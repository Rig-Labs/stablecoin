contract;

dep data_structures;
use data_structures::{Snapshots};

use libraries::stability_pool_interface::{StabilityPool};
use libraries::active_pool_interface::{ActivePool};
use libraries::numbers::*;

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
    u128::U128,
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
        let depositor_asset_gain = internal_get_depositor_asset_gain(msg_sender().unwrap());
        let compounded_usdf_deposit = internal_get_compounded_usdf_deposit(msg_sender().unwrap());
        let usdf_loss = initial_deposit - compounded_usdf_deposit;

        let new_position = compounded_usdf_deposit + msg_amount();
        internal_update_deposits_and_snapshots(msg_sender().unwrap(), new_position);
        storage.total_usdf_deposits += msg_amount();
        // Pay out FPT gains
        send_asset_gain_to_depositor(msg_sender().unwrap(), depositor_asset_gain);
    }

    #[storage(read, write)]
    fn withdraw_from_stability_pool(amount: u64) {
        let initial_deposit = storage.deposits.get(msg_sender().unwrap());

        require_user_has_initial_deposit(initial_deposit);
        // TODO Trigger FPT issuance
        let depositor_asset_gain = internal_get_depositor_asset_gain(msg_sender().unwrap());
        let compounded_usdf_deposit = internal_get_compounded_usdf_deposit(msg_sender().unwrap());
        let mut usdf_to_withdraw = amount;

        // TODO Change to min when available
        if amount >= compounded_usdf_deposit {
            usdf_to_withdraw = compounded_usdf_deposit;
        }

        let new_position = compounded_usdf_deposit - usdf_to_withdraw;

        internal_update_deposits_and_snapshots(msg_sender().unwrap(), new_position);
        send_usdf_to_depositor(msg_sender().unwrap(), usdf_to_withdraw);
        send_asset_gain_to_depositor(msg_sender().unwrap(), depositor_asset_gain);
    }

    #[storage(read, write)]
    fn withdraw_gain_to_trove(lower_hint: Identity, upper_hint: Identity) {}

    #[storage(read, write)]
    fn offset(debt_to_offset: u64, coll_to_offset: u64) {
        require_caller_is_trove_manager();
        let total_usdf = storage.total_usdf_deposits;

        if total_usdf == 0 || debt_to_offset == 0 {
            return;
        }

        let per_unit_staked_changes = compute_rewards_per_unit_staked(coll_to_offset, debt_to_offset, total_usdf);

        update_reward_sum_and_product(per_unit_staked_changes.0, per_unit_staked_changes.1);

        internal_move_offset_coll_and_debt(coll_to_offset, debt_to_offset);

    }

    #[storage(read)]
    fn get_asset() -> u64 {
        return storage.asset;
    }

    #[storage(read)]
    fn get_total_usdf_deposits() -> u64 {
        return storage.total_usdf_deposits;
    }

    #[storage(read)]
    fn get_depositor_asset_gain(depositor: Identity) -> u64 {
        return internal_get_depositor_asset_gain(depositor);
    }

    #[storage(read)]
    fn get_compounded_usdf_deposit(depositor: Identity) -> u64 {
        return internal_get_compounded_usdf_deposit(depositor);
    }
}

#[storage(read)]
fn require_usdf_is_valid_and_non_zero() {
    require(storage.usdf_address == msg_asset_id(), "USDF contract not initialized");
    require(msg_amount() > 0, "USDF amount must be greater than 0");
}

#[storage(read)]
fn internal_get_depositor_asset_gain(depositor: Identity) -> u64 {
    let initial_deposit = storage.deposits.get(depositor);

    if initial_deposit == 0 {
        return 0;
    }

    let mut snapshots = storage.deposit_snapshots.get(depositor);

    return get_asset_gain_from_snapshots(initial_deposit, snapshots)
}

