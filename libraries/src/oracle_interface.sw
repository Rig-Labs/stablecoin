library;

use std::bytes::Bytes;

abi Oracle {
    #[storage(read, write)]
    fn get_price() -> u64;

    // Testing workaround
    #[storage(write)]
    fn set_debug_timestamp(timestamp: u64);
}

pub struct Price {
    pub value: u64,
    pub time: u64,
}

impl Price {
    pub fn new(price: u64, time: u64) -> Self {
        Self {
            value: price,
            time,
        }
    }
}

// Mocked Redstone structures
abi RedstoneCore {
    #[storage(read)]
    fn read_prices(feed_ids: Vec<u256>) -> Vec<u256>;

    // Testing only, not the actual function signature of redstone
    #[storage(write)]
    fn write_prices(feed: Vec<(u256, u256)>);

    #[storage(read)]
    fn read_timestamp() -> u64;

    // Purely for setting up testing conditions
    #[storage(write)]
    fn set_timestamp(time: u64);
}
