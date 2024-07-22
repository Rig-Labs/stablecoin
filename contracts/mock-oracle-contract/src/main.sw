contract;

use libraries::mock_oracle_interface::{Oracle, OracleModule, Price};
use std::{
    block::timestamp,
    constants::ZERO_B256,
};

configurable {
    PYTH: ContractId = ContractId::from(ZERO_B256),
    REDSTONE: ContractId = ContractId::from(ZERO_B256),
    TIMEOUT: u64 = 0,
}

storage {
    /// The last price
    price: Price = Price { value: 0, time: 0 },

    // TODO: remove?
    precision: u64 = 6,

    // TODO: remove upon integration
    // Each module knows its price
    module_price: Price = Price { value: 0, time: 0 },

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
        let pyth_price = abi(OracleModule, PYTH.bits()).price();

        // If the primary module is determined to be stale then fallback to the next best metric
        if current_time - pyth_price.time > TIMEOUT || last_price.time == pyth_price.time {
            // Query the fallback module for its price

            // Fuel Bug: trait coherence
            let id = REDSTONE.bits();
            let redstone_price = abi(OracleModule, id).price();

            // if the fallback oracle is also stale then compare the oracle times and the last price 
            // to determine which value is the latest
            if current_time - redstone_price.time > TIMEOUT || last_price.time == redstone_price.time {
                // redstone is also stale so use the latest price we have available
                if redstone_price.time <= pyth_price.time {
                    if last_price.time < pyth_price.time {
                        storage.price.write(pyth_price);
                        return pyth_price.value;
                    }
                } else {
                    if last_price.time < redstone_price.time {
                        storage.price.write(redstone_price);
                        return redstone_price.value;
                    }
                }

                return last_price.value;
            }

            // oracle is live so compare if it has the latest data
            if last_price.time < redstone_price.time {
                storage.price.write(redstone_price);
                return redstone_price.value;
            }

            return last_price.value;
        }

        // oracle is live so compare if it has the latest data
        if last_price.time < pyth_price.time {
            storage.price.write(pyth_price);
            return pyth_price.value;
        }

        return last_price.value;
    }

    #[storage(read)]
    fn get_precision() -> u64 {
        storage.precision.read()
    }

    #[storage(write)]
    fn set_price(price: u64) {
        storage.legacy_price.write(price)
    }
}

// impl OracleModule for Contract {
//     #[storage(read)]
//     fn price() -> Price {
//         storage.module_price.read()
//     }

//     #[storage(write)]
//     fn set_module_price(price: u64) {
//         storage.module_price.write(Price::new(price, timestamp()));
//     }
// }
