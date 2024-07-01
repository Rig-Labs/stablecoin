library;

pub struct VestingSchedule {
    pub cliff_timestamp: u64,
    pub end_timestamp: u64,
    pub cliff_amount: u64,
    pub total_amount: u64,
    pub claimed_amount: u64,
    pub recipient: Identity,
}
