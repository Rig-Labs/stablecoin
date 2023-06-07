library utils;

use libraries::fluid_math::{fm_min, dec_pow, DECIMAL_PRECISION};
use libraries::numbers::*;
use std::{u128::U128, u256::U256};

pub const FPT_SUPPLY_CAP = 32_000_000_000_000_000;
pub const SECONDS_IN_ONE_MINUTE = 60;
pub const ISSUANCE_FACTOR = 999_998_681;

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
    let time_passed_in_minutes = (current_time - deployment_time) / SECONDS_IN_ONE_MINUTE;

    let power = dec_pow(ISSUANCE_FACTOR, time_passed_in_minutes);

    let cumulative_issuance_fraction = U256::from_u64(DECIMAL_PRECISION) - power;

    return cumulative_issuance_fraction.as_u64().unwrap()
}

pub fn test_issue_fpt(current_time: u64, deployment_time: u64, time_transition_started: u64, total_transition_time_seconds:u64, total_fpt_issued: u64, has_transitioned_rewards:bool) -> u64 {
    let latest_total_fpt_issued = (internal_get_fpt_supply_cap(time_transition_started, total_transition_time_seconds, current_time, has_transitioned_rewards) 
         * internal_get_cumulative_issuance_fraction(current_time, deployment_time)) / DECIMAL_PRECISION;
    let issuance = latest_total_fpt_issued - total_fpt_issued;
    return issuance
}

#[test]
fn test_emissions_schedule(){
    let current_time = 2;
    let time_transition_started = 1;
    let total_transition_time_seconds = 1;
    let has_transitioned_rewards = false;

    let supply_cap_before_transition = internal_get_fpt_supply_cap(time_transition_started, total_transition_time_seconds, current_time, has_transitioned_rewards);
    assert(supply_cap_before_transition == FPT_SUPPLY_CAP / 2);
    
    let current_time = 2000;
    let time_transition_started = 1000;
    let total_transition_time_seconds = 10000;
    let has_transitioned_rewards = true;

    let supply_cap_during_transition = internal_get_fpt_supply_cap(time_transition_started, total_transition_time_seconds, current_time, has_transitioned_rewards);
    assert(supply_cap_during_transition == (FPT_SUPPLY_CAP / 2) + (FPT_SUPPLY_CAP / 2) * (1/10));

    let current_time = 2000;
    let time_transition_started = 1000;
    let total_transition_time_seconds = 100;
    let has_transitioned_rewards = true;

    let supply_cap_after_transition = internal_get_fpt_supply_cap(time_transition_started, total_transition_time_seconds, current_time, has_transitioned_rewards);
    assert(supply_cap_after_transition == FPT_SUPPLY_CAP);

    let current_time = 121;
    let deployment_time = 1;
    let cumulative_issuance = internal_get_cumulative_issuance_fraction(current_time, 2);
    assert(cumulative_issuance >= 0);
    assert(cumulative_issuance <= DECIMAL_PRECISION);

    let current_time = 2;
    let deployment_time = 1;
    let time_transition_started = 1;
    let total_transition_time_seconds = 1;
    let total_fpt_issued = 1;
    let has_transitioned_rewards = false;
    let issuance = test_issue_fpt(current_time, deployment_time, time_transition_started, total_transition_time_seconds, total_fpt_issued, has_transitioned_rewards);

    //comparing two timeperiods
    //999998681
}