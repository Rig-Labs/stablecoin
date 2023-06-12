contract;

use libraries::coll_surplus_pool_interface::CollSurplusPool;
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

storage {
    is_initialized: bool = false,
    protocol_manager: Identity = null_identity_address(),
    borrow_operations: ContractId = null_contract(),
    asset_amount: StorageMap<ContractId, u64> = StorageMap {},
    balances: StorageMap<(Identity, ContractId), u64> = StorageMap {},
    valid_asset_ids: StorageMap<ContractId, bool> = StorageMap {},
    valid_trove_managers: StorageMap<Identity, bool> = StorageMap {},
}

impl CollSurplusPool for Contract {
    #[storage(read, write)]
    fn initialize(borrow_operations: ContractId, protocol_manager: Identity) {
        require(storage.is_initialized == false, "Contract is already initialized");

        storage.borrow_operations = borrow_operations;
        storage.protocol_manager = protocol_manager;
        storage.is_initialized = true;
    }

    #[storage(read, write)]
    fn add_asset(asset: ContractId, trove_manager: Identity) {
        require_is_protocol_manager();
        storage.valid_asset_ids.insert(asset, true);
        storage.valid_trove_managers.insert(trove_manager, true);
        storage.asset_amount.insert(asset, 0);
    }

    #[storage(read, write)]
    fn claim_coll(account: Identity, asset: ContractId) {
        require_is_borrow_operations();
        require_is_valid_asset_id(asset);

        let balance = storage.balances.get((account, asset));
        if balance > 0 {
            storage.balances.insert((account, asset), 0);
            let asset_amount = storage.asset_amount.get(asset);
            storage.asset_amount.insert(asset, asset_amount - balance);

            transfer(balance, asset, account);
        }
    }

    #[storage(read, write)]
    fn account_surplus(account: Identity, amount: u64, asset: ContractId) {
        require_is_trove_manager();
        require_is_valid_asset_id(asset);

        let current_asset_amount = storage.asset_amount.get(asset);
        storage.asset_amount.insert(asset, current_asset_amount + amount);

        let mut balance = storage.balances.get((account, asset));
        balance += amount;
        storage.balances.insert((account, asset), balance);
    }

    #[storage(read)]
    fn get_asset(asset: ContractId) -> u64 {
        storage.asset_amount.get(asset)
    }

    #[storage(read)]
    fn get_collateral(acount: Identity, asset: ContractId) -> u64 {
        storage.balances.get((acount, asset))
    }
}

#[storage(read)]
fn require_is_valid_asset_id(contract_id: ContractId) {
    let is_valid = storage.valid_asset_ids.get(contract_id);
    require(is_valid, "Invalid asset id");
}

#[storage(read)]
fn require_is_protocol_manager() {
    let caller = msg_sender().unwrap();
    let protocol_manager = storage.protocol_manager;
    require(caller == protocol_manager, "Caller is not ProtocolManager");
}

#[storage(read)]
fn require_is_trove_manager() {
    let caller = msg_sender().unwrap();
    let is_valid = storage.valid_trove_managers.get(caller);
    require(is_valid, "Caller is not TroveManager");
}

#[storage(read)]
fn require_is_borrow_operations() {
    let caller = msg_sender().unwrap();
    let borrow_operations = Identity::ContractId(storage.borrow_operations);
    require(caller == borrow_operations, "Caller is not TroveManager");
}
