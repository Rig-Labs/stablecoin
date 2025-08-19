contract;
// Mints a single token to gets all the rewards associated with the protocol
use libraries::fpt_token_interface::FPTToken;
use libraries::fluid_math::{DECIMAL_PRECISION,};
use standards::src20::{SetDecimalsEvent, SetNameEvent, SetSymbolEvent, SRC20, TotalSupplyEvent,};
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
configurable {
    /// Initializer identity
    INITIALIZER: Identity = Identity::Address(Address::zero()),
}

storage {
    vesting_contract: ContractId = ContractId::zero(),
    community_issuance_contract: ContractId = ContractId::zero(),
    is_initialized: bool = false,
}
// Using https://docs.fuel.network/docs/sway-standards/src-20-native-asset/ as reference
pub const TOTAL_SUPPLY: u64 = 100_000_000_000;
pub const DECIMALS: u8 = 9;
pub const SYMBOL: str[3] = __to_str_array("MFT");
pub const NAME: str[14] = __to_str_array("Moor Fee Token");

impl FPTToken for Contract {
    //////////////////////////////////////
    // Initialization method
    //////////////////////////////////////
    #[storage(read, write)]
    fn initialize(
        vesting_contract: ContractId,
        community_issuance_contract: ContractId,
    ) {
        require(
            msg_sender()
                .unwrap() == INITIALIZER,
            "FPTToken: Caller is not initializer",
        );
        require(
            storage
                .is_initialized
                .read() == false,
            "FPTToken: Contract is already initialized",
        );
        storage.vesting_contract.write(vesting_contract);
        storage
            .community_issuance_contract
            .write(community_issuance_contract);

        // Mint total supply to deployer
        let sender = msg_sender().unwrap();
        mint_to(sender, SubId::zero(), TOTAL_SUPPLY);

        SetSymbolEvent::new(
            AssetId::default(),
            Some(String::from_ascii_str(from_str_array(SYMBOL))),
            sender,
        )
            .log();
        SetDecimalsEvent::new(AssetId::default(), DECIMALS, sender)
            .log();
        SetNameEvent::new(
            AssetId::default(),
            Some(String::from_ascii_str(from_str_array(NAME))),
            sender,
        )
            .log();
        TotalSupplyEvent::new(AssetId::default(), TOTAL_SUPPLY, sender)
            .log();
        storage.is_initialized.write(true);
    }
    //////////////////////////////////////
    // Read-Only methods
    //////////////////////////////////////
    #[storage(read)]
    fn get_vesting_contract() -> ContractId {
        storage.vesting_contract.read()
    }
}

impl SRC20 for Contract {
    #[storage(read)]
    fn total_assets() -> u64 {
        return 1;
    }

    #[storage(read)]
    fn total_supply(asset: AssetId) -> Option<u64> {
        if asset == AssetId::default() {
            return Some(TOTAL_SUPPLY);
        }
        return None;
    }

    #[storage(read)]
    fn name(asset: AssetId) -> Option<String> {
        if asset == AssetId::default() {
            return Some(String::from_ascii_str(from_str_array(NAME)));
        }
        return None;
    }

    #[storage(read)]
    fn symbol(asset: AssetId) -> Option<String> {
        if asset == AssetId::default() {
            return Some(String::from_ascii_str(from_str_array(SYMBOL)));
        }
        return None;
    }

    #[storage(read)]
    fn decimals(asset: AssetId) -> Option<u8> {
        if asset == AssetId::default() {
            return Some(DECIMALS);
        }
        return None;
    }
}
