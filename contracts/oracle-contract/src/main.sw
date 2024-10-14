contract;
// This contract, Oracle, serves as an interface to query asset prices from either Pyth or Redstone oracles.
//
// Key functionalities include:
// - Providing a unified interface to fetch price data from different oracle sources
// - Converting price data from different precisions to a standardized format
// - Implementing safeguards against potential precision issues
// - Prioritizing price sources based on availability and recency
//
// Price Priority:
// 1. Pyth Oracle: The contract first attempts to fetch the price from the Pyth oracle.
// 2. Redstone Oracle: If the Pyth price is unavailable or outdated, the contract falls back to the Redstone oracle.
//
// This prioritization ensures that the most reliable and recent price data is used,
// enhancing the overall stability and accuracy of the Fluid Protocol.


use libraries::{
    fluid_math::{
        convert_precision,
        convert_precision_u256_and_downcast,
    },
    oracle_interface::RedstoneCore,
    oracle_interface::{
        Oracle,
    },
};
use std::{block::timestamp, constants::ZERO_B256,};
use pyth_interface::{data_structures::price::{Price, PriceFeedId}, PythCore};

// // Hack: Sway does not provide a downcast to u64
// // If redstone provides a strangely high u256 which shouldn't be cast down
// // then other parts of the code must be adjusted to use u256

configurable {
    /// Contract Address
    PYTH: ContractId = ContractId::zero(),
    /// Price feed to query
    PYTH_PRICE_ID: PriceFeedId = ZERO_B256,
    /// Contract Address
    REDSTONE: ContractId = ContractId::zero(),
    /// Price feed to query
    REDSTONE_PRICE_ID: u256 = u256::min(),
    /// Precision of value returned by Redstone
    REDSTONE_PRECISION: u32 = 9,
    /// Timeout in seconds
    DEBUG: bool = false,
}
// Timeout period for considering oracle data as stale (4 hours in seconds)
const TIMEOUT: u64 = 14400;

storage {
    /// The last valid price from either Pyth or Redstone
    last_good_price: Price = Price {
        confidence: 0,
        exponent: 0,
        price: 0,
        publish_time: 0,
    },
    // Used for simulating different timestamps during testing
    debug_timestamp: u64 = 0,
}

impl Oracle for Contract {
    #[storage(read, write)]
    fn get_price() -> u64 {
        // Determine the current timestamp based on debug mode
        let current_time = match DEBUG {
            true => storage.debug_timestamp.read(),
            false => timestamp(),
        };
        // Read the last stored valid price
        let last_price = storage.last_good_price.read();
        // Step 1: Query the Pyth oracle (primary source)
        let mut pyth_price = abi(PythCore, PYTH.bits()).price(PYTH_PRICE_ID);
        pyth_price.price = convert_precision(pyth_price.price, pyth_price.exponent);
        pyth_price.confidence = convert_precision(pyth_price.confidence, pyth_price.exponent); // Convert confidence to match precision
        // Check if Pyth data is stale or outside confidence
        if is_pyth_price_stale_or_outside_confidence(pyth_price, current_time) {
            // Step 2: Pyth is stale or outside confidence, query Redstone oracle (fallback source)
            let mut feed = Vec::with_capacity(1);
            feed.push(REDSTONE_PRICE_ID);

            // Fuel Bug workaround: trait coherence
            let id = REDSTONE.bits();
            let redstone = abi(RedstoneCore, id);
            let redstone_prices = redstone.read_prices(feed);
            let redstone_timestamp = redstone.read_timestamp();
            let redstone_price_u64 = redstone_prices.get(0).unwrap();
            // By default redstone uses 8 decimal precision so it is generally safe to cast down
            let redstone_price = convert_precision_u256_and_downcast(redstone_price_u64, REDSTONE_PRECISION);
            // Check if Redstone data is also stale
            if current_time > redstone_timestamp + TIMEOUT {
                // Both oracles are stale, use the most recent data available
                if redstone_timestamp <= pyth_price.publish_time {
                    // Pyth data is more recent
                    if last_price.publish_time < pyth_price.publish_time {
                        let price: Price = pyth_price;
                        storage.last_good_price.write(price);
                        return price.price;
                    }
                } else {
                    // Redstone data is more recent
                    if last_price.publish_time < redstone_timestamp {
                        let price = Price::new(0, 0, redstone_price, redstone_timestamp);
                        storage.last_good_price.write(price);
                        return price.price;
                    }
                }
                // If both new prices are older than the last stored price, return the last price
                return last_price.price;
            }

            // Redstone data is fresh, update if it's newer than the last stored price
            if last_price.publish_time < redstone_timestamp {
                let price = Price::new(0, 0, redstone_price, redstone_timestamp);
                storage.last_good_price.write(price);
                return price.price;
            }

            // Otherwise, return the last stored price
            return last_price.price;
        }
        // Pyth data is fresh, update if it's newer than the last stored price
        if last_price.publish_time < pyth_price.publish_time {
            let price: Price = pyth_price;
            storage.last_good_price.write(price);
            return price.price;
        }

        // If the new Pyth price is older than the last stored price, return the last price
        return last_price.price;
    }

    #[storage(write)]
    fn set_debug_timestamp(timestamp: u64) {
        // Allow setting a custom timestamp for testing, but only in debug mode
        require(DEBUG, "ORACLE: Debug is not enabled");
        storage.debug_timestamp.write(timestamp);
    }
}

fn is_pyth_price_stale_or_outside_confidence(pyth_price: Price, current_time: u64) -> bool {
    // confidence within 4% is considered safe 
    let confidence_threshold = pyth_price.price / 25;
    return current_time > pyth_price.publish_time + TIMEOUT || pyth_price.confidence > confidence_threshold;
}

#[test]
fn test_is_pyth_price_not_stale_or_outside_confidence() {
    let pyth_price = Price {
        confidence: 2,
        publish_time: 100,
        price: 100,
        exponent: 0,
    };
    let is_stale_or_outside_confidence = is_pyth_price_stale_or_outside_confidence(pyth_price, 100);
    assert(is_stale_or_outside_confidence == false);
}

#[test]
fn test_is_price_outside_confidence() {
    // confidence is 5, which is outside 5% threshold 
    let pyth_price = Price {
        confidence: 5,
        publish_time: 100,
        price: 100,
        exponent: 0,
    };
    let is_stale_or_outside_confidence = is_pyth_price_stale_or_outside_confidence(pyth_price, 100);
    assert(is_stale_or_outside_confidence == true);
}

#[test]
fn test_is_price_stale() {
    // price is stale because it's published more than 4 hours ago
    let publish_time = 10000;
    let pyth_price = Price {
        confidence: 2,
        publish_time: publish_time,
        price: 100,
        exponent: 0,
    };
    let is_stale_or_outside_confidence = is_pyth_price_stale_or_outside_confidence(pyth_price, publish_time + TIMEOUT + 1);
    assert(is_stale_or_outside_confidence == true);
}
