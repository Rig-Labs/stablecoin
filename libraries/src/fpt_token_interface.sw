library;

pub struct TokenInitializeConfig {
    name: str[32],
    symbol: str[8],
    decimals: u8,
}

abi FPTToken {
    // Initialize contract
    #[storage(read, write)]
    fn initialize(config: TokenInitializeConfig, vesting_contract: ContractId, community_issuance_contract: ContractId);

    #[storage(read)]
    fn get_vesting_contract() -> ContractId;

    fn total_supply() -> u64;

    #[storage(read)]
    fn config() -> TokenInitializeConfig;
}
