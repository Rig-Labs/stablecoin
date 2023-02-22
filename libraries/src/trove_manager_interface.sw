library trove_manager_interface;

abi TroveManager {
    #[storage(read)]
    fn get_nominal_icr(id: Identity) -> u64;

    #[storage(read, write)]
    fn initialize(id: ContractId);

    #[storage(read, write)]
    fn set_nominal_icr(id: Identity, value: u64);

    #[storage(read, write)]
    fn remove(id: Identity);

    #[storage(read, write)]
    fn set_nominal_icr_and_insert(id: Identity, value: u64, prev_id: Identity, next_id: Identity);
}
