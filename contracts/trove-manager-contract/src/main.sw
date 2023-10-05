contract;

mod data_structures;
mod utils;

use ::utils::{add_liquidation_vals_to_totals, get_offset_and_redistribution_vals};
use ::data_structures::{
    EntireTroveDebtAndColl,
    LiquidationTotals,
    LiquidationValues,
    LocalVariablesLiquidationSequence,
    LocalVariablesOuterLiquidationFunction,
    RedemptionTotals,
    RewardSnapshot,
    Trove,
};

use libraries::trove_manager_interface::{TroveManager};
use libraries::usdf_token_interface::{USDFToken};
use libraries::sorted_troves_interface::{SortedTroves};
use libraries::stability_pool_interface::{StabilityPool};
use libraries::default_pool_interface::{DefaultPool};
use libraries::active_pool_interface::{ActivePool};
use libraries::coll_surplus_pool_interface::{CollSurplusPool};
use libraries::mock_oracle_interface::{MockOracle};
use libraries::trove_manager_interface::data_structures::{SingleRedemptionValues, Status};
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
    hash::Hash,
    logging::log,
    storage::storage_vec::*,
    token::transfer,
    u128::U128,
};

storage {
    protocol_manager_contract: ContractId = ContractId::from(ZERO_B256),
    sorted_troves_contract: ContractId = ContractId::from(ZERO_B256),
    borrow_operations_contract: ContractId = ContractId::from(ZERO_B256),
    stability_pool_contract: ContractId = ContractId::from(ZERO_B256),
    oracle_contract: ContractId = ContractId::from(ZERO_B256),
    active_pool_contract: ContractId = ContractId::from(ZERO_B256),
    default_pool_contract: ContractId = ContractId::from(ZERO_B256),
    coll_surplus_pool_contract: ContractId = ContractId::from(ZERO_B256),
    usdf_contract: ContractId = ContractId::from(ZERO_B256),
    asset_contract: AssetId = ZERO_B256,
    total_stakes: u64 = 0,
    total_stakes_snapshot: u64 = 0,
    total_collateral_snapshot: u64 = 0,
    l_asset: u64 = 0,
    l_usdf: u64 = 0,
    last_asset_error_redistribution: u64 = 0,
    last_usdf_error_redistribution: u64 = 0,
    troves: StorageMap<Identity, Trove> = StorageMap::<Identity, Trove> {},
    trove_owners: StorageVec<Identity> = StorageVec {},
    reward_snapshots: StorageMap<Identity, RewardSnapshot> = StorageMap::<Identity, RewardSnapshot> {},
    is_initialized: bool = false,
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
        asset_contract: AssetId,
        protocol_manager: ContractId,
    ) {
        require(storage.is_initialized.read() == false, "TM: Contract is already initialized");
        storage.sorted_troves_contract.write(sorted_troves);
        storage.borrow_operations_contract.write(borrow_operations);
        storage.stability_pool_contract.write(stability_pool);
        storage.oracle_contract.write(oracle);
        storage.default_pool_contract.write(default_pool);
        storage.active_pool_contract.write(active_pool);
        storage.coll_surplus_pool_contract.write(coll_surplus_pool);
        storage.usdf_contract.write(usdf_contract);
        storage.asset_contract.write(asset_contract);
        storage.protocol_manager_contract.write(protocol_manager);
        storage.is_initialized.write(true);
    }

    #[storage(read)]
    fn get_nominal_icr(id: Identity) -> u64 {
        let trove = storage.troves.get(id).read();

        return fm_compute_nominal_cr(trove.coll, trove.debt);
    }

    #[storage(read, write)]
    fn apply_pending_rewards(id: Identity) {
        require_caller_is_borrow_operations_contract_or_protocol_manager();
        internal_apply_pending_rewards(id);
    }

    #[storage(read)]
    fn has_pending_rewards(id: Identity) -> bool {
        internal_has_pending_rewards(id)
    }

    #[storage(read, write)]
    fn redeem_collateral_from_trove(
        borrower: Identity,
        max_usdf_amount: u64,
        price: u64,
        partial_redemption_hint: u64,
        upper_partial_hint: Identity,
        lower_partial_hint: Identity,
    ) -> SingleRedemptionValues {
        require_caller_is_protocol_manager_contract();
        internal_redeem_collateral_from_trove(borrower, max_usdf_amount, price, partial_redemption_hint, upper_partial_hint, lower_partial_hint)
    }

    #[storage(read)]
    fn get_current_icr(id: Identity, price: u64) -> u64 {
        internal_get_current_icr(id, price)
    }

    #[storage(read)]
    fn get_entire_system_debt() -> u64 {
        internal_get_entire_system_debt()
    }

    #[storage(read)]
    fn get_entire_debt_and_coll(id: Identity) -> (u64, u64, u64, u64) {
        let res = internal_get_entire_debt_and_coll(id);

        return (
            res.entire_trove_debt,
            res.entire_trove_coll,
            res.pending_debt_rewards,
            res.pending_coll_rewards,
        )
    }

    #[storage(read)]
    fn get_trove_stake(id: Identity) -> u64 {
        internal_get_trove_stake(id)
    }

    #[storage(read, write)]
    fn set_trove_status(id: Identity, status: Status) {
        require_caller_is_borrow_operations_contract();

        let mut trove = storage.troves.get(id).read();
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

        let mut trove = storage.troves.get(id).read();
        trove.array_index = indx;
        storage.troves.insert(id, trove);

        return indx;
    }

    #[storage(read)]
    fn get_trove_debt(id: Identity) -> u64 {
        let trove = storage.troves.get(id).read();

        return trove.debt;
    }
    #[storage(read)]
    fn get_trove_coll(id: Identity) -> u64 {
        let trove = storage.troves.get(id).read();

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
        let trove = storage.troves.get(id).read();

        return trove.status;
    }
    #[storage(read, write)]
    fn batch_liquidate_troves(
        borrowers: Vec<Identity>,
        upper_partial_hint: Identity,
        lower_partial_hint: Identity,
    ) {
        internal_batch_liquidate_troves(borrowers, upper_partial_hint, lower_partial_hint);
    }

    #[storage(read, write)]
    fn liquidate(
        id: Identity,
        upper_partial_hint: Identity,
        lower_partial_hint: Identity,
    ) {
        require_trove_is_active(id);

        let mut borrowers: Vec<Identity> = Vec::new();
        borrowers.push(id);

        internal_batch_liquidate_troves(borrowers, upper_partial_hint, lower_partial_hint);
    }
    #[storage(read, write)]
    fn liquidate_troves(
        num_troves: u64,
        upper_partial_hint: Identity,
        lower_partial_hint: Identity,
    ) {}

    // TODO
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
    let mut reward_snapshot = storage.reward_snapshots.get(id).read();

    reward_snapshot.asset = storage.l_asset.read();
    reward_snapshot.usdf_debt = storage.l_usdf.read();

    storage.reward_snapshots.insert(id, reward_snapshot);
}
#[storage(read, write)]
fn internal_apply_pending_rewards(borrower: Identity) {
    if (internal_has_pending_rewards(borrower)) {
        let pending_asset = internal_get_pending_asset_reward(borrower);
        let pending_usdf = internal_get_pending_usdf_reward(borrower);

        let mut trove = storage.troves.get(borrower).read();
        trove.coll += pending_asset;
        trove.debt += pending_usdf;
        storage.troves.insert(borrower, trove);

        internal_update_trove_reward_snapshots(borrower);
        internal_move_pending_trove_rewards_to_active_pool(pending_asset, pending_usdf);
    }
}

