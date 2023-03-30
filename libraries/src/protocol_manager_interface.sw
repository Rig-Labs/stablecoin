library protocol_manager_interface;

abi ProtocolManager {
    #[storage(read, write)]
    fn initialize(borrow_operations: ContractId, stability_pool: ContractId, usdf:ContractId, admin: Identity);

    #[storage(read, write)]
    fn register_asset(asset_address: ContractId, active_pool: ContractId, trove_manager: ContractId, coll_surplus_pool: ContractId, oracle: ContractId, sorted_troves: ContractId);

    #[storage(read, write)]
    fn renounce_admin();
}