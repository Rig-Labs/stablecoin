library fpt_staking_interface;

abi FPTStaking {
    #[storage(read, write)]
    fn stake(id: Identity);

    #[storage(read, write)]
    fn unstake(id: Identity, amount: u64);
    
    #[storage(read, write)]
    fn add_asset(
        trove_manager_address: ContractId,
        active_pool_address: ContractId,
        sorted_troves_address: ContractId,
        asset_address: ContractId,
        oracle_address: ContractId,
    );

     #[storage(read, write)]
    fn initialize(
        protocol_manager: ContractId,
        trove_manager_address: ContractId,
        borrower_operations_address: ContractId,
        fpt_address: ContractId,
        usdf_address: ContractId,
    );

    #[storage(read)]
    fn get_pending_asset_gain(id: Identity, asset_address: ContractId) -> u64;

    #[storage(read)]
    fn get_pending_usdf_gain(id: Identity) -> u64;

    #[storage(read, write)]
    fn increase_f_usdf(usdf_fee_amount: u64);

    #[storage(read, write)]
    fn increase_f_asset(asset_fee_amount: u64, asset_address: ContractId);

}