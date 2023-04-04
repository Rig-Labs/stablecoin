contract;

dep data_structures;
use data_structures::{AssetContracts, Snapshots};

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
    storage::{
        StorageMap,
        StorageVec,
    },
    token::transfer,
    u128::U128,
};

const SCALE_FACTOR = 1_000_000_000;
const DECIMAL_PRECISION = 1_000_000;

storage {
    // List of assets tracked by the Stability Pool
    valid_assets: StorageVec<ContractId> = StorageVec {},
    // Asset amounts held by the Stability Pool to be claimed
    asset: StorageMap<ContractId, u64> = StorageMap {},
    // Total amount of USDF held by the Stability Pool
    total_usdf_deposits: u64 = 0,
    // Amount of USDF deposited by each user
    deposits: StorageMap<Identity, u64> = StorageMap {},
    // Starting S for each asset when deposited by each user
    deposit_snapshot_s_per_asset: StorageMap<(Identity, ContractId), U128> = StorageMap {},
    // Snapshot of P, G, epoch and scale when each deposit was made
    deposit_snapshots: StorageMap<Identity, Snapshots> = StorageMap {},
    // Each time the scale of P shifts by SCALE_FACTOR, the scale is incremented by 1
    current_scale: u64 = 0,
    // With each offset that fully empties the Pool, the epoch is incremented by 1
    current_epoch: u64 = 0,
    /* Asset Gain sum 'S(asset)': During its lifetime, each deposit d_t earns an Asset gain of ( d_t * [S(asset) - S_t(asset)] )/P_t, 
    * where S_t(asset) is the depositor's snapshot of S(asset) taken at the time t when the deposit was made.
    *
    * The 'S(asset)' sums are stored in a nested mapping (epoch => scale => asset => sum):
    *
    * - The inner mapping records the sum S(asset) at different scales
    * - The outer mapping records the (scale => sum) mappings, for different epochs.
    */
    epoch_to_scale_to_sum: StorageMap<(u64, u64, ContractId), U128> = StorageMap {},
    /*
    * Similarly, the sum 'G' is used to calculate FPT gains. During it's lifetime, each deposit d_t earns a FPT gain of
    *  ( d_t * [G - G_t] )/P_t, where G_t is the depositor's snapshot of G taken at time t when  the deposit was made.
    *
    *  FPT reward events occur are triggered by depositor operations (new deposit, topup, withdrawal), and liquidations.
    *  In each case, the FPT reward is issued (i.e. G is updated), before other state changes are made.
    */
    epoch_to_scale_to_gain: StorageMap<(u64, u64), U128> = StorageMap {},
    p: U128 = U128::from_u64(DECIMAL_PRECISION),
    last_fpt_error: U128 = U128::from_u64(0),
    last_asset_error_offset: StorageMap<ContractId, U128> = StorageMap {},
    last_usdf_error_offset: U128 = U128::from_u64(0),
    borrow_operations_address: ContractId = null_contract(),
    protocol_manager_address: ContractId = null_contract(),
    usdf_address: ContractId = null_contract(),
    community_issuance_address: ContractId = null_contract(),
    is_initialized: bool = false,
    // Asset specific addresses
    asset_contracts: StorageMap<ContractId, AssetContracts> = StorageMap {},
}

