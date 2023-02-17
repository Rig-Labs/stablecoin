library data_structures;

pub struct VestingSchedule {
    start: u64,
    end: u64,
    cliff_amount: u64,
    total_amount: u64,
    revocable: bool,
    recipient: Identity
}
