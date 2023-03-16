library utils;

dep data_structures;
use data_structures::{LiquidatedTroveValsInner, LiquidationTotals, LiquidationValues};
use libraries::fluid_math::*;
use libraries::numbers::*;

use std::{logging::log, u128::U128};

pub fn calculate_liqudated_trove_values(
    coll: u64,
    debt: u64,
    price: u64,
) -> LiquidatedTroveValsInner {
    let trove_debt_numerator: U128 = U128::from_u64(debt) * U128::from_u64(POST_COLLATERAL_RATIO) - U128::from_u64(coll) * U128::from_u64(price);
    let trove_debt_denominator: U128 = U128::from_u64(POST_COLLATERAL_RATIO) - U128::from_u64(ONE + STABILITY_POOL_FEE);

    let trove_debt_to_repay = (trove_debt_numerator / trove_debt_denominator).as_u64().unwrap();
    let trove_coll_liquidated_numerator: U128 = U128::from_u64(trove_debt_to_repay) * (U128::from_u64(ONE) + U128::from_u64(STABILITY_POOL_FEE));
    let mut trove_coll_liquidated = (trove_coll_liquidated_numerator / U128::from_u64(price)).as_u64().unwrap();

    let remaining_debt = debt - trove_debt_to_repay;

    if remaining_debt < MIN_NET_DEBT {
        trove_coll_liquidated = debt * (ONE + STABILITY_POOL_FEE) / price;
        return LiquidatedTroveValsInner {
            trove_coll_liquidated,
            trove_debt_to_repay: debt,
            is_partial_liquidation: false,
        }
    }

    return LiquidatedTroveValsInner {
        trove_coll_liquidated,
        trove_debt_to_repay,
        is_partial_liquidation: true,
    }
}

pub fn get_offset_and_redistribution_vals(
    coll: u64,
    debt: u64,
    usdf_in_stab_pool: u64,
    price: u64,
) -> LiquidationValues {
    let mut vars: LiquidationValues = LiquidationValues::default();

    vars.entire_trove_coll = coll;
    vars.entire_trove_debt = debt;

    let liquidated_position_vals = calculate_liqudated_trove_values(coll, debt, price);
    if (liquidated_position_vals.is_partial_liquidation) {
        vars.is_partial_liquidation = true;
        vars.remaining_trove_coll = coll - liquidated_position_vals.trove_coll_liquidated;
        vars.remaining_trove_debt = debt - liquidated_position_vals.trove_debt_to_repay;
    } else {
        vars.is_partial_liquidation = false;
        vars.remaining_trove_coll = 0;
        vars.remaining_trove_debt = 0;
        vars.coll_surplus = coll - liquidated_position_vals.trove_coll_liquidated;
    }
    if (usdf_in_stab_pool > 0) {   
        // If the Stability Pool doesnt have enough USDF to offset the entire debt, offset as much as possible
        vars.debt_to_offset = fm_min(liquidated_position_vals.trove_debt_to_repay, usdf_in_stab_pool);
        // Send collateral to the Stability Pool proportional to the amount of debt offset
        let coll_to_send_to_sp_u128: U128 = U128::from_u64(liquidated_position_vals.trove_coll_liquidated) * U128::from_u64(vars.debt_to_offset) / U128::from_u64(liquidated_position_vals.trove_debt_to_repay);
        vars.coll_to_send_to_sp = coll_to_send_to_sp_u128.as_u64().unwrap();
        // If stability pool doesn't have enough USDF to offset the entire debt, redistribute the remaining debt and collateral
        vars.debt_to_redistribute = liquidated_position_vals.trove_debt_to_repay - vars.debt_to_offset;
        vars.coll_to_redistribute = liquidated_position_vals.trove_coll_liquidated - vars.coll_to_send_to_sp;
    } else {
        vars.debt_to_redistribute = liquidated_position_vals.trove_debt_to_repay;
        vars.coll_to_redistribute = liquidated_position_vals.trove_coll_liquidated;
    }

    return vars;
}

pub fn add_liquidation_vals_to_totals(
    old_totals: LiquidationTotals,
    vals: LiquidationValues,
) -> LiquidationTotals {
    let mut new_totals = old_totals;
    new_totals.total_debt_in_sequence += vals.entire_trove_debt;
    new_totals.total_coll_in_sequence += vals.entire_trove_coll;
    new_totals.total_debt_to_offset += vals.debt_to_offset;
    new_totals.total_coll_to_send_to_sp += vals.coll_to_send_to_sp;
    new_totals.total_debt_to_redistribute += vals.debt_to_redistribute;
    new_totals.total_coll_to_redistribute += vals.coll_to_redistribute;
    new_totals.total_coll_gas_compensation += vals.coll_gas_compensation;
    new_totals.total_usdf_gas_compensation += vals.usdf_gas_compensation;
    new_totals.total_coll_surplus += vals.coll_surplus;
    return new_totals;
}

