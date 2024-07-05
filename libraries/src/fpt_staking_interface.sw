library;

pub struct ReadStorage {
    pub f_usdf: u64,
    pub total_fpt_staked: u64,
    pub protocol_manager_address: ContractId,
    pub borrower_operations_address: ContractId,
    pub fpt_address: AssetId,
    pub usdf_address: AssetId,
    pub is_initialized: bool,
}

abi FPTStaking {
    #[storage(read, write), payable]
    fn stake();

    #[storage(read, write)]
    fn unstake(amount: u64);

    #[storage(read, write)]
    fn add_asset(asset_address: AssetId);

    #[storage(read, write)]
    fn initialize(
        protocol_manager: ContractId,
        borrower_operations_address: ContractId,
        fpt_address: AssetId,
        usdf_address: AssetId,
    );

    #[storage(read)]
    fn get_storage() -> ReadStorage;

    #[storage(read)]
    fn get_pending_asset_gain(id: Identity, asset_address: AssetId) -> u64;

    #[storage(read)]
    fn get_pending_usdf_gain(id: Identity) -> u64;

    #[storage(read, write)]
    fn increase_f_usdf(usdf_fee_amount: u64);

    #[storage(read, write)]
    fn increase_f_asset(asset_fee_amount: u64, asset_address: AssetId);
}
