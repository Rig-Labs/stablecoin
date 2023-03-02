contract;

use libraries::trove_manager_interface::{TroveManager};
use libraries::fluid_math::*;
use libraries::sorted_troves_interface::{SortedTroves};
use libraries::data_structures::{Status};

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
    logging::log,
    storage::{
        StorageMap,
        StorageVec,
    },
    token::transfer,
};

const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

storage {
    sorted_troves_contract: ContractId = ContractId::from(ZERO_B256),
    borrow_operations_contract: ContractId = ContractId::from(ZERO_B256),
    nominal_icr: StorageMap<Identity, u64> = StorageMap {},
}

impl TroveManager for Contract {
    #[storage(read, write)]
    fn initialize(borrow_operations: ContractId, sorted_troves: ContractId) {
        storage.sorted_troves_contract = sorted_troves;
        storage.borrow_operations_contract = borrow_operations;
    }

    #[storage(read)]
    fn get_nominal_icr(id: Identity) -> u64 {
        storage.nominal_icr.get(id)
    }

    #[storage(read, write)]
    fn set_nominal_icr_and_insert(
        id: Identity,
        value: u64,
        prev_id: Identity,
        next_id: Identity,
    ) {
        storage.nominal_icr.insert(id, value);

        let sorted_troves_contract = abi(SortedTroves, storage.sorted_troves_contract.value);
        sorted_troves_contract.insert(id, value, prev_id, next_id);
    }

    #[storage(read, write)]
    fn remove(id: Identity) {
        storage.nominal_icr.insert(id, 0);
        let sorted_troves_contract = abi(SortedTroves, storage.sorted_troves_contract.into());
        sorted_troves_contract.remove(id);
    }

    #[storage(read, write)]
    fn set_trove_status(id: Identity, status: Status) {}

    #[storage(read, write)]
    fn increase_trove_coll(id: Identity, coll: u64) -> u64 {
       return 0 
    }

    #[storage(read, write)]
    fn increase_trove_debt(id: Identity, debt: u64) -> u64 {
        return 0 
    }

    #[storage(read, write)]
    fn decrease_trove_coll(id: Identity, value: u64) -> u64 {
        return 0 
    }

    #[storage(read, write)]
    fn decrease_trove_debt(id: Identity, value: u64) -> u64 {
        return 0 
    }

    #[storage(read, write)]
    fn add_trove_owner_to_array(id: Identity) -> u64 {
        return 0
    }

    #[storage(read)]
    fn get_trove_debt(id: Identity) -> u64 {
        return 0
    }

    #[storage(read)]
    fn get_trove_coll(id: Identity) -> u64 {
        return 0
    }

    #[storage(read, write)]
    fn close_trove(id: Identity) {
        require_caller_is_borrow_operations_contract();

        internal_close_trove(id, Status::ClosedByOwner);
    }

    #[storage(read, write)]
    fn remove_stake(id: Identity) {}

    #[storage(read, write)]
    fn liquidate(id: Identity) {}

    #[storage(read, write)]
    fn liquidate_troves(num_troves: u64) {}

    #[storage(read, write)]
    fn batch_liquidate_troves(ids: Vec<Identity>) {}
}

#[storage(read, write)]
fn internal_close_trove(id: Identity, close_status: Status) {
    let sorted_troves_contract = abi(SortedTroves, storage.sorted_troves_contract.into());
    sorted_troves_contract.remove(id);
}


#[storage(read)]
fn require_caller_is_borrow_operations_contract() {
    let caller = msg_sender().unwrap();
    let borrow_operations_contract = Identity::ContractId(storage.borrow_operations_contract);
    require(caller == borrow_operations_contract, "Caller is not the Borrow Operations contract");
}

