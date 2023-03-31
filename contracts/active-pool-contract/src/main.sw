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
    protocol_manager_contract: ContractId = null_contract(),
    asset_id: ContractId = null_contract(),
    asset_amount: u64 = 0,
    usdf_debt_amount: u64 = 0,
    is_initialized: bool = false,
}

impl ActivePool for Contract {
    #[storage(read, write)]
    fn initialize(
        borrow_operations: Identity,
        trove_manager: Identity,
        stability_pool: Identity,
        asset_id: ContractId,
        default_pool: ContractId,
        protocol_manager: ContractId,
    ) {
        require(storage.is_initialized == false, "Already initialized");

        storage.borrow_operations_contract = borrow_operations;
        storage.trove_manager_contract = trove_manager;
        storage.stability_pool_contract = stability_pool;
        storage.asset_id = asset_id;
        storage.default_pool_contract = default_pool;
        storage.protocol_manager_contract = protocol_manager;
        storage.is_initialized = true;
    }

    // --- Getters for public variables. Required by Pool interface ---
    /*
    * Returns the Asset state variable.
    *
    * Not necessarily equal to the the contract's raw Asset balance - Assets can be forcibly sent to contracts.
    */
    #[storage(read)]
    fn get_asset() -> u64 {
        return storage.asset_amount;
    }

    #[storage(read)]
    fn get_usdf_debt() -> u64 {
        return storage.usdf_debt_amount;
    }

    // --- Pool functionality ---
    #[storage(read, write)]
    fn send_asset(address: Identity, amount: u64) {
        require_caller_is_bo_or_tm_or_sp_pm();
        transfer(amount, storage.asset_id, address);
        storage.asset_amount -= amount;
    }

    #[storage(read, write)]
    fn increase_usdf_debt(amount: u64) {
        require_caller_is_bo_or_tm();
        storage.usdf_debt_amount += amount;
    }

    #[storage(read, write)]
    fn decrease_usdf_debt(amount: u64) {
        require_caller_is_bo_or_tm_or_sp_pm();
        storage.usdf_debt_amount -= amount;
    }

    #[storage(read, write)]
    fn send_asset_to_default_pool(amount: u64) {
        require_caller_is_bo_or_tm_or_sp_pm();
        storage.asset_amount -= amount;
        let dafault_pool = abi(ActivePool, storage.default_pool_contract.value);

        dafault_pool.recieve {
            coins: amount,
            asset_id: storage.asset_id.value,
        }();
    }

    // --- Receive functionality ---
    // Required to record the contract's raw Asset balance since there is no FallBack function
    #[storage(read, write), payable]
    fn recieve() {
        require_caller_is_borrow_operations_or_default_pool();
        require_is_asset_id();
        storage.asset_amount += msg_amount();
    }
}

// --- Helper functions ---
#[storage(read)]
fn require_is_asset_id() {
    let asset_id = msg_asset_id();
    require(asset_id == storage.asset_id, "Asset ID is not correct");
}

#[storage(read)]
fn require_caller_is_bo_or_tm_or_sp_pm() {
    let caller = msg_sender().unwrap();
    let borrow_operations_contract = storage.borrow_operations_contract;
    let trove_manager_contract = storage.trove_manager_contract;
    let stability_pool_contract = storage.stability_pool_contract;
    let protocol_manager_contract = Identity::ContractId(storage.protocol_manager_contract);
    require(caller == protocol_manager_contract || caller == borrow_operations_contract || caller == trove_manager_contract || caller == stability_pool_contract, "Caller is not BorrowOperations, TroveManager or DefaultPool");
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
