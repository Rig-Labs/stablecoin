contract;

dep data_structures;
use data_structures::ReadStorage;
use libraries::numbers::*;
use libraries::fluid_math::{
    DECIMAL_PRECISION,
    fm_min,
    fm_multiply_ratio,
    null_contract,
    null_identity_address,
};
use libraries::fpt_staking_interface::{FPTStaking};
use std::{
    auth::msg_sender,
    call_frames::{
        msg_asset_id,
    },
    context::{
        msg_amount,
    },
    logging::log,
    storage::{
        StorageMap,
        StorageVec,
    },
    token::transfer,
    u128::U128,
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
    borrower_operations_address: ContractId = null_contract(),
    fpt_address: ContractId = null_contract(),
    usdf_address: ContractId = null_contract(),
    is_initialized: bool = false,
}

impl FPTStaking for Contract {
    #[storage(read, write)]
    fn initialize(
        protocol_manager_address: ContractId,
        borrower_operations_address: ContractId,
        fpt_address: ContractId,
        usdf_address: ContractId,
    ) {
        require(storage.is_initialized == false, "Contract is already initialized");
        storage.protocol_manager_address = protocol_manager_address;
        storage.borrower_operations_address = borrower_operations_address;
        storage.fpt_address = fpt_address;
        storage.usdf_address = usdf_address;
        storage.is_initialized = true;
    }

    #[storage(read)]
    fn get_storage() -> ReadStorage {
        ReadStorage {
            f_usdf: storage.f_usdf,
            total_fpt_staked: storage.total_fpt_staked,
            protocol_manager_address: storage.protocol_manager_address,
            borrower_operations_address: storage.borrower_operations_address,
            fpt_address: storage.fpt_address,
            usdf_address: storage.usdf_address,
            is_initialized: storage.is_initialized,
        }
    }

    #[storage(read, write), payable]
    fn stake() {
        let id = msg_sender().unwrap();

        require_fpt_is_valid_and_non_zero();

        let amount = msg_amount();

        let current_stake = storage.stakes.get(id);

        if (current_stake != 0) {
            let usdf_gain = internal_get_pending_usdf_gain(id);
            internal_send_usdf_gain_to_user(usdf_gain);

            internal_send_asset_gain_to_user(id);
        }

        update_user_snapshots(id);

        let new_stake = current_stake + amount;
        storage.stakes.insert(id, new_stake); //overwrite previous balance
        storage.total_fpt_staked += amount;
    }

    #[storage(read, write)]
    fn unstake(amount: u64) {
        let id = msg_sender().unwrap();

        let current_stake = storage.stakes.get(id);
        require_user_has_stake(current_stake, amount);

        let usdf_gain = internal_get_pending_usdf_gain(id);
        internal_send_usdf_gain_to_user(usdf_gain);
        internal_send_asset_gain_to_user(id);

        update_user_snapshots(id);

        if (amount > 0) {
            let amount_to_withdraw = fm_min(amount, current_stake);
            let new_stake = current_stake - amount_to_withdraw;
            storage.stakes.insert(id, new_stake); //overwrite previous balance
            storage.total_fpt_staked -= amount_to_withdraw;

            if (amount_to_withdraw > 0) {
                // transfer the FPT tokens to the user
                transfer(amount_to_withdraw, storage.fpt_address, msg_sender().unwrap());
            }
        }
    }

    // called from the protocol manager contract in the `register_asset` fn
    #[storage(read, write)]
    fn add_asset(asset_address: ContractId) {
        require_is_protocol_manager();
        storage.valid_assets.push(asset_address);
        storage.f_asset.insert(asset_address, 0);
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
    fn increase_f_usdf(usdf_fee_amount: u64) {
        require_is_borrower_operations();
        if (storage.total_fpt_staked > 0) {
            let usdf_fee_per_fpt_staked = fm_multiply_ratio(usdf_fee_amount, DECIMAL_PRECISION, storage.total_fpt_staked);
            storage.f_usdf += usdf_fee_per_fpt_staked;
        }
    }

    #[storage(read, write)]
    fn increase_f_asset(asset_fee_amount: u64, asset_address: ContractId) {
        require_is_protocol_manager(); // we have redeem function in protocol manager, not trove manager in liquity
        if (storage.total_fpt_staked > 0) {
            let asset_fee_per_fpt_staked = fm_multiply_ratio(asset_fee_amount, DECIMAL_PRECISION, storage.total_fpt_staked);
            let mut new_f_asset = storage.f_asset.get(asset_address) + asset_fee_per_fpt_staked;
            storage.f_asset.insert(asset_address, new_f_asset);
        }
    }
}

#[storage(read)]
fn internal_get_pending_asset_gain(id: Identity, asset_address: ContractId) -> u64 {
    let f_asset_snapshot = storage.asset_snapshot.get((id, asset_address));
    let asset_gain = fm_multiply_ratio(storage.stakes.get(id), storage.f_asset.get(asset_address) - f_asset_snapshot, DECIMAL_PRECISION);
    asset_gain
}

#[storage(read)]
fn internal_get_pending_usdf_gain(id: Identity) -> u64 {
    let f_usdf_snapshot = storage.usdf_snapshot.get(id);
    let usdf_gain = fm_multiply_ratio(storage.stakes.get(id), storage.f_usdf - f_usdf_snapshot, DECIMAL_PRECISION);
    usdf_gain
}

#[storage(read, write)]
fn update_user_snapshots(id: Identity) {
    storage.usdf_snapshot.insert(id, storage.f_usdf);

    let mut ind = 0;
    while ind < storage.valid_assets.len() {
        let current_asset_address = storage.valid_assets.get(ind).unwrap();
        let f_asset = storage.f_asset.get(current_asset_address);
        storage.asset_snapshot.insert((id, current_asset_address), f_asset);
        ind += 1;
    }
}

fn require_user_has_stake(current_stake_amount: u64, unstake_amount: u64) {
    require(current_stake_amount > 0, "User must have stake greater than 0");
    require(current_stake_amount >= unstake_amount, "Cannot unstake more than current staked amount");
}

#[storage(read)]
fn require_is_protocol_manager() {
    let protocol_manager = Identity::ContractId(storage.protocol_manager_address);
    require(msg_sender().unwrap() == protocol_manager, "Caller is not the protocol manager");
}

#[storage(read)]
fn require_is_borrower_operations() {
    let borrower_operations = Identity::ContractId(storage.borrower_operations_address);
    require(msg_sender().unwrap() == borrower_operations, "Caller is not the Borrower Operations");
}

#[storage(read)]
fn require_fpt_is_valid_and_non_zero() {
    require(storage.fpt_address == msg_asset_id(), "FPT contract not initialized, or wrong token");
    require(msg_amount() > 0, "FPT amount must be greater than 0");
}

#[storage(read)]
fn internal_send_asset_gain_to_user(id: Identity) {
    // when fuel adds a .contains or .indexOf for StorageVec, double check asset address is in valid_assets here
    let mut ind = 0;
    while ind < storage.valid_assets.len() {
        let current_asset_address = storage.valid_assets.get(ind).unwrap();
        let asset_gain = internal_get_pending_asset_gain(id, current_asset_address);
        if (asset_gain > 0) {
            transfer(asset_gain, current_asset_address, msg_sender().unwrap());
        }
        ind += 1;
    }
}

#[storage(read)]
fn internal_send_usdf_gain_to_user(amount: u64) {
    if (amount > 0) {
        transfer(amount, storage.usdf_address, msg_sender().unwrap());
    }
}
