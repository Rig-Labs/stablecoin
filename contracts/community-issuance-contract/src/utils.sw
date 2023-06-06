library utils;

use libraries::fluid_math::{fm_min, dec_pow, DECIMAL_PRECISION};
use libraries::numbers::*;

pub const FPT_SUPPLY_CAP = 32_000_000_000_000_000;
pub const SECONDS_IN_ONE_MINUTE = 60;
pub const ISSUANCE_FACTOR = 999998681227695000;

pub fn internal_get_fpt_supply_cap(time_transition_started: u64, total_transition_time_seconds:u64, current_time: u64, has_transitioned_rewards: bool) -> u64 {
    if (!has_transitioned_rewards){
        return FPT_SUPPLY_CAP / 2;
    } else {
        let time_since_transition_started_seconds = current_time - time_transition_started;
        let change_in_fpt_supply_cap = time_since_transition_started_seconds / total_transition_time_seconds;
        return (FPT_SUPPLY_CAP / 2) + (fm_min(1, change_in_fpt_supply_cap) * (FPT_SUPPLY_CAP / 2));
    }
}

pub fn internal_get_cumulative_issuance_fraction(current_time: u64, deployment_time: u64 ) -> u64 {
    let time_passed_in_minutes = current_time - deployment_time / SECONDS_IN_ONE_MINUTE;

    // TODO: as_u64() is not implemented for U256
    let power = dec_pow(ISSUANCE_FACTOR, time_passed_in_minutes).d;

    let cumulative_issuance_fraction = DECIMAL_PRECISION - power;

    require(cumulative_issuance_fraction <= DECIMAL_PRECISION, "Cumulative issuance fraction must be in range [0,1]");

    return cumulative_issuance_fraction
}

pub fn issue_fpt(current_time: u64, deployment_time: u64, time_transition_started: u64, total_transition_time_seconds:u64, total_fpt_issued: u64, has_transitioned_rewards:bool) -> u64 {
    let latest_total_fpt_issued = (internal_get_fpt_supply_cap(time_transition_started, total_transition_time_seconds, current_time, has_transitioned_rewards) 
         * internal_get_cumulative_issuance_fraction(current_time, deployment_time)) / DECIMAL_PRECISION;
    let issuance = latest_total_fpt_issued - total_fpt_issued;
    return issuance
}

#[test]
fn test_emissions_schedule(){
    let current_time = 0;
    let deployment_time = 0;
    let time_transition_started = 0;
    let total_transition_time_seconds = 0;
    let total_fpt_issued = 0;
    let has_transitioned_rewards = false;
    let issuance = issue_fpt(current_time, deployment_time, time_transition_started, total_transition_time_seconds, total_fpt_issued, has_transitioned_rewards);
    assert(issuance == 0);
}