#[storage(read, write)]
fn internal_close_trove(id: Identity, close_status: Status) {
    require(close_status != Status::NonExistent || close_status != Status::Active, "TM: Invalid status");
    let asset_contract_cache = storage.asset_contract.read();
    let trove_owner_array_length = storage.trove_owners.len();
    let sorted_troves_contract_cache = storage.sorted_troves_contract.read();
    let sorted_troves = abi(SortedTroves, sorted_troves_contract_cache.into());
    require_more_than_one_trove_in_system(trove_owner_array_length, asset_contract_cache, sorted_troves_contract_cache);

    let mut trove = storage.troves.get(id).read();
    trove.status = close_status;
    trove.coll = 0;
    trove.debt = 0;
    storage.troves.insert(id, trove);

    let mut rewards_snapshot = storage.reward_snapshots.get(id).read();
    rewards_snapshot.asset = 0;
    rewards_snapshot.usdf_debt = 0;
    storage.reward_snapshots.insert(id, rewards_snapshot);

    internal_remove_trove_owner(id, trove_owner_array_length);

    sorted_troves.remove(id, asset_contract_cache);
}
#[storage(read, write)]
fn internal_remove_trove_owner(_borrower: Identity, _trove_array_owner_length: u64) {
    let mut trove = storage.troves.get(_borrower).read();

    require(trove.status != Status::NonExistent && trove.status != Status::Active, "TM: Trove does not exist");

    let index = trove.array_index;
    let length = _trove_array_owner_length;
    let indx_last = length - 1;

    require(index <= indx_last, "TM: Trove does not exist");
    let address_to_move = storage.trove_owners.get(indx_last).unwrap().read();

    let mut trove_to_move = storage.troves.get(address_to_move).read();
    trove_to_move.array_index = index;
    storage.troves.insert(address_to_move, trove_to_move);

    let _ = storage.trove_owners.swap_remove(index);
}
#[storage(read)]
fn require_trove_is_active(id: Identity) {
    let trove = storage.troves.get(id).read();
    require(trove.status == Status::Active, "TM: Trove is not active");
}

