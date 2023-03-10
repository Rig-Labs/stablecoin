contract;

use libraries::default_pool_interface::DefaultPool;
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
    trove_manager_contract: Identity = Identity::ContractId(ContractId::from(ZERO_B256)),
    active_pool: Identity = Identity::ContractId(ContractId::from(ZERO_B256)),
    asset_id: ContractId = ContractId::from(ZERO_B256),
    asset_amount: u64 = 0,
    usdf_debt_amount: u64 = 0,
}

impl DefaultPool for Contract {
    #[storage(read, write)]
    fn initialize(
        trove_manager: Identity,
        active_pool: Identity,
        asset_id: ContractId,
    ) {
        require(storage.trove_manager_contract == Identity::ContractId(ContractId::from(ZERO_B256)), "TroveManager contract is already set");
        require(storage.asset_id == ContractId::from(ZERO_B256), "Asset ID is already set");

        storage.trove_manager_contract = trove_manager;
        storage.active_pool = active_pool;
        storage.asset_id = asset_id;
    }

    #[storage(read, write)]
    fn send_asset_to_active_pool(amount: u64) {
        require_is_trove_manager();
        transfer(amount, storage.asset_id, storage.active_pool);
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
        require_is_trove_manager();
        storage.usdf_debt_amount += amount;
    }

    #[storage(read, write)]
    fn decrease_usdf_debt(amount: u64) {
        require_is_trove_manager();
        storage.usdf_debt_amount -= amount;
    }

    #[storage(read, write), payable]
    fn recieve() {
        require_is_active_pool();
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
fn require_is_trove_manager() {
    let caller = msg_sender().unwrap();
    let trove_manager_contract = storage.trove_manager_contract;
    require(caller == trove_manager_contract, "Caller is not TroveManager");
}

#[storage(read)]
fn require_is_active_pool() {
    let caller = msg_sender().unwrap();
    let active_pool_contract = storage.active_pool;
    require(caller == active_pool_contract, "Caller is not ActivePool");
}
