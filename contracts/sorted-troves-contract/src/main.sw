contract;

use libraries::sorted_troves_interface::{SortedTroves};
use libraries::trove_manager_interface::{TroveManager};
use libraries::data_structures::Node;
use libraries::fluid_math::{null_contract, null_identity_address};

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

storage {
    size: u64 = 0,
    max_size: u64 = 0,
    trove_manager_contract_id: ContractId = null_contract(),
    borrower_operations_contract_id: ContractId = null_contract(),
    head: Identity = null_identity_address(),
    tail: Identity = null_identity_address(),
    nodes: StorageMap<Identity, Node> = StorageMap {},
    is_initialized: bool = false,
}

impl SortedTroves for Contract {
    #[storage(read, write)]
    fn set_params(
        size: u64,
        trove_manager_contract_id: ContractId,
        borrower_operations_contract_id: ContractId,
    ) {
        require(size > 0, "size must be greater than 0");
        require(storage.is_initialized == false, "Contract is already initialized");

        storage.max_size = size;
        storage.trove_manager_contract_id = trove_manager_contract_id;
        storage.borrower_operations_contract_id = borrower_operations_contract_id;
        storage.is_initialized = true;
    }

    #[storage(read, write)]
    fn insert(
        _id: Identity,
        _nicr: u64,
        _prev_id: Identity,
        _next_id: Identity,
    ) {
        require_is_bo_or_tm();

        internal_insert(_id, _nicr, _prev_id, _next_id);
    }

    #[storage(read, write)]
    fn remove(id: Identity) {
        require_is_trove_manager();

        internal_remove(id);
    }

    #[storage(read, write)]
    fn re_insert(
        id: Identity,
        nicr: u64,
        prev_id: Identity,
        next_id: Identity,
    ) {
        require_is_bo_or_tm();

        require(internal_contains(id), "id must exist");
        require(nicr > 0, "nicr must be greater than 0");

        internal_remove(id);
        internal_insert(id, nicr, prev_id, next_id);
    }

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

