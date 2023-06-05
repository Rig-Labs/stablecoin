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
    trove_manager_contract: Identity = null_identity_address(),
    borrow_operations: ContractId = null_contract(),
    asset_id: ContractId = null_contract(),
    asset_amount: StorageMap<ContractId, u64> = StorageMap {},
    balances: StorageMap<(Identity, ContractId), u64> = StorageMap {},
    is_initialized: bool = false,
    valid_asset_ids: StorageMap<ContractId, bool> = StorageMap {},
}

impl CollSurplusPool for Contract {
    #[storage(read, write)]
    fn initialize(
        trove_manager: Identity,
        active_pool: ContractId,
        borrow_operations: ContractId,
        asset_id: ContractId,
    ) {
        require(storage.is_initialized == false, "Contract is already initialized");

        storage.trove_manager_contract = trove_manager;
        storage.borrow_operations = borrow_operations;
        storage.asset_id = asset_id;
        storage.is_initialized = true;
    }

    #[storage(read, write)]
    fn claim_coll(account: Identity, asset: ContractId) {
        require_is_borrow_operations();
        let balance = storage.balances.get((account, asset));
        if balance > 0 {
            storage.balances.insert((account, asset), 0);
            let asset_amount = storage.asset_amount.get(storage.asset_id);
            storage.asset_amount.insert(storage.asset_id, asset_amount - balance);

            transfer(balance, storage.asset_id, account);
        }
    }

    #[storage(read, write)]
    fn account_surplus(account: Identity, amount: u64, asset: ContractId) {
        require_is_trove_manager();

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
fn require_is_trove_manager() {
    let caller = msg_sender().unwrap();
    let trove_manager_contract = storage.trove_manager_contract;
    require(caller == trove_manager_contract, "Caller is not TroveManager");
}

#[storage(read)]
fn require_is_borrow_operations() {
    let caller = msg_sender().unwrap();
    let borrow_operations = Identity::ContractId(storage.borrow_operations);
    require(caller == borrow_operations, "Caller is not TroveManager");
}
