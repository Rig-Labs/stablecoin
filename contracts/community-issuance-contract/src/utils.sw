library utils;

use libraries::fluid_math::{fm_min, dec_pow, DECIMAL_PRECISION, fm_abs_diff};
use libraries::numbers::*;
use std::{u128::U128, u256::U256};

// todo: if sway supports constant multiplications inherit decimal precision from fluid math
// 32_000_000 * 1_000_000_000


pub const FPT_SUPPLY_CAP = 32_000_000_000_000_000;
pub const SECONDS_IN_ONE_MINUTE = 60;
pub const ISSUANCE_FACTOR = 999_998_681;


pub fn internal_get_fpt_supply_cap(time_transition_started: u64, total_transition_time_seconds:u64, current_time: u64, has_transitioned_rewards: bool) -> u64 {
    if (!has_transitioned_rewards){
        return FPT_SUPPLY_CAP / 2;
    } else {
        let time_since_transition_started_seconds = U128::from_u64(current_time - time_transition_started) * U128::from_u64(10_000);
        let change_in_fpt_supply_cap = time_since_transition_started_seconds / U128::from_u64(total_transition_time_seconds);
        let supply_cap = U128::from_u64(FPT_SUPPLY_CAP / 2) 
            + (
                (
                    U128::from_u64(
                        fm_min(10_000, change_in_fpt_supply_cap.as_u64().unwrap())
                    ) 
                    / 
                    U128::from_u64(10_000)
                    * 
                    U128::from_u64(FPT_SUPPLY_CAP / 2)
                    
                )
            );
        return supply_cap.as_u64().unwrap();
    }
}

pub fn internal_get_cumulative_issuance_fraction(current_time: u64, deployment_time: u64 ) -> u64 {
    let time_passed_in_minutes = (current_time - deployment_time) / SECONDS_IN_ONE_MINUTE;

    let power = dec_pow(ISSUANCE_FACTOR, time_passed_in_minutes);

    let cumulative_issuance_fraction = U128::from_u64(DECIMAL_PRECISION) - power;

    return cumulative_issuance_fraction.as_u64().unwrap()
}

// this will overflow without u256

pub fn test_issue_fpt(current_time: u64, deployment_time: u64, time_transition_started: u64, total_transition_time_seconds:u64, total_fpt_issued: u64, has_transitioned_rewards:bool) -> u64 {
    let latest_total_fpt_issued = (
            U128::from_u64(internal_get_fpt_supply_cap(time_transition_started, total_transition_time_seconds, current_time, has_transitioned_rewards)) 
            * U128::from_u64(internal_get_cumulative_issuance_fraction(current_time, deployment_time))
         ) / U128::from_u64(DECIMAL_PRECISION);
    let issuance = latest_total_fpt_issued.as_u64().unwrap() - total_fpt_issued;
    return issuance
}

