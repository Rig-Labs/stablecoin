library utils;

dep data_structures;

use data_structures::VestingSchedule;

pub fn calculate_redeemable_amount(current_time: u64, vesting_schedule: VestingSchedule) -> u64 {
    if current_time < vesting_schedule.cliff_timestamp {
        return 0;
    }

    if current_time >= vesting_schedule.end_timestamp {
        return vesting_schedule.total_amount;
    }

    let mut amount_redeemable = vesting_schedule.cliff_amount;

    let total_minus_cliff = vesting_schedule.total_amount - vesting_schedule.cliff_amount;
    let total_vesting_duration = vesting_schedule.end_timestamp - vesting_schedule.cliff_timestamp;
    let time_elapsed = current_time - vesting_schedule.cliff_timestamp;

    let fraction_amount_claimable = total_minus_cliff * (time_elapsed) / total_vesting_duration;

    amount_redeemable += fraction_amount_claimable;

    return amount_redeemable;
}
