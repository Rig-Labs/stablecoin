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
    head: StorageMap<ContractId, Identity> = StorageMap {},
    tail: StorageMap<ContractId, Identity> = StorageMap {},
    nodes: StorageMap<(Identity, ContractId), Node> = StorageMap {},
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
        id: Identity,
        nicr: u64,
        prev_id: Identity,
        next_id: Identity,
        asset: ContractId,
    ) {
        require_is_bo_or_tm();

        internal_insert(id, nicr, prev_id, next_id, asset);
    }

    #[storage(read, write)]
    fn remove(id: Identity, asset: ContractId) {
        require_is_trove_manager();

        internal_remove(id, asset);
    }

    #[storage(read, write)]
    fn re_insert(
        id: Identity,
        nicr: u64,
        prev_id: Identity,
        next_id: Identity,
        asset: ContractId,
    ) {
        require_is_bo_or_tm();

        require(internal_contains(id, asset), "id must exist");
        require(nicr > 0, "nicr must be greater than 0");

        internal_remove(id, asset);
        internal_insert(id, nicr, prev_id, next_id, asset);
    }

    #[storage(read)]
    fn contains(id: Identity, asset: ContractId) -> bool {
        return internal_contains(id, asset);
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
    fn get_first(asset: ContractId) -> Identity {
        return internal_get_first(asset);
    }

    #[storage(read)]
    fn get_last(asset: ContractId) -> Identity {
        return internal_get_last(asset);
    }

    #[storage(read)]
    fn get_next(id: Identity, asset: ContractId) -> Identity {
        return internal_get_next(id, asset);
    }

    #[storage(read)]
    fn get_prev(id: Identity, asset: ContractId) -> Identity {
        return internal_get_prev(id, asset);
    }

    #[storage(read)]
    fn valid_insert_position(
        icr: u64,
        next_id: Identity,
        prev_id: Identity,
        asset: ContractId,
    ) -> bool {
        return internal_valid_insert_position(icr, next_id, prev_id, asset);
    }

    #[storage(read)]
    fn find_insert_position(
        icr: u64,
        next_id: Identity,
        prev_id: Identity,
        asset: ContractId,
    ) -> (Identity, Identity) {
        return internal_find_insert_position(icr, next_id, prev_id, asset);
    }
}

#[storage(read)]
fn internal_get_first(asset: ContractId) -> Identity {
    return storage.head.get(asset);
}

#[storage(read)]
fn internal_get_last(asset: ContractId) -> Identity {
    return storage.tail.get(asset);
}

#[storage(read)]
fn internal_get_next(id: Identity, asset: ContractId) -> Identity {
    return storage.nodes.get((id, asset)).next_id;
}

