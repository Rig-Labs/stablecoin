library coll_surplus_pool_interface;

abi CollSurplusPool {
    #[storage(read, write)]
    fn initialize(trove_manager: Identity, active_pool: ContractId, borrow_operations: ContractId, asset_id: ContractId);

    #[storage(read, write)]
    fn claim_coll(acount: Identity, asset: ContractId);

    #[storage(read)]
    fn get_asset(asset: ContractId) -> u64;

    #[storage(read)]
    fn get_collateral(acount: Identity, asset: ContractId) -> u64;

    #[storage(read, write)]
    fn account_surplus(account: Identity, asset:ContractId, amount: u64);
}