impl StabilityPool for Contract {
    #[storage(read, write)]
    fn initialize(
        borrow_operations_address: ContractId,
        usdf_address: ContractId,
        community_issuance_address: ContractId,
        protocol_manager: ContractId,
    ) {
        require(storage.is_initialized == false, "Contract is already initialized");

        storage.borrow_operations_address = borrow_operations_address;
        storage.usdf_address = usdf_address;
        storage.community_issuance_address = community_issuance_address;
        storage.protocol_manager_address = protocol_manager;
        storage.is_initialized = true;
    }

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
        storage.last_asset_error_offset.insert(asset_address, U128::from_u64(0));
        storage.asset_contracts.insert(asset_address, AssetContracts {
            trove_manager: trove_manager_address,
            active_pool: active_pool_address,
            sorted_troves: sorted_troves_address,
            oracle: oracle_address,
        });
    }

    /*
    * - Triggers a FPT issuance, based on time passed since the last issuance. The FPT issuance is shared between *all* depositors
    * - Sends depositor's accumulated gains (FPT, Asset1, Asset2...) to depositor
    * - Sends the tagged front end's accumulated FPT gains to the tagged front end
    * - Increases deposit stake, and takes new snapshots for each.
    */
    #[storage(read, write), payable]
    fn provide_to_stability_pool() {
        require_usdf_is_valid_and_non_zero();

        let initial_deposit = storage.deposits.get(msg_sender().unwrap());
        // TODO Trigger FPT issuance
        let compounded_usdf_deposit = internal_get_compounded_usdf_deposit(msg_sender().unwrap());
        let usdf_loss = initial_deposit - compounded_usdf_deposit;

        let mut i = 0;
        while i < storage.valid_assets.len() {
            let asset_address = storage.valid_assets.get(i).unwrap();
            let asset_gain = internal_get_depositor_asset_gain(msg_sender().unwrap(), asset_address);
            send_asset_gain_to_depositor(msg_sender().unwrap(), asset_gain, asset_address);
            i += 1;
        }
        let new_position = compounded_usdf_deposit + msg_amount();
        internal_update_deposits_and_snapshots(msg_sender().unwrap(), new_position);

        storage.total_usdf_deposits += msg_amount();
        // Pay out FPT gains
    }

     /*
    * - Triggers a FPT issuance, based on time passed since the last issuance. The FPT issuance is shared between *all* depositors
    * - Sends all depositor's accumulated gains (FPT, Asset1, Asset2...) to depositor
    * - Decreases deposit stake, and takes new snapshots for each.
    *
    * If amount > userDeposit, the user withdraws all of their compounded deposit.
    */
    #[storage(read, write)]
    fn withdraw_from_stability_pool(amount: u64) {
        let initial_deposit = storage.deposits.get(msg_sender().unwrap());

        require_user_has_initial_deposit(initial_deposit);
        // TODO Trigger FPT issuance
        let compounded_usdf_deposit = internal_get_compounded_usdf_deposit(msg_sender().unwrap());
        let usdf_to_withdraw = fm_min(amount, compounded_usdf_deposit);

        let new_position = compounded_usdf_deposit - usdf_to_withdraw;

        let mut i = 0;
        while i < storage.valid_assets.len() {
            let asset_address = storage.valid_assets.get(i).unwrap();
            let asset_gain = internal_get_depositor_asset_gain(msg_sender().unwrap(), asset_address);
            send_asset_gain_to_depositor(msg_sender().unwrap(), asset_gain, asset_address);
            i += 1;
        }

        internal_update_deposits_and_snapshots(msg_sender().unwrap(), new_position);
        send_usdf_to_depositor(msg_sender().unwrap(), usdf_to_withdraw);
    }

     /* 
    * - Triggers a FPT issuance, based on time passed since the last issuance. The FPT issuance is shared between *all* depositors
    * - Sends all depositor's FPT gain to depositor
    * - Transfers the depositor's entire Asset gain from the Stability Pool to the caller's trove
    * - Leaves their compounded deposit in the Stability Pool
    * - Updates snapshots for deposit and tagged front end stake */
    #[storage(read, write)]
    fn withdraw_gain_to_trove(
        lower_hint: Identity,
        upper_hint: Identity,
        asset_address: ContractId,
    ) {
        let sender = msg_sender().unwrap();
        let initial_deposit = storage.deposits.get(sender);
        let borrower_operations = abi(BorrowOperations, storage.borrow_operations_address.value);
        let asset_contracts = storage.asset_contracts.get(asset_address);
        require_user_has_initial_deposit(initial_deposit);
        require_user_has_asset_gain(sender, asset_address);
        require_user_has_trove(sender, asset_contracts.trove_manager);

        // TODO Trigger FPT issuance
        let depositor_asset_gain = internal_get_depositor_asset_gain(sender, asset_address);
        let compounded_usdf_deposit = internal_get_compounded_usdf_deposit(sender);

        let usdf_loss = initial_deposit - compounded_usdf_deposit;
        internal_update_deposits_and_snapshots(sender, compounded_usdf_deposit);

        let mut current_asset_amount = storage.asset.get(asset_address);
        current_asset_amount -= depositor_asset_gain;
        storage.asset.insert(asset_address, current_asset_amount);

        borrower_operations.move_asset_gain_to_trove {
            coins: depositor_asset_gain,
            asset_id: asset_address.value,
        }(sender, lower_hint, upper_hint, asset_address);
    }

    #[storage(read, write)]
    fn offset(
        debt_to_offset: u64,
        coll_to_offset: u64,
        asset_address: ContractId,
    ) {
        require_caller_is_trove_manager();
        let total_usdf = storage.total_usdf_deposits;

        if total_usdf == 0 || debt_to_offset == 0 {
            return;
        }

        let asset_addresses_cache = storage.asset_contracts.get(asset_address);

        let per_unit_staked_changes = compute_rewards_per_unit_staked(coll_to_offset, debt_to_offset, total_usdf, asset_address);

        update_reward_sum_and_product(per_unit_staked_changes.0, per_unit_staked_changes.1, asset_address);

        internal_move_offset_coll_and_debt(coll_to_offset, debt_to_offset, asset_address, asset_addresses_cache);
    }

    #[storage(read)]
    fn get_asset(asset_address: ContractId) -> u64 {
        return storage.asset.get(asset_address);
    }

    #[storage(read)]
    fn get_total_usdf_deposits() -> u64 {
        return storage.total_usdf_deposits;
    }

    #[storage(read)]
    fn get_depositor_asset_gain(depositor: Identity, asset_address: ContractId) -> u64 {
        return internal_get_depositor_asset_gain(depositor, asset_address);
    }

    #[storage(read)]
    fn get_compounded_usdf_deposit(depositor: Identity) -> u64 {
        return internal_get_compounded_usdf_deposit(depositor);
    }
}

