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
    storage::{
        StorageMap,
    },
    token::transfer,
};

storage {
    trove_manager_contract: Identity = null_identity_address(),
    active_pool: ContractId = null_contract(),
    borrow_operations: ContractId = null_contract(),
    is_initialized: bool = false,
    balances: StorageMap<(Identity, ContractId), u64> = StorageMap {},
    valid_assets: StorageMap<ContractId, bool> = StorageMap {},
    asset_amounts: StorageMap<ContractId, u64> = StorageMap {},
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
        storage.active_pool = active_pool;
        storage.is_initialized = true;

        initialize_valid_asset(asset_id);
    }

    #[storage(read, write)]
    fn claim_coll(account: Identity, asset_id: ContractId) {
        require_is_borrow_operations();
        let balance = storage.balances.get((account, asset_id));
        if balance > 0 {
            storage.balances.insert((account, asset_id), 0);

            let mut asset_amount = storage.asset_amounts.get(asset_id);
            asset_amount -= balance;
            storage.asset_amounts.insert(asset_id, asset_amount);

            transfer(balance, asset_id, account);
        }
    }

    #[storage(read)]
    fn get_asset(asset: ContractId) -> u64 {
        storage.asset_amounts.get(asset)
    }

    #[storage(read)]
    fn get_collateral(acount: Identity, asset: ContractId) -> u64 {
        storage.balances.get((acount, asset))
    }

    #[storage(read, write)]
    fn account_surplus(account: Identity, asset_id: ContractId, amount: u64) {
        require_is_trove_manager();
        let mut asset_amount = storage.asset_amounts.get(asset_id);
        asset_amount += amount;
        storage.asset_amounts.insert(asset_id, asset_amount);

        let mut balance = storage.balances.get((account, asset_id));
        balance += amount;
        storage.balances.insert((account, asset_id), balance);
    }
}

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