#[storage(read)]
fn internal_get_prev(id: Identity, asset: ContractId) -> Identity {
    return storage.nodes.get((id, asset)).prev_id;
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
fn internal_contains(id: Identity, asset: ContractId) -> bool {
    return storage.nodes.get((id, asset)).exists;
}

#[storage(read)]
fn internal_valid_insert_position(
    icr: u64,
    next_id: Identity,
    prev_id: Identity,
    asset: ContractId,
) -> bool {
    let trove_manager_contract = abi(TroveManager, storage.trove_manager_contract_id.value);

    if (next_id == null_identity_address()
        && prev_id == null_identity_address())
    {
        return internal_is_empty();
    } else if (prev_id == null_identity_address()) {
        return storage.head.get(asset) == next_id && icr >= trove_manager_contract.get_nominal_icr(next_id);
    } else if (next_id == null_identity_address()) {
        return storage.tail.get(asset) == prev_id && icr <= trove_manager_contract.get_nominal_icr(prev_id);
    } else {
        return storage.nodes.get((prev_id, asset)).next_id == next_id && trove_manager_contract.get_nominal_icr(prev_id) >= icr && icr >= trove_manager_contract.get_nominal_icr(next_id);
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
    asset: ContractId,
) -> (Identity, Identity) {
    let trove_manager_contract = abi(TroveManager, storage.trove_manager_contract_id.value);

    let mut next_id: Identity = _next_id;
    let mut prev_id: Identity = _prev_id;

    if (prev_id != null_identity_address()) {
        if (!internal_contains(prev_id, asset)
            || icr > trove_manager_contract.get_nominal_icr(prev_id))
        {
            prev_id = null_identity_address()
        }
    }

    if (next_id != null_identity_address()) {
        if (!internal_contains(next_id, asset)
            || icr < trove_manager_contract.get_nominal_icr(prev_id))
        {
            next_id = null_identity_address()
        }
    }

    if (prev_id == null_identity_address()
        && next_id == null_identity_address())
    {
        return internal_descend_list(icr, storage.head.get(asset), asset);
    } else if (prev_id == null_identity_address()) {
        return internal_ascend_list(icr, next_id, asset);
    } else if (next_id == null_identity_address()) {
        return internal_descend_list(icr, prev_id, asset);
    } else {
        return internal_descend_list(icr, prev_id, asset);
    }
}

#[storage(read)]
fn internal_descend_list(nicr: u64, start_id: Identity, asset: ContractId) -> (Identity, Identity) {
    let trove_manager_contract = abi(TroveManager, storage.trove_manager_contract_id.value);

    if (storage.head.get(asset) == start_id
        && nicr >= trove_manager_contract.get_nominal_icr(start_id))
    {
        return (null_identity_address(), start_id);
    }

    let mut prev_id = start_id;
    let mut next_id = storage.nodes.get((prev_id, asset)).next_id;

    while (prev_id != null_identity_address() && !internal_valid_insert_position(nicr, next_id, prev_id, asset)) {
        prev_id = storage.nodes.get((prev_id, asset)).next_id;
        next_id = storage.nodes.get((prev_id, asset)).next_id;
    }

    return (prev_id, next_id);
}

#[storage(read)]
fn internal_ascend_list(nicr: u64, start_id: Identity, asset: ContractId) -> (Identity, Identity) {
    let trove_manager_contract = abi(TroveManager, storage.trove_manager_contract_id.value);

    if (storage.tail.get(asset) == start_id
        && nicr <= trove_manager_contract.get_nominal_icr(start_id))
    {
        return (start_id, null_identity_address());
    }

    let mut next_id = start_id;
    let mut prev_id = storage.nodes.get((next_id, asset)).prev_id;

    while (next_id != null_identity_address() && !internal_valid_insert_position(nicr, next_id, prev_id, asset)) {
        next_id = storage.nodes.get((next_id, asset)).prev_id;
        prev_id = storage.nodes.get((next_id, asset)).prev_id;
    }

    return (prev_id, next_id);
}

#[storage(read, write)]
fn internal_insert(
    id: Identity,
    nicr: u64,
    _prev_id: Identity,
    _next_id: Identity,
    asset: ContractId,
) {
    let trove_manager_contract = abi(TroveManager, storage.trove_manager_contract_id.into());
    require(!internal_is_full(), "list is full");
    require(!internal_contains(id, asset), "id already exists");
    require(null_identity_address() != id, "id must not be zero");
    require(nicr > 0, "icr must be greater than 0");

    let mut next_id: Identity = _next_id;
    let mut prev_id: Identity = _prev_id;

    if (!internal_valid_insert_position(nicr, prev_id, next_id, asset))
    {
        let res = internal_find_insert_position(nicr, prev_id, next_id, asset);
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
        storage.head.insert(asset, id);
        storage.tail.insert(asset, id);
    } else if (prev_id == null_identity_address()) {
        new_node.next_id = storage.head.get(asset);

        edit_node_neighbors(storage.head.get(asset), Option::Some(id), Option::None, asset);

        storage.head.insert(asset, id);
    } else if (next_id == null_identity_address()) {
        new_node.prev_id = storage.tail.get(asset);

        edit_node_neighbors(storage.tail.get(asset), Option::None, Option::Some(id), asset);

        storage.tail.insert(asset, id);
    } else {
        new_node.prev_id = prev_id;
        new_node.next_id = next_id;

        edit_node_neighbors(prev_id, Option::None, Option::Some(id), asset);

        edit_node_neighbors(next_id, Option::Some(id), Option::None, asset);
    }

    storage.nodes.insert((id, asset), new_node);
    storage.size += 1;
}

#[storage(read, write)]
fn edit_node_neighbors(
    id: Identity,
    prev_id: Option<Identity>,
    next_id: Option<Identity>,
    asset: ContractId,
) {
    // TODO Update when StorageMap supports updating values
    let mut node = storage.nodes.get((id, asset));

    if (prev_id.is_some()) {
        node.prev_id = prev_id.unwrap();
    }

    if (next_id.is_some()) {
        node.next_id = next_id.unwrap();
    }

    storage.nodes.insert((id, asset), node);
}

#[storage(read, write)]
fn internal_remove(id: Identity, asset: ContractId) {
    require(internal_contains(id, asset), "id does not exist");

    let mut node = storage.nodes.get((id, asset));

    if (storage.size > 1) {
        if (id == storage.head.get(asset)) {
            storage.head.insert(asset, node.next_id);

            edit_node_neighbors(node.next_id, Option::Some(null_identity_address()), Option::None, asset);
        } else if (id == storage.tail.get(asset)) {
            storage.tail.insert(asset, node.prev_id);

            edit_node_neighbors(node.prev_id, Option::None, Option::Some(null_identity_address()), asset);
        } else {
            edit_node_neighbors(node.prev_id, Option::None, Option::Some(node.next_id), asset);

            edit_node_neighbors(node.next_id, Option::Some(node.prev_id), Option::None, asset);
        }
    } else {
        storage.head.insert(asset, null_identity_address());
        storage.tail.insert(asset, null_identity_address());
    }

    // TODO use storage.nodes.remove(id) when available
    node.exists = false;
    node.prev_id = null_identity_address();
    node.next_id = null_identity_address();
    storage.nodes.insert((id, asset), node);

    storage.size -= 1;
}
