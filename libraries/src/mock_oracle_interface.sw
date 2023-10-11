library;

abi MockOracle {
    #[storage(write)]
    fn set_price(price: u64);

    #[storage(read)]
    fn get_price() -> u64;

    #[storage(read)]
    fn get_precision() -> u64;
}
