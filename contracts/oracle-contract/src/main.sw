contract;
// This contract, Oracle, serves as an interface to query asset prices from Pyth or Redstone oracles. stFuel is taken solely from Stork oracle.
//
// Key functionalities include:
// - Providing a unified interface to fetch price data from different oracle sources
// - Converting price data from different precisions to a standardized format
// - Implementing safeguards against potential precision issues
// - Prioritizing price sources based on availability and recency
//
// Price Priority:
// 1. Snork Orcale: Used only for stFuel price.
// 2. Pyth Oracle: used initially for all other assets.
// 3. Redstone Oracle: If the Pyth price is unavailable or outdated, the contract falls back to the Redstone oracle.
//
// This prioritization ensures that the most reliable and recent price data is used,
// enhancing the overall stability and accuracy of the Fluid Protocol.
// 
// IMPORTANT: The price returned by this contract should be the price in USD for 1_000_000_000 units of the asset in the Fuel network.
// Example 1: if the price of ETH is $3,000 USD with a 9 decimal representation in the fuel network, then the price of ETH is 3_000_000_000_000.
// Example 2: the price of BTC is $60,000 USD with an 8 decimal representation in the fuel network, then the price of 1 BTC is 60_000_000_000_000.
// But 1_000_000_000 units of BTC in the fuelvm with 8 decimal precision is 10 BTC, so the price returned by this contract should be 600_000_000_000_000 for 10 BTC.
// Refer to https://github.com/FuelLabs/verified-assets/blob/main/assets.json for the number of units that 1_000_000_000 units of an asset is in the fuelvm.

mod errors;

use standards::{
    src5::{SRC5, State},
};
use std::{
    block::timestamp,
    constants::ZERO_B256,
    u128::U128,
};
use pyth_interface::{
    data_structures::price::{
        Price, PriceFeedId,
    },
    PythCore,
};
use sway_libs::{
    signed_integers::i128::I128,
    ownership::{
        _owner,
        initialize_ownership,
        transfer_ownership,
        only_owner,
    },
};
use libraries::{
    fluid_math::{
        convert_precision,
        convert_precision_u256_and_downcast,
    },
    oracle_interface::RedstoneCore,
    oracle_interface::{
        Oracle,
        StorkConfig,
        PythConfig,
        RedstoneConfig,
    },
    stork_interface::Stork,
};

use errors::*;


// // Hack: Sway does not provide a downcast to u64
// // If redstone provides a strangely high u256 which shouldn't be cast down
// // then other parts of the code must be adjusted to use u256

configurable {
    FUEL_DECIMAL_REPRESENTATION: u32 = 9,
    DEBUG: bool = false,
    INITIAL_OWNER: Identity = Identity::Address(Address::zero()),
}

// Timeout period for considering oracle data as stale (10 minutes in seconds)
const TIMEOUT: u64 = 600;

// Nanoseconds to seconds conversion factor
const NS_TO_SECONDS: u64 = 1_000_000_000;

storage {
    /// The last valid price from Stork/Pyth/Redstone.
    last_good_price: Price = Price {
        confidence: 0,
        exponent: 0,
        price: 0,
        publish_time: 0,
    },
    // Used for simulating different timestamps during testing
    debug_timestamp: u64 = 0,
    /// Oracle config for Stork oracle.
    stork_config: Option<StorkConfig> = None,
    /// Oracle config for Pyth oracle.
    pyth_config: Option<PythConfig> = None,
    /// Oracle config for Redstone oracle.
    redstone_config: Option<RedstoneConfig> = None,
}

// Oracle contract, that reads from Stork, Pyth, and Redstone.
impl Oracle for Contract {

    /// ------------------------ ORACLE FUNCTIONS ------------------------

    // Initialize the oracle contract with owners and configs.
    #[storage(read, write)]
    fn initialize(
        stork_config: Option<StorkConfig>,
        pyth_config: Option<PythConfig>,
        redstone_config: Option<RedstoneConfig>,
    ) {
        initialize_ownership(INITIAL_OWNER);

        // Set the configs for Stork, Pyth, and Redstone.
        _set_stork_config(stork_config);
        _set_pyth_config(pyth_config);
        _set_redstone_config(redstone_config);
    }

