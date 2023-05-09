contract;

use libraries::numbers::*;
use libraries::fluid_math::{ null_contract, null_identity_address };
use libraries::fpt_staking_interface::{FPTStaking};
use std::{
    auth::msg_sender,
    call_frames::{
        msg_asset_id,
    },
    storage::{
        StorageMap,
        StorageVec,
    },
    u128::U128,
    token::transfer,
    context::{
        msg_amount,
    },
};

storage {
    valid_assets: StorageVec<ContractId> = StorageVec {},
    stakes: StorageMap<Identity, u64> = StorageMap {}, 
    usdf_snapshot: StorageMap<Identity, u64> = StorageMap {}, 
    asset_snapshot: StorageMap<(Identity, ContractId), u64> = StorageMap {}, 
    f_asset: StorageMap<ContractId, u64> = StorageMap {}, 
    f_usdf: u64 = 0, 
    total_fpt_staked: u64 = 0,
    protocol_manager_address: ContractId = null_contract(),
    trove_manager_address: ContractId = null_contract(),
    borrower_operations_address: ContractId = null_contract(),
    fpt_address: ContractId = null_contract(),
    usdf_address: ContractId = null_contract(),
    is_initialized: bool = false,
}

const DECIMAL_PRECISION: U128 = U128::from_u64(1); //todo: import from fluidmath once we switch this to u128

impl FPTStaking for Contract {

    #[storage(read, write)]
    fn initialize(
        protocol_manager_address: ContractId,
        trove_manager_address: ContractId,
        borrower_operations_address: ContractId,
        fpt_address: ContractId,
        usdf_address: ContractId,
    ) {
        require(storage.is_initialized == false, "Contract is already initialized");
        storage.protocol_manager_address = protocol_manager_address;
        storage.trove_manager_address = trove_manager_address;
        storage.borrower_operations_address = borrower_operations_address;
        storage.fpt_address = fpt_address;
        storage.usdf_address = usdf_address;
        storage.is_initialized = true;
    }

    #[storage(read, write)]
    fn stake(id: Identity) {

        require_fpt_is_valid_and_non_zero();

        let amount = msg_amount();

        require_non_zero(amount);
        let current_stake = storage.stakes.get(id);

        let mut usdf_gain = 0;
        if (current_stake != 0) {
            usdf_gain = internal_get_pending_usdf_gain(id);
            internal_send_usdf_gain_to_user(usdf_gain);
        }

        let mut x = 0;
        while x < storage.valid_assets.len(){
            let mut asset_gain = 0;
            let current_asset_address = storage.valid_assets.get(x).unwrap();
            if (current_stake != 0) {
                asset_gain = internal_get_pending_asset_gain(id, current_asset_address);
                internal_send_asset_gain_to_user(asset_gain, current_asset_address);
            }
            x += 1;
        }

        update_user_snapshots(id);

        let new_stake = current_stake + amount;
        storage.stakes.insert(id, new_stake); //overwrite previous balance
        storage.total_fpt_staked = storage.total_fpt_staked + amount;
    }

    #[storage(read, write)]
    fn unstake(id: Identity, amount: u64) {
        let current_stake = storage.stakes.get(id);
        require_user_has_stake(current_stake);

        let usdf_gain = internal_get_pending_usdf_gain(id);
        internal_send_usdf_gain_to_user(usdf_gain);

        let mut x = 0;
        while x < storage.valid_assets.len(){
            let mut asset_gain = 0;
            let current_asset_address = storage.valid_assets.get(x).unwrap();
            asset_gain = internal_get_pending_asset_gain(id, current_asset_address);
            internal_send_asset_gain_to_user(asset_gain, current_asset_address);
            x += 1;
        }

        update_user_snapshots(id);

        if (amount > 0){
            let amount_to_withdraw = min(amount, current_stake);
            let new_stake = current_stake - amount_to_withdraw;
            storage.stakes.insert(id, new_stake); //overwrite previous balance
            storage.total_fpt_staked = storage.total_fpt_staked - amount_to_withdraw;

            // here we need to actually transfer the FPT tokens to the user
            transfer(amount_to_withdraw, storage.fpt_address, msg_sender().unwrap());
        }
    }

    // later we need to call this from the protocol manager contract in the `register_asset` fn
    #[storage(read, write)]
    fn add_asset(
        trove_manager_address: ContractId,
        active_pool_address: ContractId,
        sorted_troves_address: ContractId,
        asset_address: ContractId,
        oracle_address: ContractId,
    ) {
        require_is_protocol_manager();
        storage.valid_assets.push(asset_address);

        //todo, determine if this is 128 or 64
        //storage.f_asset.insert(asset_address, U128::from_u64(0));
    }

