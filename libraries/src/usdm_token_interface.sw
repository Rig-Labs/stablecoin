library;

use std::string::String;

abi USDMToken {
    // Initialize contract
    #[storage(read, write)]
    fn initialize(
        protocol_manager: ContractId,
        stability_pool: Identity,
        borrower_operations: Identity,
    );

    #[storage(read, write)]
    fn add_trove_manager(trove_manager: ContractId);
}