#[test]
fn test_issuance_factor(){

    let current_time = 121;
    let deployment_time = 1;
    let cumulative_issuance = internal_get_cumulative_issuance_fraction(current_time, deployment_time);
    assert(cumulative_issuance >= 0);
    assert(cumulative_issuance <= DECIMAL_PRECISION);

    let current_time = 18_400_000_000_000_000_000;
    let deployment_time = 1;
    let cumulative_issuance = internal_get_cumulative_issuance_fraction(current_time, deployment_time);
    assert(cumulative_issuance >= 0);
    assert(cumulative_issuance <= DECIMAL_PRECISION);

    let current_time = 60;
    let deployment_time = 0;
    let cumulative_issuance = internal_get_cumulative_issuance_fraction(current_time, deployment_time);
    let diff = fm_abs_diff(cumulative_issuance, 1318);
    assert(diff <= 10);

    let current_time = 60 * 60;
    let deployment_time = 0;
    let cumulative_issuance = internal_get_cumulative_issuance_fraction(current_time, deployment_time);
    let diff = fm_abs_diff(cumulative_issuance, 79123);
    assert(diff <= 100);

    let current_time = 60 * 60 * 24;
    let deployment_time = 0;
    let cumulative_issuance = internal_get_cumulative_issuance_fraction(current_time, deployment_time);
    let diff = fm_abs_diff(cumulative_issuance, 1897231);
    assert(diff <= 1_000);

    let current_time = 60 * 60 * 24 * 30;
    let deployment_time = 0;
    let cumulative_issuance = internal_get_cumulative_issuance_fraction(current_time, deployment_time);
    let diff = fm_abs_diff(cumulative_issuance, 55378538);
    assert(diff <= 100_000);

    let current_time = 60 * 60 * 24 * 30 * 12;
    let deployment_time = 0;
    let cumulative_issuance = internal_get_cumulative_issuance_fraction(current_time, deployment_time);
    let diff = fm_abs_diff(cumulative_issuance, 500_000_000);
    assert(diff <= 10_000_000);

    let current_time = 60 * 60 * 24 * 30 * 12 * 2;
    let deployment_time = 0;
    let cumulative_issuance = internal_get_cumulative_issuance_fraction(current_time, deployment_time);
    let diff = fm_abs_diff(cumulative_issuance, 750_000_000);
    assert(diff <= 10_000_000);

    let current_time = 60 * 60 * 24 * 30 * 12 * 4;
    let deployment_time = 0;
    let cumulative_issuance = internal_get_cumulative_issuance_fraction(current_time, deployment_time);
    let diff = fm_abs_diff(cumulative_issuance, 937_500_000);
    assert(diff <= 10_000_000);

    let current_time = 60 * 60 * 24 * 30 * 12 * 10;
    let deployment_time = 0;
    let cumulative_issuance = internal_get_cumulative_issuance_fraction(current_time, deployment_time);
    let diff = fm_abs_diff(cumulative_issuance, 999_000_000);
    assert(diff <= 10_000_000);
}

#[test]
fn test_supply_cap_transition_overflow(){
    let current_time = 2;
    let time_transition_started = 1;
    let total_transition_time_seconds = 1;
    let has_transitioned_rewards = false;

    let supply_cap_before_transition = internal_get_fpt_supply_cap(time_transition_started, total_transition_time_seconds, current_time, has_transitioned_rewards);
    
}

#[test]
fn test_supply_cap_transition(){

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

    let current_time = 60 * 60 * 24 * 30 * 12 * 100;
    let time_transition_started = current_time - 1;
    let total_transition_time_seconds = 1;
    let has_transitioned_rewards = true;

    let supply_cap_after_transition = internal_get_fpt_supply_cap(time_transition_started, total_transition_time_seconds, current_time, has_transitioned_rewards);
    assert(supply_cap_after_transition == FPT_SUPPLY_CAP);

}

