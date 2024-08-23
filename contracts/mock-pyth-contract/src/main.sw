contract;

use libraries::oracle_interface::{PythCore, PythPrice, PythError, PythPriceFeedId, PythPriceFeed};
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
            storage.latest_price_feed.insert(feeds.get(feed_index).unwrap().0, feeds.get(feed_index).unwrap().1);
            feed_index += 1;
        }
    }
}
