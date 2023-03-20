contract;

dep data_structures;
use data_structures::{Snapshots};

use libraries::data_structures::{Status};
use libraries::stability_pool_interface::{StabilityPool};
use libraries::usdf_token_interface::{USDFToken};
use libraries::active_pool_interface::{ActivePool};
use libraries::trove_manager_interface::{TroveManager};
use libraries::borrow_operations_interface::{BorrowOperations};
use libraries::numbers::*;
use libraries::fluid_math::{fm_min, null_contract, null_identity_address};

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

const SCALE_FACTOR = 1_000_000_000;
const DECIMAL_PRECISION = 1_000_000;

storage {
    asset: u64 = 0,
    total_usdf_deposits: u64 = 0,
    deposits: StorageMap<Identity, u64> = StorageMap {},
    deposit_snapshots: StorageMap<Identity, Snapshots> = StorageMap {},
    current_scale: u64 = 0,
    current_epoch: u64 = 0,
    epoch_to_scale_to_sum: StorageMap<(u64, u64), U128> = StorageMap {},
    epoch_to_scale_to_gain: StorageMap<(u64, u64), U128> = StorageMap {},
    last_fpt_error: U128 = U128::from_u64(0),
    last_asset_error_offset: U128 = U128::from_u64(0),
    last_usdf_error_offset: U128 = U128::from_u64(0),
    borrow_operations_address: ContractId = null_contract(),
    trove_manager_address: ContractId = null_contract(),
    active_pool_address: ContractId = null_contract(),
    usdf_address: ContractId = null_contract(),
    sorted_troves_address: ContractId = null_contract(),
    oracle_address: ContractId = null_contract(),
    community_issuance_address: ContractId = null_contract(),
    asset_address: ContractId = null_contract(),
    p: U128 = U128::from_u64(DECIMAL_PRECISION),
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
        require(storage.borrow_operations_address == null_contract(), "Already initialized");

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
        let usdf_to_withdraw = fm_min(amount, compounded_usdf_deposit);

        let new_position = compounded_usdf_deposit - usdf_to_withdraw;

        internal_update_deposits_and_snapshots(msg_sender().unwrap(), new_position);
        send_usdf_to_depositor(msg_sender().unwrap(), usdf_to_withdraw);
        send_asset_gain_to_depositor(msg_sender().unwrap(), depositor_asset_gain);
    }

    #[storage(read, write)]
    fn withdraw_gain_to_trove(lower_hint: Identity, upper_hint: Identity) {
        let sender = msg_sender().unwrap();
        let initial_deposit = storage.deposits.get(sender);
        let borrower_operations = abi(BorrowOperations, storage.borrow_operations_address.value);
        require_user_has_initial_deposit(initial_deposit);
        require_user_has_asset_gain(sender);
        require_user_has_trove(sender);

        // TODO Trigger FPT issuance
        let depositor_asset_gain = internal_get_depositor_asset_gain(sender);
        let compounded_usdf_deposit = internal_get_compounded_usdf_deposit(sender);

        let usdf_loss = initial_deposit - compounded_usdf_deposit;
        internal_update_deposits_and_snapshots(sender, compounded_usdf_deposit);

        storage.asset -= depositor_asset_gain;

        borrower_operations.move_asset_gain_to_trove {
            coins: depositor_asset_gain,
            asset_id: storage.asset_address.value,
        }(sender, lower_hint, upper_hint);
    }

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
fn require_user_has_trove(address: Identity) {
    let trove_manager = abi(TroveManager, storage.trove_manager_address.value);
    let status = trove_manager.get_trove_status(address);
    require(status == Status::Active, "User does not have an active trove");
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
    let second_portion = storage.epoch_to_scale_to_gain.get((epoch_snapshot, scale_snapshot + 1)) / U128::from_u64(SCALE_FACTOR);

    let gain = (U128::from_u64(initial_deposit) * (first_portion + second_portion)) / p_snapshot / U128::from_u64(DECIMAL_PRECISION);

    return gain.as_u64().unwrap();
}
#[storage(read)]
fn get_compounded_stake_from_snapshots(initial_stake: u64, snapshots: Snapshots) -> u64 {
    let epoch_snapshot = snapshots.epoch;
    let scale_snapshot = snapshots.scale;
    let p_snapshot = snapshots.P;

    if (epoch_snapshot < storage.current_epoch) {
        return 0;
    }

    let mut compounded_stake: U128 = U128::from_u64(0);
    let scale_diff = storage.current_scale - scale_snapshot;

    if (scale_diff == 0) {
        compounded_stake = U128::from_u64(initial_stake) * storage.p / p_snapshot;
    } else if (scale_diff == 1) {
        compounded_stake = U128::from_u64(initial_stake) * storage.p / p_snapshot / U128::from_u64(SCALE_FACTOR);
    } else {
        compounded_stake = U128::from_u64(0);
    }

    if (compounded_stake < U128::from_u64(initial_stake) / U128::from_u64(DECIMAL_PRECISION))
    {
        return 0;
    }

    return compounded_stake.as_u64().unwrap();
}

