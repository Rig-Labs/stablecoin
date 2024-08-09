library;

abi Oracle {
    // TODO: remove
    #[storage(write)]
    fn set_price(price: u64);

    // TODO: return Price
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