#[test]
fn test_calculate_liqudated_trove_values() {
    // Full liquidation
    let starting_coll = 1_100_000_000;
    let starting_debt = 1_000_000_000;
    let liquidation_vals = calculate_liqudated_trove_values(1_100_000_000, 1_000_000_000, 1_000_000);

    // Value of debt + 5% stability fee
    assert(liquidation_vals.trove_coll_liquidated == (starting_debt * (ONE + STABILITY_POOL_FEE)) / 1_000_000);
    assert(liquidation_vals.trove_debt_to_repay == starting_debt);
    assert(liquidation_vals.is_partial_liquidation == false);

    // Partial liquidation
    let starting_coll = 12_000_000_000;
    let starting_debt = 10_000_000_000;
    let price = 1_000_000;
    let liquidation_vals = calculate_liqudated_trove_values(starting_coll, starting_debt, 1_000_000);

    let ending_coll = starting_coll - liquidation_vals.trove_coll_liquidated;
    let ending_debt = starting_debt - liquidation_vals.trove_debt_to_repay;

    let pcr = fm_compute_cr(ending_coll, ending_debt, price);
    assert(pcr == POST_COLLATERAL_RATIO);
}

#[test]
fn test_get_offset_and_redistribution_vals_full_liquidation_empty_pool() {
    // Full liquidation, Empty Stability Pool
    let starting_coll = 1_100_000_000;
    let starting_debt = 1_000_000_000;
    let liquidation_vals = get_offset_and_redistribution_vals(starting_coll, starting_debt, 0, 1_000_000);

    assert(liquidation_vals.entire_trove_coll == starting_coll);
    assert(liquidation_vals.entire_trove_debt == starting_debt);
    assert(liquidation_vals.is_partial_liquidation == false);
    assert(liquidation_vals.remaining_trove_coll == 0);
    assert(liquidation_vals.remaining_trove_debt == 0);
    assert(liquidation_vals.coll_surplus == starting_coll - (starting_debt * (ONE + STABILITY_POOL_FEE)) / 1_000_000);
    assert(liquidation_vals.debt_to_offset == 0);
    assert(liquidation_vals.coll_to_send_to_sp == 0);
    assert(liquidation_vals.debt_to_redistribute == starting_debt);
    assert(liquidation_vals.coll_to_redistribute == 1_050_000_000);
}

#[test]
fn test_get_offset_and_redistribution_vals_full_liquidation_enough_pool() {
    // Full liquidation, Stability Pool has enough USDF to offset the entire debt
    let starting_coll = 1_100_000_000;
    let starting_debt = 1_000_000_000;
    let liquidation_vals = get_offset_and_redistribution_vals(starting_coll, starting_debt, 2_000_000_000, 1_000_000);

    assert(liquidation_vals.entire_trove_coll == starting_coll);
    assert(liquidation_vals.entire_trove_debt == starting_debt);
    assert(liquidation_vals.is_partial_liquidation == false);
    assert(liquidation_vals.remaining_trove_coll == 0);
    assert(liquidation_vals.remaining_trove_debt == 0);
    assert(liquidation_vals.coll_surplus == starting_coll - (starting_debt * (ONE + STABILITY_POOL_FEE)) / 1_000_000);
    assert(liquidation_vals.debt_to_offset == 1_000_000_000);
    assert(liquidation_vals.coll_to_send_to_sp == 1_050_000_000);
    assert(liquidation_vals.debt_to_redistribute == 0);
    assert(liquidation_vals.coll_to_redistribute == 0);
}

#[test]
fn test_get_offset_and_redistribution_vals_full_liquidation_partial_pool() {
    // Full liquidation, Stability Pool doesn't have enough USDF to offset the entire debt
    let starting_coll = 1_100_000_000;
    let starting_debt = 1_000_000_000;
    let liquidation_vals = get_offset_and_redistribution_vals(starting_coll, starting_debt, 500_000_000, 1_000_000);

    assert(liquidation_vals.entire_trove_coll == starting_coll);
    assert(liquidation_vals.entire_trove_debt == starting_debt);
    assert(liquidation_vals.is_partial_liquidation == false);
    assert(liquidation_vals.remaining_trove_coll == 0);
    assert(liquidation_vals.remaining_trove_debt == 0);
    assert(liquidation_vals.coll_surplus == starting_coll - (starting_debt * (ONE + STABILITY_POOL_FEE)) / 1_000_000);
    assert(liquidation_vals.debt_to_offset == 500_000_000);
    assert(liquidation_vals.coll_to_send_to_sp == 525_000_000);
    assert(liquidation_vals.debt_to_redistribute == 500_000_000);
    assert(liquidation_vals.coll_to_redistribute == 525_000_000);
}

