library data_structures;

use libraries::data_structures::{Status};

pub struct Trove {
    debt: u64,
    coll: u64,
    stake: u64,
    array_index: u64,
    status: Status,
}

impl Trove {
    pub fn default() -> Self {
        Trove {
            debt: 0,
            coll: 0,
            stake: 0,
            array_index: 0,
            status: Status::Active,
        }
    }
}

pub struct RewardSnapshot {
    asset: u64,
    usdf_debt: u64,
}
