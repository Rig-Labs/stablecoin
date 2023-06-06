library community_issuance_interface;

abi CommunityIssuance {
    // Initialize contract
    #[storage(read, write)]
    fn initialize(
        stability_pool_contract: ContractId,
        fpt_token_contract: ContractId,
        admin: Identity,
        debugging: bool,
        time: u64,
    );

    #[storage(read, write)]
    fn start_rewards_increase_transition(total_transition_time_seconds: u64);

    #[storage(read, write)]
    fn issue_fpt() -> u64;

    #[storage(read)]
    fn send_fpt(account: Identity, amount: u64);

    #[storage(read)]
    fn get_current_time() -> u64;

    #[storage(write, read)]
    fn set_current_time(time: u64);
}
