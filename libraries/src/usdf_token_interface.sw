library;

pub struct TokenInitializeConfig {
    name: str[32],
    symbol: str[8],
    decimals: u8,
}

abi USDFToken {
    // Initialize contract
    #[storage(read, write)]
    fn initialize(config: TokenInitializeConfig, protocol_manager: ContractId, stability_pool: Identity, borrower_operations: Identity);

    #[storage(read, write)]
    fn add_trove_manager(trove_manager: ContractId);

    // Mint token coins to a given address
    #[storage(read, write)]
    fn mint(amount: u64, address: Identity);
    // Get balance of the contract coins
    // Burn token coins
    #[storage(read, write), payable]
    fn burn();
    // Transfer a contract coins to a given output  
    // Config of token
    #[storage(read)]
    fn total_supply() -> u64;

    #[storage(read)]
    fn config() -> TokenInitializeConfig;
}