    // Get the price of the asset.
    #[storage(read, write)]
    fn get_price() -> u64 {

        // Determine the current timestamp based on debug mode
        let current_time = match DEBUG {
            true => storage.debug_timestamp.read(),
            false => timestamp(),
        };

        // Get the configs for Stork, Pyth, and Redstone.
        let stork_config = _get_stork_config();
        let pyth_config = _get_pyth_config();
        let redstone_config = _get_redstone_config();
        let last_good_price = _get_last_good_price();

        let mut stork_price: u64 = 0;
        let mut pyth_price: Price = Price {
            confidence: 0,
            exponent: 0,
            price: 0,
            publish_time: 0,
        };
        let mut redstone_price: u64 = 0;
        let mut stork_timestamp: u64 = 0;
        let mut redstone_timestamp: u64 = 0;

        // Step 1: Attempt to get the price from Stork if configured.
        if stork_config.is_some() {
            // Get the price from Stork.
            let (sp, st) = _get_stork_price(stork_config.unwrap());

            // Assign after returning from _get_stork_price
            stork_price = sp;
            stork_timestamp = st;

            // Check if we have alternative oracles configured.
            if pyth_config.is_none() && redstone_config.is_none() {

                // Set the last good price to the Stork price, fn will check if its more recent.
                _set_last_good_price(last_good_price, stork_price, stork_timestamp);

                // If Pyth and Redstone are not configured, return the Stork price
                // regardless of staleness and confidence.
                return stork_price;
            }

            // Do a staleness check on the Stork price if we have alternative oracles configured.
            if current_time <= stork_timestamp + TIMEOUT {

                // Set the last good price to the Stork price, fn will check if its more recent.
                _set_last_good_price(last_good_price, stork_price, stork_timestamp);

                // If the Stork price is not stale, return it.
                return stork_price;
            }

            // Otherwise keep going by checking alternative oracles.
        }

        // Step 2: Attempt to get the price from Pyth if configured.
        if pyth_config.is_some() {

            // Get the price from Pyth.
            pyth_price = _get_pyth_price(pyth_config.unwrap());

            // Check if we have alternative oracles configured.
            if redstone_config.is_none() && stork_config.is_none() {
                // Set the last good price to the Pyth price, fn will check if its more recent.
                _set_last_good_price(last_good_price, pyth_price.price, pyth_price.publish_time);

                // If Redstone is not configured, return the Pyth price regardless of staleness and confidence.
                return pyth_price.price;
            }
            
            // Do a staleness check on the Pyth price if we have alternative oracles configured.
            if current_time <= pyth_price.publish_time + TIMEOUT {

                // Set the last good price to the Pyth price, fn will check if its more recent.
                _set_last_good_price(last_good_price, pyth_price.price, pyth_price.publish_time);

                // If the Pyth price is not stale or outside confidence, return it.
                return pyth_price.price;
            }

            // Otherwise keep going by checking Redstone.
        }

        // Step 3: Attempt to get the price from Redstone, if not then we get last good price.
        if redstone_config.is_some() {
            // Get the price from Redstone.
            let (rp, rt) = _get_redstone_price(redstone_config.unwrap());

            // Assign after returning from _get_redstone_price
            redstone_price = rp;
            redstone_timestamp = rt;

            // Check if Redstone is not stale.
            if current_time <= redstone_timestamp + TIMEOUT {

                // Set the last good price to the Redstone price, fn will check if its more recent.
                _set_last_good_price(last_good_price, redstone_price, redstone_timestamp);

                // Redstone is not stale, return the price.
                return redstone_price;
            }
        } 

        // If we get here, all oracle prices are stale or we've run out of oracles to try.
        // Compare timestamps to find the most recent price
        let last_price = _get_last_good_price();
        let mut most_recent_timestamp = last_price.publish_time;
        let mut most_recent_price = last_price.price;

        // Compare with Stork if it was queried.
        if stork_config.is_some() && stork_timestamp > most_recent_timestamp {
            most_recent_timestamp = stork_timestamp;
            most_recent_price = stork_price;
        }

        // Compare with Pyth if it was queried.
        if pyth_config.is_some() && pyth_price.publish_time > most_recent_timestamp {
            most_recent_timestamp = pyth_price.publish_time;
            most_recent_price = pyth_price.price;
        }

        // Compare with Redstone if it was queried.
        if redstone_config.is_some() && redstone_timestamp > most_recent_timestamp {
            most_recent_timestamp = redstone_timestamp;
            most_recent_price = redstone_price;
        }

        // Update storage with the most recent price.
        _set_last_good_price(last_good_price, most_recent_price, most_recent_timestamp);

        // Return the most recent price.
        return most_recent_price;
    }

