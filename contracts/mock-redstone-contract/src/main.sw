contract;

use libraries::mock_oracle_interface::RedstoneCore;
use std::block::timestamp;

storage {
    timestamp: u64 = 0
}

impl RedstoneCore for Contract {
    #[storage(read)]
    fn read_prices(feed_ids: Vec<u256>) -> Vec<u256> {
        let mut prices = Vec::with_capacity(1);
        prices.push(u256::from(0_u64));
        prices
    }

    #[storage(read)]
    fn read_timestamp() -> u64 {
        storage.timestamp.read()
    }

    #[storage(write)]
    fn set_timestamp(time: u64) {
        storage.timestamp.write(time)
    }
}