#[storage(read)]
fn internal_get_compounded_usdf_deposit(depositor: Identity) -> u64 {
    let initial_deposit = storage.deposits.get(depositor);

    if initial_deposit == 0 {
        return 0;
    }

    let mut snapshots = storage.deposit_snapshots.get(depositor);

    return get_compounded_stake_from_snapshots(initial_deposit, snapshots)
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

    if (epoch_snapshot < storage.current_epoch) {
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

    if (compounded_stake < initial_stake / 1_000_000_000) {
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

#[storage(read, write)]
fn send_asset_gain_to_depositor(depositor: Identity, gain: u64) {
    if (gain == 0) {
        return;
    }
    storage.asset -= gain;
    let asset_address = storage.asset_address;

    transfer(gain, asset_address, depositor);
}

#[storage(read, write)]
fn send_usdf_to_depositor(depositor: Identity, amount: u64) {
    if (amount == 0) {
        return;
    }
    storage.total_usdf_deposits -= amount;
    let usdf_address = storage.usdf_address;
    transfer(amount, usdf_address, depositor);
}

#[storage(read, write)]
fn require_caller_is_trove_manager() {
    // TODO May have to generalize with multiple trove managers
    let trove_manager_address = Identity::ContractId(storage.trove_manager_address);
    require(msg_sender().unwrap() == trove_manager_address, "Caller is not the TroveManager");
}

fn require_user_has_initial_deposit(deposit: u64) {
    require(deposit > 0, "User has no initial deposit");
}

#[storage(read, write)]
fn compute_rewards_per_unit_staked(
    coll_to_add: u64,
    debt_to_offset: u64,
    total_usdf_deposits: u64,
) -> (u64, u64) {
    let asset_numerator: U128 = U128::from_u64(coll_to_add) * U128::from_u64(DECIMAL_PRECISION) + U128::from_u64(storage.last_asset_error_offset);

    require(debt_to_offset <= total_usdf_deposits, "Debt offset exceeds total USDF deposits");

    let mut usdf_loss_per_unit_staked: U128 = U128::from_u64(0);

    if (debt_to_offset == total_usdf_deposits) {
        usdf_loss_per_unit_staked = U128::from_u64(DECIMAL_PRECISION);
        storage.last_usdf_error_offset = 0;
    } else {
        let usdf_loss_per_unit_staked_numerator: U128 = U128::from_u64(debt_to_offset) * U128::from_u64(DECIMAL_PRECISION) - U128::from_u64(storage.last_usdf_error_offset);


        usdf_loss_per_unit_staked = usdf_loss_per_unit_staked_numerator / U128::from_u64(total_usdf_deposits) + U128::from_u64(1);

        let last_usdf_error_offset = (usdf_loss_per_unit_staked_numerator * U128::from_u64(total_usdf_deposits) - usdf_loss_per_unit_staked_numerator);


        // storage.last_usdf_error_offset = (usdf_loss_per_unit_staked_numerator * U128::from_u64(total_usdf_deposits) - usdf_loss_per_unit_staked_numerator).as_u64().unwrap();

    }

    let asset_gain_per_unit_staked = asset_numerator / U128::from_u64(total_usdf_deposits);


    // storage.last_asset_error_offset = (asset_numerator - (asset_gain_per_unit_staked * total_usdf_de4(posits)).as_u64().u)nwrap();
    return (
        asset_gain_per_unit_staked.as_u64().unwrap(),
        usdf_loss_per_unit_staked.as_u64().unwrap(),
    );
}

#[storage(read, write)]
fn update_reward_sum_and_product(
    asset_gain_per_unit_staked: u64,
    usdf_loss_per_unit_staked: u64,
) {
    let current_p = storage.p;
    let mut new_p: u64 = 0;

    let new_product_factor = DECIMAL_PRECISION - usdf_loss_per_unit_staked;
    let current_epoch = storage.current_epoch;
    let current_scale = storage.current_scale;

    let current_s = storage.epoch_to_scale_to_sum.get((current_epoch, current_scale));

    let marginal_asset_gain = asset_gain_per_unit_staked * current_p;
    let new_sum = current_s + marginal_asset_gain;

    storage.epoch_to_scale_to_sum.insert((current_epoch, current_scale), new_sum);

    if (new_product_factor == 0) {
        storage.current_epoch += 1;
        storage.current_scale = 0;
        new_p = DECIMAL_PRECISION;
    } else if (current_p * new_product_factor / DECIMAL_PRECISION < SCALE_FACTOR)
    {
        new_p = current_p * SCALE_FACTOR / DECIMAL_PRECISION;
        storage.current_scale += 1;
    } else {
        new_p = current_p * new_product_factor / DECIMAL_PRECISION;
    }

    require(new_p > 0, "New p is 0");

    storage.p = new_p;
}

#[storage(read, write)]
fn internal_move_offset_coll_and_debt(coll_to_add: u64, debt_to_offset: u64) {
    let active_pool_address = storage.active_pool_address;

    let active_pool = abi(ActivePool, active_pool_address.value);

    active_pool.decrease_usdf_debt(debt_to_offset);

    // TODO Burn the offset usdf debt    
    active_pool.send_asset(Identity::ContractId(contract_id()), coll_to_add);
}
