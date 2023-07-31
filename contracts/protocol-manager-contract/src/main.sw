contract;

dep data_structures;
use data_structures::{AssetContracts, AssetInfo, RedemptionTotals, SingleRedemptionValues};
use libraries::fluid_math::{null_contract, null_identity_address};
use libraries::stability_pool_interface::{StabilityPool};
use libraries::trove_manager_interface::{TroveManager};
use libraries::borrow_operations_interface::{BorrowOperations};
use libraries::sorted_troves_interface::{SortedTroves};
use libraries::active_pool_interface::{ActivePool};
use libraries::default_pool_interface::{DefaultPool};
use libraries::coll_surplus_pool_interface::{CollSurplusPool};
use libraries::{MockOracle};
use libraries::protocol_manager_interface::{ProtocolManager};
use libraries::usdf_token_interface::{USDFToken};
use libraries::fpt_staking_interface::{FPTStaking};
use libraries::fluid_math::*;

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
};

storage {
    admin: Identity = null_identity_address(),
    borrow_operations_contract: ContractId = null_contract(),
    fpt_staking_contract: ContractId = null_contract(),
    usdf_token_contract: ContractId = null_contract(),
    stability_pool_contract: ContractId = null_contract(),
    coll_surplus_pool_contract: ContractId = null_contract(),
    default_pool_contract: ContractId = null_contract(),
    active_pool_contract: ContractId = null_contract(),
    sorted_troves_contract: ContractId = null_contract(),
    asset_contracts: StorageMap<ContractId, AssetContracts> = StorageMap {},
    assets: StorageVec<ContractId> = StorageVec {},
    is_initialized: bool = false,
}

impl ProtocolManager for Contract {
    #[storage(read, write)]
    fn initialize(
        borrow_operations: ContractId,
        stability_pool: ContractId,
        fpt_staking: ContractId,
        usdf_token: ContractId,
        coll_surplus_pool: ContractId,
        default_pool: ContractId,
        active_pool: ContractId,
        sorted_troves: ContractId,
        admin: Identity,
    ) {
        require(storage.is_initialized == false, "PM: Already initialized");

        storage.admin = admin;
        storage.borrow_operations_contract = borrow_operations;
        storage.fpt_staking_contract = fpt_staking;
        storage.stability_pool_contract = stability_pool;
        storage.usdf_token_contract = usdf_token;
        storage.coll_surplus_pool_contract = coll_surplus_pool;
        storage.default_pool_contract = default_pool;
        storage.active_pool_contract = active_pool;
        storage.sorted_troves_contract = sorted_troves;
        storage.is_initialized = true;
    }

    #[storage(read, write)]
    fn register_asset(
        asset_address: ContractId,
        trove_manager: ContractId,
        oracle: ContractId,
    ) {
        require_is_admin();
        let stability_pool = abi(StabilityPool, storage.stability_pool_contract.value);
        let borrow_operations = abi(BorrowOperations, storage.borrow_operations_contract.value);
        let usdf_token = abi(USDFToken, storage.usdf_token_contract.value);
        let fpt_staking = abi(FPTStaking, storage.fpt_staking_contract.value);
        let coll_surplus_pool = abi(CollSurplusPool, storage.coll_surplus_pool_contract.value);
        let default_pool = abi(DefaultPool, storage.default_pool_contract.value);
        let active_pool = abi(ActivePool, storage.active_pool_contract.value);
        let sorted_troves = abi(SortedTroves, storage.sorted_troves_contract.value);

        storage.asset_contracts.insert(asset_address, AssetContracts {
            trove_manager,
            oracle,
            asset_address,
        });
        storage.assets.push(asset_address);

        borrow_operations.add_asset(asset_address, trove_manager, oracle);
        coll_surplus_pool.add_asset(asset_address, Identity::ContractId(trove_manager));
        active_pool.add_asset(asset_address, Identity::ContractId(trove_manager));
        default_pool.add_asset(asset_address, Identity::ContractId(trove_manager));
        stability_pool.add_asset(trove_manager, asset_address, oracle);
        sorted_troves.add_asset(asset_address, trove_manager);
        fpt_staking.add_asset(asset_address);
        usdf_token.add_trove_manager(trove_manager);
    }

