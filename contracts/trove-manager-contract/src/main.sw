contract;

dep data_structures;
dep utils;

use utils::{add_liquidation_vals_to_totals, get_offset_and_redistribution_vals};
use data_structures::{
    EntireTroveDebtAndColl,
    LiquidationTotals,
    LiquidationValues,
    LocalVariablesLiquidationSequence,
    LocalVariablesOuterLiquidationFunction,
    RedemptionTotals,
    RewardSnapshot,
    SingleRedemptionValues,
    Trove,
};
use libraries::numbers::*;
use libraries::trove_manager_interface::{TroveManager};
use libraries::usdf_token_interface::{USDFToken};
use libraries::sorted_troves_interface::{SortedTroves};
use libraries::stability_pool_interface::{StabilityPool};
use libraries::default_pool_interface::{DefaultPool};
use libraries::active_pool_interface::{ActivePool};
use libraries::coll_surplus_pool_interface::{CollSurplusPool};
use libraries::{MockOracle};
use libraries::data_structures::{Status};
use libraries::fluid_math::*;

use std::{
    address::Address,
    auth::msg_sender,
    block::{
        height,
        timestamp,
    },
    call_frames::{
        contract_id,
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
    sorted_troves_contract: ContractId = null_contract(),
    borrow_operations_contract: ContractId = null_contract(),
    stability_pool_contract: ContractId = null_contract(),
    oracle_contract: ContractId = null_contract(),
    active_pool_contract: ContractId = null_contract(),
    default_pool_contract: ContractId = null_contract(),
    coll_surplus_pool_contract: ContractId = null_contract(),
    usdf_contract: ContractId = null_contract(),
    fpt_token: ContractId = null_contract(),
    fpt_staking_contract: ContractId = null_contract(),
    total_stakes: u64 = 0,
    total_stakes_snapshot: u64 = 0,
    total_collateral_snapshot: u64 = 0,
    l_asset: u64 = 0,
    l_usdf: u64 = 0,
    last_asset_error_redistribution: u64 = 0,
    last_usdf_error_redistribution: u64 = 0,
    nominal_icr: StorageMap<Identity, u64> = StorageMap {},
    troves: StorageMap<Identity, Trove> = StorageMap {},
    trove_owners: StorageVec<Identity> = StorageVec {},
    reward_snapshots: StorageMap<Identity, RewardSnapshot> = StorageMap {},
}

impl TroveManager for Contract {
    #[storage(read, write)]
    fn initialize(
        borrow_operations: ContractId,
        sorted_troves: ContractId,
        oracle: ContractId,
        stability_pool: ContractId,
        default_pool: ContractId,
        active_pool: ContractId,
        coll_surplus_pool: ContractId,
        usdf_contract: ContractId,
    ) {
        // TODO Require not already initialized
        storage.sorted_troves_contract = sorted_troves;
        storage.borrow_operations_contract = borrow_operations;
        storage.stability_pool_contract = stability_pool;
        storage.oracle_contract = oracle;
        storage.default_pool_contract = default_pool;
        storage.active_pool_contract = active_pool;
        storage.coll_surplus_pool_contract = coll_surplus_pool;
        storage.usdf_contract = usdf_contract;
    }

    #[storage(read)]
    fn get_nominal_icr(id: Identity) -> u64 {
        let trove = storage.troves.get(id);

        return fm_compute_nominal_cr(trove.coll, trove.debt);
    }

    #[storage(read, write)]
    fn set_nominal_icr_and_insert(
        id: Identity,
        value: u64,
        prev_id: Identity,
        next_id: Identity,
    ) {
        // TODO Remove this function 
        storage.nominal_icr.insert(id, value);
        let sorted_troves_contract = abi(SortedTroves, storage.sorted_troves_contract.value);
        let _ = internal_increase_trove_coll(id, value);
        let _ = internal_increase_trove_debt(id, 1);

        sorted_troves_contract.insert(id, fm_compute_nominal_cr(value, 1), prev_id, next_id);
    }

    #[storage(read, write)]
    fn remove(id: Identity) {
        // TODO Remove this function
        storage.nominal_icr.insert(id, 0);
        let sorted_troves_contract = abi(SortedTroves, storage.sorted_troves_contract.into());
        sorted_troves_contract.remove(id);
    }

    #[storage(read, write)]
    fn apply_pending_rewards(id: Identity) {
        require_caller_is_borrow_operations_contract();
        internal_apply_pending_rewards(id);
    }

    #[storage(read)]
    fn has_pending_rewards(id: Identity) -> bool {
        internal_has_pending_rewards(id)
    }

    #[storage(read, write)]
    fn redeem_collateral(
        max_itterations: u64,
        max_fee_percentage: u64,
        partial_redemption_hint: u64,
        upper_partial_hint: Identity,
        lower_partial_hint: Identity,
    ) {
        // TODO Require functions
        require_valid_usdf_id();
        require(msg_amount() > 0, "Redemption amount must be greater than 0");

        let mut totals: RedemptionTotals = RedemptionTotals::default();
        let oracle_contract = abi(MockOracle, storage.oracle_contract.into());
        let sorted_troves_contract = abi(SortedTroves, storage.sorted_troves_contract.into());
        let active_pool_contract = abi(ActivePool, storage.active_pool_contract.into());
        let usdf_contract = abi(USDFToken, storage.usdf_contract.into());

        totals.remaining_usdf = msg_amount();
        totals.price = oracle_contract.get_price();
        totals.total_usdf_supply_at_start = internal_get_entire_system_debt();

        let mut current_borrower = sorted_troves_contract.get_last();

        while (current_borrower != null_identity_address() && internal_get_current_icr(current_borrower, totals.price) < MCR) {
            let current_trove = sorted_troves_contract.get_prev(current_borrower);
        }

        let mut remaining_itterations = max_itterations;

        while (current_borrower != null_identity_address() && totals.remaining_usdf > 0 && remaining_itterations > 0) {
            remaining_itterations -= 1;

            let next_user_to_check = sorted_troves_contract.get_prev(current_borrower);
            internal_apply_pending_rewards(current_borrower);

            let single_redemption = internal_redeem_collateral_from_trove(current_borrower, totals.remaining_usdf, totals.price, partial_redemption_hint, upper_partial_hint, lower_partial_hint);

            if (single_redemption.cancelled_partial) {
                break;
            }

            totals.total_usdf_to_redeem += single_redemption.usdf_lot;
            totals.total_asset_drawn += single_redemption.asset_lot;

            totals.remaining_usdf -= single_redemption.usdf_lot;
            current_borrower = next_user_to_check;
        }

        require(totals.total_asset_drawn > 0, "No collateral to redeem");

        internal_update_base_rate_from_redemption(0, 0, 0);

        totals.asset_fee = internal_get_redemption_fee(totals.total_asset_drawn);
        // Consider spliting fee with person being redeemed from
        // TODO require user accepts fee
        // TODO active pool send fee to stakers
        // TODO lqty staking increase f_asset
        totals.asset_to_send_to_redeemer = totals.total_asset_drawn - totals.asset_fee;

        usdf_contract.burn {
            coins: totals.total_usdf_to_redeem,
            asset_id: storage.usdf_contract.value,
        }();

        active_pool_contract.decrease_usdf_debt(totals.total_usdf_to_redeem);
        active_pool_contract.send_asset(msg_sender().unwrap(), totals.asset_to_send_to_redeemer);
    }

    #[storage(read)]
    fn get_current_icr(id: Identity, price: u64) -> u64 {
        internal_get_current_icr(id, price)
    }

    #[storage(read)]
    fn get_entire_debt_and_coll(id: Identity) -> (u64, u64, u64, u64) {
        return (0, 0, 0, 0)
        // TODO
    }

    #[storage(read)]
    fn get_redemption_rate() -> u64 {
        // TODO
        return 0;
    }

    #[storage(read)]
    fn get_redemption_rate_with_decay() -> u64 {
        // TODO
        return 0;
    }

    #[storage(read)]
    fn get_borrowing_fee(debt: u64) -> u64 {
        // TODO
        return 0
    }

    #[storage(read, write)]
    fn decay_base_rate_from_borrowing() {}

        // TODO
    #[storage(read)]
    fn get_trove_stake(id: Identity) -> u64 {
        internal_get_trove_stake(id)
    }

    #[storage(read)]
    fn get_borrowing_fee_with_decay(debt: u64) -> u64 {
        // TODO
        return 0
    }

    #[storage(read)]
    fn get_borrowing_rate() -> u64 {
        // TODO
        return 0
    }

    #[storage(read)]
    fn get_borrowing_rate_with_decay() -> u64 {
        // TODO
        return 0
    }

    #[storage(read)]
    fn get_tcr() -> u64 {
        // TODO
        return 0
    }

    #[storage(read, write)]
    fn set_trove_status(id: Identity, status: Status) {
        require_caller_is_borrow_operations_contract();

        let mut trove = storage.troves.get(id);
        trove.status = status;
        storage.troves.insert(id, trove);
    }

    #[storage(read, write)]
    fn increase_trove_coll(id: Identity, coll: u64) -> u64 {
        require_caller_is_borrow_operations_contract();

        internal_increase_trove_coll(id, coll)
    }

    #[storage(read, write)]
    fn update_stake_and_total_stakes(id: Identity) -> u64 {
        require_caller_is_borrow_operations_contract();

        internal_update_stake_and_total_stakes(id)
    }

    #[storage(read, write)]
    fn increase_trove_debt(id: Identity, debt: u64) -> u64 {
        require_caller_is_borrow_operations_contract();

        internal_increase_trove_debt(id, debt)
    }

    #[storage(read, write)]
    fn decrease_trove_coll(id: Identity, value: u64) -> u64 {
        require_caller_is_borrow_operations_contract();

        internal_decrease_trove_coll(id, value)
    }

    #[storage(read, write)]
    fn decrease_trove_debt(id: Identity, value: u64) -> u64 {
        require_caller_is_borrow_operations_contract();

        internal_decrease_trove_debt(id, value)
    }

    #[storage(read, write)]
    fn add_trove_owner_to_array(id: Identity) -> u64 {
        require_caller_is_borrow_operations_contract();

        storage.trove_owners.push(id);
        let indx = storage.trove_owners.len() - 1;

        let mut trove = storage.troves.get(id);
        trove.array_index = indx;
        storage.troves.insert(id, trove);

        return indx;
    }

    #[storage(read)]
    fn get_trove_debt(id: Identity) -> u64 {
        let trove = storage.troves.get(id);

        return trove.debt;
    }
    #[storage(read)]
    fn get_trove_coll(id: Identity) -> u64 {
        let trove = storage.troves.get(id);

        return trove.coll;
    }

    #[storage(read, write)]
    fn close_trove(id: Identity) {
        require_caller_is_borrow_operations_contract();

        internal_close_trove(id, Status::ClosedByOwner);
    }

    #[storage(read, write)]
    fn remove_stake(id: Identity) {}

    #[storage(read)]
    fn get_trove_status(id: Identity) -> Status {
        let trove = storage.troves.get(id);

        return trove.status;
    }
    #[storage(read, write)]
    fn batch_liquidate_troves(borrowers: Vec<Identity>) {
        internal_batch_liquidate_troves(borrowers);
    }

    #[storage(read, write)]
    fn liquidate(id: Identity) {
        require_trove_is_active(id);

        let mut borrowers: Vec<Identity> = Vec::new();
        borrowers.push(id);

        internal_batch_liquidate_troves(borrowers);
    }

    #[storage(read, write)]
    fn liquidate_troves(num_troves: u64) {}

    #[storage(read, write)]
    fn update_trove_reward_snapshots(id: Identity) {
        require_caller_is_borrow_operations_contract();

        internal_update_trove_reward_snapshots(id);
    }

    #[storage(read)]
    fn get_pending_usdf_rewards(address: Identity) -> u64 {
        internal_get_pending_usdf_reward(address)
    }

    #[storage(read)]
    fn get_pending_asset_rewards(id: Identity) -> u64 {
        internal_get_pending_asset_reward(id)
    }
}

#[storage(read, write)]
fn internal_update_trove_reward_snapshots(id: Identity) {
    let mut reward_snapshot = storage.reward_snapshots.get(id);

    reward_snapshot.asset = storage.l_asset;
    reward_snapshot.usdf_debt = storage.l_usdf;

    storage.reward_snapshots.insert(id, reward_snapshot);
}

#[storage(read, write)]
fn internal_apply_pending_rewards(borrower: Identity) {
    if (internal_has_pending_rewards(borrower)) {
        let pending_asset = internal_get_pending_asset_reward(borrower);
        let pending_usdf = internal_get_pending_usdf_reward(borrower);

        let mut trove = storage.troves.get(borrower);
        trove.coll += pending_asset;
        trove.debt += pending_usdf;
        storage.troves.insert(borrower, trove);

        internal_update_trove_reward_snapshots(borrower);
        internal_pending_trove_rewards_to_active_pool(pending_asset, pending_usdf);
    }
}

#[storage(read, write)]
fn internal_close_trove(id: Identity, close_status: Status) {
    require(close_status != Status::NonExistent || close_status != Status::Active, "Invalid status");

    let trove_owner_array_length = storage.trove_owners.len();
    require_more_than_one_trove_in_system(trove_owner_array_length);

    let mut trove = storage.troves.get(id);
    trove.status = close_status;
    trove.coll = 0;
    trove.debt = 0;
    storage.troves.insert(id, trove);

    // TODO Reward snapshot
    internal_remove_trove_owner(id, trove_owner_array_length);
    let sorted_troves_contract = abi(SortedTroves, storage.sorted_troves_contract.into());
    sorted_troves_contract.remove(id);
}

#[storage(read, write)]
fn internal_remove_trove_owner(_borrower: Identity, _trove_array_owner_length: u64) {
    let mut trove = storage.troves.get(_borrower);

    require(trove.status != Status::NonExistent && trove.status != Status::Active, "Trove does not exist");

    let index = trove.array_index;
    let length = _trove_array_owner_length;
    let indx_last = length - 1;

    require(index <= indx_last, "Trove does not exist");
    let address_to_move = storage.trove_owners.get(indx_last).unwrap();

    let mut trove_to_move = storage.troves.get(address_to_move);
    trove_to_move.array_index = index;
    storage.troves.insert(address_to_move, trove_to_move);

    let _ = storage.trove_owners.swap_remove(index);
}
#[storage(read)]
fn require_trove_is_active(id: Identity) {
    let trove = storage.troves.get(id);
    require(trove.status == Status::Active, "Trove is not active");
}

#[storage(read, write)]
fn internal_batch_liquidate_troves(borrowers: Vec<Identity>) {
    require(borrowers.len() > 0, "No borrowers to liquidate");

    let mut vars = LocalVariablesOuterLiquidationFunction::default();
    let oracle = abi(MockOracle, storage.oracle_contract.into());

    vars.price = oracle.get_price();
    let stability_pool = abi(StabilityPool, storage.stability_pool_contract.into());
    let total_usdf_in_sp = stability_pool.get_total_usdf_deposits();

    let totals = internal_get_totals_from_batch_liquidate(vars.price, total_usdf_in_sp, borrowers);

    require(totals.total_debt_in_sequence > 0, "No debt to liquidate");
    stability_pool.offset(totals.total_debt_to_offset, totals.total_coll_to_send_to_sp);

    if (totals.total_coll_surplus > 0) {
        // TODO Change add to coll_surplus_pool and also 
        let active_pool = abi(ActivePool, storage.active_pool_contract.into());
        active_pool.send_asset(Identity::ContractId(storage.coll_surplus_pool_contract), totals.total_coll_surplus);
    }

    internal_redistribute_debt_and_coll(totals.total_debt_to_redistribute, totals.total_coll_to_redistribute);
}

#[storage(read)]
fn require_caller_is_borrow_operations_contract() {
    let caller = msg_sender().unwrap();
    let borrow_operations_contract = Identity::ContractId(storage.borrow_operations_contract);
    require(caller == borrow_operations_contract, "Caller is not the Borrow Operations contract");
}

#[storage(read, write)]
fn internal_increase_trove_coll(id: Identity, coll: u64) -> u64 {
    let mut trove = storage.troves.get(id);
    trove.coll += coll;
    storage.troves.insert(id, trove);

    return trove.coll;
}

#[storage(read, write)]
fn internal_increase_trove_debt(id: Identity, debt: u64) -> u64 {
    let mut trove = storage.troves.get(id);
    trove.debt += debt;
    storage.troves.insert(id, trove);

    return trove.debt;
}

#[storage(read, write)]
fn internal_decrease_trove_coll(id: Identity, coll: u64) -> u64 {
    let mut trove = storage.troves.get(id);
    trove.coll -= coll;
    storage.troves.insert(id, trove);

    return trove.coll;
}

#[storage(read, write)]
fn internal_decrease_trove_debt(id: Identity, debt: u64) -> u64 {
    let mut trove = storage.troves.get(id);
    trove.debt -= debt;
    storage.troves.insert(id, trove);

    return trove.debt;
}

#[storage(read, write)]
fn internal_get_totals_from_batch_liquidate(
    price: u64,
    usdf_in_stability_pool: u64,
    borrowers: Vec<Identity>,
) -> LiquidationTotals {
    let mut vars = LocalVariablesLiquidationSequence::default();
    vars.remaining_usdf_in_stability_pool = usdf_in_stability_pool;
    let mut single_liquidation = LiquidationValues::default();
    let mut i = 0;
    let mut totals = LiquidationTotals::default();

    while i < borrowers.len() {
        vars.borrower = borrowers.get(i).unwrap();
        vars.icr = internal_get_current_icr(vars.borrower, price);

        if vars.icr < MCR {
            let position = get_entire_debt_and_coll(vars.borrower);

            internal_pending_trove_rewards_to_active_pool(position.pending_coll_rewards, position.pending_debt_rewards);

            single_liquidation = get_offset_and_redistribution_vals(position.entire_trove_coll, position.entire_trove_debt, usdf_in_stability_pool, price);

            internal_apply_liquidation(vars.borrower, single_liquidation);
            vars.remaining_usdf_in_stability_pool -= single_liquidation.debt_to_offset;
            totals = add_liquidation_vals_to_totals(totals, single_liquidation);
        } else {
            break;
        }
    }
    return totals;
}

#[storage(read)]
fn require_more_than_one_trove_in_system(trove_owner_array_length: u64) {
    let sorted_troves_contract = abi(SortedTroves, storage.sorted_troves_contract.into());
    let size = sorted_troves_contract.get_size();
    require(trove_owner_array_length > 1 && size > 1, "There is only one trove in the system");
}

#[storage(read)]
fn internal_get_current_icr(borrower: Identity, price: u64) -> u64 {
    let position = get_entire_debt_and_coll(borrower);

    return fm_compute_cr(position.entire_trove_coll, position.entire_trove_debt, price);
}

#[storage(read)]
fn internal_get_trove_stake(borrower: Identity) -> u64 {
    let trove = storage.troves.get(borrower);
    return trove.stake;
}

#[storage(read, write)]
fn internal_remove_stake(borrower: Identity) {
    let mut trove = storage.troves.get(borrower);
    storage.total_stakes -= trove.stake;
    // TODO use update function when available
    trove.stake = 0;
    storage.troves.insert(borrower, trove);
}

#[storage(read)]
fn get_entire_debt_and_coll(borrower: Identity) -> EntireTroveDebtAndColl {
    let trove = storage.troves.get(borrower);
    let coll = trove.coll;
    let debt = trove.debt;

    let pending_coll_rewards = internal_get_pending_asset_reward(borrower);
    let pending_debt_rewards = internal_get_pending_usdf_reward(borrower);

    return EntireTroveDebtAndColl {
        entire_trove_debt: debt + pending_debt_rewards,
        entire_trove_coll: coll + pending_coll_rewards,
        pending_debt_rewards,
        pending_coll_rewards,
    }
}

#[storage(read, write)]
fn internal_apply_liquidation(borrower: Identity, liquidation_values: LiquidationValues) {
    if (liquidation_values.is_partial_liquidation) {
        let mut trove = storage.troves.get(borrower);
        trove.coll = liquidation_values.remaining_trove_coll;
        trove.debt = liquidation_values.remaining_trove_debt;
        storage.troves.insert(borrower, trove);

        let _ = internal_update_stake_and_total_stakes(borrower);

        let new_ncr = fm_compute_nominal_cr(trove.coll, trove.debt);
        let sorted_troves_contract = abi(SortedTroves, storage.sorted_troves_contract.into());
        sorted_troves_contract.re_insert(borrower, new_ncr, null_identity_address(), null_identity_address());
    } else {
        let coll_surplus_contract = abi(CollSurplusPool, storage.coll_surplus_pool_contract.into());
        internal_remove_stake(borrower);
        internal_close_trove(borrower, Status::ClosedByLiquidation());
        coll_surplus_contract.account_surplus(borrower, liquidation_values.coll_surplus);
    }
}

#[storage(read, write)]
fn internal_redistribute_debt_and_coll(debt: u64, coll: u64) {
    if (debt == 0) {
        return;
    }

    let asset_numerator: U128 = U128::from_u64(coll) * U128::from_u64(DECIMAL_PRECISION) + U128::from_u64(storage.last_asset_error_redistribution);
    let usdf_numerator: U128 = U128::from_u64(debt) * U128::from_u64(DECIMAL_PRECISION) + U128::from_u64(storage.last_usdf_error_redistribution);

    let asset_reward_per_unit_staked = asset_numerator / U128::from_u64(storage.total_stakes);
    let usdf_reward_per_unit_staked = usdf_numerator / U128::from_u64(storage.total_stakes);

    storage.last_asset_error_redistribution = (asset_numerator - (asset_reward_per_unit_staked * U128::from_u64(storage.total_stakes))).as_u64().unwrap();
    storage.last_usdf_error_redistribution = (usdf_numerator - (usdf_reward_per_unit_staked * U128::from_u64(storage.total_stakes))).as_u64().unwrap();

    storage.l_asset += asset_reward_per_unit_staked.as_u64().unwrap();
    storage.l_usdf += usdf_reward_per_unit_staked.as_u64().unwrap();

    let active_pool = abi(ActivePool, storage.active_pool_contract.into());
    let default_pool = abi(DefaultPool, storage.default_pool_contract.into());

    active_pool.decrease_usdf_debt(debt);
    default_pool.increase_usdf_debt(debt);
    active_pool.send_asset_to_default_pool(coll);
}

#[storage(read, write)]
fn internal_update_stake_and_total_stakes(address: Identity) -> u64 {
    let mut trove = storage.troves.get(address);
    let new_stake = internal_compute_new_stake(trove.coll);

    let old_stake = trove.stake;
    trove.stake = new_stake;
    storage.troves.insert(address, trove);

    let old_total_stakes = storage.total_stakes;
    storage.total_stakes = old_total_stakes + new_stake - old_stake;
    return new_stake;
}

#[storage(read)]
fn internal_compute_new_stake(coll: u64) -> u64 {
    if (storage.total_collateral_snapshot == 0) {
        return coll;
    } else {
        require(storage.total_stakes_snapshot > 0, "Total stakes snapshot is zero");
        let stake = (U128::from_u64(coll) * U128::from_u64(storage.total_stakes_snapshot)) / U128::from_u64(storage.total_collateral_snapshot);
        return stake.as_u64().unwrap();
    }
}

#[storage(read)]
fn internal_get_pending_asset_reward(address: Identity) -> u64 {
    let snapshot_asset = storage.reward_snapshots.get(address).asset;
    let reward_per_unit_staked = storage.l_asset - snapshot_asset;

    if (reward_per_unit_staked == 0
        || storage.troves.get(address).status != Status::Active())
    {
        return 0;
    }
    let stake = storage.troves.get(address).stake;
    let pending_asset_reward = (U128::from_u64(reward_per_unit_staked) * U128::from_u64(stake)) / U128::from_u64(DECIMAL_PRECISION);

    return pending_asset_reward.as_u64().unwrap();
}

#[storage(read)]
fn internal_get_pending_usdf_reward(address: Identity) -> u64 {
    let snapshot_usdf = storage.reward_snapshots.get(address).usdf_debt;
    let reward_per_unit_staked = storage.l_usdf - snapshot_usdf;

    if (reward_per_unit_staked == 0
        || storage.troves.get(address).status != Status::Active())
    {
        return 0;
    }
    let stake = storage.troves.get(address).stake;
    let pending_usdf_reward = (U128::from_u64(reward_per_unit_staked) * U128::from_u64(stake)) / U128::from_u64(DECIMAL_PRECISION);

    return pending_usdf_reward.as_u64().unwrap();
}

#[storage(read)]
fn internal_has_pending_rewards(address: Identity) -> bool {
    if (storage.troves.get(address).status != Status::Active())
    {
        return false;
    }

    return (storage.reward_snapshots.get(address).asset < storage.l_asset);
}

#[storage(read)]
fn internal_pending_trove_rewards_to_active_pool(coll: u64, debt: u64) {
    if (coll == 0 && debt == 0) {
        return;
    }
    let default_pool = abi(DefaultPool, storage.default_pool_contract.into());
    let active_pool = abi(ActivePool, storage.active_pool_contract.into());

    default_pool.decrease_usdf_debt(debt);
    active_pool.increase_usdf_debt(debt);

    default_pool.send_asset_to_active_pool(coll);
}

#[storage(read)]
fn internal_get_entire_system_debt() -> u64 {
    let active_pool = abi(ActivePool, storage.active_pool_contract.into());
    let default_pool = abi(DefaultPool, storage.default_pool_contract.into());

    return active_pool.get_usdf_debt() + default_pool.get_usdf_debt();
}

#[storage(read, write)]
fn internal_redeem_collateral_from_trove(
    borrower: Identity,
    max_usdf_amount: u64,
    price: u64,
    partial_redemption_hint: u64,
    upper_partial_hint: Identity,
    lower_partial_hint: Identity,
) -> SingleRedemptionValues {
    let mut single_redemption_values = SingleRedemptionValues::default();
    let sorted_troves_contract = abi(SortedTroves, storage.sorted_troves_contract.into());
    // Determine the remaining amount (lot) to be redeemed, capped by the entire debt of the Trove minus the liquidation reserve
    let trove = storage.troves.get(borrower);
    single_redemption_values.usdf_lot = fm_min(max_usdf_amount, trove.debt);
    single_redemption_values.asset_lot = ((U128::from_u64(single_redemption_values.usdf_lot) * U128::from_u64(ORACLE_PRICE_PRECISION)) / U128::from_u64(price)).as_u64().unwrap();
    let new_debt = trove.debt - single_redemption_values.usdf_lot;
    let new_coll = trove.coll - single_redemption_values.asset_lot;

    // If the Trove is now empty, close it
    if (new_debt == 0) {
        internal_remove_stake(borrower);
        internal_close_trove(borrower, Status::ClosedByRedemption);
        internal_redeem_close_trove(borrower, 0, new_coll);
    } else {
        let new_nicr = fm_compute_nominal_cr(new_coll, new_debt);

        // TODO Consider removing this check
        // if (new_nicr != partial_redemption_hint
        //     || new_debt < MIN_NET_DEBT)
        // {
        //     single_redemption_values.cancelled_partial = true;
        //     return single_redemption_values;
        // }
        sorted_troves_contract.re_insert(borrower, new_nicr, upper_partial_hint, lower_partial_hint);
        let mut trove = storage.troves.get(borrower);
        trove.debt = new_debt;
        trove.coll = new_coll;
        storage.troves.insert(borrower, trove);

        internal_update_stake_and_total_stakes(borrower);
    }

    return single_redemption_values;
}

#[storage(read, write)]
fn internal_redeem_close_trove(borrower: Identity, usdf: u64, asset: u64) {
    let usdf_contract = abi(USDFToken, storage.usdf_contract.into());
    let active_pool = abi(ActivePool, storage.active_pool_contract.into());
    let coll_surplus_pool = abi(CollSurplusPool, storage.coll_surplus_pool_contract.into());

    usdf_contract.burn {
        coins: usdf,
        asset_id: storage.usdf_contract.value,
    }();

    active_pool.decrease_usdf_debt(usdf);
    coll_surplus_pool.account_surplus(borrower, asset);
    active_pool.send_asset(Identity::ContractId(storage.coll_surplus_pool_contract), asset);
}

#[storage(read, write)]
fn internal_update_base_rate_from_redemption(asset_drawn: u64, price: u64, total_usdf_supply: u64) {}

    // TODO FEE
#[storage(read)]
fn internal_minutes_passed_since_last_fee_op() -> u64 {
    // TODO FEE
    return 0;
}

#[storage(read)]
fn internal_get_redemption_fee(asset_drawn: u64) -> u64 {
    // TODO FEE
    return 0;
}

#[storage(read)]
fn require_valid_usdf_id() {
    require(msg_asset_id() == storage.usdf_contract, "Invalid asset being transfered");
}
