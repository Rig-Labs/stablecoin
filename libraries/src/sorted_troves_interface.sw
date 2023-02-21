library sorted_troves_interface;

abi SortedTroves {
    #[storage(read,write)]
    fn set_params(size: u64, trove_manager_identity: Identity, borrower_operations_identity: Identity);

    #[storage(read,write)]
    fn insert(id: Identity, icr: u64, prev_id: Identity, next_id: Identity);

    #[storage(read,write)]
    fn remove(id: Identity);

    #[storage(read,write)]
    fn re_insert(id: Identity, new_icr: u64, prev_id: Identity, next_id: Identity);

    #[storage(read)]
    fn contains(id: Identity) -> bool;

    #[storage(read)]
    fn is_full() -> bool;

    #[storage(read)]
    fn is_empty() -> bool;

    #[storage(read)]
    fn get_size() -> u64;

    #[storage(read)]
    fn get_max_size() -> u64;

    #[storage(read)]
    fn get_first() -> Identity;

    #[storage(read)]
    fn get_last() -> Identity;

    #[storage(read)]
    fn get_next(id: Identity) -> Identity;

     #[storage(read)]
    fn get_prev(id: Identity) -> Identity;

     #[storage(read)]
    fn valid_insert_position(icr: u64, next_id: Identity,prev_id: Identity) -> bool;

     #[storage(read)]
    fn find_insert_position(icr: u64, next_id: Identity,prev_id: Identity) -> (Identity,Identity);
}