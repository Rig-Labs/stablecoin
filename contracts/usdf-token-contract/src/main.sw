contract;

use libraries::usdf_token_interface::{TokenInitializeConfig, USDFToken};
use libraries::fluid_math::{null_contract, null_identity_address};

use std::{
    address::*,
    auth::{
        AuthError,
        msg_sender,
    },
    call_frames::{
        contract_id,
        msg_asset_id,
    },
    context::{
        balance_of,
        msg_amount,
    },
    contract_id::ContractId,
    identity::{
        Identity,
    },
    revert::require,
    storage::*,
    token::*,
};

storage {
    config: TokenInitializeConfig = TokenInitializeConfig {
        name: "                                ",
        symbol: "        ",
        decimals: 1u8,
    },
    trove_managers: StorageVec<ContractId> = StorageVec {},
    protocol_manager: ContractId = null_contract(),
    stability_pool: Identity = null_identity_address(),
    borrower_operations: Identity = null_identity_address(),
    total_supply: u64 = 0,
    is_initialized: bool = false,
}

enum Error {
    NotAuthorized: (),
}

impl USDFToken for Contract {//////////////////////////////////////
    // Owner methods
    //////////////////////////////////////
    #[storage(read, write)]
    fn initialize(
        config: TokenInitializeConfig,
        protocol_manager: ContractId,
        stability_pool: Identity,
        borrower_operations: Identity,
    ) {
        require(storage.is_initialized == false, "Contract is already initialized");
        storage.stability_pool = stability_pool;
        storage.protocol_manager = protocol_manager;
        storage.borrower_operations = borrower_operations;
        storage.config = config;
        storage.is_initialized = true;
    }

    #[storage(read, write)]
    fn mint(amount: u64, address: Identity) {
        require_caller_is_borrower_operations();
        mint_to(amount, address);
        storage.total_supply += amount;
    }

    #[storage(read, write), payable]
    fn burn() {
        require_caller_is_bo_or_tm_or_sp_or_pm();
        let burn_amount = msg_amount();
        burn(burn_amount);
        storage.total_supply -= burn_amount;
    }

    #[storage(read, write)]
    fn add_trove_manager(trove_manager: ContractId) {
        require_caller_is_protocol_manager();
        storage.trove_managers.push(trove_manager);
    }

    //////////////////////////////////////
    // Read-Only methods
    //////////////////////////////////////#[storage(read)]
    #[storage(read)]
    fn total_supply() -> u64 {
        storage.total_supply
    }

    #[storage(read)]
    fn config() -> TokenInitializeConfig {
        storage.config
    }
}

#[storage(read)]
fn require_caller_is_protocol_manager() {
    require(msg_sender().unwrap() == Identity::ContractId(storage.protocol_manager), Error::NotAuthorized);
}

#[storage(read)]
fn require_caller_is_borrower_operations() {
    require(msg_sender().unwrap() == storage.borrower_operations, Error::NotAuthorized);
}

#[storage(read)]
fn require_caller_is_bo_or_tm_or_sp_or_pm() {
    let sender = msg_sender().unwrap();
    let protocol_manager_id = Identity::ContractId(storage.protocol_manager);
    
    let mut i = 0;
    while i < storage.trove_managers.len() {
        let manager = Identity::ContractId(storage.trove_managers.get(i).unwrap());
        if manager == sender {
            return
        }
        i += 1;
    }
    require(sender == storage.borrower_operations || sender == storage.stability_pool || sender == protocol_manager_id, Error::NotAuthorized);
}
