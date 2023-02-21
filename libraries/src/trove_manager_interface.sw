library trove_manager_interface;

abi TroveManager {
    #[storage(read)]
    fn get_nominal_icr(id: Identity) -> u64;

    #[storage(read, write)]
    fn set_nominal_icr(id: Identity, value: u64);
}
