contract;

use std::token::{mint_to_address, mint_to_contract};

abi Asset {
    fn mint_and_send_to_address(amount: u64, recipient: Address) -> bool;

    fn mint_and_send_to_contract(amount: u64, to: ContractId);
}

impl Asset for Contract {
    fn mint_and_send_to_address(amount: u64, recipient: Address) -> bool {
        mint_to_address(amount, recipient);
        true
    }

    fn mint_and_send_to_contract(amount: u64, to: ContractId) {
        mint_to_contract(amount, to);
    }
}
