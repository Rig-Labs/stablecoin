contract;

use libraries::{
    mock_oracle_interface::{PythPriceFeedId, PythPrice, PythCore},
    mock_oracle_interface::{Oracle, Price},
    mock_oracle_interface::RedstoneCore,
};
use std::{
    block::timestamp,
    constants::ZERO_B256,
};

// Hack because of Sway Compiler consuming < 64GB RAM in library import location
impl From<PythPrice> for Price {
    fn from(p: PythPrice) -> Self {
        Self {
            value: p.price,
            time: p.publish_time
        }
    }
}

// Hack: Sway does not provide a downcast to u64
// If redstone provides a strangely high u256 which shouldn't be cast down
// then other parts of the code must be adjusted to use u256
impl u256 {
    fn to_u64(self) -> u64 {
        let (_a, _b, _c, d): (u64, u64, u64, u64) = asm(r1: self) {
            r1: (u64, u64, u64, u64)
        };

        d
    }
}

configurable {
    /// Contract Address
    PYTH: ContractId = ContractId::from(ZERO_B256),
    /// Price feed to query
    PYTH_PRICE_ID: PythPriceFeedId = ZERO_B256,
    /// Contract Address
    REDSTONE: ContractId = ContractId::from(ZERO_B256),
    /// Price feed to query
    REDSTONE_PRICE_ID: u256 = u256::min(),
    /// Timeout in seconds
    TIMEOUT: u64 = 0,

    // Workaround for testing timestamps
    DEBUG: bool = false,
}

storage {
    /// The last price from either Pyth or Redstone
    price: Price = Price { value: 0, time: 0 },

    // Workaround for testing timestamps
    debug_timestamp: u64 = 0,
}

impl Oracle for Contract {
    #[storage(read, write)]
    fn get_price() -> u64 {
        // Fetch the current time and price to evaluate if a price may be stale
        let current_time = match DEBUG {
            true => storage.debug_timestamp.read(),
            false => timestamp(),
        };
        let last_price = storage.price.read();

        // Query the primary module for its price feed
        let pyth_price = abi(PythCore, PYTH.bits()).price(PYTH_PRICE_ID);

        // If the primary module is determined to be stale then fallback to the next best metric
        if current_time - pyth_price.publish_time > TIMEOUT || last_price.time == pyth_price.publish_time {
            // Query the fallback module for its price

            // Define the redstone price feed arguments
            let mut feed = Vec::with_capacity(1);
            feed.push(REDSTONE_PRICE_ID);

            // Fuel Bug: trait coherence
            let id = REDSTONE.bits();
            let redstone = abi(RedstoneCore, id);
            let redstone_prices = redstone.read_prices(feed);
            let redstone_timestamp = redstone.read_timestamp();
            let redstone_price = redstone_prices.get(0).unwrap().to_u64();

            // if the fallback oracle is also stale then compare the oracle times and the last price 
            // to determine which value is the latest
            if current_time - redstone_timestamp > TIMEOUT || last_price.time == redstone_timestamp {
                // redstone is also stale so use the latest price we have available
                if redstone_timestamp <= pyth_price.publish_time {
                    if last_price.time < pyth_price.publish_time {
                        let price: Price = pyth_price.into();
                        storage.price.write(price);
                        return price.value;
                    }
                } else {
                    if last_price.time < redstone_timestamp {
                        let price = Price::new(redstone_price, redstone_timestamp);
                        storage.price.write(price);
                        return price.value;
                    }
                }
                return last_price.value;
            }

            // oracle is live so compare if it has the latest data
            if last_price.time < redstone_timestamp {
                let price = Price::new(redstone_price, redstone_timestamp);
                storage.price.write(price);
                return price.value;
            }

            return last_price.value;
        }

        // oracle is live so compare if it has the latest data
        if last_price.time < pyth_price.publish_time {
            let price: Price = pyth_price.into();
            storage.price.write(price);
            return price.value;
        }

        return last_price.value;
    }

    #[storage(write)]
    fn set_debug_timestamp(timestamp: u64) {
        storage.debug_timestamp.write(timestamp);
    }
}