   /// ------------------------ PUBLIC SETTERS ------------------------ ///

    // Set the config for the Stork oracle, can be set to None.
    #[storage(read, write)]
    fn set_stork_config(config: Option<StorkConfig>) {
        only_owner();

        _set_stork_config(config);
    }

    // Set the config for the Pyth oracle, can be set to None.
    #[storage(read, write)]
    fn set_pyth_config(config: Option<PythConfig>) {
        only_owner();

        _set_pyth_config(config);
    }

    // Set the config for the Redstone oracle, can be set to None.
    #[storage(read, write)]
    fn set_redstone_config(config: Option<RedstoneConfig>) {
        only_owner();

        _set_redstone_config(config);
    }

    // Set the debug timestamp, only in debug mode.
    #[storage(write)]
    fn set_debug_timestamp(timestamp: u64) {
        only_owner();

        // Allow setting a custom timestamp for testing, but only in debug mode
        require(DEBUG, OracleError::DebugNotEnabled);
        storage.debug_timestamp.write(timestamp);
    }

    // Transfer ownership of the contract.
    #[storage(read, write)]
    fn transfer_ownership(new_owner: Identity) {
        transfer_ownership(new_owner);
    }

   /// ------------------------ PUBLIC GETTERS ------------------------ ///

    #[storage(read)]
    fn get_stork_config() -> Option<StorkConfig> {
        _get_stork_config()
    }
    
    #[storage(read)]
    fn get_pyth_config() -> Option<PythConfig> {
        _get_pyth_config()
    }
    
    #[storage(read)]
    fn get_redstone_config() -> Option<RedstoneConfig> {
        _get_redstone_config()
    }

    #[storage(read)]
    fn get_last_good_price() -> Price {
        _get_last_good_price()
    }

    #[storage(read)]
    fn get_stork_price() -> Option<(u64, u64)> {  
        let stork_config = _get_stork_config();
        if stork_config.is_some() {
            let (stork_price, stork_timestamp) = _get_stork_price(stork_config.unwrap());
            return Some((stork_price, stork_timestamp));
        }
        return None;
    }
    
    #[storage(read)]
    fn get_pyth_price() -> Option<Price> {
        let pyth_config = _get_pyth_config();
        if pyth_config.is_some() {
            return Some(_get_pyth_price(pyth_config.unwrap()));
        }
        return None;
    }

    #[storage(read)]
    fn get_redstone_price() -> Option<(u64, u64)> {
        let redstone_config = _get_redstone_config();
        if redstone_config.is_some() {
            let (redstone_price, redstone_timestamp) = _get_redstone_price(redstone_config.unwrap());
            return Some((redstone_price, redstone_timestamp));
        }
        return None;
    }
}

// --------------- SRC5 IMPLEMENTATION --------------- ///

impl SRC5 for Contract {
    #[storage(read)]
    fn owner() -> State {
        _owner()
    }
}

/// ------------------------ PRIVATE SETTERS ------------------------ ///

/// Set the last good price, to be used if all oracles are stale.
#[storage(write)]
fn _set_last_good_price(last_good_price: Price, price: u64, timestamp: u64) {
    // Only set the last good price if the new price is more recent.
    if timestamp > last_good_price.publish_time {
        let new_price = Price::new(0, 0, price, timestamp);
        storage.last_good_price.write(new_price);
    }
}

/// Set the stork config, can be set to None.
#[storage(write)]
fn _set_stork_config(config: Option<StorkConfig>) {
    storage.stork_config.write(config);
}

