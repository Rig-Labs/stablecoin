library data_structures;

pub struct Trove {
    debt :u64,
    coll :u64,
    stake:u64,
    array_index:u64,
    status:Status,
}

pub enum Status {
    NonExistent : (),
    Active : (),
    ClosedByOwner : (),
    ClosedByLiquidation : (),
    ClosedByRedemption : (),
}

pub struct RewardSnapshot {
    asset: u64,
    usdf_debt: u64,
}