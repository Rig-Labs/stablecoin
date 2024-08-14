contract;

use libraries::{
    mock_oracle_interface::{PythPriceFeedId, PythPrice, PythCore},
    mock_oracle_interface::{Oracle, Price},
    mock_oracle_interface::RedstoneCore,
};
// use pyth_interface::{data_structures::price::{PriceFeedId, Price as PythPrice}, PythCore};
use std::{
    block::timestamp,
    bytes::Bytes,
    constants::ZERO_B256,
};

// Hack because of Sway Compiler consuming < 64GB RAM in library import location
impl From<PythPrice> for Price {
    fn from(p: PythPrice) -> Self {
        Self {
            value: p.price.into(),
            time: p.publish_time
        }
    }
}

configurable {
    PYTH: ContractId = ContractId::from(ZERO_B256),
    PYTH_PRICE_ID: PythPriceFeedId = ZERO_B256,
    REDSTONE: ContractId = ContractId::from(ZERO_B256),
    REDSTONE_PRICE_ID: u256 = u256::min(),
    TIMEOUT: u64 = 0,
}

storage {
    /// The last price
    price: Price = Price { value: 0, time: 0 },

    // TODO: remove and clean up tests later
    legacy_price: u64 = 0
}

impl Oracle for Contract {
    #[storage(read, write)]
    fn get_price(redstone_payload: Bytes) -> Price {
        // Fetch the current time and price to evaluate if a price may be stale
        let current_time = timestamp();
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
            let (redstone_prices, redstone_timestamp) = abi(RedstoneCore, id).get_prices(feed, redstone_payload);
            let redstone_price = redstone_prices.get(0).unwrap();

            // if the fallback oracle is also stale then compare the oracle times and the last price 
            // to determine which value is the latest
            if current_time - redstone_timestamp > TIMEOUT || last_price.time == redstone_timestamp {
                // redstone is also stale so use the latest price we have available
                if redstone_timestamp <= pyth_price.publish_time {
                    if last_price.time < pyth_price.publish_time {
                        let price: Price = pyth_price.into();
                        storage.price.write(price);
                        return price;
                    }
                } else {
                    if last_price.time < redstone_timestamp {
                        let price = Price::new(redstone_price, redstone_timestamp);
                        storage.price.write(price);
                        return price;
                    }
                }

                return last_price;
            }

            // oracle is live so compare if it has the latest data
            if last_price.time < redstone_timestamp {
                let price = Price::new(redstone_price, redstone_timestamp);
                storage.price.write(price);
                return price;
            }

            return last_price;
        }

        // oracle is live so compare if it has the latest data
        if last_price.time < pyth_price.publish_time {
            let price: Price = pyth_price.into();
            storage.price.write(price);
            return price;
        }

        return last_price;
    }

    #[storage(write)]
    fn set_price(price: u64) {
        storage.legacy_price.write(price)
    }
}
