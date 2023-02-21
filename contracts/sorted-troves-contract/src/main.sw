contract;

use libraries::sorted_troves_interface::{SortedTroves};
use libraries::trove_manager_interface::{TroveManager};
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
    identity::Identity,
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
    max_size: u64 = 0,
    trove_manager_contract_id: ContractId = ContractId::from(ZERO_B256),
    borrower_operations_identity: ContractId = ContractId::from(ZERO_B256),
    head: Identity = Identity::Address(Address::from(ZERO_B256)),
    tail: Identity = Identity::Address(Address::from(ZERO_B256)),
    nodes: StorageMap<Identity, Node> = StorageMap {},
}

impl SortedTroves for Contract {
    #[storage(read, write)]
    fn set_params(
        size: u64,
        trove_manager_contract_id: ContractId,
        borrower_operations_contract_id: ContractId,
    ) {
        require(size > 0, "size must be greater than 0");
        require(storage.trove_manager_contract_id == ContractId::from(ZERO_B256), "trove_manager_identity must not be set");
        require(storage.borrower_operations_identity == ContractId::from(ZERO_B256), "borrower_operations_identity must not be set");

        storage.max_size = size;
        storage.trove_manager_contract_id = trove_manager_contract_id;
        storage.borrower_operations_identity = borrower_operations_contract_id;
    }

    #[storage(read, write)]
    fn insert(id: Identity, icr: u64, prev_id: Identity, next_id: Identity) {
        require_is_bo_or_tm();

        let trove_manager_contract = abi(TroveManager, storage.trove_manager_contract_id.value);
        require(!internal_is_full(), "list is full");
        require(!internal_contains(id), "id already exists");
        require(Identity::Address(Address::from(ZERO_B256)) != id, "id must not be zero");
        require(icr > 0, "icr must be greater than 0");
    }

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
        return internal_contains(id);
    }

    #[storage(read)]
    fn is_full() -> bool {
        return internal_is_full();
    }

    #[storage(read)]
    fn is_empty() -> bool {
        return internal_is_empty();
    }

    #[storage(read)]
    fn get_size() -> u64 {
        return internal_get_size();
    }

    #[storage(read)]
    fn get_max_size() -> u64 {
        return internal_get_max_size();
    }

    #[storage(read)]
    fn get_first() -> Identity {
        return internal_get_first();
    }

    #[storage(read)]
    fn get_last() -> Identity {
        return internal_get_last();
    }

    #[storage(read)]
    fn get_next(id: Identity) -> Identity {
        return internal_get_next(id);
    }

    #[storage(read)]
    fn get_prev(id: Identity) -> Identity {
        return internal_get_prev(id);
    }

    #[storage(read)]
    fn valid_insert_position(icr: u64, next_id: Identity, prev_id: Identity) -> bool {
        return internal_valid_insert_position(icr, next_id, prev_id);
    }

    #[storage(read)]
    fn find_insert_position(icr: u64, next_id: Identity, prev_id: Identity) -> (Identity, Identity) {
        return internal_find_insert_position(icr, next_id, prev_id);
    }
}

#[storage(read)]
fn internal_get_first() -> Identity {
    return storage.head;
}

#[storage(read)]
fn internal_get_last() -> Identity {
    return storage.tail;
}

#[storage(read)]
fn internal_get_next(id: Identity) -> Identity {
    return storage.nodes.get(id).next_id;
}

#[storage(read)]
fn internal_get_prev(id: Identity) -> Identity {
    return storage.nodes.get(id).prev_id;
}

#[storage(read)]
fn internal_get_max_size() -> u64 {
    return storage.max_size;
}

#[storage(read)]
fn internal_get_size() -> u64 {
    return storage.size;
}

#[storage(read)]
fn internal_is_full() -> bool {
    return storage.size == storage.max_size;
}

#[storage(read)]
fn internal_is_empty() -> bool {
    return storage.size == 0;
}

#[storage(read)]
fn internal_contains(id: Identity) -> bool {
    return storage.nodes.get(id).exists;
}

#[storage(read)]
fn internal_valid_insert_position(icr: u64, next_id: Identity, prev_id: Identity) -> bool {
    let trove_manager_contract = abi(TroveManager, storage.trove_manager_contract_id.value);

    if (next_id == Identity::Address(Address::from(ZERO_B256))
        && prev_id == Identity::Address(Address::from(ZERO_B256)))
    {
        return internal_is_empty();
    } else if (prev_id == Identity::Address(Address::from(ZERO_B256)))
    {
        return storage.head == next_id && icr >= trove_manager_contract.get_nominal_irc(next_id);
    } else if (next_id == Identity::Address(Address::from(ZERO_B256)))
    {
        return storage.tail == prev_id && icr <= trove_manager_contract.get_nominal_irc(prev_id);
    } else {
        return storage.nodes.get(prev_id).next_id == next_id && trove_manager_contract.get_nominal_irc(prev_id) >= icr && icr >= trove_manager_contract.get_nominal_irc(next_id);
    }

    return true;
}

#[storage(read)]
fn require_is_trove_manager() {
    let sender = msg_sender().unwrap();
    require(Identity::ContractId(storage.trove_manager_contract_id) == sender, "Access denied");
}

#[storage(read)]
fn require_is_bo_or_tm() {
    let sender = msg_sender().unwrap();
    require(Identity::ContractId(storage.trove_manager_contract_id) == sender || Identity::ContractId(storage.trove_manager_contract_id) == sender, "Access denied");
}

#[storage(read)]
fn internal_find_insert_position(
    icr: u64,
    next_id: Identity,
    prev_id: Identity,
) -> (Identity, Identity) {
    return (
        Identity::Address(Address::from(ZERO_B256)),
        Identity::Address(Address::from(ZERO_B256)),
    )
}

#[storage(read)]
fn internal_descend_list(nicr: u64, start_id: Identity) -> (Identity, Identity) {
    let trove_manager_contract = abi(TroveManager, storage.trove_manager_contract_id.value);

    if (storage.head == start_id
        && nicr >= trove_manager_contract.get_nominal_irc(start_id))
    {
        return (Identity::Address(Address::from(ZERO_B256)), start_id);
    }

    let mut prev_id = start_id;
    let mut next_id = storage.nodes.get(prev_id).next_id;

    while (prev_id != Identity::Address(Address::from(ZERO_B256)) && !internal_valid_insert_position(nicr, next_id, prev_id)) {
        prev_id = storage.nodes.get(prev_id).next_id;
        next_id = storage.nodes.get(prev_id).next_id;
    }

    return (prev_id, next_id);
}

#[storage(read)]
fn internal_ascend_list(nicr: u64, start_id: Identity) -> (Identity, Identity) {
    let trove_manager_contract = abi(TroveManager, storage.trove_manager_contract_id.value);

    if (storage.tail == start_id
        && nicr <= trove_manager_contract.get_nominal_irc(start_id))
    {
        return (start_id, Identity::Address(Address::from(ZERO_B256)));
    }

    let mut next_id = start_id;
    let mut prev_id = storage.nodes.get(next_id).prev_id;

    while (next_id != Identity::Address(Address::from(ZERO_B256)) && !internal_valid_insert_position(nicr, next_id, prev_id)) {
        next_id = storage.nodes.get(next_id).prev_id;
        prev_id = storage.nodes.get(next_id).prev_id;
    }

    return (prev_id, next_id);
}
