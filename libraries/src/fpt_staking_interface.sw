library fpt_staking_interface;
use std::{
    storage::{
        StorageMap,
        StorageVec,
    },
};

pub struct ReadStorage {
    f_usdf: u64, 
    total_fpt_staked: u64,
    protocol_manager_address: ContractId,
    trove_manager_address: ContractId,
    borrower_operations_address: ContractId,
    fpt_address: ContractId,
    usdf_address: ContractId,
    is_initialized: bool,
}

abi FPTStaking {
    #[storage(read, write), payable]
    fn stake();

    #[storage(read, write)]
    fn unstake(amount: u64);
    
    #[storage(read, write)]
    fn add_asset(
        asset_address: ContractId,
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
    fn get_storage() -> ReadStorage;


    #[storage(read)]
    fn get_pending_asset_gain(id: Identity, asset_address: ContractId) -> u64;

    #[storage(read)]
    fn get_pending_usdf_gain(id: Identity) -> u64;

    #[storage(read, write)]
    fn increase_f_usdf(usdf_fee_amount: u64);

    #[storage(read, write)]
    fn increase_f_asset(asset_fee_amount: u64, asset_address: ContractId);

}