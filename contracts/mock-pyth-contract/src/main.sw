contract;
// This contract, MockPyth, is a mock implementation of the Pyth oracle interface.
// It is used for testing and simulation purposes within the Fluid Protocol.
//
// Key functionalities include:
// - Simulating price feeds for testing purposes
// - Providing a mock interface for interacting with the Pyth oracle
// - Ensuring compatibility with the Pyth oracle interface for testing
//
// To the auditor: This contract is not used in the system. It is only used for testing.

use libraries::oracle_interface::{PythCore, PythError, PythPrice, PythPriceFeed, PythPriceFeedId};
use std::{block::timestamp, hash::Hash};

storage {
    latest_price_feed: StorageMap<PythPriceFeedId, PythPriceFeed> = StorageMap {},
}

impl PythCore for Contract {
    #[storage(read)]
    fn price(price_feed_id: PythPriceFeedId) -> PythPrice {
        let price_feed = storage.latest_price_feed.get(price_feed_id).try_read();
        require(price_feed.is_some(), PythError::PriceFeedNotFound);

        price_feed.unwrap().price
    }

    #[storage(write)]
    fn update_price_feeds(feeds: Vec<(PythPriceFeedId, PythPriceFeed)>) {
        let mut feed_index = 0;

        while feed_index < feeds.len() {
            storage
                .latest_price_feed
                .insert(
                    feeds
                        .get(feed_index)
                        .unwrap().0,
                    feeds
                        .get(feed_index)
                        .unwrap().1,
                );
            feed_index += 1;
        }
    }
}
