contract;

use libraries::{MockOracle};

storage {
    price: u64 = 0,
    precision: u64 = 6,
}

// TODO Add migration ability
// TODO Add renounce ownership ability
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
