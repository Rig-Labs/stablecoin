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
}
