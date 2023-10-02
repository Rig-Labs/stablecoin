library;

pub struct VestingSchedule {
    cliff_timestamp: u64,
    end_timestamp: u64,
    cliff_amount: u64,
    total_amount: u64,
    claimed_amount: u64,
    recipient: Identity,
}
