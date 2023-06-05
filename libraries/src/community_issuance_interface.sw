library community_issuance_interface;

abi CommunityIssuance {
    // Initialize contract
    #[storage(read, write)]
    fn initialize();
}
