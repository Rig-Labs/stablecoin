contract;

use libraries::usdf_token_interface::USDFToken;
use libraries::fluid_math::{get_default_asset_id, ZERO_B256};
use std::{
    address::*,
    asset::{
        burn,
        mint_to,
        transfer,
    },
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
    hash::Hash,
    storage::storage_vec::*,
    string::String,
};
storage {
    trove_managers: StorageVec<ContractId> = StorageVec {},
    protocol_manager: ContractId = ContractId::zero(),
    stability_pool: Identity = Identity::Address(Address::zero()),
    borrower_operations: Identity = Identity::Address(Address::zero()),
    default_asset: AssetId = AssetId::zero(),
    total_supply: u64 = 0,
    is_initialized: bool = false,
}
// Using https://docs.fuel.network/docs/sway-standards/src-20-native-asset/ as reference
impl USDFToken for Contract {
    //////////////////////////////////////
    // Initialization method
    //////////////////////////////////////
    #[storage(read, write)]
    fn initialize(
        protocol_manager: ContractId,
        stability_pool: Identity,
        borrower_operations: Identity,
    ) {
        require(
            storage
                .is_initialized
                .read() == false,
            "USDFToken: Contract is already initialized",
        );
        storage.stability_pool.write(stability_pool);
        storage.protocol_manager.write(protocol_manager);
        storage.borrower_operations.write(borrower_operations);
        storage
            .default_asset
            .write(get_default_asset_id(ContractId::this()));
        storage.is_initialized.write(true);
    }
    #[storage(read, write)]
    fn mint(amount: u64, address: Identity) {
        require_caller_is_borrower_operations();
        mint_to(address, ZERO_B256, amount);
        storage
            .total_supply
            .write(storage.total_supply.try_read().unwrap_or(0) + amount);
    }
    #[storage(read, write), payable]
    fn burn() {
        require_caller_is_bo_or_tm_or_sp_or_pm();
        let burn_amount = msg_amount();
        burn(ZERO_B256, burn_amount);
        storage
            .total_supply
            .write(storage.total_supply.read() - burn_amount);
    }
    #[storage(read, write)]
    fn add_trove_manager(trove_manager: ContractId) {
        require_caller_is_protocol_manager();
        storage.trove_managers.push(trove_manager);
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
            return Some(storage.total_supply.try_read().unwrap_or(0))
        }
        return None;
    }

    #[storage(read)]
    fn name(asset: AssetId) -> Option<String> {
        if asset == storage.default_asset.read() {
            return Some(String::from_ascii_str("USDF"));
        }
        return None;
    }

    #[storage(read)]
    fn symbol(asset: AssetId) -> Option<String> {
        if asset == storage.default_asset.read() {
            return Some(String::from_ascii_str("USDF"));
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
#[storage(read)]
fn require_caller_is_protocol_manager() {
    require(
        msg_sender()
            .unwrap() == Identity::ContractId(storage.protocol_manager.read()),
        "USDFToken: NotAuthorized",
    );
}
#[storage(read)]
fn require_caller_is_borrower_operations() {
    require(
        msg_sender()
            .unwrap() == storage
            .borrower_operations
            .read(),
        "USDFToken: NotAuthorized",
    );
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
    require(
        sender == storage
            .borrower_operations
            .read() || sender == storage
            .stability_pool
            .read() || sender == protocol_manager_id,
        "USDFToken: NotAuthorized",
    );
}
