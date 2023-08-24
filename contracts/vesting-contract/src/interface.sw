library interface;

dep data_structures;

use data_structures::{VestingSchedule};

abi VestingContract {
    #[storage(write, read)]
    fn constructor(asset: ContractId, schedules: Vec<VestingSchedule>, debugging: bool);

    #[storage(read, write)]
    fn claim_vested_tokens();

    #[storage(read)]
    fn get_vesting_schedule(address: Identity) -> Option<VestingSchedule>;

    #[storage(read)]
    fn get_redeemable_amount(timestamp: u64, address: Identity) -> u64;

    #[storage(read)]
    fn get_current_time() -> u64;

    #[storage(write, read)]
    fn set_current_time(time: u64);
}
