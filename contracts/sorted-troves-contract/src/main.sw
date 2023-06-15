contract;

use libraries::sorted_troves_interface::{SortedTroves};
use libraries::trove_manager_interface::{TroveManager};
use libraries::data_structures::Node;
use libraries::fluid_math::{null_contract, null_identity_address};
/*
* A sorted doubly linked list with nodes sorted in descending order.
*
* Nodes map to active Troves in the system - the ID property is the address of a Trove owner.
* Nodes are ordered according to their current nominal individual collateral ratio (NICR),
* which is like the ICR but without the price, i.e., just collateral / debt.
*
* The list optionally accepts insert position hints.
*
* NICRs are computed dynamically at runtime, and not stored on the Node. This is because NICRs of active Troves
* change dynamically as liquidation events occur.
*
* The list relies on the fact that liquidation events preserve ordering: a liquidation decreases the NICRs of all active Troves,
* but maintains their order. A node inserted based on current NICR will maintain the correct position,
* relative to it's peers, as rewards accumulate, as long as it's raw collateral and debt have not changed.
* Thus, Nodes remain sorted by current NICR.
*
* Nodes need only be re-inserted upon a Trove operation - when the owner adds or removes collateral or debt
* to their position.
*
* Ordering checks for insertion are performed by comparing an NICR argument to the current NICR, calculated at runtime.
* The list relies on the property that ordering by ICR is maintained as the ASSET:USD price varies.
*
*/
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
    max_size: u64 = 0,
    protocol_manager: ContractId = null_contract(),
    borrower_operations_contract: ContractId = null_contract(),
    head: StorageMap<ContractId, Identity> = StorageMap {},
    tail: StorageMap<ContractId, Identity> = StorageMap {},
    nodes: StorageMap<(Identity, ContractId), Node> = StorageMap {},
    size: StorageMap<ContractId, u64> = StorageMap {},
    asset_trove_manager: StorageMap<ContractId, ContractId> = StorageMap {},
    valid_trove_manager: StorageMap<Identity, bool> = StorageMap {},
    is_initialized: bool = false,
}

impl SortedTroves for Contract {
    #[storage(read, write)]
    fn set_params(
        size: u64,
        protocol_manager: ContractId,
        borrower_operations_contract: ContractId,
    ) {
        require(size > 0, "size must be greater than 0");
        require(storage.is_initialized == false, "Contract is already initialized");

        storage.max_size = size;
        storage.protocol_manager = protocol_manager;
        storage.borrower_operations_contract = borrower_operations_contract;
        storage.is_initialized = true;
    }

    /*
     * @dev Add a node to the list
     * @param _id Node's id
     * @param _NICR Node's NICR
     * @param _prevId Id of previous node for the insert position
     * @param _nextId Id of next node for the insert position
     */
    #[storage(read, write)]
    fn insert(
        id: Identity,
        nicr: u64,
        hint_prev_id: Identity,
        hint_next_id: Identity,
        asset: ContractId,
    ) {
        require_is_bo_or_tm();

        internal_insert(id, nicr, hint_prev_id, hint_next_id, asset);
    }

    #[storage(read, write)]
    fn add_asset(asset: ContractId, trove_manager: ContractId) {
        require_is_protocol_manager();
        storage.asset_trove_manager.insert(asset, trove_manager);
        storage.valid_trove_manager.insert(Identity::ContractId(trove_manager), true);
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

        require(internal_contains(id, asset), "ST: Id must exist");
        require(nicr > 0, "ST: nicr must be greater than 0");

        internal_remove(id, asset);
        internal_insert(id, nicr, prev_id, next_id, asset);
    }

    #[storage(read)]
    fn contains(id: Identity, asset: ContractId) -> bool {
        return internal_contains(id, asset);
    }

    #[storage(read)]
    fn is_full(asset: ContractId) -> bool {
        return internal_is_full(asset);
    }

    #[storage(read)]
    fn is_empty(asset: ContractId) -> bool {
        return internal_is_empty(asset);
    }

