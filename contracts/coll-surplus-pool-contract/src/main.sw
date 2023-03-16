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
    active_pool: ContractId = null_contract(),
    borrow_operations: ContractId = null_contract(),
    asset_id: ContractId = null_contract(),
    asset_amount: u64 = 0,
    balances: StorageMap<Identity, u64> = StorageMap {},
}

impl CollSurplusPool for Contract {
    #[storage(read, write)]
    fn initialize(
        trove_manager: Identity,
        active_pool: ContractId,
        borrow_operations: ContractId,
        asset_id: ContractId,
    ) {
        require(storage.trove_manager_contract == null_identity_address(), "TroveManager contract is already set");
        require(storage.asset_id == null_contract(), "Asset ID is already set");

        storage.trove_manager_contract = trove_manager;
        storage.borrow_operations = borrow_operations;
        storage.active_pool = active_pool;
        storage.asset_id = asset_id;
    }

    #[storage(read, write)]
    fn claim_coll(account: Identity) {
        require_is_borrow_operations();
        let balance = storage.balances.get(account);
        if balance > 0 {
            storage.balances.insert(account, 0);
            storage.asset_amount -= balance;

            transfer(balance, storage.asset_id, account);
        }
    }

    #[storage(read)]
    fn get_asset() -> u64 {
        storage.asset_amount
    }

    #[storage(read)]
    fn get_collateral(acount: Identity) -> u64 {
        storage.balances.get(acount)
    }

    #[storage(read, write)]
    fn account_surplus(account: Identity, amount: u64) {
        require_is_trove_manager();
        storage.asset_amount += amount;

        let mut balance = storage.balances.get(account);
        balance += amount;
        storage.balances.insert(account, balance);
    }
}

#[storage(read)]
fn require_is_asset_id() {
    let asset_id = msg_asset_id();
    require(asset_id == storage.asset_id, "Asset ID is not correct");
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
