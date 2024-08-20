library;

use std::bytes::Bytes;

abi Oracle {
    // TODO: remove
    #[storage(write)]
    fn set_price(price: u64);

    #[storage(read, write)]
    fn get_price() -> u64;
}

pub struct Price {
    pub value: u64,
    pub time: u64
}

impl Price {
    pub fn new(price: u64, time: u64) -> Self {
        Self {
            value: price,
            time
        }
    }
}

// Mocked Pyth related structures to simulate Pyth integration
pub type PythPriceFeedId = b256;

abi PythCore {
    #[storage(read)]
    fn price(price_feed_id: PythPriceFeedId) -> PythPrice;

    // Directly exposed but logic is simplified
    #[storage(write)]
    fn update_price_feeds(feeds: Vec<(PythPriceFeedId, PythPriceFeed)>);
}

pub struct PythPrice {
    pub price: u64,
    pub publish_time: u64,
}

pub struct PythPriceFeed {
    pub price: PythPrice,
}

pub enum PythError {
    PriceFeedNotFound: (),
}

// Mocked Redstone structures
abi RedstoneCore {
    #[storage(read)]
    fn read_prices(feed_ids: Vec<u256>) -> Vec<u256>;

    #[storage(read)]
    fn read_timestamp() -> u64;

    // Purely for setting up testing conditions
    #[storage(write)]
    fn set_timestamp(time: u64);
}
