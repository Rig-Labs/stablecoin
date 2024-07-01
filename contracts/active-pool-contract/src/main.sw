contract;

use libraries::active_pool_interface::ActivePool;
use libraries::fluid_math::ZERO_B256;
use std::{
    asset::transfer,
    auth::msg_sender,
    call_frames::{
        msg_asset_id,
    },
    context::{
        msg_amount,
    },
    hash::Hash,
    logging::log,
};
storage {
    borrow_operations_contract: Identity = Identity::Address(Address::from(ZERO_B256)),
    stability_pool_contract: Identity = Identity::Address(Address::from(ZERO_B256)),
    default_pool_contract: ContractId = ContractId::from(ZERO_B256),
    protocol_manager_contract: Identity = Identity::Address(Address::from(ZERO_B256)),
    asset_amount: StorageMap<AssetId, u64> = StorageMap::<AssetId, u64> {},
    usdf_debt_amount: StorageMap<AssetId, u64> = StorageMap::<AssetId, u64> {},
    valid_asset_ids: StorageMap<AssetId, bool> = StorageMap::<AssetId, bool> {},
    valid_trove_managers: StorageMap<Identity, bool> = StorageMap::<Identity, bool> {},
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
        require(
            storage
                .is_initialized
                .read() == false,
            "Already initialized",
        );
        storage.borrow_operations_contract.write(borrow_operations);
        storage.stability_pool_contract.write(stability_pool);
        storage.default_pool_contract.write(default_pool);
        storage.protocol_manager_contract.write(protocol_manager);
        storage.is_initialized.write(true);
    }
    // --- Getters for public variables. Required by Pool interface ---
    /*
    * Returns the Asset state variable.
    *
    * Not necessarily equal to the the contract's raw Asset balance - Assets can be forcibly sent to contracts.
    */
    #[storage(read)]
    fn get_asset(asset_id: AssetId) -> u64 {
        return storage.asset_amount.get(asset_id).try_read().unwrap_or(0);
    }
    #[storage(read)]
    fn get_usdf_debt(asset_id: AssetId) -> u64 {
        return storage.usdf_debt_amount.get(asset_id).try_read().unwrap_or(0);
    }
    // --- Support multiple assets functionality ---
    #[storage(read, write)]
    fn add_asset(asset: AssetId, trove_manager: Identity) {
        require_is_protocol_manager();
        storage.valid_asset_ids.insert(asset, true);
        storage.valid_trove_managers.insert(trove_manager, true);
        storage.asset_amount.insert(asset, 0);
        storage.usdf_debt_amount.insert(asset, 0);
    }
    // --- Pool functionality ---
    #[storage(read, write)]
    fn send_asset(address: Identity, amount: u64, asset_id: AssetId) {
        require_caller_is_bo_or_tm_or_sp_or_pm();
        let new_amount = storage.asset_amount.get(asset_id).read() - amount;
        storage.asset_amount.insert(asset_id, new_amount);
        transfer(address, asset_id, amount);
    }
    #[storage(read, write)]
    fn increase_usdf_debt(amount: u64, asset_id: AssetId) {
        require_caller_is_bo_or_tm();
        let new_debt = storage.usdf_debt_amount.get(asset_id).try_read().unwrap_or(0) + amount;
        storage.usdf_debt_amount.insert(asset_id, new_debt);
    }
    #[storage(read, write)]
    fn decrease_usdf_debt(amount: u64, asset_id: AssetId) {
        require_caller_is_bo_or_tm_or_sp_or_pm();
        let new_debt = storage.usdf_debt_amount.get(asset_id).read() - amount;
        storage.usdf_debt_amount.insert(asset_id, new_debt);
    }
    #[storage(read, write)]
    fn send_asset_to_default_pool(amount: u64, asset_id: AssetId) {
        require_caller_is_bo_or_tm_or_sp_or_pm();
        let new_amount = storage.asset_amount.get(asset_id).read() - amount;
        storage.asset_amount.insert(asset_id, new_amount);
        let dafault_pool = abi(ActivePool, storage.default_pool_contract.read().bits());
        dafault_pool
            .recieve {
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
        let new_amount = storage.asset_amount.get(asset_id).try_read().unwrap_or(0) + msg_amount();
        storage.asset_amount.insert(asset_id, new_amount);
    }
}
// --- Helper functions ---
#[storage(read)]
fn require_is_asset_id(asset_id: AssetId) {
    let valid_asset_id = storage.valid_asset_ids.get(asset_id).try_read().unwrap_or(false);
    require(valid_asset_id, "Asset ID is not correct");
}
#[storage(read)]
fn require_caller_is_bo_or_tm_or_sp_or_pm() {
    let caller = msg_sender().unwrap();
    let borrow_operations_contract = storage.borrow_operations_contract.read();
    let valid_trove_manager = storage.valid_trove_managers.get(caller).try_read().unwrap_or(false);
    let stability_pool_contract = storage.stability_pool_contract.read();
    let protocol_manager_contract = storage.protocol_manager_contract.read();
    require(
        caller == protocol_manager_contract || caller == borrow_operations_contract || valid_trove_manager || caller == stability_pool_contract,
        "Caller is not BorrowOperations, TroveManager, ProtocolManager, or DefaultPool",
    );
}
#[storage(read)]
fn require_caller_is_bo_or_tm() {
    let caller = msg_sender().unwrap();
    let borrow_operations_contract = storage.borrow_operations_contract.read();
    let valid_trove_manager = storage.valid_trove_managers.get(caller).try_read().unwrap_or(false);
    require(
        caller == borrow_operations_contract || valid_trove_manager,
        "Caller is not BorrowOperations or TroveManager",
    );
}
#[storage(read)]
fn require_caller_is_borrow_operations_or_default_pool() {
    let caller = msg_sender().unwrap();
    let borrow_operations_contract = storage.borrow_operations_contract.read();
    let default_pool_contract = Identity::ContractId(storage.default_pool_contract.read());
    require(
        caller == borrow_operations_contract || caller == default_pool_contract,
        "Caller is not BorrowOperations or DefaultPool",
    );
}
#[storage(read)]
fn require_is_protocol_manager() {
    let caller = msg_sender().unwrap();
    let protocol_manager = storage.protocol_manager_contract.read();
    require(caller == protocol_manager, "Caller is not ProtocolManager");
}
