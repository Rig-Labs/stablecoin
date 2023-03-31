contract;

dep data_structures;
use data_structures::{RedemptionTotals, SingleRedemptionValues};
use libraries::fluid_math::{null_contract, null_identity_address};
use libraries::stability_pool_interface::{StabilityPool};
use libraries::trove_manager_interface::{TroveManager};
use libraries::borrow_operations_interface::{BorrowOperations};
use libraries::sorted_troves_interface::{SortedTroves};
use libraries::active_pool_interface::{ActivePool};
use libraries::{MockOracle};
use libraries::protocol_manager_interface::{ProtocolManager};
use libraries::usdf_token_interface::{USDFToken};
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

pub struct AssetContracts {
    trove_manager: ContractId,
    active_pool: ContractId,
    coll_surplus_pool: ContractId,
    oracle: ContractId,
    sorted_troves: ContractId,
}

storage {
    admin: Identity = null_identity_address(),
    borrow_operations_contract: ContractId = null_contract(),
    usdf_token_contract: ContractId = null_contract(),
    stability_pool_contract: ContractId = null_contract(),
    asset_contracts: StorageMap<ContractId, AssetContracts> = StorageMap {},
    assets: StorageVec<ContractId> = StorageVec {},
    is_initialized: bool = false,
}

impl ProtocolManager for Contract {
    #[storage(read, write)]
    fn initialize(
        borrow_operations: ContractId,
        stability_pool: ContractId,
        usdf_token: ContractId,
        admin: Identity,
    ) {
        require(storage.is_initialized == false, "Already initialized");

        storage.admin = admin;
        storage.borrow_operations_contract = borrow_operations;
        storage.stability_pool_contract = stability_pool;
        storage.usdf_token_contract = usdf_token;
        storage.is_initialized = true;
    }

    #[storage(read, write)]
    fn register_asset(
        asset_address: ContractId,
        active_pool: ContractId,
        trove_manager: ContractId,
        coll_surplus_pool: ContractId,
        oracle: ContractId,
        sorted_troves: ContractId,
    ) {
        require_is_admin();
        let stability_pool = abi(StabilityPool, storage.stability_pool_contract.value);
        let borrow_operations = abi(BorrowOperations, storage.borrow_operations_contract.value);
        let usdf_token = abi(USDFToken, storage.usdf_token_contract.value);

        storage.asset_contracts.insert(asset_address, AssetContracts {
            trove_manager,
            active_pool,
            coll_surplus_pool,
            oracle,
            sorted_troves,
        });
        storage.assets.push(asset_address);

        borrow_operations.add_asset(asset_address, trove_manager, sorted_troves, oracle, active_pool, coll_surplus_pool);
        stability_pool.add_asset(trove_manager, active_pool, sorted_troves, asset_address, oracle);
        usdf_token.add_trove_manager(trove_manager);
    }

    #[storage(read, write)]
    fn renounce_admin() {
        require_is_admin();
        storage.admin = null_identity_address();
    }

    #[storage(read, write), payable]
    fn redeem_collateral(
        max_itterations: u64,
        max_fee_percentage: u64,
        partial_redemption_hint: u64,
        upper_partial_hint: Identity,
        lower_partial_hint: Identity,
    ) {
        // TODO Require functions
        // TODO Require bootstrap mode
        require_valid_usdf_id();
        require(msg_amount() > 0, "Redemption amount must be greater than 0");

        let asset = storage.assets.get(0).unwrap();
        let asset_contracts = storage.asset_contracts.get(asset);

        let mut totals: RedemptionTotals = RedemptionTotals::default();
        let trove_manager_contract = abi(TroveManager, asset_contracts.trove_manager.into());
        let oracle_contract = abi(MockOracle, asset_contracts.oracle.into());
        let sorted_troves_contract = abi(SortedTroves, asset_contracts.sorted_troves.into());
        let active_pool_contract = abi(ActivePool, asset_contracts.active_pool.into());
        let usdf_contract = abi(USDFToken, storage.usdf_token_contract.into());

        totals.remaining_usdf = msg_amount();
        totals.price = oracle_contract.get_price();

        totals.total_usdf_supply_at_start = trove_manager_contract.get_entire_system_debt();
        let mut current_borrower = sorted_troves_contract.get_last();

        while (current_borrower != null_identity_address() && trove_manager_contract.get_current_icr(current_borrower, totals.price) < MCR) {
            let current_trove = sorted_troves_contract.get_prev(current_borrower);
        }

        let mut remaining_itterations = max_itterations;
        while (current_borrower != null_identity_address() && totals.remaining_usdf > 0 && remaining_itterations > 0) {
            remaining_itterations -= 1;
            let next_user_to_check = sorted_troves_contract.get_prev(current_borrower);
            trove_manager_contract.apply_pending_rewards(current_borrower);
            let single_redemption = trove_manager_contract.redeem_collateral_from_trove(current_borrower, totals.remaining_usdf, totals.price, partial_redemption_hint, upper_partial_hint, lower_partial_hint);
            if (single_redemption.cancelled_partial) {
                break;
            }
            totals.total_usdf_to_redeem += single_redemption.usdf_lot;
            totals.total_asset_drawn += single_redemption.asset_lot;
            totals.remaining_usdf -= single_redemption.usdf_lot;
            current_borrower = next_user_to_check;
        }
        require(totals.total_asset_drawn > 0, "No collateral to redeem");

        trove_manager_contract.update_base_rate_from_redemption(totals.total_asset_drawn, totals.price, totals.total_usdf_supply_at_start);
        totals.asset_fee = trove_manager_contract.get_redemption_fee(totals.total_asset_drawn);
        // Consider spliting fee with person being redeemed from
        // TODO require user accepts fee
        // TODO active pool send fee to stakers
        // TODO lqty staking increase f_asset
        totals.asset_to_send_to_redeemer = totals.total_asset_drawn - totals.asset_fee;
        // TODO Send to stakers instead of oracle when implemented
        active_pool_contract.send_asset(Identity::ContractId(asset_contracts.oracle), totals.asset_fee);

        usdf_contract.burn {
            coins: totals.total_usdf_to_redeem,
            asset_id: storage.usdf_token_contract.value,
        }();

        active_pool_contract.decrease_usdf_debt(totals.total_usdf_to_redeem);
        active_pool_contract.send_asset(msg_sender().unwrap(), totals.asset_to_send_to_redeemer);
    }
}

// --- Helper functions ---
#[storage(read)]
fn require_is_admin() {
    let caller = msg_sender().unwrap();
    let admin = storage.admin;
    require(caller == admin, "Caller is not admin");
}

#[storage(read)]
fn require_valid_usdf_id() {
    require(msg_asset_id() == storage.usdf_token_contract, "Invalid asset being transfered");
}
