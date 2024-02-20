contract;

use libraries::mock_oracle_interface::MockOracle;
storage {

    price: u64 = 0,
    precision: u64 = 6,
}
// To the auditor: This is a mock oracle contract that is used for testing purposes.
// It is not meant to be used in production. We are waiting for the oracle interfaces 
// to be finalized before we implement the real oracle contract. 
impl MockOracle
 for Contract {
    #[storage(read)]
    fn get_price() -> u64 {
        storage.price.read()
    }
    #[storage(read)]
    fn get_precision() -> u64 {
        storage.precision.read()
    }
    #[storage(write)]
    fn set_price(price: u64) {
        storage.price.write(price)
    }
}
