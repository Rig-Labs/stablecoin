contract;

use libraries::trove_manager_interface::{TroveManager};
use libraries::sorted_troves_interface::{SortedTroves};

use std::{identity::Identity};

const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

storage {
    nominal_icr: StorageMap<Identity, u64> = StorageMap {},
    sorted_troves_contract: ContractId = ContractId::from(ZERO_B256),
}

impl TroveManager for Contract {
    #[storage(read, write)]
    fn initialize(id: ContractId) {
        storage.sorted_troves_contract = id;
    }

    #[storage(read)]
    fn get_nominal_icr(id: Identity) -> u64 {
        return storage.nominal_icr.get(id);
    }

    #[storage(read, write)]
    fn set_nominal_icr(id: Identity, value: u64) {
        return storage.nominal_icr.insert(id, value);
    }

    #[storage(read, write)]
    fn set_nominal_icr_and_insert(
        id: Identity,
        value: u64,
        prev_id: Identity,
        next_id: Identity,
    ) {
        storage.nominal_icr.insert(id, value);
        let sorted_troves_contract = abi(SortedTroves, storage.sorted_troves_contract.into());
        sorted_troves_contract.insert(id, value, prev_id, next_id);
    }
}