#[storage(read, write)]
fn internal_batch_liquidate_troves(
    borrowers: Vec<Identity>,
    upper_partial_hint: Identity,
    lower_partial_hint: Identity,
) {
    require(borrowers.len() > 0, "TM: No borrowers to liquidate");

    let mut vars = LocalVariablesOuterLiquidationFunction::default();
    let oracle = abi(MockOracle, storage.oracle_contract.read().into());
    let asset_contract_cache = storage.asset_contract.read();

    vars.price = oracle.get_price();
    let stability_pool = abi(StabilityPool, storage.stability_pool_contract.read().into());
    let total_usdf_in_sp = stability_pool.get_total_usdf_deposits();

    let totals = internal_get_totals_from_batch_liquidate(vars.price, total_usdf_in_sp, borrowers, upper_partial_hint, lower_partial_hint);

    require(totals.total_debt_in_sequence > 0, "TM: No debt to liquidate");
    stability_pool.offset(totals.total_debt_to_offset, totals.total_coll_to_send_to_sp, storage.asset_contract.read());
    let active_pool = abi(ActivePool, storage.active_pool_contract.read().into());

    if (totals.total_coll_surplus > 0) {
        active_pool.send_asset(Identity::ContractId(storage.coll_surplus_pool_contract.read()), totals.total_coll_surplus, asset_contract_cache);
    }
    if (totals.total_coll_gas_compensation > 0) {
        active_pool.send_asset(msg_sender().unwrap(), totals.total_coll_gas_compensation, asset_contract_cache);
    }

    internal_redistribute_debt_and_coll(totals.total_debt_to_redistribute, totals.total_coll_to_redistribute);
}

#[storage(read)]
fn require_caller_is_borrow_operations_contract() {
    let caller = msg_sender().unwrap();
    let borrow_operations_contract = Identity::ContractId(storage.borrow_operations_contract.read());
    require(caller == borrow_operations_contract, "TM: Caller is not the Borrow Operations contract");
}

#[storage(read)]
fn require_caller_is_protocol_manager_contract() {
    let caller = msg_sender().unwrap();
    let protocol_manager_contract = Identity::ContractId(storage.protocol_manager_contract.read());
    require(caller == protocol_manager_contract, "TM: Caller is not the Protocol Manager contract");
}

#[storage(read)]
fn require_caller_is_borrow_operations_contract_or_protocol_manager() {
    let caller = msg_sender().unwrap();
    let borrow_operations_contract = Identity::ContractId(storage.borrow_operations_contract.read());
    let protocol_manager_contract = Identity::ContractId(storage.protocol_manager_contract.read());
    require(caller == borrow_operations_contract || caller == protocol_manager_contract, "TM: Caller is not the Borrow Operations or Protocol Manager contract");
}

