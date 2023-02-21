library trove_manager_interface;

abi TroveManager {
     #[storage(read)]
    fn get_nominal_irc(id: Identity) -> u64;
}