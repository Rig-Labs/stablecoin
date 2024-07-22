contract;

use libraries::fpt_token_interface::FPTToken;
use libraries::fluid_math::{DECIMAL_PRECISION, get_default_asset_id, ZERO_B256,};
use std::{
    address::*,
    asset::*,
    auth::{
        AuthError,
    },
    call_frames::{
        msg_asset_id,
    },
    context::{
        balance_of,
        msg_amount,
    },
    storage::*,
    string::String,
};

storage {
    vesting_contract: ContractId = ContractId::from(ZERO_B256),
    community_issuance_contract: ContractId = ContractId::from(ZERO_B256),
    is_initialized: bool = false,
    default_asset: AssetId = AssetId::from(ZERO_B256),
}

// import fluid math decinals here
pub const TOTAL_SUPPLY: u64 = 100_000_000;
impl FPTToken for Contract {
    //////////////////////////////////////
    // Owner methods
    //////////////////////////////////////
    #[storage(read, write)]
    fn initialize(
        vesting_contract: ContractId,
        community_issuance_contract: ContractId,
    ) {
        require(
            storage
                .is_initialized
                .read() == false,
            "Contract is already initialized",
        );
        storage.vesting_contract.write(vesting_contract);
        storage
            .community_issuance_contract
            .write(community_issuance_contract);
        // storage.config.write(config);
        mint_to(
            Identity::ContractId(vesting_contract),
            ZERO_B256,
            TOTAL_SUPPLY * 68 / 100 * DECIMAL_PRECISION,
        );
        mint_to(
            Identity::ContractId(community_issuance_contract),
            ZERO_B256,
            TOTAL_SUPPLY * 32 / 100 * DECIMAL_PRECISION,
        );
        storage
            .default_asset
            .write(get_default_asset_id(ContractId::this()));
        storage.is_initialized.write(true);
    }
    //////////////////////////////////////
    // Read-Only methods
    //////////////////////////////////////
    #[storage(read)]
    fn get_vesting_contract() -> ContractId {
        storage.vesting_contract.read()
    }
    //////////////////////////////////////
    // SRC-20 Read-Only methods
    //////////////////////////////////////
    fn total_assets() -> u64 {
        return 1;
    }

    #[storage(read)]
    fn total_supply(asset: AssetId) -> Option<u64> {
        if asset == storage.default_asset.read() {
            return Some(TOTAL_SUPPLY * DECIMAL_PRECISION);
        }
        return None;
    }

    #[storage(read)]
    fn name(asset: AssetId) -> Option<String> {
        if asset == storage.default_asset.read() {
            return Some(String::from_ascii_str("Fluid Protocol Token"));
        }
        return None;
    }

    #[storage(read)]
    fn symbol(asset: AssetId) -> Option<String> {
        if asset == storage.default_asset.read() {
            return Some(String::from_ascii_str("FPT"));
        }
        return None;
    }

    #[storage(read)]
    fn decimals(asset: AssetId) -> Option<u8> {
        if asset == storage.default_asset.read() {
            return Some(9u8);
        }
        return None;
    }
}