#[storage(read, write)]
fn internal_increase_trove_coll(id: Identity, coll: u64) -> u64 {
    let mut trove = storage.troves.get(id).read();
    trove.coll += coll;
    storage.troves.insert(id, trove);

    return trove.coll;
}

#[storage(read, write)]
fn internal_increase_trove_debt(id: Identity, debt: u64) -> u64 {
    let mut trove = storage.troves.get(id).read();
    trove.debt += debt;
    storage.troves.insert(id, trove);

    return trove.debt;
}

#[storage(read, write)]
fn internal_decrease_trove_coll(id: Identity, coll: u64) -> u64 {
    let mut trove = storage.troves.get(id).read();
    trove.coll -= coll;
    storage.troves.insert(id, trove);

    return trove.coll;
}

#[storage(read, write)]
fn internal_decrease_trove_debt(id: Identity, debt: u64) -> u64 {
    let mut trove = storage.troves.get(id).read();
    trove.debt -= debt;
    storage.troves.insert(id, trove);

    return trove.debt;
}

#[storage(read, write)]
fn internal_get_totals_from_batch_liquidate(
    price: u64,
    usdf_in_stability_pool: u64,
    borrowers: Vec<Identity>,
    upper_partial_hint: Identity,
    lower_partial_hint: Identity,
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
            let position = internal_get_entire_debt_and_coll(vars.borrower);

            internal_move_pending_trove_rewards_to_active_pool(position.pending_coll_rewards, position.pending_debt_rewards);

            single_liquidation = get_offset_and_redistribution_vals(position.entire_trove_coll, position.entire_trove_debt, usdf_in_stability_pool, price);

            internal_apply_liquidation(vars.borrower, single_liquidation, upper_partial_hint, lower_partial_hint);
            vars.remaining_usdf_in_stability_pool -= single_liquidation.debt_to_offset;
            totals = add_liquidation_vals_to_totals(totals, single_liquidation);
        } else {
            break;
        }

        i += 1;
    }
    return totals;
}

#[storage(read)]
fn require_more_than_one_trove_in_system(
    trove_owner_array_length: u64,
    asset_contract: AssetId,
    sorted_troves_contract: ContractId,
) {
    let sorted_troves = abi(SortedTroves, sorted_troves_contract.into());
    let size = sorted_troves.get_size(asset_contract);
    require(trove_owner_array_length > 1 && size > 1, "TM: There is only one trove in the system");
}

#[storage(read)]
fn internal_get_current_icr(borrower: Identity, price: u64) -> u64 {
    let position = internal_get_entire_debt_and_coll(borrower);

    return fm_compute_cr(position.entire_trove_coll, position.entire_trove_debt, price);
}

#[storage(read)]
fn internal_get_trove_stake(borrower: Identity) -> u64 {
    let trove = storage.troves.get(borrower).read();
    return trove.stake;
}

#[storage(read, write)]
fn internal_remove_stake(borrower: Identity) {
    let mut trove = storage.troves.get(borrower).read();
    storage.total_stakes.write(storage.total_stakes.read() - trove.stake);
    // TODO use update function when available
    trove.stake = 0;
    storage.troves.insert(borrower, trove);
}

