library coll_surplus_pool_interface;

abi CollSurplusPool {
    #[storage(read, write)]
    fn initialize(borrow_operations: ContractId, protocol_manager: Identity);

    #[storage(read, write)]
    fn add_asset(asset: ContractId, trove_manager: Identity);

    #[storage(read, write)]
    fn claim_coll(acount: Identity, asset: ContractId);

    #[storage(read)]
    fn get_asset(asset: ContractId) -> u64;

    #[storage(read)]
    fn get_collateral(acount: Identity, asset: ContractId) -> u64;

    #[storage(read, write)]
    fn account_surplus(account: Identity, amount: u64, asset: ContractId);
}
