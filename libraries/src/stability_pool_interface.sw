library stability_pool_interface;

abi StabilityPool {
    #[storage(read, write)]
    fn initialize(borrow_operations_address: ContractId, trove_manager_address: ContractId, active_pool_address: ContractId, usdf_address: ContractId, sorted_troves_address: ContractId, oracle_address: ContractId, community_issuance_address: ContractId, asset_address: ContractId);

    #[storage(read, write)]
    fn provide_to_stability_pool();

    #[storage(read, write)]
    fn withdraw_from_stability_pool(amount: u64);

    #[storage(read, write)]
    fn withdraw_gain_to_trove(lower_hint: Identity, upper_hint: Identity);

    #[storage(read, write)]
    fn offset(debt: u64, coll: u64);

    #[storage(read)]
    fn get_asset() -> u64;

    #[storage(read)]
    fn get_total_usdf_deposits() -> u64;

    #[storage(read)]
    fn get_depositor_asset_gain(depositor: Identity) -> u64;

    #[storage(read)]
    fn get_compounded_usdf_deposit(depositor: Identity) -> u64;
}
