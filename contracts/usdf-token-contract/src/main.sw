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
    trove_manager: Identity = null_identity_address(),
    stability_pool: Identity = null_identity_address(),
    borrower_operations: Identity = null_identity_address(),
    total_supply: u64 = 0,
}

enum Error {
    CannotReinitialize: (),
    NotAuthorized: (),
}

impl USDFToken for Contract {//////////////////////////////////////
    // Owner methods
    //////////////////////////////////////
    #[storage(read, write)]
    fn initialize(
        config: TokenInitializeConfig,
        trove_manager: Identity,
        stability_pool: Identity,
        borrower_operations: Identity,
    ) {
        require(storage.trove_manager == null_identity_address(), Error::CannotReinitialize);
        require(storage.stability_pool == null_identity_address(), Error::CannotReinitialize);
        require(storage.borrower_operations == null_identity_address(), Error::CannotReinitialize);

        storage.trove_manager = trove_manager;
        storage.stability_pool = stability_pool;
        storage.borrower_operations = borrower_operations;
        storage.config = config;
    }

    #[storage(read, write)]
    fn mint(amount: u64, address: Identity) {
        require_caller_is_borrower_operations();
        mint_to(amount, address);
        storage.total_supply += amount;
    }

    #[storage(read, write), payable]
    fn burn() {
        require_caller_is_bo_or_tm_or_sp();
        let burn_amount = msg_amount();
        burn(burn_amount);
        storage.total_supply -= burn_amount;
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
fn require_caller_is_borrower_operations() {
    require(msg_sender().unwrap() == storage.borrower_operations, Error::NotAuthorized);
}

#[storage(read)]
fn require_caller_is_bo_or_tm_or_sp() {
    require(msg_sender().unwrap() == storage.borrower_operations || msg_sender().unwrap() == storage.trove_manager || msg_sender().unwrap() == storage.stability_pool, Error::NotAuthorized);
}
