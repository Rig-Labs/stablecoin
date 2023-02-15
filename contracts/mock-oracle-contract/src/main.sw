contract;

storage {
    price: u64 = 0,
    precision: u64 = 6,
}

abi MockOracle {
    #[storage(write)]
    fn set_price(price: u64);

    #[storage(read)]
    fn get_price() -> u64;

    #[storage(read)]
    fn get_precision() -> u64;
}

impl MockOracle for Contract {
    #[storage(read)]
    fn get_price() -> u64 {
        storage.price
    }

    #[storage(read)]
    fn get_precision() -> u64 {
        storage.precision
    }

    #[storage(write)]
    fn set_price(_price: u64) {
        storage.price = _price
    }
}
