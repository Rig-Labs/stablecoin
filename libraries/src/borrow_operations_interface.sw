library borrow_operations_interface;

abi BorrowOperations {
    #[storage(read, write)]
    fn initialize(usdf_contract: ContractId, fpt_staking_contract: ContractId, stability_pool: ContractId);

    #[storage(read, write)]
    fn add_asset(asset_contract: ContractId, trove_manager_contract: ContractId, sorted_troves_contract: ContractId, oracle_contract: ContractId, active_pool: ContractId, coll_surplus_pool: ContractId);

    #[storage(read, write), payable]
    fn open_trove(usdf_amount: u64, upper_hint: Identity, lower_hint: Identity, asset: ContractId);

    #[storage(read, write), payable]
    fn add_coll(upper_hint: Identity, lower_hint: Identity, asset: ContractId);

    #[storage(read, write), payable]
    fn move_asset_gain_to_trove(id: Identity, upper_hint: Identity, lower_hint: Identity, asset: ContractId);

    #[storage(read, write)]
    fn withdraw_coll(amount: u64, upper_hint: Identity, lower_hint: Identity, asset: ContractId);

    #[storage(read, write)]
    fn withdraw_usdf(amount: u64, upper_hint: Identity, lower_hint: Identity, asset: ContractId);

    #[storage(read, write), payable]
    fn repay_usdf(upper_hint: Identity, lower_hint: Identity, asset: ContractId);

    #[storage(read, write), payable]
    fn close_trove(asset: ContractId);

    #[storage(read, write), payable]
    fn adjust_trove(coll_withdrawl: u64, debt_change: u64, is_debt_increase: bool, upper_hint: Identity, lower_hint: Identity, asset: ContractId);

    #[storage(read)]
    fn claim_collateral(asset: ContractId);

    #[storage(read)]
    fn get_composite_debt(id: Identity) -> u64;
}
