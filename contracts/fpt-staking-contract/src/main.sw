contract;

//dep data_structures;

use libraries::numbers::*;
use libraries::fluid_math::{ null_contract };
use std::{
    auth::msg_sender,
    storage::{
        StorageMap,
        StorageVec,
    },
    u128::U128,
};


//todo: switch anything that involves multiplication to U128
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
    is_initialized: bool = false,
}

const DECIMAL_PRECISION = 1; //temporary until we figure this out!
// use shared library decimal precision var

// TODO Migrate this to follow the other contracts
// move ABI to libraries/src
abi FPTStaking {
    #[storage(read, write)]
    fn stake(id: Identity, amount: u64);

    #[storage(read)]
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
    );

    #[storage(read)]
    fn get_pending_asset_gain(id: Identity, asset_address: ContractId) -> u64 {};

    #[storage(read)]
    fn get_pending_usdf_gain(id: Identity) -> u64 {};

}

impl FPTStaking for Contract {

    #[storage(read, write)]
    fn initialize(
        protocol_manager: ContractId,
    ) {
        require(storage.is_initialized == false, "Contract is already initialized");
        storage.protocol_manager_address = protocol_manager;
        storage.is_initialized = true;
    }

    #[storage(read, write)]
    fn stake(id: Identity, amount: u64) {

        require_non_zero(amount);
        let current_stake = storage.stakes.get(id).unwrap();

        let mut usdf_gain = 0;
        if (current_stake != 0) {
            usdf_gain = get_pending_usdf_gain(id);
            internal_send_usdf_gain_to_user(usdf_gain);
        }

        let mut x = 0;
        while x < storage.valid_assets.length{
            let mut asset_gain = 0;
            let current_asset_address = storage.valid_assets[x];
            if (current_stake != 0) {
                asset_gain = get_pending_asset_gain(id, current_asset_address);
                internal_send_asset_gain_to_user(asset_gain, current_asset_address);
            }
        }

        update_user_snapshots(id);

        let new_stake = current_stake + amount;
        storage.stakes.insert(id, new_stake); //overwrite previous balance
        storage.total_fpt_staked = storage.total_fpt_staked + amount;
        //here we actually need to transfer the FPT tokens from the user to this contract

    }

    #[storage(read)]
    fn unstake(id: Identity, amount: u64) {
        let current_stake = storage.stakes.get(id).unwrap();
        require_user_has_stake(current_stake);

        let usdf_gain = get_pending_usdf_gain(id);
        internal_send_usdf_gain_to_user(usdf_gain);

        let mut x = 0;
        while x < storage.valid_assets.length{
            let current_asset_address = storage.valid_assets[x];
            asset_gain = get_pending_asset_gain(id, current_asset_address);
            internal_send_asset_gain_to_user(asset_gain, current_asset_address);
        }

        update_user_snapshots(id);

        if (amount > 0){
            let amount_to_withdraw = min(amount, current_stake);
            let new_stake = current_stake - amount_to_withdraw;
            storage.stakes.insert(id, new_stake); //overwrite previous balance
            storage.total_fpt_staked = storage.total_fpt_staked - amount_to_withdraw;

            // here we need to actually transfer the FPT tokens to the user
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
        require_is_trove_manager();
        let mut usdf_fee_per_fpt_staked;
        if (storage.total_fpt_staked > 0){
            usdf_fee_per_fpt_staked = (usdf_fee_amount * DECIMAL_PRECISION) / storage.total_fpt_staked;
        }
        storage.f_usdf = storage.f_usdf + usdf_fee_per_fpt_staked;
    }

    #[storage(read, write)]
    fn increase_f_asset(asset_fee_amount: u64, asset_address: ContractId){
        require_is_trove_manager();
        let mut asset_fee_per_fpt_staked;
        if (storage.total_fpt_staked > 0){
            asset_fee_per_fpt_staked = (asset_fee_amount * DECIMAL_PRECISION) / storage.total_fpt_staked;
        }
        let mut new_f_asset = storage.f_asset.get(asset_address).unwrap() + asset_fee_per_fpt_staked;
        storage.f_asset.insert(asset_address, new_f_asset);
    }
}


#[storage(read)]
fn internal_get_pending_asset_gain(id: Identity, asset_address: ContractId) -> u64 {
    let f_asset_snapshot = storage.asset_snapshot.get((id, asset_address)).unwrap();
    let asset_gain = (storage.stakes.get(id).unwrap() * (storage.f_asset.get(asset_address).unwrap() - f_asset_snapshot)) / DECIMAL_PRECISION;
    asset_gain
}

#[storage(read)]
fn internal_get_pending_usdf_gain(id: Identity) -> u64 {
    let f_usdf_snapshot = storage.usdf_snapshot.get(id).unwrap();
    let usdf_gain = (storage.stakes.get(id).unwrap() * (storage.f_usdf - f_usdf_snapshot)) / DECIMAL_PRECISION;
    usdf_gain
}

#[storage(read, write)]
fn update_user_snapshots(id: Identity) {

    storage.usdf_snapshot.insert(id, storage.f_usdf);
 
    let mut x = 0;
    while x < storage.valid_assets.length{
        let current_asset_address = storage.valid_assets[x];
        let f_asset = storage.f_asset.get(current_asset_address);
        storage.asset_snapshot.insert(id, current_asset_address, f_asset);
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
fn internal_send_asset_gain_to_user(amount: u64, asset_address: ContractId) {
 //todo
}

fn min(amount_one: u64, amount_two:u64) -> u64 {
    if (amount_one >= amount_two){
        amount_two
    } else {
        amount_one
    }
}

#[storage(read)]
fn internal_send_usdf_gain_to_user(amount: u64) {
// todo
}
