contract;

use libraries::fpt_token_interface::{TokenInitializeConfig, FPTToken};
use libraries::fluid_math::{null_contract, null_identity_address, DECIMAL_PRECISION};

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
    vesting_contract: ContractId = null_contract(),
    community_issuance_contract: ContractId = null_contract(),
    is_initialized: bool = false,
}

// import fluid math decinals here
pub const TOTAL_SUPPLY: u64 = 100_000_000;

impl FPTToken for Contract {
    //////////////////////////////////////
    // Owner methods
    //////////////////////////////////////
    #[storage(read, write)]
    fn initialize(
        config: TokenInitializeConfig,
        vesting_contract: ContractId,
        community_issuance_contract: ContractId,
    ) {
        require(storage.is_initialized == false, "Contract is already initialized");
        storage.vesting_contract = vesting_contract;
        storage.community_issuance_contract = community_issuance_contract;
        storage.config = config;
        mint_to(TOTAL_SUPPLY * 68 / 100 * DECIMAL_PRECISION, Identity::ContractId(vesting_contract));
        mint_to(TOTAL_SUPPLY * 32 / 100 * DECIMAL_PRECISION, Identity::ContractId(community_issuance_contract));
        storage.is_initialized = true;
    }

    //////////////////////////////////////
    // Read-Only methods
    //////////////////////////////////////#[storage(read)]
    #[storage(read)]
    fn get_vesting_contract() -> ContractId {
        storage.vesting_contract
    }

    fn total_supply() -> u64 {
        TOTAL_SUPPLY * DECIMAL_PRECISION
    }

    #[storage(read)]
    fn config() -> TokenInitializeConfig {
        storage.config
    }
}