    #[storage(read)]
    fn get_size(asset: ContractId) -> u64 {
        return internal_get_size(asset);
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
        let trove_manager = storage.asset_trove_manager.get(asset);
        return internal_find_insert_position(icr, next_id, prev_id, asset, trove_manager);
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
fn internal_get_size(asset: ContractId) -> u64 {
    return storage.size.get(asset);
}

#[storage(read)]
fn internal_is_full(asset: ContractId) -> bool {
    return storage.size.get(asset) == storage.max_size;
}

#[storage(read)]
fn internal_is_empty(asset: ContractId) -> bool {
    return storage.size.get(asset) == 0;
}

#[storage(read)]
fn internal_contains(id: Identity, asset: ContractId) -> bool {
    return storage.nodes.get((id, asset)).exists;
}

#[storage(read)]
fn internal_valid_insert_position(
    icr: u64,
    prev_id: Identity,
    next_id: Identity,
    asset: ContractId,
) -> bool {
    let trove_manager_contract = storage.asset_trove_manager.get(asset);
    let trove_manager = abi(TroveManager, trove_manager_contract.value);

    if (next_id == null_identity_address()
        && prev_id == null_identity_address())
    {
        return internal_is_empty(asset);
    } else if (prev_id == null_identity_address()) {
        return storage.head.get(asset) == next_id && icr >= trove_manager.get_nominal_icr(next_id);
    } else if (next_id == null_identity_address()) {
        return storage.tail.get(asset) == prev_id && icr <= trove_manager.get_nominal_icr(prev_id);
    } else {
        return storage.nodes.get((prev_id, asset)).next_id == next_id && trove_manager.get_nominal_icr(prev_id) >= icr && icr >= trove_manager.get_nominal_icr(next_id);
    }

    return true;
}

#[storage(read)]
fn require_is_protocol_manager() {
    let sender = msg_sender().unwrap();
    require(Identity::ContractId(storage.protocol_manager) == sender, "ST: Not PM");
}

#[storage(read)]
fn require_is_trove_manager() {
    let sender = msg_sender().unwrap();

    require(storage.valid_trove_manager.get(sender), "ST: Not TM");
}

#[storage(read)]
fn require_is_bo_or_tm() {
    let sender = msg_sender().unwrap();
    let borrow_operations = Identity::ContractId(storage.borrower_operations_contract);
    let is_trove_manager = storage.valid_trove_manager.get(sender);
    require(borrow_operations == sender || is_trove_manager, "ST: Not BO or TM");
}

#[storage(read)]
fn internal_find_insert_position(
    nicr: u64,
    prev_id: Identity,
    next_id: Identity,
    asset: ContractId,
    trove_manager_contract: ContractId,
) -> (Identity, Identity) {
    let trove_manager = abi(TroveManager, trove_manager_contract.value);

    let mut next_id: Identity = next_id;
    let mut prev_id: Identity = prev_id;

    if (prev_id != null_identity_address()) {
        if (!internal_contains(prev_id, asset)
            || nicr > trove_manager.get_nominal_icr(prev_id))
        {
            prev_id = null_identity_address()
        }
    }

    if (next_id != null_identity_address()) {
        if (!internal_contains(next_id, asset)
            || nicr < trove_manager.get_nominal_icr(prev_id))
        {
            next_id = null_identity_address()
        }
    }

    if (prev_id == null_identity_address()
        && next_id == null_identity_address())
    {
        return internal_descend_list(nicr, storage.head.get(asset), asset, trove_manager_contract);
    } else if (prev_id == null_identity_address()) {
        return internal_ascend_list(nicr, next_id, asset, trove_manager_contract);
    } else if (next_id == null_identity_address()) {
        return internal_descend_list(nicr, prev_id, asset, trove_manager_contract);
    } else {
        return internal_descend_list(nicr, prev_id, asset, trove_manager_contract);
    }
}

#[storage(read)]
fn internal_descend_list(
    nicr: u64,
    start_id: Identity,
    asset: ContractId,
    trove_manager_contract: ContractId,
) -> (Identity, Identity) {
    let trove_manager = abi(TroveManager, trove_manager_contract.value);
    if (storage.head.get(asset) == start_id
        && nicr >= trove_manager.get_nominal_icr(start_id))
    {
        return (null_identity_address(), start_id);
    }

    let mut prev_id = start_id;
    let mut next_id = storage.nodes.get((prev_id, asset)).next_id;
    while (prev_id != null_identity_address() && !internal_valid_insert_position(nicr, prev_id, next_id, asset)) {
        prev_id = storage.nodes.get((prev_id, asset)).next_id;
        next_id = storage.nodes.get((prev_id, asset)).next_id;
    }
    return (prev_id, next_id);
}

#[storage(read)]
fn internal_ascend_list(
    nicr: u64,
    start_id: Identity,
    asset: ContractId,
    trove_manager_contract: ContractId,
) -> (Identity, Identity) {
    let trove_manager = abi(TroveManager, trove_manager_contract.value);

    if (storage.tail.get(asset) == start_id
        && nicr <= trove_manager.get_nominal_icr(start_id))
    {
        return (start_id, null_identity_address());
    }

    let mut next_id = start_id;
    let mut prev_id = storage.nodes.get((next_id, asset)).prev_id;

    while (next_id != null_identity_address() && !internal_valid_insert_position(nicr, prev_id, next_id, asset)) {
        next_id = storage.nodes.get((next_id, asset)).prev_id;
        prev_id = storage.nodes.get((next_id, asset)).prev_id;
    }

    return (prev_id, next_id);
}

#[storage(read, write)]
fn internal_insert(
    id: Identity,
    nicr: u64,
    hint_prev_id: Identity,
    hint_next_id: Identity,
    asset: ContractId,
) {
    let trove_manager_contract = storage.asset_trove_manager.get(asset);
    let trove_manager = abi(TroveManager, trove_manager_contract.value);
    require(!internal_is_full(asset), "ST: list is full");
    require(!internal_contains(id, asset), "ST: id already exists");
    require(null_identity_address() != id, "ST: id must not be zero");
    require(nicr > 0, "ST: icr must be greater than 0");

    let mut next_id: Identity = hint_next_id;
    let mut prev_id: Identity = hint_prev_id;

    if (!internal_valid_insert_position(nicr, prev_id, next_id, asset))
    {
        // Sender's hint was not a valid insert position
        // Use sender's hint to find a valid insert position
        let res = internal_find_insert_position(nicr, prev_id, next_id, asset, trove_manager_contract);
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
        // Insert as head and tail
        storage.head.insert(asset, id);
        storage.tail.insert(asset, id);
    } else if (prev_id == null_identity_address()) {
        // Insert before `prev_id` as the head
        new_node.next_id = storage.head.get(asset);

        edit_node_neighbors(storage.head.get(asset), Option::Some(id), Option::None, asset);

        storage.head.insert(asset, id);
    } else if (next_id == null_identity_address()) {
         // Insert after `next_id` as the tail
        new_node.prev_id = storage.tail.get(asset);

        edit_node_neighbors(storage.tail.get(asset), Option::None, Option::Some(id), asset);

        storage.tail.insert(asset, id);
    } else {
        // Insert at insert position between `prev_id` and `next_id`
        new_node.prev_id = prev_id;
        new_node.next_id = next_id;

        edit_node_neighbors(prev_id, Option::None, Option::Some(id), asset);

        edit_node_neighbors(next_id, Option::Some(id), Option::None, asset);
    }

    storage.nodes.insert((id, asset), new_node);
    let new_size = storage.size.get(asset) + 1;
    storage.size.insert(asset, new_size);
}

#[storage(read, write)]
fn edit_node_neighbors(
    id: Identity,
    prev_id: Option<Identity>,
    next_id: Option<Identity>,
    asset: ContractId,
) {
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
    require(internal_contains(id, asset), "ST: Id does not exist");

    let mut node = storage.nodes.get((id, asset));

    if (storage.size.get(asset) > 1) {
        if (id == storage.head.get(asset)) {
            // The removed node is the head
            // Set head to next node
            storage.head.insert(asset, node.next_id);
            // Set prev pointer of new head to null
            edit_node_neighbors(node.next_id, Option::Some(null_identity_address()), Option::None, asset);
        } else if (id == storage.tail.get(asset)) {
            // The removed node is the tail
            // Set tail to previous node
            storage.tail.insert(asset, node.prev_id);
            // Set next pointer of new tail to null
            edit_node_neighbors(node.prev_id, Option::None, Option::Some(null_identity_address()), asset);
        } else {
            // The removed node is neither the head nor the tail
            // Set next pointer of previous node to the next node
            edit_node_neighbors(node.prev_id, Option::None, Option::Some(node.next_id), asset);
            // Set prev pointer of next node to the previous node
            edit_node_neighbors(node.next_id, Option::Some(node.prev_id), Option::None, asset);
        }
    } else {
        // List contains a single node
        // Set the head and tail to null
        storage.head.insert(asset, null_identity_address());
        storage.tail.insert(asset, null_identity_address());
    }

    // TODO use storage.nodes.remove(id) when available
    node.exists = false;
    node.prev_id = null_identity_address();
    node.next_id = null_identity_address();
    storage.nodes.insert((id, asset), node);

    let new_size = storage.size.get(asset) - 1;
    storage.size.insert(asset, new_size);
}
