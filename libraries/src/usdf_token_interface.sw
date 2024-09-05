library;

use std::string::String;

abi USDFToken {
    // Initialize contract
    #[storage(read, write)]
    fn initialize(
        protocol_manager: ContractId,
        stability_pool: Identity,
        borrower_operations: Identity,
    );

    #[storage(read, write)]
    fn add_trove_manager(trove_manager: ContractId);

    // Mint token coins to a given address
    #[storage(read, write)]
    fn mint(amount: u64, address: Identity);
    // Get balance of the contract coins
    // Burn token coins
    #[storage(read, write), payable]
    fn burn();

    fn total_assets() -> u64;

    #[storage(read)]
    fn total_supply(asset: AssetId) -> Option<u64>;

    #[storage(read)]
    fn name(asset: AssetId) -> Option<String>;

    #[storage(read)]
    fn symbol(asset: AssetId) -> Option<String>;

    #[storage(read)]
    fn decimals(asset: AssetId) -> Option<u8>;
}
