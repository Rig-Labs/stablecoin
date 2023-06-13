library data_structures;
use libraries::numbers::*;
use std::{u128::U128};

pub struct AssetContracts {
    trove_manager: ContractId,
    oracle: ContractId,
    sorted_troves: ContractId,
}

pub struct Snapshots {
    P: U128,
    G: U128,
    scale: u64,
    epoch: u64,
}

impl Snapshots {
    pub fn default() -> Self {
        Snapshots {
            P: U128::from_u64(0),
            G: U128::from_u64(0),
            scale: 0,
            epoch: 0,
        }
    }
}
