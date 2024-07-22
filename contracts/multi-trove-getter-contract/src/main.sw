contract;

// To the auditor: This contract is not used in the system. It is only used for querying the system.

use libraries::trove_manager_interface::TroveManager;
use libraries::sorted_troves_interface::SortedTroves;
use libraries::fluid_math::*;
use std::{
    asset::transfer,
    auth::msg_sender,
    call_frames::{
        msg_asset_id,
    },
    context::{
        msg_amount,
    },
    hash::Hasher,
    logging::log,
};

storage {}

abi MultiTroveGetter {
    // #[storage(read)]
    // fn get_multiple_sorted_troves(sorted_troves_contract: ContractId, trove_manager_contract: ContractId, start_indx: u64, count: u8) -> Vec<CombinedTroveData>;
}

struct CombinedTroveData {
    address: Identity,
    collateral: u64,
    debt: u64,
    stake: u64,
    snapshot_collateral: u64,
    snapshot_debt: u64,
}

impl MultiTroveGetter for Contract {
    // #[storage(read)]
    // fn get_multiple_sorted_troves(
    //     sorted_troves_contract: ContractId,
    //     trove_manager_contract: ContractId,
    //     start_indx: u64,
    //     count: u8,
    // ) -> Vec<CombinedTroveData> {
    //     let mut troves = Vec::new();
    //     let mut index = start_indx;
    //     let mut current_count = 0;

    //     return Vec::new();
    // }
}

#[storage(read)]
fn get_multiple_sorted_troves_from_head(
    sorted_troves_contract: ContractId,
    trove_manager_contract: ContractId,
    asset_id: AssetId,
    start_indx: u64,
    count: u8,
) -> Vec<CombinedTroveData> {
    let sorted_troves = abi(SortedTroves, sorted_troves_contract.bits());
    let trove_manager = abi(TroveManager, trove_manager_contract.bits());

    let mut index = start_indx;
    let mut curr_index = 0;
    let mut current_count: u8 = 0;

    let mut current_trove_owner = sorted_troves.get_first(asset_id);
    let mut troves: Vec<CombinedTroveData> = Vec::new();

    while curr_index < index {
        current_trove_owner = sorted_troves.get_next(current_trove_owner, asset_id);
        curr_index += 1;
    }

    while current_count < count {}

    return troves;
}
