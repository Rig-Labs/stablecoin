contract;

use libraries::fpt_token_interface::{FPTToken, TokenInitializeConfig};
use libraries::fluid_math::{DECIMAL_PRECISION, null_contract, null_identity_address, ZERO_B256};

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
    // config: TokenInitializeConfig = TokenInitializeConfig {
    //     name: "                                ",
    //     symbol: "        ",
    //     decimals: 1u8,
    // },
    vesting_contract: ContractId = ContractId::from(ZERO_B256),
    community_issuance_contract: ContractId = ContractId::from(ZERO_B256),
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
        require(storage.is_initialized.read() == false, "Contract is already initialized");
        storage.vesting_contract.write(vesting_contract);
        storage.community_issuance_contract.write(community_issuance_contract);
        // storage.config.write(config);
        mint_to(Identity::ContractId(vesting_contract), ZERO_B256, TOTAL_SUPPLY * 68 / 100 * DECIMAL_PRECISION);
        mint_to(Identity::ContractId(community_issuance_contract), ZERO_B256, TOTAL_SUPPLY * 32 / 100 * DECIMAL_PRECISION);
        storage.is_initialized.write(true);
    }

    //////////////////////////////////////
    // Read-Only methods
    //////////////////////////////////////#[storage(read)]
    #[storage(read)]
    fn get_vesting_contract() -> ContractId {
        storage.vesting_contract.read()
    }

    fn total_supply() -> u64 {
        TOTAL_SUPPLY * DECIMAL_PRECISION
    }

    // #[storage(read)]
    // fn config() -> TokenInitializeConfig {
    //     storage.config.read()
    // }
}
