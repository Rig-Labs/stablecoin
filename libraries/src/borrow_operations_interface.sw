library;

abi BorrowOperations {
    #[storage(read, write)]
    fn initialize(
        usdf_contract: ContractId,
        fpt_staking_contract: ContractId,
        protocol_manager: ContractId,
        coll_surplus_pool_contract: ContractId,
        active_pool: ContractId,
        sorted_troves_contract: ContractId,
    );

    #[storage(read, write)]
    fn add_asset(
        asset_contract: AssetId,
        trove_manager_contract: ContractId,
        oracle_contract: ContractId,
    );

    #[storage(read), payable]
    fn open_trove(usdf_amount: u64, upper_hint: Identity, lower_hint: Identity);

    #[storage(read,), payable]
    fn add_coll(upper_hint: Identity, lower_hint: Identity);

    #[storage(read)]
    fn withdraw_coll(
        amount: u64,
        upper_hint: Identity,
        lower_hint: Identity,
        asset: AssetId,
    );

    #[storage(read)]
    fn withdraw_usdf(
        amount: u64,
        upper_hint: Identity,
        lower_hint: Identity,
        asset: AssetId,
    );

    #[storage(read), payable]
    fn repay_usdf(upper_hint: Identity, lower_hint: Identity, asset: AssetId);

    #[storage(read), payable]
    fn close_trove(asset: AssetId);

    #[storage(read)]
    fn claim_collateral(asset: AssetId);

    #[storage(read)]
    fn get_usdf_asset_id() -> AssetId;
}