/// Set the python config, can be set to None.
#[storage(read, write)]
fn _set_pyth_config(config: Option<PythConfig>) {
    storage.pyth_config.write(config);
}

/// Set the redstone config, can be set to None.
#[storage(read, write)]
fn _set_redstone_config(config: Option<RedstoneConfig>) {
    storage.redstone_config.write(config);
}

/// ------------------------ PRIVATE GETTERS ------------------------ ///

/// Get the last good price, to be used if all oracles are stale.
#[storage(read)]
fn _get_last_good_price() -> Price {
    storage.last_good_price.read()
}

fn _get_stork_price(stork_config: StorkConfig) -> (u64, u64) {

    // Get the stork contract.
    let stork_contract = abi(Stork, stork_config.contract_id.bits());

    // Get the stork price.
    let stork_price_result = stork_contract.get_temporal_numeric_value_unchecked_v1(
        stork_config.feed_id,
    );
    // Use direct field access to avoid method resolution issues across ABI boundaries
    let stork_timestamp_ns = stork_price_result.timestamp_ns;
    let stork_timestamp = stork_timestamp_ns / NS_TO_SECONDS;
    let stork_quantized_value = stork_price_result.quantized_value;

    // Convert to formatted price, throws an error if cannot convert
    let stork_price_u64 = _stork_quantized_to_fuel_u64(
        stork_quantized_value,
        FUEL_DECIMAL_REPRESENTATION,
    );

    return (stork_price_u64, stork_timestamp);
}

/// Get the pyth price, if set otherwise return None.
fn _get_pyth_price(pyth_config: PythConfig) -> Price {

    // Get the pyth contract.
    let pyth_contract = abi(PythCore, pyth_config.contract_id.bits());
    let mut pyth_price = pyth_contract.price_unsafe(pyth_config.feed_id);

    // Adjust the price to the fuel VM precision.
    pyth_price = _pyth_price_with_fuel_vm_precision_adjustment(
        pyth_price,
        FUEL_DECIMAL_REPRESENTATION,
    );

    // Return the price.
    return pyth_price;
}

/// Get the redstone price, if set otherwise return None.
fn _get_redstone_price(redstone_config: RedstoneConfig) -> (u64, u64) {

    // Get the redstone contract.
    let redstone_contract = abi(RedstoneCore, redstone_config.contract_id.bits());

    // Get the redstone price.
    let mut feed = Vec::with_capacity(1);
    feed.push(redstone_config.feed_id);

    let redstone_prices = redstone_contract.read_prices(feed);
    let redstone_timestamp = redstone_contract.read_timestamp();
    let redstone_price_u64 = redstone_prices.get(0).unwrap();

    // By default redstone uses 8 decimal precision so it is generally safe to cast down
    let redstone_price = convert_precision_u256_and_downcast(
        redstone_price_u64,
        _adjust_exponent(redstone_config.precision, FUEL_DECIMAL_REPRESENTATION),
    );

    return (redstone_price, redstone_timestamp);
}

/// Get the stork config, if set otherwise return None.
#[storage(read)]
fn _get_stork_config() -> Option<StorkConfig> {
    storage.stork_config.try_read().unwrap_or(None)
}

/// Get the python config, if set otherwise return None.
#[storage(read)]
fn _get_pyth_config() -> Option<PythConfig> {
    storage.pyth_config.try_read().unwrap_or(None)
}

/// Get the redstone config, if set otherwise return None.
#[storage(read)]
fn _get_redstone_config() -> Option<RedstoneConfig> {
    storage.redstone_config.try_read().unwrap_or(None)
}

// Assets in the fuel VM can have a different decimal representation
// This function adjusts the price to align with the decimal representation of the Fuel VM
fn _pyth_price_with_fuel_vm_precision_adjustment(
    pyth_price: Price,
    fuel_vm_decimals: u32,
) -> Price {
    let adjusted_exponent = _adjust_exponent(pyth_price.exponent, fuel_vm_decimals);
    return Price {
        confidence: convert_precision(pyth_price.confidence, adjusted_exponent),
        price: convert_precision(pyth_price.price, adjusted_exponent),
        publish_time: pyth_price.publish_time,
        exponent: adjusted_exponent,
    };
}

