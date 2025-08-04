library;

abi BorrowOperations {
    #[storage(read, write)]
    fn initialize(
        usdm_contract: ContractId,
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

    #[storage(read, write)]
    fn set_pauser(pauser: Identity);

    #[storage(read, write)]
    fn transfer_owner(new_owner: Identity);

    #[storage(read, write)]
    fn renounce_owner();

    #[storage(read), payable]
    fn open_trove(usdm_amount: u64, upper_hint: Identity, lower_hint: Identity);

    #[storage(read, write), payable]
    fn add_coll(upper_hint: Identity, lower_hint: Identity);

    #[storage(read, write)]
    fn withdraw_coll(
        amount: u64,
        upper_hint: Identity,
        lower_hint: Identity,
        asset: AssetId,
    );

    #[storage(read, write)]
    fn withdraw_usdm(
        amount: u64,
        upper_hint: Identity,
        lower_hint: Identity,
        asset: AssetId,
    );

    #[storage(read, write), payable]
    fn repay_usdm(upper_hint: Identity, lower_hint: Identity, asset: AssetId);

    #[storage(read, write), payable]
    fn close_trove(asset: AssetId);

    #[storage(read)]
    fn claim_collateral(asset: AssetId);

    #[storage(read)]
    fn get_usdm_asset_id() -> AssetId;

    #[storage(read, write)]
    fn set_pause_status(is_paused: bool);

    #[storage(read)]
    fn get_pauser() -> Identity;

    #[storage(read)]
    fn get_is_paused() -> bool;
}
