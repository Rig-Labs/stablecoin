contract;

use libraries::mock_oracle_interface::{Oracle, Price};
use pyth_interface::{data_structures::price::{PriceFeedId, Price as PythPrice}, PythCore};
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

configurable {
    PYTH: ContractId = ContractId::from(ZERO_B256),
    PYTH_PRICE_ID: PriceFeedId = ZERO_B256,
    REDSTONE: ContractId = ContractId::from(ZERO_B256),
    TIMEOUT: u64 = 0,
}

storage {
    /// The last price
    price: Price = Price { value: 0, time: 0 },

    // TODO: remove and clean up tests later
    legacy_price: u64 = 0
}

// NOTE: WIP
impl Oracle for Contract {
    #[storage(read, write)]
    fn get_price() -> u64 {
        // Fetch the current time and price to evaluate if a price may be stale
        let current_time = timestamp();
        let last_price = storage.price.read();

        // Query the primary module for its price feed
        let pyth_price = abi(PythCore, PYTH.bits()).price(PYTH_PRICE_ID);

        // If the primary module is determined to be stale then fallback to the next best metric
        if current_time - pyth_price.publish_time > TIMEOUT || last_price.time == pyth_price.publish_time {
            // Query the fallback module for its price

            // Fuel Bug: trait coherence
            let id = REDSTONE.bits();
            let redstone_price = abi(PythCore, id).price(PYTH_PRICE_ID); // TODO

            // if the fallback oracle is also stale then compare the oracle times and the last price 
            // to determine which value is the latest
            if current_time - redstone_price.publish_time > TIMEOUT || last_price.time == redstone_price.publish_time {
                // redstone is also stale so use the latest price we have available
                if redstone_price.publish_time <= pyth_price.publish_time {
                    if last_price.time < pyth_price.publish_time {
                        storage.price.write(pyth_price.into());
                        return pyth_price.price;
                    }
                } else {
                    if last_price.time < redstone_price.publish_time {
                        storage.price.write(redstone_price.into());
                        return redstone_price.price;
                    }
                }

                return last_price.value;
            }

            // oracle is live so compare if it has the latest data
            if last_price.time < redstone_price.publish_time {
                storage.price.write(redstone_price.into());
                return redstone_price.price;
            }

            return last_price.value;
        }

        // oracle is live so compare if it has the latest data
        if last_price.time < pyth_price.publish_time {
            storage.price.write(pyth_price.into());
            return pyth_price.price;
        }

        return last_price.value;
    }

    #[storage(write)]
    fn set_price(price: u64) {
        storage.legacy_price.write(price)
    }
}
