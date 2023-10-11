contract;

mod data_structures;
use ::data_structures::{AssetContracts, Snapshots};

use libraries::trove_manager_interface::data_structures::{Status};
use libraries::stability_pool_interface::{StabilityPool};
use libraries::usdf_token_interface::{USDFToken};
use libraries::active_pool_interface::{ActivePool};
use libraries::trove_manager_interface::{TroveManager};
use libraries::borrow_operations_interface::{BorrowOperations};
use libraries::community_issuance_interface::{CommunityIssuance};
use libraries::fluid_math::numbers::*;
use libraries::fluid_math::{
    DECIMAL_PRECISION,
    fm_min,
    null_contract,
    null_identity_address,
    ZERO_B256,
    get_default_asset_id,
};

use std::{
    auth::msg_sender,
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

const SCALE_FACTOR = 1_000_000_000;

storage {
    asset_contracts: StorageMap<AssetId, AssetContracts> = StorageMap::<AssetId, AssetContracts> {},
    active_pool_contract: ContractId = ContractId::from(ZERO_B256),
    protocol_manager_address: ContractId = ContractId::from(ZERO_B256),
    usdf_contract: ContractId = ContractId::from(ZERO_B256),
    usdf_asset_id: AssetId = AssetId::from(ZERO_B256),
    community_issuance_contract: ContractId = ContractId::from(ZERO_B256),
    // List of assets tracked by the Stability Pool
    valid_assets: StorageVec<AssetId> = StorageVec {},
    // Asset amounts held by the Stability Pool to be claimed
    asset: StorageMap<AssetId, u64> = StorageMap::<AssetId, u64> {},
    // Total amount of USDF held by the Stability Pool
    total_usdf_deposits: u64 = 0,
    // Amount of USDF deposited by each user
    deposits: StorageMap<Identity, u64> = StorageMap::<Identity, u64> {},
    // Starting S for each asset when deposited by each user
    deposit_snapshot_s_per_asset: StorageMap<(Identity, AssetId), U128> = StorageMap::<(Identity, AssetId), U128> {},
    // Snapshot of P, G, epoch and scale when each deposit was made
    deposit_snapshots: StorageMap<Identity, Snapshots> = StorageMap::<Identity, Snapshots> {},
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
    epoch_to_scale_to_sum: StorageMap<(u64, u64, AssetId), U128> = StorageMap::<(u64, u64, AssetId), U128> {},
    /*
    * Similarly, the sum 'G' is used to calculate FPT gains. During it's lifetime, each deposit d_t earns a FPT gain of
    *  ( d_t * [G - G_t] )/P_t, where G_t is the depositor's snapshot of G taken at time t when  the deposit was made.
    *
    *  FPT reward events occur are triggered by depositor operations (new deposit, topup, withdrawal), and liquidations.
    *  In each case, the FPT reward is issued (i.e. G is updated), before other state changes are made.
    */
    epoch_to_scale_to_gain: StorageMap<(u64, u64), U128> = StorageMap::<(u64, u64), U128> {},
    p: U128 = U128::from_u64(DECIMAL_PRECISION),
    last_fpt_error: U128 = U128::from_u64(0),
    last_asset_error_offset: StorageMap<AssetId, U128> = StorageMap::<AssetId, U128> {},
    last_usdf_error_offset: U128 = U128::from_u64(0),
    is_initialized: bool = false,
}

impl StabilityPool for Contract {
    #[storage(read, write)]
    fn initialize(
        usdf_contract: ContractId,
        community_issuance_contract: ContractId,
        protocol_manager: ContractId,
        active_pool_contract: ContractId,
    ) {
        require(storage.is_initialized.read() == false, "Contract is already initialized");

        storage.usdf_contract.write(usdf_contract);
        storage.community_issuance_contract.write(community_issuance_contract);
        storage.protocol_manager_address.write(protocol_manager);
        storage.active_pool_contract.write(active_pool_contract);
        storage.is_initialized.write(true);
        storage.usdf_asset_id.write(get_default_asset_id(usdf_contract));
        // Super weird, updated from 0.38.1 to 0.41.0 and initial storage assignment was not working
        storage.p.write( U128::from_u64(DECIMAL_PRECISION));
    }

    #[storage(read, write)]
    fn add_asset(
        trove_manager_contract: ContractId,
        asset_contract: AssetId,
        oracle_contract: ContractId,
    ) {
        require_is_protocol_manager();
        storage.valid_assets.push(asset_contract);
        storage.last_asset_error_offset.insert(asset_contract, U128::from_u64(0));
        storage.asset_contracts.insert(asset_contract, AssetContracts {
            trove_manager: trove_manager_contract,
            oracle: oracle_contract,
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
        
        let initial_deposit = storage.deposits.get(msg_sender().unwrap()).try_read().unwrap_or(0);
        
        internal_trigger_fpt_issuance();
        
        let compounded_usdf_deposit = internal_get_compounded_usdf_deposit(msg_sender().unwrap());
        
        let usdf_loss = initial_deposit - compounded_usdf_deposit;

        internal_pay_out_asset_gains(msg_sender().unwrap()); // pay out asset gains
        
        internal_pay_out_fpt_gains(msg_sender().unwrap());
        

        let new_position = compounded_usdf_deposit + msg_amount();
        internal_update_deposits_and_snapshots(msg_sender().unwrap(), new_position);
        

        storage.total_usdf_deposits.write(storage.total_usdf_deposits.read() + msg_amount());
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
        let initial_deposit = storage.deposits.get(msg_sender().unwrap()).try_read().unwrap_or(0);

        require_user_has_initial_deposit(initial_deposit);

        internal_trigger_fpt_issuance();

        let compounded_usdf_deposit = internal_get_compounded_usdf_deposit(msg_sender().unwrap());
        let usdf_to_withdraw = fm_min(amount, compounded_usdf_deposit);

        let new_position = compounded_usdf_deposit - usdf_to_withdraw;

        internal_pay_out_asset_gains(msg_sender().unwrap()); // pay out asset gains
        internal_pay_out_fpt_gains(msg_sender().unwrap()); // pay out FPT
        internal_update_deposits_and_snapshots(msg_sender().unwrap(), new_position);
        send_usdf_to_depositor(msg_sender().unwrap(), usdf_to_withdraw);
    }

    #[storage(read, write)]
    fn offset(
        debt_to_offset: u64,
        coll_to_offset: u64,
        asset_contract: AssetId,
    ) {
        require_caller_is_trove_manager();
        let total_usdf = storage.total_usdf_deposits.read();

        if total_usdf == 0 || debt_to_offset == 0 {
            return;
        }
        internal_trigger_fpt_issuance();

        let asset_contractes_cache = storage.asset_contracts.get(asset_contract).read();

        let per_unit_staked_changes = compute_rewards_per_unit_staked(coll_to_offset, debt_to_offset, total_usdf, asset_contract);

        update_reward_sum_and_product(per_unit_staked_changes.0, per_unit_staked_changes.1, asset_contract);

        internal_move_offset_coll_and_debt(coll_to_offset, debt_to_offset, asset_contract, asset_contractes_cache);
    }

    #[storage(read)]
    fn get_asset(asset_contract: AssetId) -> u64 {
        return storage.asset.get(asset_contract).try_read().unwrap_or(0);
    }

    #[storage(read)]
    fn get_total_usdf_deposits() -> u64 {
        return storage.total_usdf_deposits.try_read().unwrap_or(0);
    }

    #[storage(read)]
    fn get_depositor_asset_gain(depositor: Identity, asset_contract: AssetId) -> u64 {
        return internal_get_depositor_asset_gain(depositor, asset_contract);
    }

    #[storage(read)]
    fn get_compounded_usdf_deposit(depositor: Identity) -> u64 {
        return internal_get_compounded_usdf_deposit(depositor);
    }

    #[storage(read)]
    fn get_depositor_fpt_gain(depositor: Identity) -> u64 {
        return internal_get_depositor_fpt_gain(depositor);
    }
}

// --- Internal functions ---
#[storage(read, write)]
fn internal_pay_out_asset_gains(depositor: Identity) {
    let mut i = 0;
    while i < storage.valid_assets.len() {
        let asset_contract = storage.valid_assets.get(i).unwrap().read();
        let asset_gain = internal_get_depositor_asset_gain(depositor, asset_contract);
        send_asset_gain_to_depositor(depositor, asset_gain, asset_contract);
        i += 1;
    }
}

#[storage(read, write)]
fn internal_trigger_fpt_issuance() {
    let community_issuance_contract = abi(CommunityIssuance, storage.community_issuance_contract.read().value);
    let fpt_issuance = community_issuance_contract.issue_fpt();
    
    internal_update_g(fpt_issuance);
    
}

#[storage(read, write)]
fn internal_update_g(fpt_issuance: u64) {
    if (storage.total_usdf_deposits.read() == 0 || fpt_issuance == 0) {
        return;
    }
    let fpt_per_unit_staked = internal_compute_fpt_per_unit_staked(fpt_issuance, storage.total_usdf_deposits.read());
    let marginal_fpt_gain = U128::from_u64(fpt_per_unit_staked) * storage.p.read();
    let current_epoch = storage.current_epoch.read();
    let current_scale = storage.current_scale.read();
    let new_epoch_to_scale_to_gain = storage.epoch_to_scale_to_gain.get((current_epoch, current_scale)).try_read().unwrap_or(U128::from_u64(0)) + marginal_fpt_gain;
    storage.epoch_to_scale_to_gain.insert((current_epoch, current_scale), new_epoch_to_scale_to_gain);
}

#[storage(read, write)]
fn internal_compute_fpt_per_unit_staked(fpt_issuance: u64, total_usdf_deposits: u64) -> u64 {
    let fpt_numerator = U128::from_u64(fpt_issuance) * U128::from_u64(DECIMAL_PRECISION) + storage.last_fpt_error.read();
    let fpt_per_unit_staked = fpt_numerator / U128::from_u64(total_usdf_deposits);
    storage.last_fpt_error.write( fpt_numerator - (fpt_per_unit_staked * U128::from_u64(total_usdf_deposits)));
    fpt_per_unit_staked.as_u64().unwrap()
}

#[storage(read)]
fn internal_pay_out_fpt_gains(depositor: Identity) {
    let depositor_fpt_gain = internal_get_depositor_fpt_gain(depositor);
    if (depositor_fpt_gain > 0) {
        let community_issuance_contract = abi(CommunityIssuance, storage.community_issuance_contract.read().value);
        community_issuance_contract.send_fpt(depositor, depositor_fpt_gain);
    }
}

#[storage(read)]
fn internal_get_depositor_fpt_gain(depositor: Identity) -> u64 {
    let initial_deposit = storage.deposits.get(depositor).try_read().unwrap_or(0);
    if (initial_deposit == 0) {
        return 0;
    }
    let snapshots = storage.deposit_snapshots.get(depositor).try_read().unwrap_or(Snapshots::default());
    let fpt_gain = internal_get_fpt_gain_from_snapshots(initial_deposit, snapshots);
    fpt_gain
}

#[storage(read)]
fn internal_get_fpt_gain_from_snapshots(initial_stake: u64, snapshots: Snapshots) -> u64 {
    let epoch_snapshot = snapshots.epoch;
    let scale_snapshot = snapshots.scale;

    let g_snapshot = snapshots.G;
    let p_snapshot = snapshots.P;

    let first_portion = storage.epoch_to_scale_to_gain.get((epoch_snapshot, scale_snapshot)).try_read().unwrap_or(U128::from_u64(0)) - g_snapshot;
    let second_portion = storage.epoch_to_scale_to_gain.get((epoch_snapshot, scale_snapshot + 1)).try_read().unwrap_or(U128::from_u64(0)) / U128::from_u64(SCALE_FACTOR);

    let gain = (U128::from_u64(initial_stake) * (first_portion + second_portion)) / p_snapshot / U128::from_u64(DECIMAL_PRECISION);

    return gain.as_u64().unwrap();
}

#[storage(read)]
fn require_is_protocol_manager() {
    let protocol_manager = Identity::ContractId(storage.protocol_manager_address.read());
    require(msg_sender().unwrap() == protocol_manager, "SP: Caller is not the protocol manager");
}

#[storage(read)]
fn require_usdf_is_valid_and_non_zero() {
    require(storage.usdf_asset_id.read() == msg_asset_id(), "SP: USDF address is invalid");
    require(msg_amount() > 0, "SP: USDF amount must be greater than 0");
}

#[storage(read)]
fn require_user_has_trove(address: Identity, trove_manager_contract: ContractId) {
    let trove_manager = abi(TroveManager, trove_manager_contract.value);
    let status = trove_manager.get_trove_status(address);
    require(status == Status::Active, "SP: User does not have an active trove");
}

// --- Reward calculator functions for depositor and front end ---
#[storage(read)]
fn internal_get_depositor_asset_gain(depositor: Identity, asset: AssetId) -> u64 {
    let initial_deposit = storage.deposits.get(depositor).try_read().unwrap_or(0);

    if initial_deposit == 0 {
        return 0;
    }

    let s_snapshot = storage.deposit_snapshot_s_per_asset.get((depositor, asset)).try_read().unwrap_or(U128::from_u64(0));
    let mut snapshots = storage.deposit_snapshots.get(depositor).try_read().unwrap_or(Snapshots::default());

    return internal_get_asset_gain_from_snapshots(initial_deposit, snapshots, s_snapshot, asset);
}

#[storage(read)]
fn internal_get_asset_gain_from_snapshots(
    initial_deposit: u64,
    snapshots: Snapshots,
    s_snapshot: U128,
    asset: AssetId,
) -> u64 {
    let epoch_snapshot = snapshots.epoch;
    let scale_snapshot = snapshots.scale;

    let p_snapshot = snapshots.P;

    let first_portion = storage.epoch_to_scale_to_sum.get((epoch_snapshot, scale_snapshot, asset)).try_read().unwrap_or(U128::from_u64(0)) - s_snapshot;
    let second_portion = storage.epoch_to_scale_to_sum.get((epoch_snapshot, scale_snapshot + 1, asset)).try_read().unwrap_or(U128::from_u64(0)) / U128::from_u64(SCALE_FACTOR);

    let gain = (U128::from_u64(initial_deposit) * (first_portion + second_portion)) / p_snapshot / U128::from_u64(DECIMAL_PRECISION);

    return gain.as_u64().unwrap();
}

#[storage(read)]
fn internal_get_compounded_usdf_deposit(depositor: Identity) -> u64 {
    let initial_deposit = storage.deposits.get(depositor).try_read().unwrap_or(0);

    if initial_deposit == 0 {
        return 0;
    }
    let mut snapshots = storage.deposit_snapshots.get(depositor).read();

    return get_compounded_stake_from_snapshots(initial_deposit, snapshots)
}

#[storage(read)]
fn get_compounded_stake_from_snapshots(initial_stake: u64, snapshots: Snapshots) -> u64 {
    let epoch_snapshot = snapshots.epoch;
    let scale_snapshot = snapshots.scale;
    let p_snapshot = snapshots.P;

    if (epoch_snapshot < storage.current_epoch.read()) {
        return 0;
    }

    let mut compounded_stake: U128 = U128::from_u64(0);
    let scale_diff = storage.current_scale.read() - scale_snapshot;

    if (scale_diff == 0) {
        compounded_stake = U128::from_u64(initial_stake) * storage.p.read() / p_snapshot;
    } else if (scale_diff == 1) {
        compounded_stake = U128::from_u64(initial_stake) * storage.p.read() / p_snapshot / U128::from_u64(SCALE_FACTOR);
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
    storage.total_usdf_deposits.write(storage.total_usdf_deposits.read() - total_usdf_to_decrease);
}

#[storage(read, write)]
fn internal_increase_asset(total_asset_to_increase: u64, asset_contract: AssetId) {
    let mut asset_amount = storage.asset.get(asset_contract).try_read().unwrap_or(0);
    asset_amount += total_asset_to_increase;
    storage.asset.insert(asset_contract, asset_amount);
}

#[storage(read, write)]
fn internal_update_deposits_and_snapshots(depositor: Identity, amount: u64) {
    storage.deposits.insert(depositor, amount);

    if (amount == 0) {
        // TODO use storage remove when available
        storage.deposit_snapshots.insert(depositor, Snapshots::default());
    }

    let current_epoch = storage.current_epoch.read();
    let current_scale = storage.current_scale.read();
    let current_p = storage.p.read();

    let current_g = storage.epoch_to_scale_to_gain.get((current_epoch, current_scale)).try_read().unwrap_or(U128::from_u64(0));

    let snapshots = Snapshots {
        epoch: current_epoch,
        scale: current_scale,
        P: current_p,
        G: current_g,
    };

    // TODO use itterator when available
    let mut i = 0;
    while i < storage.valid_assets.len() {
        let asset = storage.valid_assets.get(i).unwrap().read();
        let current_s: U128 = storage.epoch_to_scale_to_sum.get((current_epoch, current_scale, asset)).try_read().unwrap_or(U128::from_u64(0));
        storage.deposit_snapshot_s_per_asset.insert((depositor, asset), current_s);
        i += 1;
    }
    storage.deposit_snapshots.insert(depositor, snapshots);
}

#[storage(read, write), payable]
fn send_asset_gain_to_depositor(depositor: Identity, gain: u64, asset_contract: AssetId) {
    if (gain == 0) {
        return;
    }
    let mut asset_amount = storage.asset.get(asset_contract).read();
    asset_amount -= gain;
    storage.asset.insert(asset_contract, asset_amount);
    transfer(depositor, asset_contract, gain );
}

#[storage(read, write)]
fn send_usdf_to_depositor(depositor: Identity, amount: u64) {
    if (amount == 0) {
        return;
    }
    storage.total_usdf_deposits.write(storage.total_usdf_deposits.read() - amount);
    let usdf_asset_id = storage.usdf_asset_id.read();
    transfer(depositor, usdf_asset_id, amount);
}

#[storage(read)]
fn require_user_has_asset_gain(depositor: Identity, asset_contract: AssetId) {
    let gain = internal_get_depositor_asset_gain(depositor, asset_contract);
    require(gain > 0, "SP: User has no asset gain");
}

#[storage(read)]
fn require_caller_is_trove_manager() {
    let mut i = 0;
    while i < storage.valid_assets.len() {
        let asset = storage.valid_assets.get(i).unwrap().read();
        let trove_manager_contract = Identity::ContractId(storage.asset_contracts.get(asset).read().trove_manager);
        if (msg_sender().unwrap() == trove_manager_contract) {
            return;
        }
        i += 1;
    }
    require(false, "SP: Caller is not a trove manager");
}

fn require_user_has_initial_deposit(deposit: u64) {
    require(deposit > 0, "SP: User has no initial deposit");
}

#[storage(read, write)]
fn compute_rewards_per_unit_staked(
    coll_to_add: u64,
    debt_to_offset: u64,
    total_usdf_deposits: u64,
    asset_contract: AssetId,
) -> (U128, U128) {
    let asset_numerator: U128 = U128::from_u64(coll_to_add) * U128::from_u64(DECIMAL_PRECISION) + storage.last_asset_error_offset.get(asset_contract).try_read().unwrap_or(U128::from_u64(0));

    require(debt_to_offset <= total_usdf_deposits, "SP: Debt offset exceeds total USDF deposits");

    let mut usdf_loss_per_unit_staked: U128 = U128::from_u64(0);
    if (debt_to_offset == total_usdf_deposits) {
        usdf_loss_per_unit_staked = U128::from_u64(DECIMAL_PRECISION);
        storage.last_usdf_error_offset.write( U128::from_u64(0));
    } else {
        let usdf_loss_per_unit_staked_numerator: U128 = U128::from_u64(debt_to_offset) * U128::from_u64(DECIMAL_PRECISION) - storage.last_usdf_error_offset.read();
        usdf_loss_per_unit_staked = usdf_loss_per_unit_staked_numerator / U128::from_u64(total_usdf_deposits) + U128::from_u64(1);

        storage.last_usdf_error_offset.write(usdf_loss_per_unit_staked * U128::from_u64(total_usdf_deposits) - usdf_loss_per_unit_staked_numerator);
    }

    let asset_gain_per_unit_staked = asset_numerator / U128::from_u64(total_usdf_deposits);

    storage.last_asset_error_offset.insert(asset_contract, asset_numerator - (asset_gain_per_unit_staked * U128::from_u64(total_usdf_deposits)));

    return (asset_gain_per_unit_staked, usdf_loss_per_unit_staked);
}
#[storage(read, write)]
fn update_reward_sum_and_product(
    asset_gain_per_unit_staked: U128,
    usdf_loss_per_unit_staked: U128,
    asset: AssetId,
) {
    let current_p = storage.p.read();
    let mut new_p: U128 = U128::from_u64(0);
    let new_product_factor = U128::from_u64(DECIMAL_PRECISION) - usdf_loss_per_unit_staked;
    let current_epoch = storage.current_epoch.read();
    let current_scale = storage.current_scale.read();

    let current_s = storage.epoch_to_scale_to_sum.get((current_epoch, current_scale, asset)).try_read().unwrap_or(U128::from_u64(0));

    let marginal_asset_gain: U128 = asset_gain_per_unit_staked * current_p;
    let new_sum = current_s + marginal_asset_gain;

    storage.epoch_to_scale_to_sum.insert((current_epoch, current_scale, asset), new_sum);
    if (new_product_factor == U128::from_u64(0)) {
        storage.current_epoch.write(storage.current_epoch.read() + 1);
        storage.current_scale.write( 0);
        new_p = U128::from_u64(DECIMAL_PRECISION);
    } else if (current_p * new_product_factor / U128::from_u64(DECIMAL_PRECISION) < U128::from_u64(SCALE_FACTOR))
    {
        new_p = current_p * new_product_factor * U128::from_u64(SCALE_FACTOR) / U128::from_u64(DECIMAL_PRECISION);
        storage.current_scale.write(storage.current_scale.read() + 1);
    } else {
        new_p = current_p * new_product_factor / U128::from_u64(DECIMAL_PRECISION);
    }
    require(new_p > U128::from_u64(0), "SP: New p is 0");

    storage.p.write(new_p);
}

#[storage(read, write)]
fn internal_move_offset_coll_and_debt(
    coll_to_add: u64,
    debt_to_offset: u64,
    asset_contract: AssetId,
    asset_contractes_cache: AssetContracts,
) {
    let active_pool = abi(ActivePool, storage.active_pool_contract.read().value);
    let usdf_contract = abi(USDFToken, storage.usdf_contract.read().value);

    internal_decrease_usdf(debt_to_offset);
    internal_increase_asset(coll_to_add, asset_contract);
    active_pool.decrease_usdf_debt(debt_to_offset, asset_contract);

    usdf_contract.burn {
        coins: debt_to_offset,
        asset_id: storage.usdf_asset_id.read().value,
    }();
    active_pool.send_asset(Identity::ContractId(contract_id()), coll_to_add, asset_contract);
}