    #[storage(read, write)]
    fn renounce_admin() {
        require_is_admin();
        storage.admin = null_identity_address();
    }

    #[storage(read), payable]
    fn redeem_collateral(
        max_itterations: u64,
        partial_redemption_hint: u64,
        upper_partial_hint: Identity,
        lower_partial_hint: Identity,
    ) {
        // TODO Require functions
        // TODO Require bootstrap mode
        require_valid_usdf_id();
        require(msg_amount() > 0, "Redemption amount must be greater than 0");
        let usdf_contract_cache = storage.usdf_token_contract;
        let fpt_staking_contract_cache = storage.fpt_staking_contract;

        let usdf = abi(USDFToken, usdf_contract_cache.value);
        let sorted_troves = abi(SortedTroves, storage.sorted_troves_contract.value);
        let active_pool = abi(ActivePool, storage.active_pool_contract.value);
        let fpt_staking = abi(FPTStaking, fpt_staking_contract_cache.value);

        let mut assets_info = get_all_assets_info();
        let mut remaining_usdf = msg_amount();

        let (mut current_borrower, mut index) = find_min_borrower(assets_info.current_borrowers, assets_info.current_crs);

        let mut remaining_itterations = max_itterations;
        while (current_borrower != null_identity_address() && remaining_usdf > 0 && remaining_itterations > 0) {
            let contracts_cache = assets_info.asset_contracts.get(index).unwrap();
            let trove_manager_contract = abi(TroveManager, contracts_cache.trove_manager.value);

            let price = assets_info.prices.get(index).unwrap();
            let mut totals = assets_info.redemption_totals.get(index).unwrap();

            remaining_itterations -= 1;

            let next_user_to_check = sorted_troves.get_prev(current_borrower, contracts_cache.asset_address);
            trove_manager_contract.apply_pending_rewards(current_borrower);

            let single_redemption = trove_manager_contract.redeem_collateral_from_trove(current_borrower, remaining_usdf, price, partial_redemption_hint, upper_partial_hint, lower_partial_hint);
            if (single_redemption.cancelled_partial) {
                break;
            }

            totals.total_usdf_to_redeem += single_redemption.usdf_lot;
            totals.total_asset_drawn += single_redemption.asset_lot;
            remaining_usdf -= single_redemption.usdf_lot;

            let next_cr = trove_manager_contract.get_current_icr(next_user_to_check, price);
            assets_info.current_crs.set(index, next_cr);
            assets_info.current_borrowers.set(index, next_user_to_check);
            assets_info.redemption_totals.set(index, totals);

            let next_borrower = find_min_borrower(assets_info.current_borrowers, assets_info.current_crs);
            current_borrower = next_borrower.0;
            index = next_borrower.1;
        }

        let mut total_usdf_redeemed = 0;
        let mut ind = 0;
        while (ind < assets_info.assets.len()) {
            let contracts_cache = assets_info.asset_contracts.get(ind).unwrap();
            let trove_manager_contract = abi(TroveManager, contracts_cache.trove_manager.value);

            let price = assets_info.prices.get(ind).unwrap();
            let mut totals = assets_info.redemption_totals.get(ind).unwrap();

            let total_usdf_supply_at_start = usdf.total_supply();
            if (totals.total_usdf_to_redeem == 0) {
                ind += 1;
                continue;
            }

            totals.asset_fee = fm_compute_redemption_fee(totals.total_asset_drawn); 
            // TODO require user accepts fee
            totals.asset_to_send_to_redeemer = totals.total_asset_drawn - totals.asset_fee;
            // Send to stakers instead of oracle when implemented
            active_pool.send_asset(Identity::ContractId(fpt_staking_contract_cache), totals.asset_fee, contracts_cache.asset_address);
            fpt_staking.increase_f_asset(totals.asset_fee, assets_info.assets.get(ind).unwrap());

            total_usdf_redeemed += totals.total_usdf_to_redeem;
            active_pool.decrease_usdf_debt(totals.total_usdf_to_redeem, contracts_cache.asset_address);
            active_pool.send_asset(msg_sender().unwrap(), totals.asset_to_send_to_redeemer, contracts_cache.asset_address);
            ind += 1;
        }

        usdf.burn {
            coins: total_usdf_redeemed,
            asset_id: storage.usdf_token_contract.value,
        }();

        if (remaining_usdf > 0) {
            // Return remaining usdf to redeemer
            transfer(remaining_usdf, usdf_contract_cache, msg_sender().unwrap());
        }
    }
}

