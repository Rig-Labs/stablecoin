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

        let new_stake = current_stake + amount;
        storage.stakes.insert(id, new_stake); //overwrite previous balance
        storage.total_fpt_staked = storage.total_fpt_staked + amount;
        //here we actually need to transfer the FPT tokens from the user to this contract

        update_user_snapshots(id);
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

        //todo, determine if this is 126 or 64
        //storage.f_asset.insert(asset_address, U128::from_u64(0));
    }

}

// doesn't really look like in Sway we need to have a public wrapper of a private function for these 
#[storage(read)]
fn get_pending_asset_gain(id: Identity, asset_address: ContractId) -> u64 {
    let f_asset_snapshot = storage.asset_snapshot.get((id, asset_address)).unwrap();
    let asset_gain = (storage.stakes.get(id).unwrap() * (storage.f_asset.get(asset_address).unwrap() - f_asset_snapshot)) / DECIMAL_PRECISION;
    asset_gain
}

#[storage(read)]
fn get_pending_usdf_gain(id: Identity) -> u64 {
    let f_usdf_snapshot = storage.usdf_snapshot.get(id).unwrap();
    let usdf_gain = (storage.stakes.get(id).unwrap() * (storage.f_usdf - f_usdf_snapshot)) / DECIMAL_PRECISION;
    usdf_gain
}

#[storage(read, write)]
fn update_user_snapshots(id: Identity) {
    /*let mut user_snapshot = storage.snapshots.get(id);
    user_snapshot.f_usdf_snapshot = storage.f_usdf;
    user_snapshot.f_asset_snapshot = storage.f_asset;
    storage.snapshots.insert(id, user_snapshot);*/
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
fn internal_send_asset_gain_to_user(amount: u64, asset_address: ContractId) {

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

}