#[storage(read)]
fn internal_get_entire_debt_and_coll(borrower: Identity) -> EntireTroveDebtAndColl {
    let trove = storage.troves.get(borrower).read();
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
fn internal_apply_liquidation(
    borrower: Identity,
    liquidation_values: LiquidationValues,
    upper_partial_hint: Identity,
    lower_partial_hint: Identity,
) {
    let asset_contract_cache = storage.asset_contract.read();

    if (liquidation_values.is_partial_liquidation) {
        let mut trove = storage.troves.get(borrower).read();
        trove.coll = liquidation_values.remaining_trove_coll;
        trove.debt = liquidation_values.remaining_trove_debt;
        storage.troves.insert(borrower, trove);

        let _ = internal_update_stake_and_total_stakes(borrower);

        let new_ncr = fm_compute_nominal_cr(trove.coll, trove.debt);
        let sorted_troves_contract = abi(SortedTroves, storage.sorted_troves_contract.read().into());
        sorted_troves_contract.re_insert(borrower, new_ncr, upper_partial_hint, lower_partial_hint, asset_contract_cache);
    } else {
        let coll_surplus_contract = abi(CollSurplusPool, storage.coll_surplus_pool_contract.read().into());
        internal_remove_stake(borrower);
        coll_surplus_contract.account_surplus(borrower, liquidation_values.coll_surplus, asset_contract_cache);
    }
}

#[storage(read, write)]
fn internal_redistribute_debt_and_coll(debt: u64, coll: u64) {
    let asset_contract_cache = storage.asset_contract.read();
    if (debt == 0) {
        return;
    }

    let asset_numerator: U128 = U128::from_u64(coll) * U128::from_u64(DECIMAL_PRECISION) + U128::from_u64(storage.last_asset_error_redistribution.read());
    let usdf_numerator: U128 = U128::from_u64(debt) * U128::from_u64(DECIMAL_PRECISION) + U128::from_u64(storage.last_usdf_error_redistribution.read());

    let asset_reward_per_unit_staked = asset_numerator / U128::from_u64(storage.total_stakes.read());
    let usdf_reward_per_unit_staked = usdf_numerator / U128::from_u64(storage.total_stakes.read());

    storage.last_asset_error_redistribution.write((asset_numerator - (asset_reward_per_unit_staked * U128::from_u64(storage.total_stakes.read()))).as_u64().unwrap());
    storage.last_usdf_error_redistribution.write((usdf_numerator - (usdf_reward_per_unit_staked * U128::from_u64(storage.total_stakes.read()))).as_u64().unwrap());

    storage.l_asset.write(storage.l_asset.read() + asset_reward_per_unit_staked.as_u64().unwrap());
    storage.l_usdf.write(storage.l_usdf.read() + usdf_reward_per_unit_staked.as_u64().unwrap());

    let active_pool = abi(ActivePool, storage.active_pool_contract.read().into());
    let default_pool = abi(DefaultPool, storage.default_pool_contract.read().into());

    active_pool.decrease_usdf_debt(debt, asset_contract_cache);
    default_pool.increase_usdf_debt(debt, asset_contract_cache);
    active_pool.send_asset_to_default_pool(coll, asset_contract_cache);
}

#[storage(read, write)]
fn internal_update_stake_and_total_stakes(address: Identity) -> u64 {
    let mut trove = storage.troves.get(address).read();
    let new_stake = internal_compute_new_stake(trove.coll);

    let old_stake = trove.stake;
    trove.stake = new_stake;
    storage.troves.insert(address, trove);

    let old_total_stakes = storage.total_stakes.read();
    storage.total_stakes.write(old_total_stakes + new_stake - old_stake);
    return new_stake;
}

#[storage(read)]
fn internal_compute_new_stake(coll: u64) -> u64 {
    if (storage.total_collateral_snapshot.read() == 0) {
        return coll;
    } else {
        require(storage.total_stakes_snapshot.read() > 0, "TM: Total stakes snapshot is zero");
        let stake = (U128::from_u64(coll) * U128::from_u64(storage.total_stakes_snapshot.read())) / U128::from_u64(storage.total_collateral_snapshot.read());
        return stake.as_u64().unwrap();
    }
}

#[storage(read)]
fn internal_get_pending_asset_reward(address: Identity) -> u64 {
    let snapshot_asset = storage.reward_snapshots.get(address).read().asset;
    let reward_per_unit_staked = storage.l_asset.read() - snapshot_asset;

    if (reward_per_unit_staked == 0
        || storage.troves.get(address).read().status != Status::Active)
    {
        return 0;
    }
    let stake = storage.troves.get(address).read().stake;
    let pending_asset_reward = fm_multiply_ratio(reward_per_unit_staked, stake, DECIMAL_PRECISION);

    return pending_asset_reward;
}

#[storage(read)]
fn internal_get_pending_usdf_reward(address: Identity) -> u64 {
    let snapshot_usdf = storage.reward_snapshots.get(address).read().usdf_debt;
    let reward_per_unit_staked = storage.l_usdf.read() - snapshot_usdf;

    if (reward_per_unit_staked == 0
        || storage.troves.get(address).read().status != Status::Active)
    {
        return 0;
    }
    let stake = storage.troves.get(address).read().stake;
    let pending_usdf_reward = fm_multiply_ratio(reward_per_unit_staked, stake, DECIMAL_PRECISION);

    return pending_usdf_reward;
}

#[storage(read)]
fn internal_has_pending_rewards(address: Identity) -> bool {
    if (storage.troves.get(address).read().status != Status::Active)
    {
        return false;
    }

    return (storage.reward_snapshots.get(address).read().asset < storage.l_asset.read());
}

#[storage(read)]
fn internal_move_pending_trove_rewards_to_active_pool(coll: u64, debt: u64) {
    if (coll == 0 && debt == 0) {
        return;
    }
    let asset_contract_cache = storage.asset_contract.read();
    let default_pool = abi(DefaultPool, storage.default_pool_contract.read().into());
    let active_pool = abi(ActivePool, storage.active_pool_contract.read().into());

    default_pool.decrease_usdf_debt(debt, asset_contract_cache);
    active_pool.increase_usdf_debt(debt, asset_contract_cache);

    default_pool.send_asset_to_active_pool(coll, asset_contract_cache);
}
#[storage(read)]
fn internal_get_entire_system_debt() -> u64 {
    let active_pool = abi(ActivePool, storage.active_pool_contract.read().into());
    let default_pool = abi(DefaultPool, storage.default_pool_contract.read().into());
    let asset_contract_cache = storage.asset_contract.read();

    return active_pool.get_usdf_debt(asset_contract_cache) + default_pool.get_usdf_debt(asset_contract_cache);
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
    let sorted_troves = abi(SortedTroves, storage.sorted_troves_contract.read().into());
    let asset_contract_cache = storage.asset_contract.read();
    // Determine the remaining amount (lot) to be redeemed, capped by the entire debt of the Trove minus the liquidation reserve
    let trove = storage.troves.get(borrower).read();
    single_redemption_values.usdf_lot = fm_min(max_usdf_amount, trove.debt);
    single_redemption_values.asset_lot = fm_multiply_ratio(single_redemption_values.usdf_lot, DECIMAL_PRECISION, price);
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
        if (new_debt < MIN_NET_DEBT) {
            single_redemption_values.cancelled_partial = true;
            return single_redemption_values;
        }
        sorted_troves.re_insert(borrower, new_nicr, upper_partial_hint, lower_partial_hint, asset_contract_cache);
        let mut trove = storage.troves.get(borrower).read();
        trove.debt = new_debt;
        trove.coll = new_coll;
        storage.troves.insert(borrower, trove);

        let _ = internal_update_stake_and_total_stakes(borrower);
    }

    return single_redemption_values;
}

#[storage(read, write)]
fn internal_redeem_close_trove(borrower: Identity, usdf_amount: u64, asset_amount: u64) {
    let asset_contract = storage.asset_contract.read();
    let coll_surplus_pool_contract = storage.coll_surplus_pool_contract.read();

    let usdf_contract = abi(USDFToken, storage.usdf_contract.read().into());
    let active_pool = abi(ActivePool, storage.active_pool_contract.read().into());
    let coll_surplus_pool = abi(CollSurplusPool, coll_surplus_pool_contract.into());

    usdf_contract.burn {
        coins: usdf_amount,
        asset_id: storage.usdf_contract.read().value,
    }();

    active_pool.decrease_usdf_debt(usdf_amount, asset_contract);
    coll_surplus_pool.account_surplus(borrower, asset_amount, asset_contract);
    active_pool.send_asset(Identity::ContractId(coll_surplus_pool_contract), asset_amount, asset_contract);
}

#[storage(read)]
fn require_valid_usdf_id() {
    // require(msg_asset_id() == storage.usdf_contract.read(), "TM: Invalid asset being transfered");
}
