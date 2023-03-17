library data_structures;
use libraries::numbers::*;
use std::{u128::U128};

pub struct Snapshots {
    S: U128,
    P: U128,
    G: U128,
    scale: u64,
    epoch: u64,
}

impl Snapshots {
    pub fn default() -> Self {
        Snapshots {
            S: U128::from_u64(0),
            P: U128::from_u64(0),
            G: U128::from_u64(0),
            scale: 0,
            epoch: 0,
        }
    }
}
// TODO Change to u128 when possible