// --- Helper functions ---
#[storage(read)]
fn require_is_admin() {
    let caller = msg_sender().unwrap();
    let admin = storage.admin;
    require(caller == admin, "PM: Caller is not admin");
}

#[storage(read)]
fn require_valid_usdf_id() {
    require(msg_asset_id() == storage.usdf_token_contract, "PM: Invalid asset being transfered");
}

#[storage(read)]
fn get_all_assets_info() -> AssetInfo {
    let mut assets: Vec<ContractId> = Vec::new();
    let mut asset_contracts: Vec<AssetContracts> = Vec::new();
    let mut prices: Vec<u64> = Vec::new();
    let mut system_debt: Vec<u64> = Vec::new();
    let mut redemption_totals: Vec<RedemptionTotals> = Vec::new();
    let mut current_borrowers: Vec<Identity> = Vec::new();
    let mut current_crs: Vec<u64> = Vec::new();
    let sorted_troves = abi(SortedTroves, storage.sorted_troves_contract.value);
    let length = storage.assets.len();

    let mut ind = 0;
    while (ind < length) {
        assets.push(storage.assets.get(ind).unwrap());
        asset_contracts.push(storage.asset_contracts.get(assets.get(ind).unwrap()));
        ind += 1;
    }

    let mut i = 0;
    while (i < length) {
        let oracle = abi(MockOracle, asset_contracts.get(i).unwrap().oracle.into());
        let trove_manager = abi(TroveManager, asset_contracts.get(i).unwrap().trove_manager.into());

        let asset = assets.get(i).unwrap();
        let price = oracle.get_price();
        let mut current_borrower = sorted_troves.get_last(asset);
        let mut current_cr = trove_manager.get_current_icr(current_borrower, price);

        prices.push(price);
        system_debt.push(trove_manager.get_entire_system_debt());
        redemption_totals.push(RedemptionTotals::default());

        while (current_borrower != null_identity_address() && current_cr < MCR) {
            current_borrower = sorted_troves.get_prev(current_borrower, asset);
            current_cr = trove_manager.get_current_icr(current_borrower, price);
        }

        current_borrowers.push(current_borrower);
        current_crs.push(current_cr);
        i += 1;
    }

    AssetInfo {
        assets: assets,
        asset_contracts: asset_contracts,
        prices: prices,
        system_debts: system_debt,
        redemption_totals: redemption_totals,
        current_borrowers: current_borrowers,
        current_crs: current_crs,
    }
}

// TODO write comments
fn find_min_borrower(current_borrowers: Vec<Identity>, current_crs: Vec<u64>) -> (Identity, u64) {
    let mut min_borrower = current_borrowers.get(0).unwrap();
    let mut min_cr = current_crs.get(0).unwrap();
    let mut min_index = 0;
    let mut i = 1;
    while (i < current_borrowers.len()) {
        if (current_crs.get(i).unwrap() < min_cr) {
            min_borrower = current_borrowers.get(i).unwrap();
            min_cr = current_crs.get(i).unwrap();
            min_index = i;
        }
        i += 1;
    }
    (min_borrower, min_index)
}
