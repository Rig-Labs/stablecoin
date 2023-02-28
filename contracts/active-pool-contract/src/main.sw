contract;

use libraries::active_pool_interface::ActivePool;
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

const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

storage {
    borrow_operations_contract: Identity = Identity::ContractId(ContractId::from(ZERO_B256)),
    trove_manager_contract: Identity = Identity::ContractId(ContractId::from(ZERO_B256)),
    stability_pool_contract: Identity = Identity::ContractId(ContractId::from(ZERO_B256)),
    default_pool_contract: Identity = Identity::ContractId(ContractId::from(ZERO_B256)),
    asset_id: ContractId = ContractId::from(ZERO_B256),
    asset_amount: u64 = 0,
    usdf_debt_amount: u64 = 0,
}

impl ActivePool for Contract {
    #[storage(read, write)]
    fn send_asset(address: Identity, amount: u64) {
        require_caller_is_bo_or_tm_or_sp();
        transfer(amount, storage.asset_id, address);
        storage.asset_amount -= amount;
    }

    #[storage(read)]
    fn get_asset() -> u64 {
        return storage.asset_amount;
    }

    #[storage(read)]
    fn get_usdf_debt() -> u64 {
        return storage.usdf_debt_amount;
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
    fn recieve() {
        require_caller_is_borrow_operations_or_default_pool();
        require_is_asset_id();
        storage.asset_amount += msg_amount();
    }
}

#[storage(read)]
fn require_is_asset_id() {
    let asset_id = msg_asset_id();
    require(asset_id == storage.asset_id, "Asset ID is not correct");
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
    require(caller == borrow_operations_contract || caller == default_pool_contract, "Caller is not BorrowOperations or DefaultPool");
}
