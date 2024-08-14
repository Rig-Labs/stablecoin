library;

use std::bytes::Bytes;

abi Oracle {
    // TODO: remove
    #[storage(write)]
    fn set_price(price: u64);

    #[storage(read, write)]
    fn get_price(redstone_payload: Bytes) -> Price;
}

pub struct Price {
    pub value: u256,
    pub time: u64
}

impl Price {
    pub fn new(price: u256, time: u64) -> Self {
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
    fn get_prices(feed_ids: Vec<u256>, payload_bytes: Bytes) -> (Vec<u256>, u64);
}