#[storage(read)]
fn require_is_protocol_manager() {
    let protocol_manager = Identity::ContractId(storage.protocol_manager_address);
    require(msg_sender().unwrap() == protocol_manager, "Caller is not the protocol manager");
}

#[storage(read)]
fn require_usdf_is_valid_and_non_zero() {
    require(storage.usdf_address == msg_asset_id(), "USDF contract not initialized");
    require(msg_amount() > 0, "USDF amount must be greater than 0");
}

#[storage(read)]
fn require_user_has_trove(address: Identity, trove_manager_address: ContractId) {
    let trove_manager = abi(TroveManager, trove_manager_address.value);
    let status = trove_manager.get_trove_status(address);
    require(status == Status::Active, "User does not have an active trove");
}

// --- Reward calculator functions for depositor and front end ---
/* Calculates the asset gain earned by the deposit since its last snapshots were taken.
* Given by the formula:  E = d0 * (S(asset) - S(0,asset))/P(0)
* where S(0,asset) and P(0) are the depositor's snapshots of the sum S(asset) and product P, respectively.
* d0 is the last recorded deposit value.
*/
#[storage(read)]
fn internal_get_depositor_asset_gain(depositor: Identity, asset: ContractId) -> u64 {
    let initial_deposit = storage.deposits.get(depositor);

    if initial_deposit == 0 {
        return 0;
    }

    let s_snapshot = storage.deposit_snapshot_s_per_asset.get((depositor, asset));
    let mut snapshots = storage.deposit_snapshots.get(depositor);

    return internal_get_asset_gain_from_snapshots(initial_deposit, snapshots, s_snapshot, asset);
}

