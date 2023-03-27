contract;

use libraries::default_pool_interface::DefaultPool;
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
 * The Default Pool holds the Asset and USDF debt (but not USDF tokens) from liquidations that have been redistributed
 * to active troves but not yet "applied", i.e. not yet recorded on a recipient active trove's struct.
 *
 * When a trove makes an operation that applies its pending Asset and USDF debt, its pending Asset and USDF debt is moved
 * from the Default Pool to the Active Pool.
 */
storage {
    trove_manager_contract: Identity = null_identity_address(),
    active_pool: ContractId = null_contract(),
    usdf_debt_amount: u64 = 0,
    is_initialized: bool = false,
    valid_assets: StorageMap<ContractId, bool> = StorageMap {},
    asset_amounts: StorageMap<ContractId, u64> = StorageMap {},
}

impl DefaultPool for Contract {
    #[storage(read, write)]
    fn initialize(
        trove_manager: Identity,
        active_pool: ContractId,
        asset_id: ContractId,
    ) {
        require(storage.is_initialized == false, "Contract is already initialized");

        storage.trove_manager_contract = trove_manager;
        storage.active_pool = active_pool;
        storage.is_initialized = true;

        initialize_valid_asset(asset_id);
    }

    #[storage(read, write)]
    fn send_asset_to_active_pool(amount: u64, asset_id: ContractId) {
        require_is_trove_manager();
        let mut asset_amount = storage.asset_amounts.get(asset_id);
        asset_amount -= amount;
        storage.asset_amounts.insert(asset_id, asset_amount);

        let active_pool = abi(ActivePool, storage.active_pool.value);
        active_pool.recieve {
            coins: amount,
            asset_id: asset_id.value,
        }();
    }

    #[storage(read)]
    fn get_asset(asset_id: ContractId) -> u64 {
        return storage.asset_amounts.get(asset_id);
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

        let asset_id = msg_asset_id();

        let mut asset_amount = storage.asset_amounts.get(asset_id);
        asset_amount += msg_amount();
        storage.asset_amounts.insert(asset_id, asset_amount);
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
fn require_is_active_pool() {
    let caller = msg_sender().unwrap();
    let active_pool_contract = storage.active_pool;
    require(caller == Identity::ContractId(active_pool_contract), "Caller is not ActivePool");
}