/// Adjust the exponent of the price to the fuel VM precision.
fn _adjust_exponent(
    current_exponent: u32,
    fuel_vm_decimals: u32,
) -> u32 {
    if fuel_vm_decimals > 9u32 {
        // If the Fuel VM has more decimals than 9 we need to remove the extra precision
        // For example, if the Fuel VM has 10 decimals then we need to divide the price by 10^1
        // to get the correct price for 1_000_000_000 units
        return current_exponent + (fuel_vm_decimals - 9u32);
    } else if fuel_vm_decimals < 9u32 {
        // If the Fuel VM has less decimals than 9 we need to add the missing precision
        // Smaller precision means we need to add more precision to the price
        return current_exponent - (9u32 - fuel_vm_decimals);
    }
    return current_exponent;
}

// Convert Stork I128 quantized value (biased by 2^127) into u64 with decimal conversion and rounding up
fn _stork_quantized_to_fuel_u64(value: I128, fuel_decimal_representation: u32) -> u64 {
    let underlying = value.underlying();
    _stork_underlying_to_fuel_u64(underlying, fuel_decimal_representation)
}

// Convert Stork underlying U128 (which includes the 2^127 bias) to u64 with decimal conversion and rounding up
fn _stork_underlying_to_fuel_u64(underlying: U128, fuel_decimal_representation: u32) -> u64 {

    let bias = U128::from((1u64 << 63, 0u64)); // 2^127

	// If the underlying is below the bias, the logical signed value is negative
	require(underlying >= bias, OracleError::NegativeValue);
	let magnitude = underlying - bias;

	// Apply decimal conversion from 18 to fuel_decimal_representation with rounding up on downscale
	let adjusted_value = if fuel_decimal_representation < 18u32 {
		let precision = 18u32 - fuel_decimal_representation;
		let divisor = U128::from(10u32).pow(precision);

        (magnitude + (divisor - U128::from(1u32))).divide(divisor)
    } else if fuel_decimal_representation > 18u32 {
		let decimal_diff = fuel_decimal_representation - 18u32;
		let multiplier = U128::from(10u32).pow(decimal_diff);
		require(
			magnitude <= U128::from(u64::max()).divide(multiplier),
			OracleError::MultiplicationWouldExceedU64Maximum
		);
		magnitude * multiplier
	} else {
		magnitude
	};

    // Bound to u64
	require(adjusted_value <= U128::from(u64::max()), OracleError::PriceValueExceedsU64Maximum);
	adjusted_value.as_u64().unwrap()
}

#[test]
fn test_pyth_price_adjustment_fuel_vm_decimals_equal() {
    // Fuel VM has 9 decimals
    let fuel_vm_decimals = 9;
    let original_price = Price {
        confidence: 100,
        price: 1_000_000_000,
        publish_time: 1000,
        exponent: 9,
    };
    let adjusted_price = _pyth_price_with_fuel_vm_precision_adjustment(original_price, fuel_vm_decimals);
    assert(adjusted_price.price == 1_000_000_000);
    assert(adjusted_price.confidence == 100);
    assert(adjusted_price.exponent == 9);
    assert(adjusted_price.publish_time == 1000);
}

#[test]
fn test_pyth_price_adjustment_fuel_vm_decimals_greater_than_9() {
    // Fuel VM has 10 decimals
    let fuel_vm_decimals = 10;
    // With 10 decimals, the price of 1_000_000_000 units of an asset is 100_000_000
    let original_price = Price {
        confidence: 100,
        price: 1_000_000_000,
        publish_time: 1000,
        exponent: 9,
    };
    let adjusted_price = _pyth_price_with_fuel_vm_precision_adjustment(original_price, fuel_vm_decimals);
    assert(adjusted_price.price == 100_000_000);
    assert(adjusted_price.confidence == 10);
    assert(adjusted_price.exponent == 10);
    assert(adjusted_price.publish_time == 1000);
}

