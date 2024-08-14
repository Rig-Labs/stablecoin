contract;

use libraries::mock_oracle_interface::RedstoneCore;
use std::{block::timestamp, bytes::Bytes};

storage {}

impl RedstoneCore for Contract {
    #[storage(read)]
    fn get_prices(feed_ids: Vec<u256>, payload_bytes: Bytes) -> (Vec<u256>, u64) {
        let mut prices = Vec::with_capacity(1);
        prices.push(u256::from(42_u64));
        (prices, timestamp())
    }
}