#[storage(read)]
fn internal_get_asset_gain_from_snapshots(
    initial_deposit: u64,
    snapshots: Snapshots,
    s_snapshot: U128,
    asset: ContractId,
) -> u64 {
    let epoch_snapshot = snapshots.epoch;
    let scale_snapshot = snapshots.scale;

    let p_snapshot = snapshots.P;

    let first_portion = storage.epoch_to_scale_to_sum.get((epoch_snapshot, scale_snapshot, asset)) - s_snapshot;
    let second_portion = storage.epoch_to_scale_to_sum.get((epoch_snapshot, scale_snapshot + 1, asset)) / U128::from_u64(SCALE_FACTOR);

    let gain = (U128::from_u64(initial_deposit) * (first_portion + second_portion)) / p_snapshot / U128::from_u64(DECIMAL_PRECISION);

    return gain.as_u64().unwrap();
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
fn internal_increase_asset(total_asset_to_increase: u64, asset_address: ContractId) {
    let mut asset_amount = storage.asset.get(asset_address);
    asset_amount += total_asset_to_increase;
    storage.asset.insert(asset_address, asset_amount);
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

    let current_g = storage.epoch_to_scale_to_gain.get((current_epoch, current_scale));

    let snapshots = Snapshots {
        epoch: current_epoch,
        scale: current_scale,
        P: current_p,
        G: current_g,
    };

    // TODO use itterator when available
    let mut i = 0;
    while i < storage.valid_assets.len() {
        let asset = storage.valid_assets.get(i).unwrap();
        let current_s: U128 = storage.epoch_to_scale_to_sum.get((current_epoch, current_scale, asset));
        storage.deposit_snapshot_s_per_asset.insert((depositor, asset), current_s);
        i += 1;
    }

    storage.deposit_snapshots.insert(depositor, snapshots);
}

#[storage(read, write), payable]
fn send_asset_gain_to_depositor(depositor: Identity, gain: u64, asset_address: ContractId) {
    if (gain == 0) {
        return;
    }
    let mut asset_amount = storage.asset.get(asset_address);
    asset_amount -= gain;
    storage.asset.insert(asset_address, asset_amount);
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
fn require_user_has_asset_gain(depositor: Identity, asset_address: ContractId) {
    let gain = internal_get_depositor_asset_gain(depositor, asset_address);
    require(gain > 0, "User has no asset gain");
}

#[storage(read)]
fn require_caller_is_trove_manager() {
    let mut i = 0;
    while i < storage.valid_assets.len() {
        let asset = storage.valid_assets.get(i).unwrap();
        let trove_manager_address = Identity::ContractId(storage.asset_contracts.get(asset).trove_manager);
        if (msg_sender().unwrap() == trove_manager_address) {
            return;
        }
        i += 1;
    }
    require(false, "Caller is not a trove manager");
}

fn require_user_has_initial_deposit(deposit: u64) {
    require(deposit > 0, "User has no initial deposit");
}

#[storage(read, write)]
fn compute_rewards_per_unit_staked(
    coll_to_add: u64,
    debt_to_offset: u64,
    total_usdf_deposits: u64,
    asset_address: ContractId,
) -> (U128, U128) {
    let asset_numerator: U128 = U128::from_u64(coll_to_add) * U128::from_u64(DECIMAL_PRECISION) + storage.last_asset_error_offset.get(asset_address);

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

    storage.last_asset_error_offset.insert(asset_address, asset_numerator - (asset_gain_per_unit_staked * U128::from_u64(total_usdf_deposits)));

    return (asset_gain_per_unit_staked, usdf_loss_per_unit_staked);
}
#[storage(read, write)]
fn update_reward_sum_and_product(
    asset_gain_per_unit_staked: U128,
    usdf_loss_per_unit_staked: U128,
    asset: ContractId,
) {
    let current_p = storage.p;
    let mut new_p: U128 = U128::from_u64(0);
    let new_product_factor = U128::from_u64(DECIMAL_PRECISION) - usdf_loss_per_unit_staked;
    let current_epoch = storage.current_epoch;
    let current_scale = storage.current_scale;

    let current_s = storage.epoch_to_scale_to_sum.get((current_epoch, current_scale, asset));

    let marginal_asset_gain: U128 = asset_gain_per_unit_staked * current_p;
    let new_sum = current_s + marginal_asset_gain;

    storage.epoch_to_scale_to_sum.insert((current_epoch, current_scale, asset), new_sum);

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
fn internal_move_offset_coll_and_debt(
    coll_to_add: u64,
    debt_to_offset: u64,
    asset_address: ContractId,
    asset_addresses_cache: AssetContracts,
) {
    let active_pool = abi(ActivePool, asset_addresses_cache.active_pool.value);
    let usdf_contract = abi(USDFToken, storage.usdf_address.value);
    internal_decrease_usdf(debt_to_offset);
    internal_increase_asset(coll_to_add, asset_address);
    active_pool.decrease_usdf_debt(debt_to_offset);

    usdf_contract.burn {
        coins: debt_to_offset,
        asset_id: storage.usdf_address.value,
    }();

    active_pool.send_asset(Identity::ContractId(contract_id()), coll_to_add);
}
