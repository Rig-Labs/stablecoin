contract;

use libraries::usdf_token_interface::{TokenInitializeConfig, USDFToken};
use libraries::fluid_math::{null_contract, null_identity_address, ZERO_B256};

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
    hash::Hash,
    identity::{
        Identity,
    },
    revert::require,
    storage::storage_vec::*,
    token::*,
};

storage {
    // config: TokenInitializeConfig = TokenInitializeConfig {
    //     name: "                                ",
    //     symbol: "        ",
    //     decimals: 1u8,
    // },
    trove_managers: StorageVec<ContractId> = StorageVec {},
    protocol_manager: ContractId = ContractId::from(ZERO_B256),
    stability_pool: Identity = Identity::Address(Address::from(ZERO_B256)),
    borrower_operations: Identity = Identity::Address(Address::from(ZERO_B256)),
    total_supply: u64 = 0,
    is_initialized: bool = false,
}

enum Error {
    NotAuthorized: (),
}

impl USDFToken for Contract { //////////////////////////////////////
    // Owner methods
    //////////////////////////////////////
    #[storage(read, write)]
    fn initialize(
        config: TokenInitializeConfig,
        protocol_manager: ContractId,
        stability_pool: Identity,
        borrower_operations: Identity,
    ) {
        require(storage.is_initialized.read() == false, "Contract is already initialized");
        storage.stability_pool.write(stability_pool);
        storage.protocol_manager.write(protocol_manager);
        storage.borrower_operations.write(borrower_operations);
        // storage.config.write(config);
        storage.is_initialized.write(true);
    }

    #[storage(read, write)]
    fn mint(amount: u64, address: Identity) {
        require_caller_is_borrower_operations();
        mint_to(address, ZERO_B256, amount);
        storage.total_supply.write(storage.total_supply.try_read().unwrap_or(0) + amount);
    }

    #[storage(read, write), payable]
    fn burn() {
        require_caller_is_bo_or_tm_or_sp_or_pm();
        let burn_amount = msg_amount();
        burn(ZERO_B256, burn_amount);
        storage.total_supply.write(storage.total_supply.read() - burn_amount);
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
        storage.total_supply.try_read().unwrap_or(0)
    }

    // #[storage(read)]
    // fn config() -> TokenInitializeConfig {
    //     // storage.config.read()
    //     TokenInitializeConfig {
    //         name: "                                ",
    //         symbol: "        ",
    //         decimals: 1u8,
    //     }
    // }
}

#[storage(read)]
fn require_caller_is_protocol_manager() {
    require(msg_sender().unwrap() == Identity::ContractId(storage.protocol_manager.read()), Error::NotAuthorized);
}

#[storage(read)]
fn require_caller_is_borrower_operations() {
    require(msg_sender().unwrap() == storage.borrower_operations.read(), Error::NotAuthorized);
}

#[storage(read)]
fn require_caller_is_bo_or_tm_or_sp_or_pm() {
    let sender = msg_sender().unwrap();
    let protocol_manager_id = Identity::ContractId(storage.protocol_manager.read());

    let mut i = 0;
    while i < storage.trove_managers.len() {
        let manager = Identity::ContractId(storage.trove_managers.get(i).unwrap().read());
        if manager == sender {
            return
        }
        i += 1;
    }
    require(sender == storage.borrower_operations.read() || sender == storage.stability_pool.read() || sender == protocol_manager_id, Error::NotAuthorized);
}
