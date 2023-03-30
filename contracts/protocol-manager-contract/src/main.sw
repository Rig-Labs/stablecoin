contract;

use libraries::fluid_math::{null_contract, null_identity_address};
use libraries::stability_pool_interface::{StabilityPool};
use libraries::borrow_operations_interface::{BorrowOperations};
use libraries::protocol_manager_interface::{ProtocolManager};
use libraries::usdf_token_interface::{USDFToken};

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
    admin: Identity = null_identity_address(),
    borrow_operations_contract: ContractId = null_contract(),
    usdf_token_contract: ContractId = null_contract(),
    stability_pool_contract: ContractId = null_contract(),
    is_initialized: bool = false,
}

impl ProtocolManager for Contract {
    #[storage(read, write)]
    fn initialize(
        borrow_operations: ContractId,
        stability_pool: ContractId,
        usdf_token: ContractId,
        admin: Identity,
    ) {
        require(storage.is_initialized == false, "Already initialized");

        storage.admin = admin;
        storage.borrow_operations_contract = borrow_operations;
        storage.stability_pool_contract = stability_pool;
        storage.usdf_token_contract = usdf_token;
        storage.is_initialized = true;
    }

    #[storage(read, write)]
    fn register_asset(
        asset_address: ContractId,
        active_pool: ContractId,
        trove_manager: ContractId,
        coll_surplus_pool: ContractId,
        oracle: ContractId,
        sorted_troves: ContractId,
    ) {
        require_is_admin();
        let stability_pool = abi(StabilityPool, storage.stability_pool_contract.value);
        let borrow_operations = abi(BorrowOperations, storage.borrow_operations_contract.value);
        let usdf_token = abi(USDFToken, storage.usdf_token_contract.value);

        borrow_operations.add_asset(asset_address, trove_manager, sorted_troves, oracle, active_pool, coll_surplus_pool);
        stability_pool.add_asset(trove_manager, active_pool, sorted_troves, asset_address, oracle);
        usdf_token.add_trove_manager(trove_manager);
    }

    #[storage(read, write)]
    fn renounce_admin() {
        require_is_admin();
        storage.admin = null_identity_address();
    }
}

// --- Helper functions ---
#[storage(read)]
fn require_is_admin() {
    let caller = msg_sender().unwrap();
    let admin = storage.admin;
    require(caller == admin, "Caller is not admin");
}
