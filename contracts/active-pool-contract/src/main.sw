contract;

use libraries::active_pool_interface::ActivePool;
use libraries::fluid_math::{null_contract, null_identity_address};

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
    },
    token::transfer,
};
/*
 * The Active Pool holds the Asset collateral and USDF debt (but not USDF tokens) for all active troves.
 *
 * When a trove is liquidated, it's Asset and USDF debt are transferred from the Active Pool, to either the
 * Stability Pool, the Default Pool, or both, depending on the liquidation conditions.
 *
 */
storage {
    borrow_operations_contract: Identity = null_identity_address(),
    trove_manager_contract: Identity = null_identity_address(),
    stability_pool_contract: Identity = null_identity_address(),
    default_pool_contract: ContractId = null_contract(),
    usdf_debt_amount: u64 = 0,
    is_initialized: bool = false,
    valid_assets: StorageMap<ContractId, bool> = StorageMap {},
    asset_amounts: StorageMap<ContractId, u64> = StorageMap {},
}

impl ActivePool for Contract {
    #[storage(read, write)]
    fn initialize(
        borrow_operations: Identity,
        trove_manager: Identity,
        stability_pool: Identity,
        asset_id: ContractId,
        default_pool: ContractId,
    ) {
        require(storage.is_initialized == false, "Already initialized");

        storage.borrow_operations_contract = borrow_operations;
        storage.trove_manager_contract = trove_manager;
        storage.stability_pool_contract = stability_pool;
        storage.default_pool_contract = default_pool;

        storage.is_initialized = true;
        initialize_valid_asset(asset_id);
    }

    // --- Getters for public variables. Required by Pool interface ---
    /*
    * Returns the Asset state variable.
    *
    * Not necessarily equal to the the contract's raw Asset balance - Assets can be forcibly sent to contracts.
    */
    #[storage(read)]
    fn get_asset(asset: ContractId) -> u64 {
        return storage.asset_amounts.get(asset);
    }

    #[storage(read)]
    fn get_usdf_debt() -> u64 {
        return storage.usdf_debt_amount;
    }

    // --- Pool functionality ---    
    #[storage(read, write)]
    fn send_asset(address: Identity, asset: ContractId, amount: u64) {
        require_caller_is_bo_or_tm_or_sp();

        transfer(amount, asset, address);
        let mut current_amount = storage.asset_amounts.get(asset);
        current_amount -= amount;
        storage.asset_amounts.insert(asset, current_amount);
    }

    #[storage(read, write)]
    fn increase_usdf_debt(amount: u64) {
        require_caller_is_bo_or_tm();
        storage.usdf_debt_amount += amount;
    }
    #[storage(read, write)]
    fn decrease_usdf_debt(amount: u64) {
        require_caller_is_bo_or_tm_or_sp();
        storage.usdf_debt_amount -= amount;
    }

    #[storage(read, write)]
    fn send_asset_to_default_pool(asset: ContractId, amount: u64) {
        require_caller_is_bo_or_tm_or_sp();

        let mut current_amount = storage.asset_amounts.get(asset);
        current_amount -= amount;
        storage.asset_amounts.insert(asset, current_amount);

        let dafault_pool = abi(ActivePool, storage.default_pool_contract.value);
        dafault_pool.recieve {
            coins: amount,
            asset_id: asset.value,
        }();
    }

    // --- Receive functionality ---
    // Required to record the contract's raw Asset balance since there is no FallBack function    
    #[storage(read, write), payable]
    fn recieve() {
        require_caller_is_borrow_operations_or_default_pool();
        require_is_asset_id();

        let mut current_amount = storage.asset_amounts.get(msg_asset_id());
        current_amount += msg_amount();
        storage.asset_amounts.insert(msg_asset_id(), current_amount);
    }
}

// --- Helper functions ---
#[storage(read, write)]
fn initialize_valid_asset(asset_id: ContractId) {
    storage.valid_assets.insert(asset_id, true);
    storage.asset_amounts.insert(asset_id, 0);
}

#[storage(read)]
fn require_is_asset_id() {
    let asset_id = msg_asset_id();
    let is_valid_asset = storage.valid_assets.get(asset_id);
    require(is_valid_asset == true, "Asset ID is not correct");
}
#[storage(read)]
fn require_caller_is_bo_or_tm_or_sp() {
    let caller = msg_sender().unwrap();
    let borrow_operations_contract = storage.borrow_operations_contract;
    let trove_manager_contract = storage.trove_manager_contract;
    let stability_pool_contract = storage.stability_pool_contract;
    require(caller == borrow_operations_contract || caller == trove_manager_contract || caller == stability_pool_contract, "Caller is not BorrowOperations, TroveManager or DefaultPool");
}
#[storage(read)]
fn require_caller_is_bo_or_tm() {
    let caller = msg_sender().unwrap();
    let borrow_operations_contract = storage.borrow_operations_contract;
    let trove_manager_contract = storage.trove_manager_contract;
    require(caller == borrow_operations_contract || caller == trove_manager_contract, "Caller is not BorrowOperations or TroveManager");
}

#[storage(read)]
fn require_caller_is_borrow_operations_or_default_pool() {
    let caller = msg_sender().unwrap();
    let borrow_operations_contract = storage.borrow_operations_contract;
    let default_pool_contract = storage.default_pool_contract;
    require(caller == borrow_operations_contract || caller == Identity::ContractId(default_pool_contract), "Caller is not BorrowOperations or DefaultPool");
}