#[test]
fn test_emissions_schedule(){
    //c+p
    // let current_time = 2;
    // let deployment_time = 1;
    // let time_transition_started = 1;
    // let total_transition_time_seconds = 1;
    // let total_fpt_issued = 1;
    // let has_transitioned_rewards = false;
    // let issuance = test_issue_fpt(current_time, deployment_time, time_transition_started, total_transition_time_seconds, total_fpt_issued, has_transitioned_rewards);
    
    // double check math
    let current_time = 2000;
    let deployment_time = 1;
    let time_transition_started = 1;
    let total_transition_time_seconds = 1;
    let total_fpt_issued = 0;
    let has_transitioned_rewards = false;
    let issuance = test_issue_fpt(current_time, deployment_time, time_transition_started, total_transition_time_seconds, total_fpt_issued, has_transitioned_rewards);
    let issuance_pre_transition = (U128::from_u64((FPT_SUPPLY_CAP / 2)) * U128::from_u64(internal_get_cumulative_issuance_fraction(current_time, deployment_time))) / U128::from_u64(DECIMAL_PRECISION);
    assert(issuance == issuance_pre_transition.as_u64().unwrap());

    // test emissions before transition

    // check that half of half of the fpt supply is issued before transitioned at 1 year mark
    let current_time = 60 * 60 * 24 * 30 * 12;
    let deployment_time = 0;
    let time_transition_started = 1;
    let total_transition_time_seconds = 1;
    let total_fpt_issued = 0;
    let has_transitioned_rewards = false;
    let issuance = test_issue_fpt(current_time, deployment_time, time_transition_started, total_transition_time_seconds, total_fpt_issued, has_transitioned_rewards);
    let diff = fm_abs_diff(issuance, FPT_SUPPLY_CAP / 4);
    assert(diff <= 100_000_000_000_000);

    // check that 75% of half of fpt supply is issued before transition after 2 years
    let current_time = 60 * 60 * 24 * 30 * 12 * 2;
    let deployment_time = 0;
    let time_transition_started = 1;
    let total_transition_time_seconds = 1;
    let total_fpt_issued = 0;
    let has_transitioned_rewards = false;
    let issuance = test_issue_fpt(current_time, deployment_time, time_transition_started, total_transition_time_seconds, total_fpt_issued, has_transitioned_rewards);
    let diff = fm_abs_diff(issuance, (FPT_SUPPLY_CAP / 2) * 75 / 100);
    assert(diff <= 100_000_000_000_000);

    // check that half of fpt supply is issued before transition after 100 years
    let current_time = 60 * 60 * 24 * 30 * 12 * 100;
    let deployment_time = 0;
    let time_transition_started = 1;
    let total_transition_time_seconds = 1;
    let total_fpt_issued = 0;
    let has_transitioned_rewards = false;
    let issuance = test_issue_fpt(current_time, deployment_time, time_transition_started, total_transition_time_seconds, total_fpt_issued, has_transitioned_rewards);
    let diff = fm_abs_diff(issuance, FPT_SUPPLY_CAP / 2);
    assert(diff <= 1_000_000_000_000);

    // test emissions after transitioned

    // // check that half of the fpt supply amount is issued after transition at 1 year mark
    // this one gets up to 1% off because of 9 decimals, but it should be ok
    let current_time = 60 * 60 * 24 * 30 * 12;
    let deployment_time = 0;
    let time_transition_started = current_time - 1;
    let total_transition_time_seconds = 1;
    let total_fpt_issued = 0;
    let has_transitioned_rewards = true;
    let issuance = test_issue_fpt(current_time, deployment_time, time_transition_started, total_transition_time_seconds, total_fpt_issued, has_transitioned_rewards);
    let diff = fm_abs_diff(issuance, FPT_SUPPLY_CAP/2);
    assert(diff <= 1_000_000_000_000_000);

    // check that full fpt supply amount is issued after transition after 100 years
    let current_time = 60 * 60 * 24 * 30 * 12 * 100;
    let deployment_time = 0;
    let time_transition_started = current_time - 1;
    let total_transition_time_seconds = 1;
    let total_fpt_issued = 0;
    let has_transitioned_rewards = true;
    let issuance = test_issue_fpt(current_time, deployment_time, time_transition_started, total_transition_time_seconds, total_fpt_issued, has_transitioned_rewards);
    let diff = fm_abs_diff(issuance, FPT_SUPPLY_CAP);
    assert(diff <= 1_000_000_000_000);


    // test emissions during transition

    // let current_time = 60 * 60 * 24 * 30 * 12;
    // let deployment_time = 0;
    // let time_transition_started = 0;
    // let total_transition_time_seconds = 60 * 60 * 24 * 30 * 12 * 2;
    // let total_fpt_issued = 0;
    // let has_transitioned_rewards = true;
    // let issuance = test_issue_fpt(current_time, deployment_time, time_transition_started, total_transition_time_seconds, total_fpt_issued, has_transitioned_rewards);
    // let diff = fm_abs_diff(issuance, FPT_SUPPLY_CAP);
    // assert(diff <= 1_000_000_000_000);

    // test comparing time periods during transition

    // test total fpt issued subtraction
}