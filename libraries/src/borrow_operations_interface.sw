library borrow_operations_interface;

abi BorrowOperations {
    #[storage(read, write)]
    fn initialize(trove_manager_contract: ContractId, sorted_troves_contract: ContractId, oracle_contract: ContractId, asset_contract: ContractId, usdf_contract: ContractId, fpt_staking_contract: ContractId);

    #[storage(read, write)]
    fn open_trove(max_fee: u64, usdf_amount: u64, upper_hint: Identity, lower_hint: Identity);

    #[storage(read, write)]
    fn add_coll(upper_hint: Identity, lower_hint: Identity);

    #[storage(read, write)]
    fn move_asset_gain_to_trove(id: Identity, upper_hint: Identity, lower_hint: Identity);

    #[storage(read, write)]
    fn withdraw_coll(amount: u64, upper_hint: Identity, lower_hint: Identity);

    #[storage(read, write)]
    fn withdraw_usdf(max_fee: u64, amount: u64, upper_hint: Identity, lower_hint: Identity);

    #[storage(read, write)]
    fn repay_usdf(amount: u64, upper_hint: Identity, lower_hint: Identity);

    #[storage(read, write)]
    fn close_trove();

    #[storage(read, write)]
    fn adjust_trove(max_fee: u64, coll_withdrawl: u64, debt_change: u64, is_debt_increase: bool, upper_hint: Identity, lower_hint: Identity);

    #[storage(read, write)]
    fn claim_collateral();

    #[storage(read, write)]
    fn get_composite_debt(id: Identity) -> u64;
}
