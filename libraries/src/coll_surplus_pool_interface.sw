library coll_surplus_pool_interface;

abi CollSurplusPool {
    #[storage(read, write)]
    fn initialize(trove_manager: Identity, active_pool: ContractId, borrow_operations:ContractId, asset_id: ContractId);

    #[storage(read, write)]
    fn claim_coll(acount: Identity);

    #[storage(read)]
    fn get_asset() -> u64;

    #[storage(read)]
    fn get_collateral(acount: Identity) -> u64;

    #[storage(read,write)]
    fn account_surplus(account: Identity, amount:u64);
}
