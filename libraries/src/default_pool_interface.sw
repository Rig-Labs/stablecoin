library default_pool_interface;

abi DefaultPool {
    #[storage(read, write)]
    fn initialize(trove_manager: Identity, active_pool: ContractId, asset_id: ContractId);

    #[storage(read, write)]
    fn send_asset_to_active_pool(amount: u64);

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
