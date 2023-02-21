contract;

use libraries::sorted_troves_interface::{SortedTroves};
use libraries::data_structures::Node;

use std::{
    address::Address,
    auth::msg_sender,
    block::{
        height,
        timestamp,
    },
    call_frames::{
        contract_id,
        msg_asset_id,
    },
    context::{
        msg_amount,
    },
    logging::log,
    storage::{
        StorageMap,
        StorageVec,
    },
    token::transfer,
};

const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

storage {
    size: u64 = 0,
    max_size: u64 = 1000,
    trove_manager_identity: Identity = Identity::Address(Address::from(ZERO_B256)),
    borrower_operations_identity: Identity = Identity::Address(Address::from(ZERO_B256)),
    head: Identity = Identity::Address(Address::from(ZERO_B256)),
    tail: Identity = Identity::Address(Address::from(ZERO_B256)),
    nodes: StorageMap<Identity, Node> = StorageMap {},
}

impl SortedTroves for Contract {
    #[storage(read, write)]
    fn set_params(
        size: u64,
        trove_manager_identity: Identity,
        borrower_operations_identity: Identity,
    ) {
        require(size > 0, "size must be greater than 0");
        require(trove_manager_identity == Identity::Address(Address::from(ZERO_B256)), "trove_manager_identity must not be set");
        require(borrower_operations_identity == Identity::Address(Address::from(ZERO_B256)), "borrower_operations_identity must not be set");

        storage.max_size = size;
        storage.trove_manager_identity = trove_manager_identity;
        storage.borrower_operations_identity = borrower_operations_identity;
    }

    #[storage(read, write)]
    fn insert(id: Identity, icr: u64, prev_id: Identity, next_id: Identity) {}

    #[storage(read, write)]
    fn remove(id: Identity) {}

    #[storage(read, write)]
    fn re_insert(
        id: Identity,
        new_icr: u64,
        prev_id: Identity,
        next_id: Identity,
    ) {}

    #[storage(read)]
    fn contains(id: Identity) -> bool {
        return storage.nodes.get(id).exists;
    }

    #[storage(read)]
    fn is_full() -> bool {
        return storage.size == storage.max_size;
    }

    #[storage(read)]
    fn is_empty() -> bool {
        return storage.size == 0;
    }

    #[storage(read)]
    fn get_size() -> u64 {
        return storage.size;
    }

    #[storage(read)]
    fn get_max_size() -> u64 {
        return storage.max_size;
    }

    #[storage(read)]
    fn get_first() -> Identity {
        return storage.head;
    }

    #[storage(read)]
    fn get_last() -> Identity {
        return storage.tail;
    }

    #[storage(read)]
    fn get_next(id: Identity) -> Identity {
        return storage.nodes.get(id).next_id;
    }

    #[storage(read)]
    fn get_prev(id: Identity) -> Identity {
        return storage.nodes.get(id).prev_id;
    }

    #[storage(read)]
    fn valid_insert_position(icr: u64, next_id: Identity, prev_id: Identity) -> bool {
        return true
    }

    #[storage(read)]
    fn find_insert_position(icr: u64, next_id: Identity, prev_id: Identity) -> (Identity, Identity) {
        return (
            Identity::Address(Address::from(ZERO_B256)),
            Identity::Address(Address::from(ZERO_B256)),
        )
    }
}

#[storage(read)]
fn require_is_trove_manager() {
    let sender = msg_sender().unwrap();
    require(storage.trove_manager_identity == sender, "Access denied");
}

#[storage(read)]
fn require_is_bo_or_tm() {
    let sender = msg_sender().unwrap();
    require(storage.borrower_operations_identity == sender || storage.trove_manager_identity == sender, "Access denied");
}
