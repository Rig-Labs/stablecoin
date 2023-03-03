contract;

dep data_structures;
use data_structures::{Snapshots};

use libraries::stability_pool_interface::{StabilityPool};

use std::{
    auth::msg_sender,
    call_frames::{
        contract_id,
        msg_asset_id,
    },
    context::{
        msg_amount,
    },
    logging::log,
    token::transfer,
};

const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
const SCALE_FACTOR = 1_000_000_000;
const DECIMAL_PRECISION = 1_000_000;

storage {
    asset: u64 = 0,
    total_usdf_deposits: u64 = 0,
    deposits: StorageMap<Identity, u64> = StorageMap {},
    deposit_snapshots: StorageMap<Identity, Snapshots> = StorageMap {},
    current_scale: u64 = 0,
    current_epoch: u64 = 0,
    epoch_to_scale_to_sum: StorageMap<(u64, u64), u64> = StorageMap {},
    epoch_to_scale_to_gain: StorageMap<(u64, u64), u64> = StorageMap {},
    last_fpt_error: u64 = 0,
    last_asset_error_offset: u64 = 0,
    last_usdf_error_offset: u64 = 0,
    borrow_operations_address: ContractId = ContractId::from(ZERO_B256),
    trove_manager_address: ContractId = ContractId::from(ZERO_B256),
    active_pool_address: ContractId = ContractId::from(ZERO_B256),
    usdf_address: ContractId = ContractId::from(ZERO_B256),
    sorted_troves_address: ContractId = ContractId::from(ZERO_B256),
    oracle_address: ContractId = ContractId::from(ZERO_B256),
    community_issuance_address: ContractId = ContractId::from(ZERO_B256),
    asset_address: ContractId = ContractId::from(ZERO_B256),
    p: u64 = DECIMAL_PRECISION,
}

impl StabilityPool for Contract {
    #[storage(read, write)]
    fn initialize(
        borrow_operations_address: ContractId,
        trove_manager_address: ContractId,
        active_pool_address: ContractId,
        usdf_address: ContractId,
        sorted_troves_address: ContractId,
        oracle_address: ContractId,
        community_issuance_address: ContractId,
        asset_address: ContractId,
    ) {
        require(storage.borrow_operations_address == ContractId::from(ZERO_B256), "Already initialized");

        storage.borrow_operations_address = borrow_operations_address;
        storage.trove_manager_address = trove_manager_address;
        storage.active_pool_address = active_pool_address;
        storage.usdf_address = usdf_address;
        storage.sorted_troves_address = sorted_troves_address;
        storage.oracle_address = oracle_address;
        storage.community_issuance_address = community_issuance_address;
        storage.asset_address = asset_address;
    }

    #[storage(read, write)]
    fn provide_to_stability_pool() {
        require_usdf_is_valid_and_non_zero();

        let initial_deposit = storage.deposits.get(msg_sender().unwrap());
        // TODO Trigger FPT issuance
        let depositor_asset_gain = get_depositor_asset_gain(msg_sender().unwrap());
        let compounded_usdf_deposit = get_compounded_usdf_deposit(msg_sender().unwrap());
        let usdf_loss = initial_deposit - compounded_usdf_deposit;

        let new_posit = compounded_usdf_deposit + msg_amount();
    }

    #[storage(read, write)]
    fn withdraw_from_stability_pool(amount: u64) {}

    #[storage(read, write)]
    fn withdraw_gain_to_trove(lower_hint: Identity, upper_hint: Identity) {}

    #[storage(read, write)]
    fn offset(debt: u64, coll: u64) {}

    #[storage(read)]
    fn get_asset() -> u64 {
        return 0
    }

    #[storage(read)]
    fn get_total_usdf_deposits() -> u64 {
        return 0
    }
}

#[storage(read)]
fn require_usdf_is_valid_and_non_zero() {
    require(storage.usdf_address == msg_asset_id(), "USDF contract not initialized");
    require(msg_amount() > 0, "USDF amount must be greater than 0");
}

#[storage(read)]
fn get_depositor_asset_gain(depositor: Identity) -> u64 {
    let initial_deposit = storage.deposits.get(depositor);

    if initial_deposit == 0 {
        return 0;
    }

    let mut snapshots = storage.deposit_snapshots.get(depositor);

    return get_asset_gain_from_snapshots(initial_deposit, snapshots)
}

#[storage(read)]
fn get_compounded_usdf_deposit(depositor: Identity) -> u64 {
    let initial_deposit = storage.deposits.get(depositor);

    if initial_deposit == 0 {
        return 0;
    }

    let mut snapshots = storage.deposit_snapshots.get(depositor);

    return 0;
}

#[storage(read)]
fn get_asset_gain_from_snapshots(initial_deposit: u64, snapshots: Snapshots) -> u64 {
    let epoch_snapshot = snapshots.epoch;
    let scale_snapshot = snapshots.scale;
    let s_snapshot = snapshots.S;
    let p_snapshot = snapshots.P;

    let first_portion = storage.epoch_to_scale_to_sum.get((epoch_snapshot, scale_snapshot));
    let second_portion = storage.epoch_to_scale_to_gain.get((epoch_snapshot, scale_snapshot + 1)) / SCALE_FACTOR;

    let gain: u64 = (initial_deposit * (first_portion + second_portion)) / p_snapshot / DECIMAL_PRECISION;

    return gain
}

#[storage(read)]
fn get_compounded_stake_from_snapshots(initial_stake: u64, snapshots: Snapshots) -> u64 {
    let epoch_snapshot = snapshots.epoch;
    let scale_snapshot = snapshots.scale;
    let p_snapshot = snapshots.P;

    if (epoch_snapshot == storage.current_epoch) {
        return 0;
    }

    let mut compounded_stake = 0;
    let scale_diff = storage.current_scale - scale_snapshot;

    if (scale_diff == 0) {
        compounded_stake = initial_stake * storage.p / p_snapshot;
    } else if (scale_diff == 1) {
        compounded_stake = initial_stake * storage.p / p_snapshot / SCALE_FACTOR;
    } else {
        compounded_stake = 0;
    }

    if (compounded_stake < initial_stake / DECIMAL_PRECISION) {
        return 0;
    }

    return compounded_stake;
}

#[storage(read, write)]
fn internal_update_deposits_and_snapshots(depositor: Identity, amount: u64) {
    storage.deposits.insert(depositor, amount);

    if (amount == 0) {
        // TODO use storage remove when available
        storage.deposit_snapshots.insert(depositor, Snapshots::default());
    }

    let current_epoch = storage.current_epoch;
    let current_scale = storage.current_scale;
    let current_p = storage.p;

    let current_s = storage.epoch_to_scale_to_sum.get((current_epoch, current_scale));
    let current_g = storage.epoch_to_scale_to_gain.get((current_epoch, current_scale));

    let snapshots = Snapshots {
        epoch: current_epoch,
        scale: current_scale,
        S: current_s,
        P: current_p,
        G: current_g,
    };

    storage.deposit_snapshots.insert(depositor, snapshots);
}
