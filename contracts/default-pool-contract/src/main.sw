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
    protocol_manager: Identity = null_identity_address(),
    active_pool_contract: ContractId = null_contract(),
    asset_amount: StorageMap<ContractId, u64> = StorageMap {},
    usdf_debt_amount: StorageMap<ContractId, u64> = StorageMap {},
    valid_asset_ids: StorageMap<ContractId, bool> = StorageMap {},
    valid_trove_managers: StorageMap<Identity, bool> = StorageMap {},
    is_initialized: bool = false,
}

impl DefaultPool for Contract {
    #[storage(read, write)]
    fn initialize(protocol_manager: Identity, active_pool_contract: ContractId) {
        require(storage.is_initialized == false, "Contract is already initialized");

        storage.protocol_manager = protocol_manager;
        storage.active_pool_contract = active_pool_contract;
        storage.is_initialized = true;
    }

    #[storage(read, write)]
    fn send_asset_to_active_pool(amount: u64, asset_id: ContractId) {
        require_is_trove_manager();
        let active_pool = abi(ActivePool, storage.active_pool_contract.value);
        let new_amount = storage.asset_amount.get(asset_id) - amount;

        storage.asset_amount.insert(asset_id, new_amount);
        active_pool.recieve {
            coins: amount,
            asset_id: asset_id.into(),
        }();
    }

    #[storage(read, write)]
    fn add_asset(asset: ContractId, trove_manager: Identity) {
        require_is_protocol_manager();
        storage.valid_asset_ids.insert(asset, true);
        storage.valid_trove_managers.insert(trove_manager, true);
        storage.asset_amount.insert(asset, 0);
        storage.usdf_debt_amount.insert(asset, 0);
    }

    #[storage(read)]
    fn get_asset(asset_id: ContractId) -> u64 {
        return storage.asset_amount.get(asset_id);
    }

    #[storage(read)]
    fn get_usdf_debt(asset_id: ContractId) -> u64 {
        return storage.usdf_debt_amount.get(asset_id);
    }

    #[storage(read, write)]
    fn increase_usdf_debt(amount: u64, asset_id: ContractId) {
        require_is_trove_manager();
        let new_debt = storage.usdf_debt_amount.get(asset_id) + amount;
        storage.usdf_debt_amount.insert(asset_id, new_debt);
    }

    #[storage(read, write)]
    fn decrease_usdf_debt(amount: u64, asset_id: ContractId) {
        require_is_trove_manager();
        let new_debt = storage.usdf_debt_amount.get(asset_id) - amount;
        storage.usdf_debt_amount.insert(asset_id, new_debt);
    }

    #[storage(read, write), payable]
    fn recieve() {
        require_is_active_pool_contract();
        require_is_asset_id();
        let new_amount = storage.asset_amount.get(msg_asset_id()) + msg_amount();
        storage.asset_amount.insert(msg_asset_id(), new_amount);
    }
}

#[storage(read)]
fn require_is_asset_id() {
    let asset_id = msg_asset_id();
    let valid_asset_id = storage.valid_asset_ids.get(asset_id);
    require(valid_asset_id, "DP: Asset is not correct");
}

#[storage(read)]
fn require_is_trove_manager() {
    let caller = msg_sender().unwrap();
    let is_valid_trove_manager = storage.valid_trove_managers.get(caller);
    require(is_valid_trove_manager, "DP: Caller is not TM");
}

#[storage(read)]
fn require_is_active_pool_contract() {
    let caller = msg_sender().unwrap();
    let active_pool_contract_contract = Identity::ContractId(storage.active_pool_contract);
    require(caller == active_pool_contract_contract, "DP: Caller is not AP");
}

#[storage(read)]
fn require_is_protocol_manager() {
    let caller = msg_sender().unwrap();
    let protocol_manager = storage.protocol_manager;
    require(caller == protocol_manager, "DP: Caller is not PM");
}
