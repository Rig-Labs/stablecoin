library;

abi Pool {
    #[storage(read)]
    fn get_asset() -> u64;

    #[storage(read)]
    fn get_usdm_debt() -> u64;

    #[storage(read, write)]
    fn increase_usdm_debt(amount: u64);

    #[storage(read, write)]
    fn decrease_usdm_debt(amount: u64);
}
