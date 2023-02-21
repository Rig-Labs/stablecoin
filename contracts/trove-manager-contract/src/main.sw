contract;

use libraries::trove_manager_interface::{TroveManager};

storage {
    nominal_icr: StorageMap<Identity, u64> = StorageMap {},
}

impl TroveManager for Contract {
    #[storage(read)]
    fn get_nominal_icr(id: Identity) -> u64 {
        return storage.nominal_icr.get(id);
    }

    #[storage(read, write)]
    fn set_nominal_icr(id: Identity, value: u64) {
        return storage.nominal_icr.insert(id, value);
    }
}
