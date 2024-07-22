contract;

use libraries::sorted_troves_interface::{Node, SortedTroves};
use libraries::trove_manager_interface::TroveManager;
use libraries::fluid_math::{null_contract, null_identity_address, ZERO_B256};
use std::{
    asset::transfer,
    block::{
        height,
        timestamp,
    },
    call_frames::{
        msg_asset_id,
    },
    context::{
        msg_amount,
    },
    hash::Hash,
};

storage {
    max_size: u64 = 0,
    protocol_manager: ContractId = ContractId::from(ZERO_B256),
    borrower_operations_contract: ContractId = ContractId::from(ZERO_B256),
    head: StorageMap<AssetId, Identity> = StorageMap::<AssetId, Identity> {},
    tail: StorageMap<AssetId, Identity> = StorageMap::<AssetId, Identity> {},
    nodes: StorageMap<(Identity, AssetId), Node> = StorageMap::<(Identity, AssetId), Node> {},
    size: StorageMap<AssetId, u64> = StorageMap::<AssetId, u64> {},
    asset_trove_manager: StorageMap<AssetId, ContractId> = StorageMap::<AssetId, ContractId> {},
    valid_trove_manager: StorageMap<Identity, bool> = StorageMap::<Identity, bool> {},
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
        require(
            storage
                .is_initialized
                .read() == false,
            "Contract is already initialized",
        );
        storage.max_size.write(size);
        storage.protocol_manager.write(protocol_manager);
        storage
            .borrower_operations_contract
            .write(borrower_operations_contract);
        storage.is_initialized.write(true);
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
        asset: AssetId,
    ) {
        require_is_bo_or_tm();
        internal_insert(id, nicr, hint_prev_id, hint_next_id, asset);
    }
    #[storage(read, write)]
    fn add_asset(asset: AssetId, trove_manager: ContractId) {
        require_is_protocol_manager();
        storage.asset_trove_manager.insert(asset, trove_manager);
        storage
            .valid_trove_manager
            .insert(Identity::ContractId(trove_manager), true);
    }
    #[storage(read, write)]
    fn remove(id: Identity, asset: AssetId) {
        require_is_trove_manager();
        internal_remove(id, asset);
    }
    #[storage(read, write)]
    fn re_insert(
        id: Identity,
        nicr: u64,
        prev_id: Identity,
        next_id: Identity,
        asset: AssetId,
    ) {
        require_is_bo_or_tm();
        require(internal_contains(id, asset), "ST: Id must exist");
        require(nicr > 0, "ST: nicr must be greater than 0");
        internal_remove(id, asset);
        internal_insert(id, nicr, prev_id, next_id, asset);
    }
    #[storage(read)]
    fn contains(id: Identity, asset: AssetId) -> bool {
        return internal_contains(id, asset);
    }
    #[storage(read)]
    fn is_full(asset: AssetId) -> bool {
        return internal_is_full(asset);
    }
    #[storage(read)]
    fn is_empty(asset: AssetId) -> bool {
        return internal_is_empty(asset);
    }
    #[storage(read)]
    fn get_size(asset: AssetId) -> u64 {
        return internal_get_size(asset);
    }
    #[storage(read)]
    fn get_max_size() -> u64 {
        return internal_get_max_size();
    }
    #[storage(read)]
    fn get_first(asset: AssetId) -> Identity {
        return internal_get_first(asset);
    }
    #[storage(read)]
    fn get_last(asset: AssetId) -> Identity {
        return internal_get_last(asset);
    }
    #[storage(read)]
    fn get_next(id: Identity, asset: AssetId) -> Identity {
        return internal_get_next(id, asset);
    }
    #[storage(read)]
    fn get_prev(id: Identity, asset: AssetId) -> Identity {
        return internal_get_prev(id, asset);
    }
    #[storage(read)]
    fn valid_insert_position(
        icr: u64,
        next_id: Identity,
        prev_id: Identity,
        asset: AssetId,
    ) -> bool {
        return internal_valid_insert_position(icr, next_id, prev_id, asset);
    }
    #[storage(read)]
    fn find_insert_position(
        icr: u64,
        next_id: Identity,
        prev_id: Identity,
        asset: AssetId,
    ) -> (Identity, Identity) {
        let trove_manager = storage.asset_trove_manager.get(asset).read();
        return internal_find_insert_position(icr, next_id, prev_id, asset, trove_manager);
    }
}
#[storage(read)]
fn internal_get_first(asset: AssetId) -> Identity {
    return storage.head.get(asset).try_read().unwrap_or(null_identity_address());
}
#[storage(read)]
fn internal_get_last(asset: AssetId) -> Identity {
    return storage.tail.get(asset).try_read().unwrap_or(null_identity_address());
}
#[storage(read)]
fn internal_get_next(id: Identity, asset: AssetId) -> Identity {
    match storage.nodes.get((id, asset)).try_read() {
        Some(node) => return node.next_id,
        None => return Identity::Address(Address::from(ZERO_B256)),
    }
}
#[storage(read)]
fn internal_get_prev(id: Identity, asset: AssetId) -> Identity {
    match storage.nodes.get((id, asset)).try_read() {
        Some(node) => return node.prev_id,
        None => return Identity::Address(Address::from(ZERO_B256)),
    }
}
#[storage(read)]
fn internal_get_max_size() -> u64 {
    return storage.max_size.read();
}
#[storage(read)]
fn internal_get_size(asset: AssetId) -> u64 {
    return storage.size.get(asset).try_read().unwrap_or(0);
}
#[storage(read)]
fn internal_is_full(asset: AssetId) -> bool {
    match storage.size.get(asset).try_read() {
        Some(size) => return size == storage.max_size.read(),
        None => return false,
    }
}
#[storage(read)]
fn internal_is_empty(asset: AssetId) -> bool {
    match storage.size.get(asset).try_read() {
        Some(size) => return size == 0,
        None => return false,
    }
}
#[storage(read)]
fn internal_contains(id: Identity, asset: AssetId) -> bool {
    match storage.nodes.get((id, asset)).try_read() {
        Some(node) => return node.exists,
        None => return false,
    }
}
#[storage(read)]
fn internal_get_head(asset: AssetId) -> Identity {
    match storage.head.get(asset).try_read() {
        Some(head) => return head,
        None => return Identity::Address(Address::from(ZERO_B256)),
    }
}
#[storage(read)]
fn internal_get_tail(asset: AssetId) -> Identity {
    match storage.tail.get(asset).try_read() {
        Some(tail) => return tail,
        None => return Identity::Address(Address::from(ZERO_B256)),
    }
}
#[storage(read)]
fn internal_valid_insert_position(
    icr: u64,
    prev_id: Identity,
    next_id: Identity,
    asset: AssetId,
) -> bool {
    let trove_manager_contract = storage.asset_trove_manager.get(asset).read();
    let trove_manager = abi(TroveManager, trove_manager_contract.bits());
    if (next_id == null_identity_address()
        && prev_id == null_identity_address())
    {
        return internal_is_empty(asset);
    } else if (prev_id == null_identity_address()) {
        return internal_get_head(asset) == next_id && icr >= trove_manager.get_nominal_icr(next_id);
    } else if (next_id == null_identity_address()) {
        return internal_get_tail(asset) == prev_id && icr <= trove_manager.get_nominal_icr(prev_id);
    } else {
        return storage.nodes.get((prev_id, asset)).read().next_id == next_id && trove_manager.get_nominal_icr(prev_id) >= icr && icr >= trove_manager.get_nominal_icr(next_id);
    }
    return true;
}
#[storage(read)]
fn require_is_protocol_manager() {
    let sender = msg_sender().unwrap();
    require(
        Identity::ContractId(storage.protocol_manager.read()) == sender,
        "ST: Not PM",
    );
}
#[storage(read)]
fn require_is_trove_manager() {
    let sender = msg_sender().unwrap();
    require(storage.valid_trove_manager.get(sender).read(), "ST: Not TM");
}
#[storage(read)]
fn require_is_bo_or_tm() {
    let sender = msg_sender().unwrap();
    let borrow_operations = Identity::ContractId(storage.borrower_operations_contract.read());
    let is_trove_manager = storage.valid_trove_manager.get(sender).try_read().unwrap_or(false);
    require(
        borrow_operations == sender || is_trove_manager,
        "ST: Not BO or TM",
    );
}
#[storage(read)]
fn internal_find_insert_position(
    nicr: u64,
    prev_id: Identity,
    next_id: Identity,
    asset: AssetId,
    trove_manager_contract: ContractId,
) -> (Identity, Identity) {
    let trove_manager = abi(TroveManager, trove_manager_contract.bits());
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
        return internal_descend_list(
            nicr,
            internal_get_head(asset),
            asset,
            trove_manager_contract,
        );
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
    asset: AssetId,
    trove_manager_contract: ContractId,
) -> (Identity, Identity) {
    let trove_manager = abi(TroveManager, trove_manager_contract.bits());
    if (internal_get_head(asset) == start_id
        && nicr >= trove_manager.get_nominal_icr(start_id))
    {
        return (Identity::Address(Address::from(ZERO_B256)), start_id);
    }
    let mut prev_id = start_id;
    let mut next_id = internal_get_next(prev_id, asset);
    while (prev_id != null_identity_address() && !internal_valid_insert_position(nicr, prev_id, next_id, asset)) {
        prev_id = internal_get_next(prev_id, asset);
        next_id = internal_get_next(prev_id, asset);
    }
    return (prev_id, next_id);
}
#[storage(read)]
fn internal_ascend_list(
    nicr: u64,
    start_id: Identity,
    asset: AssetId,
    trove_manager_contract: ContractId,
) -> (Identity, Identity) {
    let trove_manager = abi(TroveManager, trove_manager_contract.bits());
    if (internal_get_tail(asset) == start_id
        && nicr <= trove_manager.get_nominal_icr(start_id))
    {
        return (start_id, null_identity_address());
    }
    let mut next_id = start_id;
    let mut prev_id = internal_get_prev(next_id, asset);
    while (next_id != null_identity_address() && !internal_valid_insert_position(nicr, prev_id, next_id, asset)) {
        next_id = internal_get_prev(next_id, asset);
        prev_id = internal_get_prev(next_id, asset);
    }
    return (prev_id, next_id);
}
#[storage(read, write)]
fn internal_insert(
    id: Identity,
    nicr: u64,
    hint_prev_id: Identity,
    hint_next_id: Identity,
    asset: AssetId,
) {
    let trove_manager_contract = storage.asset_trove_manager.get(asset).read();
    require(!internal_is_full(asset), "ST: list is full");
    require(!internal_contains(id, asset), "ST: id already exists");
    require(null_identity_address() != id, "ST: id must not be zero");
    require(nicr > 0, "ST: icr must be greater than 0");
    let mut next_id: Identity = hint_next_id;
    let mut prev_id: Identity = hint_prev_id;
    if (!internal_valid_insert_position(nicr, prev_id, next_id, asset)) {
        // Sender's hint was not a valid insert position
        // Use sender's hint to find a valid insert position
        let res = internal_find_insert_position(nicr, prev_id, next_id, asset, trove_manager_contract);
        prev_id = res.0;
        next_id = res.1;
    }
    let mut new_node = Node {
        exists: true,
        prev_id: Identity::Address(Address::from(ZERO_B256)),
        next_id: Identity::Address(Address::from(ZERO_B256)),
    };
    if (prev_id == null_identity_address() && next_id == null_identity_address()) {
        // Insert as head and tail
        storage.head.insert(asset, id);
        storage.tail.insert(asset, id);
    } else if (prev_id == null_identity_address()) {
        // Insert before `prev_id` as the head
        new_node.next_id = internal_get_head(asset);
        edit_node_neighbors(
            internal_get_head(asset),
            Option::Some(id),
            Option::None,
            asset,
        );
        storage.head.insert(asset, id);
    } else if (next_id == null_identity_address()) {
        // Insert after `next_id` as the tail
        new_node.prev_id = internal_get_tail(asset);
        edit_node_neighbors(
            internal_get_tail(asset),
            Option::None,
            Option::Some(id),
            asset,
        );
        storage.tail.insert(asset, id);
    } else {
        // Insert at insert position between `prev_id` and `next_id`
        new_node.prev_id = prev_id;
        new_node.next_id = next_id;
        edit_node_neighbors(prev_id, Option::None, Option::Some(id), asset);
        edit_node_neighbors(next_id, Option::Some(id), Option::None, asset);
    }
    storage.nodes.insert((id, asset), new_node);
    let new_size = internal_get_size(asset) + 1;
    storage.size.insert(asset, new_size);
}
#[storage(read, write)]
fn edit_node_neighbors(
    id: Identity,
    prev_id: Option<Identity>,
    next_id: Option<Identity>,
    asset: AssetId,
) {
    let mut node = storage.nodes.get((id, asset)).read();
    if (prev_id.is_some()) {
        node.prev_id = prev_id.unwrap();
    }
    if (next_id.is_some()) {
        node.next_id = next_id.unwrap();
    }
    storage.nodes.insert((id, asset), node);
}
#[storage(read, write)]
fn internal_remove(id: Identity, asset: AssetId) {
    require(internal_contains(id, asset), "ST: Id does not exist");
    let mut node = storage.nodes.get((id, asset)).read();
    if (storage.size.get(asset).read() > 1) {
        if (id == internal_get_head(asset)) {
            // The removed node is the head
            // Set head to next node
            storage.head.insert(asset, node.next_id);
            // Set prev pointer of new head to null
            edit_node_neighbors(
                node.next_id,
                Option::Some(null_identity_address()),
                Option::None,
                asset,
            );
        } else if (id == internal_get_tail(asset)) {
            // The removed node is the tail
            // Set tail to previous node
            storage.tail.insert(asset, node.prev_id);
            // Set next pointer of new tail to null
            edit_node_neighbors(
                node.prev_id,
                Option::None,
                Option::Some(null_identity_address()),
                asset,
            );
        } else {
            // The removed node is neither the head nor the tail
            // Set next pointer of previous node to the next node
            edit_node_neighbors(
                node.prev_id,
                Option::None,
                Option::Some(node.next_id),
                asset,
            );
            // Set prev pointer of next node to the previous node
            edit_node_neighbors(
                node.next_id,
                Option::Some(node.prev_id),
                Option::None,
                asset,
            );
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
    let new_size = storage.size.get(asset).read() - 1;
    storage.size.insert(asset, new_size);
}