#[storage(read, write)]
fn internal_decrease_usdf(total_usdf_to_decrease: u64) {
    storage.total_usdf_deposits -= total_usdf_to_decrease;
}

#[storage(read, write)]
fn internal_increase_asset(total_asset_to_increase: u64) {
    storage.asset += total_asset_to_increase;
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

#[storage(read)]
fn require_user_has_asset_gain(depositor: Identity) {
    let gain = internal_get_depositor_asset_gain(depositor);
    require(gain > 0, "User has no asset gain");
}

#[storage(read)]
fn require_caller_is_trove_manager() {
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
) -> (U128, U128) {
    let asset_numerator: U128 = U128::from_u64(coll_to_add) * U128::from_u64(DECIMAL_PRECISION) + storage.last_asset_error_offset;

    require(debt_to_offset <= total_usdf_deposits, "Debt offset exceeds total USDF deposits");

    let mut usdf_loss_per_unit_staked: U128 = U128::from_u64(0);

    if (debt_to_offset == total_usdf_deposits) {
        usdf_loss_per_unit_staked = U128::from_u64(DECIMAL_PRECISION);
        storage.last_usdf_error_offset = U128::from_u64(0);
    } else {
        let usdf_loss_per_unit_staked_numerator: U128 = U128::from_u64(debt_to_offset) * U128::from_u64(DECIMAL_PRECISION) - storage.last_usdf_error_offset;
        usdf_loss_per_unit_staked = usdf_loss_per_unit_staked_numerator / U128::from_u64(total_usdf_deposits) + U128::from_u64(1);

        storage.last_usdf_error_offset = usdf_loss_per_unit_staked * U128::from_u64(total_usdf_deposits) - usdf_loss_per_unit_staked_numerator;
    }

    let asset_gain_per_unit_staked = asset_numerator / U128::from_u64(total_usdf_deposits);

    storage.last_asset_error_offset = asset_numerator - (asset_gain_per_unit_staked * U128::from_u64(total_usdf_deposits));

    return (asset_gain_per_unit_staked, usdf_loss_per_unit_staked);
}

#[storage(read, write)]
fn update_reward_sum_and_product(
    asset_gain_per_unit_staked: U128,
    usdf_loss_per_unit_staked: U128,
) {
    let current_p = storage.p;
    let mut new_p: U128 = U128::from_u64(0);
    let new_product_factor = U128::from_u64(DECIMAL_PRECISION) - usdf_loss_per_unit_staked;
    let current_epoch = storage.current_epoch;
    let current_scale = storage.current_scale;

    let current_s = storage.epoch_to_scale_to_sum.get((current_epoch, current_scale));

    let marginal_asset_gain: U128 = asset_gain_per_unit_staked * current_p;
    let new_sum = current_s + marginal_asset_gain;

    storage.epoch_to_scale_to_sum.insert((current_epoch, current_scale), new_sum);

    if (new_product_factor == U128::from_u64(0)) {
        storage.current_epoch += 1;
        storage.current_scale = 0;
        new_p = U128::from_u64(DECIMAL_PRECISION);
    } else if (current_p * new_product_factor / U128::from_u64(DECIMAL_PRECISION) < U128::from_u64(SCALE_FACTOR))
    {
        new_p = current_p * new_product_factor * U128::from_u64(SCALE_FACTOR) / U128::from_u64(DECIMAL_PRECISION);
        storage.current_scale += 1;
    } else {
        new_p = current_p * new_product_factor / U128::from_u64(DECIMAL_PRECISION);
    }
    require(new_p > U128::from_u64(0), "New p is 0");

    storage.p = new_p;
}

#[storage(read, write)]
fn internal_move_offset_coll_and_debt(coll_to_add: u64, debt_to_offset: u64) {
    let active_pool_address = storage.active_pool_address;

    let active_pool = abi(ActivePool, active_pool_address.value);
    let usdf_contract = abi(USDFToken, storage.usdf_address.value);
    internal_decrease_usdf(debt_to_offset);
    internal_increase_asset(coll_to_add);
    active_pool.decrease_usdf_debt(debt_to_offset);

    usdf_contract.burn {
        coins: debt_to_offset,
        asset_id: storage.usdf_address.value,
    }();

    active_pool.send_asset(Identity::ContractId(contract_id()), coll_to_add);
}
