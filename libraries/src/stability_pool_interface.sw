library stability_pool_interface;

abi StabilityPool {
    #[storage(read, write)]
    fn initialize(borrow_operations_address: ContractId, usdf_address: ContractId, community_issuance_address: ContractId);

    #[storage(read, write)]
    fn add_asset(trove_manager_address: ContractId, active_pool_address: ContractId, sorted_troves_address: ContractId, asset_address: ContractId, oracle_address: ContractId);

    #[storage(read, write), payable]
    fn provide_to_stability_pool();

    #[storage(read, write)]
    fn withdraw_from_stability_pool(amount: u64);

    #[storage(read, write)]
    fn withdraw_gain_to_trove(lower_hint: Identity, upper_hint: Identity, asset_address: ContractId);

    #[storage(read, write)]
    fn offset(debt_to_offset: u64, coll_to_offset: u64, asset_address: ContractId);

    #[storage(read)]
    fn get_asset(asset_address: ContractId) -> u64;

    #[storage(read)]
    fn get_total_usdf_deposits() -> u64;

    #[storage(read)]
    fn get_depositor_asset_gain(depositor: Identity, asset_address: ContractId) -> u64;

    #[storage(read)]
    fn get_compounded_usdf_deposit(depositor: Identity) -> u64;
}