#[test]
fn test_pyth_price_adjustment_fuel_vm_decimals_less_than_9() {
    // Fuel VM has 8 decimals
    let fuel_vm_decimals = 8;
    // With 8 decimals, the price of 1_000_000_000 units of an asset is 10_000_000_000
    let original_price = Price {
        confidence: 100,
        price: 1_000_000_000,
        publish_time: 1000,
        exponent: 9,
    };
    let adjusted_price = _pyth_price_with_fuel_vm_precision_adjustment(original_price, fuel_vm_decimals);
    assert(adjusted_price.price == 10_000_000_000);
    assert(adjusted_price.confidence == 1000);
    assert(adjusted_price.exponent == 8);
    assert(adjusted_price.publish_time == 1000);
}

#[test]
fn test_pyth_price_adjustment_fuel_vm_decimals_8_initial_exponent_8() {
    // Fuel VM has 8 decimals
    let fuel_vm_decimals = 8; // add 10^1 to the price and confidence
    // Initial Pyth price with exponent 8
    let original_price = Price {
        confidence: 1000,
        price: 1_000_000_000,
        publish_time: 1000,
        exponent: 8, // add a second 10^1 to the price and confidence
    };
    let adjusted_price = _pyth_price_with_fuel_vm_precision_adjustment(original_price, fuel_vm_decimals);
    // Expected: price for 1_000_000_000 units (10 units with 8 decimals) should be 1_000_000_000 ($10.00)
    assert(adjusted_price.price == 100_000_000_000);
    assert(adjusted_price.confidence == 100_000);
    assert(adjusted_price.exponent == 7);
    assert(adjusted_price.publish_time == 1000);
}

#[test]
fn test_stork_underlying_downscale_rounds_down() {
    // magnitude = 1e18 -> for fuel 9, result = 1e9
    let bias = U128::from(2u64).pow(127u32);
    let magnitude = U128::from(2_000_000_000_000_000_000u64);
    let underlying = bias + magnitude;
    let result = _stork_underlying_to_fuel_u64(underlying, 9);
    assert(result == 2_000_000_000);
}


#[test]
fn test_stork_underlying_downscale_rounds_up() {
    // magnitude = 1e18 + (1e9 - 1) -> rounds up to 1_000_000_001 when scaled to 9
    let bias = U128::from(2u64).pow(127u32);
    let magnitude = U128::from(1_000_000_000_000_000_000u64) + (U128::from(1_000_000_000u64) - U128::from(1u64));
    let underlying = bias + magnitude;
    let result = _stork_underlying_to_fuel_u64(underlying, 9);
    assert(result == 1_000_000_001);
}

#[test]
fn test_stork_underlying_same_decimals_18() {
    // No change when fuel decimals = 18
    let bias = U128::from(2u64).pow(127u32);
    let magnitude = U128::from(123_456_789_000_000_000u64);
    let underlying = bias + magnitude;
    let result = _stork_underlying_to_fuel_u64(underlying, 18);
    assert(result == 123_456_789_000_000_000);
}

#[test]
fn test_stork_underlying_upscale_safe() {
    // Upscale by +1 decimal: fuel=19 with small magnitude to avoid overflow
    let bias = U128::from(2u64).pow(127u32);
    let magnitude = U128::from(123_456_789u64); // small value
    let underlying = bias + magnitude;
    let result = _stork_underlying_to_fuel_u64(underlying, 19);
    assert(result == 1_234_567_890);
}

#[test(should_revert)]
fn test_stork_underlying_overflow_bound_direct() {
    // When fuel=18 and magnitude > u64::MAX, it should revert
    let bias = U128::from(2u64).pow(127u32);
    let magnitude = U128::from(u64::max()) + U128::from(1u64);
    let underlying = bias + magnitude;
    let _ = _stork_underlying_to_fuel_u64(underlying, 18);
}

#[test(should_revert)]
fn test_stork_underlying_overflow_on_multiply() {
    // Choose multiplier = 10^2 and magnitude exceeding u64::MAX / 100
    let bias = U128::from(2u64).pow(127u32);
    let multiplier = U128::from(10u32).pow(2u32); // 100
    let safe_max = U128::from(u64::max()) / multiplier;
    let magnitude = safe_max + U128::from(1u64);
    let underlying = bias + magnitude;
    let _ = _stork_underlying_to_fuel_u64(underlying, 20);
}
