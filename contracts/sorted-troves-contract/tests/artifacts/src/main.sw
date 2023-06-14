contract;

use libraries::fluid_math::*;
use libraries::sorted_troves_interface::{SortedTroves};
use libraries::data_structures::{Status};

abi TroveManager {
    #[storage(read, write)]
    fn initialize(borrow_operations: ContractId, sorted_troves: ContractId, asset: ContractId, stability_pool: ContractId, default_pool: ContractId, active_pool: ContractId, coll_surplus_pool: ContractId, usdf_contract: ContractId);

    #[storage(read, write)]
    fn set_nominal_icr_and_insert(id: Identity, value: u64, prev_id: Identity, next_id: Identity, asset: ContractId);

    #[storage(read, write)]
    fn remove(id: Identity, asset: ContractId);

    #[storage(read)]
    fn get_nominal_icr(id: Identity) -> u64;

    #[storage(read, write)]
    fn close_trove(id: Identity, asset: ContractId);

    #[storage(read)]
    fn get_trove_status(id: Identity) -> Status;

    #[storage(read, write)]
    fn add_asset(asset: ContractId, trove_manager: ContractId);
}

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
    fn initialize(
        borrow_operations: ContractId,
        sorted_troves: ContractId,
        oracle: ContractId,
        stability_pool: ContractId,
        default_pool: ContractId,
        active_pool: ContractId,
        coll_surplus: ContractId,
        usdf_contract: ContractId,
    ) {
        storage.sorted_troves_contract = sorted_troves;
        storage.borrow_operations_contract = borrow_operations;
    }

    #[storage(read)]
    fn get_nominal_icr(id: Identity) -> u64 {
        storage.nominal_icr.get(id)
    }

    #[storage(read, write)]
    fn add_asset(asset: ContractId, trove_manager: ContractId) {
        let sorted_troves_contract = abi(SortedTroves, storage.sorted_troves_contract.into());
        sorted_troves_contract.add_asset(asset, trove_manager);
    }

    #[storage(read, write)]
    fn set_nominal_icr_and_insert(
        id: Identity,
        value: u64,
        prev_id: Identity,
        next_id: Identity,
        asset: ContractId,
    ) {
        storage.nominal_icr.insert(id, value);

        let sorted_troves_contract = abi(SortedTroves, storage.sorted_troves_contract.value);
        sorted_troves_contract.insert(id, value, prev_id, next_id, asset);
    }

    #[storage(read, write)]
    fn remove(id: Identity, asset: ContractId) {
        storage.nominal_icr.insert(id, 0);
        let sorted_troves_contract = abi(SortedTroves, storage.sorted_troves_contract.into());
        sorted_troves_contract.remove(id, asset);
    }

    #[storage(read)]
    fn get_trove_status(id: Identity) -> Status {
        return Status::Active
    }

    #[storage(read, write)]
    fn close_trove(id: Identity, asset: ContractId) {
        require_caller_is_borrow_operations_contract();

        internal_close_trove(id, Status::ClosedByOwner, asset);
    }
}

#[storage(read, write)]
fn internal_close_trove(id: Identity, close_status: Status, asset: ContractId) {
    let sorted_troves_contract = abi(SortedTroves, storage.sorted_troves_contract.into());
    sorted_troves_contract.remove(id, asset);
}

#[storage(read)]
fn require_caller_is_borrow_operations_contract() {
    let caller = msg_sender().unwrap();
    let borrow_operations_contract = Identity::ContractId(storage.borrow_operations_contract);
    require(caller == borrow_operations_contract, "Caller is not the Borrow Operations contract");
}