    #[storage(read)]
    fn get_pending_asset_gain(id: Identity, asset_address: ContractId) -> u64 {
        internal_get_pending_asset_gain(id, asset_address)
    }

    #[storage(read)]
    fn get_pending_usdf_gain(id: Identity) -> u64 {
        internal_get_pending_usdf_gain(id)
    }

    #[storage(read, write)]
    fn increase_f_usdf(usdf_fee_amount: u64){
        require_is_borrower_operations();
        let mut usdf_fee_per_fpt_staked = 0;
        if (storage.total_fpt_staked > 0){
            usdf_fee_per_fpt_staked = ((U128::from_u64(usdf_fee_amount) * DECIMAL_PRECISION) / U128::from_u64(storage.total_fpt_staked)).as_u64().unwrap();
        }
        storage.f_usdf = storage.f_usdf + usdf_fee_per_fpt_staked;
    }

    #[storage(read, write)]
    fn increase_f_asset(asset_fee_amount: u64, asset_address: ContractId){
        require_is_trove_manager();
        let mut asset_fee_per_fpt_staked =0;
        if (storage.total_fpt_staked > 0){
            asset_fee_per_fpt_staked = ((U128::from_u64(asset_fee_amount) * DECIMAL_PRECISION) / U128::from_u64(storage.total_fpt_staked)).as_u64().unwrap();
        }
        let mut new_f_asset = storage.f_asset.get(asset_address) + asset_fee_per_fpt_staked;
        storage.f_asset.insert(asset_address, new_f_asset);
    }
}


#[storage(read)]
fn internal_get_pending_asset_gain(id: Identity, asset_address: ContractId) -> u64 {
    let f_asset_snapshot: U128 = U128::from_u64(storage.asset_snapshot.get((id, asset_address)));
    let asset_gain = ((U128::from_u64(storage.stakes.get(id)) * (U128::from_u64(storage.f_asset.get(asset_address)) - f_asset_snapshot)) / DECIMAL_PRECISION).as_u64().unwrap();
    asset_gain
}

#[storage(read)]
fn internal_get_pending_usdf_gain(id: Identity) -> u64 {
    let f_usdf_snapshot: U128  = U128::from_u64(storage.usdf_snapshot.get(id));
    let usdf_gain = ((U128::from_u64(storage.stakes.get(id)) * (U128::from_u64(storage.f_usdf) - f_usdf_snapshot)) / DECIMAL_PRECISION).as_u64().unwrap();
    usdf_gain
}

#[storage(read, write)]
fn update_user_snapshots(id: Identity) {

    storage.usdf_snapshot.insert(id, storage.f_usdf);
 
    let mut x = 0;
    while x < storage.valid_assets.len(){
        let current_asset_address = storage.valid_assets.get(x).unwrap();
        let f_asset = storage.f_asset.get(current_asset_address);
        storage.asset_snapshot.insert((id, current_asset_address), f_asset);
        x += 1;
    }

}

fn require_non_zero(amount: u64) {
    require(amount > 0, "FPT Amount must be greater than 0");
}

fn require_user_has_stake(amount: u64) {
    require(amount > 0, "User must have stake greater than 0");
}

#[storage(read)]
fn require_is_protocol_manager() {
    let protocol_manager = Identity::ContractId(storage.protocol_manager_address);
    require(msg_sender().unwrap() == protocol_manager, "Caller is not the protocol manager");
}

#[storage(read)]
fn require_is_trove_manager() {
    let trove_manager = Identity::ContractId(storage.trove_manager_address);
    require(msg_sender().unwrap() == trove_manager, "Caller is not the trove manager");
}

#[storage(read)]
fn require_is_borrower_operations() {
    let borrower_operations = Identity::ContractId(storage.borrower_operations_address);
    require(msg_sender().unwrap() == borrower_operations, "Caller is not the Borrowe Operations");
}

#[storage(read)]
fn require_fpt_is_valid_and_non_zero() {
    require(storage.fpt_address == msg_asset_id(), "FPT contract not initialized, or wrong token");
    require(msg_amount() > 0, "FPT amount must be greater than 0");
}

#[storage(read)]
fn internal_send_asset_gain_to_user(amount: u64, asset_address: ContractId) {
    //when they add a .contains or .indexOf for StorageVec, double check asset address is in valid_assets here
    transfer(amount, asset_address, msg_sender().unwrap());
}

#[storage(read)]
fn internal_send_usdf_gain_to_user(amount: u64) {
    transfer(amount, storage.usdf_address, msg_sender().unwrap());
}

fn min(amount_one: u64, amount_two:u64) -> u64 {
    if (amount_one >= amount_two){
        amount_two
    } else {
        amount_one
    }
}