    if (next_id == null_identity_address()
        && prev_id == null_identity_address())
    {
        return internal_is_empty();
    } else if (prev_id == null_identity_address()) {
        return storage.head == next_id && icr >= trove_manager_contract.get_nominal_icr(next_id);
    } else if (next_id == null_identity_address()) {
        return storage.tail == prev_id && icr <= trove_manager_contract.get_nominal_icr(prev_id);
    } else {
        return storage.nodes.get(prev_id).next_id == next_id && trove_manager_contract.get_nominal_icr(prev_id) >= icr && icr >= trove_manager_contract.get_nominal_icr(next_id);
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
    require(Identity::ContractId(storage.borrower_operations_contract_id) == sender || Identity::ContractId(storage.trove_manager_contract_id) == sender, "Access denied");
}

#[storage(read)]
fn internal_find_insert_position(
    icr: u64,
    _next_id: Identity,
    _prev_id: Identity,
) -> (Identity, Identity) {
    let trove_manager_contract = abi(TroveManager, storage.trove_manager_contract_id.value);

    let mut next_id: Identity = _next_id;
    let mut prev_id: Identity = _prev_id;

    if (prev_id != null_identity_address()) {
        if (!internal_contains(prev_id)
            || icr > trove_manager_contract.get_nominal_icr(prev_id))
        {
            prev_id = null_identity_address()
        }
    }

    if (next_id != null_identity_address()) {
        if (!internal_contains(next_id)
            || icr < trove_manager_contract.get_nominal_icr(prev_id))
        {
            next_id = null_identity_address()
        }
    }

    if (prev_id == null_identity_address()
        && next_id == null_identity_address())
    {
        return internal_descend_list(icr, storage.head);
    } else if (prev_id == null_identity_address()) {
        return internal_ascend_list(icr, next_id);
    } else if (next_id == null_identity_address()) {
        return internal_descend_list(icr, prev_id);
    } else {
        return internal_descend_list(icr, prev_id);
    }
}

#[storage(read)]
fn internal_descend_list(nicr: u64, start_id: Identity) -> (Identity, Identity) {
    let trove_manager_contract = abi(TroveManager, storage.trove_manager_contract_id.value);

    if (storage.head == start_id
        && nicr >= trove_manager_contract.get_nominal_icr(start_id))
    {
        return (null_identity_address(), start_id);
    }

    let mut prev_id = start_id;
    let mut next_id = storage.nodes.get(prev_id).next_id;

    while (prev_id != null_identity_address() && !internal_valid_insert_position(nicr, next_id, prev_id)) {
        prev_id = storage.nodes.get(prev_id).next_id;
        next_id = storage.nodes.get(prev_id).next_id;
    }

    return (prev_id, next_id);
}

#[storage(read)]
fn internal_ascend_list(nicr: u64, start_id: Identity) -> (Identity, Identity) {
    let trove_manager_contract = abi(TroveManager, storage.trove_manager_contract_id.value);

    if (storage.tail == start_id
        && nicr <= trove_manager_contract.get_nominal_icr(start_id))
    {
        return (start_id, null_identity_address());
    }

    let mut next_id = start_id;
    let mut prev_id = storage.nodes.get(next_id).prev_id;

    while (next_id != null_identity_address() && !internal_valid_insert_position(nicr, next_id, prev_id)) {
        next_id = storage.nodes.get(next_id).prev_id;
        prev_id = storage.nodes.get(next_id).prev_id;
    }

    return (prev_id, next_id);
}

#[storage(read, write)]
fn internal_insert(
    id: Identity,
    nicr: u64,
    _prev_id: Identity,
    _next_id: Identity,
) {
    let trove_manager_contract = abi(TroveManager, storage.trove_manager_contract_id.into());
    require(!internal_is_full(), "list is full");
    require(!internal_contains(id), "id already exists");
    require(null_identity_address() != id, "id must not be zero");
    require(nicr > 0, "icr must be greater than 0");

    let mut next_id: Identity = _next_id;
    let mut prev_id: Identity = _prev_id;

    if (!internal_valid_insert_position(nicr, prev_id, next_id))
    {
        let res = internal_find_insert_position(nicr, prev_id, next_id);
        prev_id = res.0;
        next_id = res.1;
    }

    let mut new_node = Node {
        exists: true,
        prev_id: null_identity_address(),
        next_id: null_identity_address(),
    };

    if (prev_id == null_identity_address()
        && next_id == null_identity_address())
    {
        storage.head = id;
        storage.tail = id;
    } else if (prev_id == null_identity_address()) {
        new_node.next_id = storage.head;

        edit_node_neighbors(storage.head, Option::Some(id), Option::None);

        storage.head = id;
    } else if (next_id == null_identity_address()) {
        new_node.prev_id = storage.tail;

        edit_node_neighbors(storage.tail, Option::None, Option::Some(id));

        storage.tail = id;
    } else {
        new_node.prev_id = prev_id;
        new_node.next_id = next_id;

        edit_node_neighbors(prev_id, Option::None, Option::Some(id));

        edit_node_neighbors(next_id, Option::Some(id), Option::None);
    }

    storage.nodes.insert(id, new_node);
    storage.size += 1;
}

#[storage(read, write)]
fn edit_node_neighbors(
    id: Identity,
    prev_id: Option<Identity>,
    next_id: Option<Identity>,
) {
    // TODO Update when StorageMap supports updating values
    let mut node = storage.nodes.get(id);

    if (prev_id.is_some()) {
        node.prev_id = prev_id.unwrap();
    }

    if (next_id.is_some()) {
        node.next_id = next_id.unwrap();
    }

    storage.nodes.insert(id, node);
}

#[storage(read, write)]
fn internal_remove(id: Identity) {
    require(internal_contains(id), "id does not exist");

    let mut node = storage.nodes.get(id);

    if (storage.size > 1) {
        if (id == storage.head) {
            storage.head = node.next_id;

            edit_node_neighbors(node.next_id, Option::Some(null_identity_address()), Option::None);
        } else if (id == storage.tail) {
            storage.tail = node.prev_id;

            edit_node_neighbors(node.prev_id, Option::None, Option::Some(null_identity_address()));
        } else {
            edit_node_neighbors(node.prev_id, Option::None, Option::Some(node.next_id));

            edit_node_neighbors(node.next_id, Option::Some(node.prev_id), Option::None);
        }
    } else {
        storage.head = null_identity_address();
        storage.tail = null_identity_address();
    }

    // TODO use storage.nodes.remove(id) when available
    node.exists = false;
    node.prev_id = null_identity_address();
    node.next_id = null_identity_address();
    storage.nodes.insert(id, node);

    storage.size -= 1;
}
