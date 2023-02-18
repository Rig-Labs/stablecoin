library interface;

dep data_structures;

use data_structures::{Asset, VestingSchedule};

abi VestingContract {
    #[storage(write, read)]
    fn constructor(admin: Identity, schedules: Vec<VestingSchedule>, asset: Asset);

    #[storage(read)]
    fn get_vesting_schedule(address: Identity) -> Option<VestingSchedule>;

    #[storage(read)]
    fn get_redeemable_amount(now: u64, address: Identity) -> u64;

    #[storage(read, write)]
    fn claim_vested_tokens(address: Identity);
}
