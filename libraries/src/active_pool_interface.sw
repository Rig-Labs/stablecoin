library active_pool_interface;

abi ActivePool {
    #[storage(read, write)]
    fn initialize(borrow_operations: Identity, trove_manager: Identity, stability_pool: Identity, asset_id: ContractId);

    #[storage(read, write)]
    fn send_asset(address: Identity, amount: u64);

    #[storage(read)]
    fn get_asset() -> u64;

    #[storage(read)]
    fn get_usdf_debt() -> u64;

    #[storage(read, write)]
    fn increase_usdf_debt(amount: u64);

    #[storage(read, write)]
    fn decrease_usdf_debt(amount: u64);

    #[storage(read, write), payable]
    fn recieve();
}