#[test]
fn test_get_offset_and_redistribution_vals_partial_liquidation_empty_pool() {
     // Partial liquidation, Empty Stability Pool
    let starting_coll = 12_000_000_000;
    let starting_debt = 10_000_000_000;
    let liquidation_vals = get_offset_and_redistribution_vals(starting_coll, starting_debt, 0, 1_000_000);
    let icr = fm_compute_cr(liquidation_vals.remaining_trove_coll, liquidation_vals.remaining_trove_debt, 1_000_000);

    assert(liquidation_vals.entire_trove_coll == starting_coll);
    assert(liquidation_vals.entire_trove_debt == starting_debt);
    assert(liquidation_vals.is_partial_liquidation == true);
    assert(icr == POST_COLLATERAL_RATIO);
    assert(liquidation_vals.coll_surplus == 0);
    assert(liquidation_vals.debt_to_offset == 0);
    assert(liquidation_vals.coll_to_send_to_sp == 0);
    assert(liquidation_vals.debt_to_redistribute == starting_debt - liquidation_vals.remaining_trove_debt);
    assert(liquidation_vals.coll_to_redistribute == starting_coll - liquidation_vals.remaining_trove_coll);
}

#[test]
fn test_get_offset_and_redistribution_vals_partial_liquidation_enough_pool() {
    // Partial liquidation, Stability Pool has enough USDF to offset the entire debt
    let starting_coll = 12_000_000_000;
    let starting_debt = 10_000_000_000;
    let liquidation_vals = get_offset_and_redistribution_vals(starting_coll, starting_debt, 20_000_000_000, 1_000_000);
    let icr = fm_compute_cr(liquidation_vals.remaining_trove_coll, liquidation_vals.remaining_trove_debt, 1_000_000);

    assert(liquidation_vals.entire_trove_coll == starting_coll);
    assert(liquidation_vals.entire_trove_debt == starting_debt);
    assert(liquidation_vals.is_partial_liquidation == true);
    assert(icr == POST_COLLATERAL_RATIO);
    assert(liquidation_vals.coll_surplus == 0);
    assert(liquidation_vals.debt_to_offset == starting_debt - liquidation_vals.remaining_trove_debt);
    assert(liquidation_vals.coll_to_send_to_sp == starting_coll - liquidation_vals.remaining_trove_coll);
    assert(liquidation_vals.debt_to_redistribute == 0);
    assert(liquidation_vals.coll_to_redistribute == 0);
}

#[test]
fn test_get_offset_and_redistribution_vals_partial_liquidation_partial_pool() {
    // Partial liquidation, Stability Pool doesn't have enough USDF to offset the entire debt
    let starting_coll = 12_000_000_000;
    let starting_debt = 10_000_000_000;
    let total_usdf = 1_000_000_000;

    let liquidation_vals = get_offset_and_redistribution_vals(starting_coll, starting_debt, total_usdf, 1_000_000);
    let icr = fm_compute_cr(liquidation_vals.remaining_trove_coll, liquidation_vals.remaining_trove_debt, 1_000_000);
    let coll_removed = starting_coll - liquidation_vals.remaining_trove_coll;

    assert(liquidation_vals.entire_trove_coll == starting_coll);
    assert(liquidation_vals.entire_trove_debt == starting_debt);
    assert(liquidation_vals.is_partial_liquidation == true);
    assert(icr == POST_COLLATERAL_RATIO);
    assert(liquidation_vals.coll_surplus == 0);
    assert(liquidation_vals.debt_to_offset == total_usdf);
    assert(liquidation_vals.coll_to_send_to_sp == (total_usdf * coll_removed) / (starting_debt - liquidation_vals.remaining_trove_debt));
    assert(liquidation_vals.debt_to_redistribute == starting_debt - liquidation_vals.remaining_trove_debt - total_usdf);
    assert(liquidation_vals.coll_to_redistribute == starting_coll - liquidation_vals.remaining_trove_coll - liquidation_vals.coll_to_send_to_sp);
}
