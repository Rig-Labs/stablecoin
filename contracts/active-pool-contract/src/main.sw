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
    stability_pool_contract: Identity = null_identity_address(),
    default_pool_contract: ContractId = null_contract(),
    protocol_manager_contract: Identity = null_identity_address(),
    asset_amount: StorageMap<ContractId, u64> = StorageMap {},
    usdf_debt_amount: StorageMap<ContractId, u64> = StorageMap {},
    valid_asset_ids: StorageMap<ContractId, bool> = StorageMap {},
    valid_trove_managers: StorageMap<Identity, bool> = StorageMap {},
    is_initialized: bool = false,
}

impl ActivePool for Contract {
    #[storage(read, write)]
    fn initialize(
        borrow_operations: Identity,
        stability_pool: Identity,
        default_pool: ContractId,
        protocol_manager: Identity,
    ) {
        require(storage.is_initialized == false, "Already initialized");

        storage.borrow_operations_contract = borrow_operations;
        storage.stability_pool_contract = stability_pool;
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
    fn get_asset(asset_id: ContractId) -> u64 {
        return storage.asset_amount.get(asset_id);
    }

    #[storage(read)]
    fn get_usdf_debt(asset_id: ContractId) -> u64 {
        return storage.usdf_debt_amount.get(asset_id);
    }

    // --- Support multiple assets functionality ---
    #[storage(read, write)]
    fn add_asset(asset: ContractId, trove_manager: Identity) {
        require_is_protocol_manager();
        storage.valid_asset_ids.insert(asset, true);
        storage.valid_trove_managers.insert(trove_manager, true);
        storage.asset_amount.insert(asset, 0);
        storage.usdf_debt_amount.insert(asset, 0);
    }

    // --- Pool functionality ---
    #[storage(read, write)]
    fn send_asset(address: Identity, amount: u64, asset_id: ContractId) {
        require_caller_is_bo_or_tm_or_sp_or_pm();
        let new_amount = storage.asset_amount.get(asset_id) - amount;
        storage.asset_amount.insert(asset_id, new_amount);
        transfer(amount, asset_id, address);
    }

    #[storage(read, write)]
    fn increase_usdf_debt(amount: u64, asset_id: ContractId) {
        require_caller_is_bo_or_tm();
        let new_debt = storage.usdf_debt_amount.get(asset_id) + amount;
        storage.usdf_debt_amount.insert(asset_id, new_debt);
    }

    #[storage(read, write)]
    fn decrease_usdf_debt(amount: u64, asset_id: ContractId) {
        require_caller_is_bo_or_tm_or_sp_or_pm();
        let new_debt = storage.usdf_debt_amount.get(asset_id) - amount;
        storage.usdf_debt_amount.insert(asset_id, new_debt);
    }

    #[storage(read, write)]
    fn send_asset_to_default_pool(amount: u64, asset_id: ContractId) {
        require_caller_is_bo_or_tm_or_sp_or_pm();
        let new_amount = storage.asset_amount.get(asset_id) - amount;
        storage.asset_amount.insert(asset_id, new_amount);
        let dafault_pool = abi(ActivePool, storage.default_pool_contract.value);

        dafault_pool.recieve {
            coins: amount,
            asset_id: asset_id.value,
        }();
    }

    // --- Receive functionality ---
    // Required to record the contract's raw Asset balance since there is no FallBack function
    #[storage(read, write), payable]
    fn recieve() {
        require_caller_is_borrow_operations_or_default_pool();
        let asset_id = msg_asset_id();
        require_is_asset_id(asset_id);
        let new_amount = storage.asset_amount.get(asset_id) + msg_amount();
        storage.asset_amount.insert(asset_id, new_amount);
    }
}

// --- Helper functions ---
#[storage(read)]
fn require_is_asset_id(asset_id: ContractId) {
    let valid_asset_id = storage.valid_asset_ids.get(asset_id);
    require(valid_asset_id, "Asset ID is not correct");
}

#[storage(read)]
fn require_caller_is_bo_or_tm_or_sp_or_pm() {
    let caller = msg_sender().unwrap();
    let borrow_operations_contract = storage.borrow_operations_contract;
    let valid_trove_manager = storage.valid_trove_managers.get(caller);
    let stability_pool_contract = storage.stability_pool_contract;
    let protocol_manager_contract = storage.protocol_manager_contract;
    require(caller == protocol_manager_contract || caller == borrow_operations_contract || valid_trove_manager || caller == stability_pool_contract, "Caller is not BorrowOperations, TroveManager, ProtocolManager, or DefaultPool");
}

#[storage(read)]
fn require_caller_is_bo_or_tm() {
    let caller = msg_sender().unwrap();
    let borrow_operations_contract = storage.borrow_operations_contract;
    let valid_trove_manager = storage.valid_trove_managers.get(caller);

    require(caller == borrow_operations_contract || valid_trove_manager, "Caller is not BorrowOperations or TroveManager");
}

#[storage(read)]
fn require_caller_is_borrow_operations_or_default_pool() {
    let caller = msg_sender().unwrap();
    let borrow_operations_contract = storage.borrow_operations_contract;
    let default_pool_contract = Identity::ContractId(storage.default_pool_contract);
    require(caller == borrow_operations_contract || caller == default_pool_contract, "Caller is not BorrowOperations or DefaultPool");
}

#[storage(read)]
fn require_is_protocol_manager() {
    let caller = msg_sender().unwrap();
    let protocol_manager = storage.protocol_manager_contract;
    require(caller == protocol_manager, "Caller is not ProtocolManager");
}
