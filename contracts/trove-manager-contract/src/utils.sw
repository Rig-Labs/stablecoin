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
    // If bad debt
    if fm_multiply_ratio(coll, price, DECIMAL_PRECISION) < debt
    {
        return LiquidatedTroveValsInner {
            trove_coll_liquidated: coll,
            trove_debt_to_repay: debt,
            is_partial_liquidation: false,
        }
    }
    let trove_debt_numerator: U128 = U128::from_u64(debt) * U128::from_u64(POST_COLLATERAL_RATIO) - U128::from_u64(coll) * U128::from_u64(price);

    let trove_debt_denominator: U128 = U128::from_u64(POST_COLLATERAL_RATIO) - U128::from_u64(ONE + STABILITY_POOL_FEE);

    let trove_debt_to_repay = (trove_debt_numerator / trove_debt_denominator).as_u64().unwrap();

    let mut trove_coll_liquidated = fm_multiply_ratio(trove_debt_to_repay, ONE + STABILITY_POOL_FEE, price);

    if debt - trove_debt_to_repay < MIN_NET_DEBT {
        trove_coll_liquidated = fm_multiply_ratio(debt, ONE + STABILITY_POOL_FEE, price);

        return LiquidatedTroveValsInner {
            trove_coll_liquidated: fm_min(trove_coll_liquidated, coll),
            trove_debt_to_repay: debt,
            is_partial_liquidation: false,
        }
    }

    return LiquidatedTroveValsInner {
        trove_coll_liquidated: fm_min(trove_coll_liquidated, coll),
        trove_debt_to_repay: fm_min(trove_debt_to_repay, debt),
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
        vars.coll_surplus = coll - liquidated_position_vals.trove_coll_liquidated;
    }
    if (usdf_in_stab_pool > 0) {   
        // If the Stability Pool doesnt have enough USDF to offset the entire debt, offset as much as possible
        vars.debt_to_offset = fm_min(liquidated_position_vals.trove_debt_to_repay, usdf_in_stab_pool);
        // Send collateral to the Stability Pool proportional to the amount of debt offset
        vars.coll_to_send_to_sp = fm_multiply_ratio(liquidated_position_vals.trove_coll_liquidated, vars.debt_to_offset, liquidated_position_vals.trove_debt_to_repay);
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
    let starting_coll = 550 * DECIMAL_PRECISION;
    let starting_debt = 500 * DECIMAL_PRECISION;
    let price = DECIMAL_PRECISION;
    let liquidation_vals = calculate_liqudated_trove_values(starting_coll, starting_debt, price);

    // Value of debt + 5% stability fee
    let coll_liquidated = U128::from_u64(starting_debt) * U128::from_u64(ONE + STABILITY_POOL_FEE) / U128::from_u64(price);
    assert(liquidation_vals.trove_coll_liquidated == coll_liquidated.as_u64().unwrap());
    assert(liquidation_vals.trove_debt_to_repay == starting_debt);
    assert(liquidation_vals.is_partial_liquidation == false);

    // Partial liquidation
    // Test passes but runs into sway issue of 'TransactionScriptLength'
    // let starting_coll = 12_000 * DECIMAL_PRECISION;
    // let starting_debt = 10_000 * DECIMAL_PRECISION;
    // let liquidation_vals = calculate_liqudated_trove_values(starting_coll, starting_debt, price);
    // let ending_coll = starting_coll - liquidation_vals.trove_coll_liquidated;
    // let ending_debt = starting_debt - liquidation_vals.trove_debt_to_repay;
    // let pcr = fm_compute_cr(ending_coll, ending_debt, price);
    // assert_within_percent_tolerance(pcr, POST_COLLATERAL_RATIO, DECIMAL_PRECISION / 100);
}

#[test]
fn test_calculate_liqudated_trove_values_bad_debt() {
    // Full liquidation bad debt
    let starting_coll = 900 * DECIMAL_PRECISION;
    let starting_debt = 1_000 * DECIMAL_PRECISION;
    let liquidation_vals = calculate_liqudated_trove_values(starting_coll, starting_debt, 1_000_000_000);

    assert(liquidation_vals.trove_coll_liquidated == starting_coll);
    assert(liquidation_vals.trove_debt_to_repay == starting_debt);
    assert(liquidation_vals.is_partial_liquidation == false);
}

#[test]
fn test_get_offset_and_redistribution_vals_full_liquidation_empty_pool() {
    // Full liquidation, Empty Stability Pool
    let starting_coll = 1_100 * DECIMAL_PRECISION;
    let starting_debt = 1_000 * DECIMAL_PRECISION;
    let price = DECIMAL_PRECISION;
    let liquidation_vals = get_offset_and_redistribution_vals(starting_coll, starting_debt, 0, price);
    let coll_liquidated = U128::from_u64(starting_debt) * U128::from_u64(ONE + STABILITY_POOL_FEE) / U128::from_u64(price);
    assert(liquidation_vals.entire_trove_coll == starting_coll);
    assert(liquidation_vals.entire_trove_debt == starting_debt);
    assert(liquidation_vals.is_partial_liquidation == false);
    assert(liquidation_vals.remaining_trove_coll == 0);
    assert(liquidation_vals.remaining_trove_debt == 0);
    assert(liquidation_vals.coll_surplus == starting_coll - coll_liquidated.as_u64().unwrap());
    assert(liquidation_vals.debt_to_offset == 0);
    assert(liquidation_vals.coll_to_send_to_sp == 0);
    assert(liquidation_vals.debt_to_redistribute == starting_debt);
    assert(liquidation_vals.coll_to_redistribute == 1_050 * DECIMAL_PRECISION);
}

#[test]
fn test_get_offset_and_redistribution_vals_full_liquidation_enough_pool() {
    // Full liquidation, Stability Pool has enough USDF to offset the entire debt
    let starting_coll = 1_100 * DECIMAL_PRECISION;
    let starting_debt = 1_000 * DECIMAL_PRECISION;
    let amount_in_pool = 2_000 * DECIMAL_PRECISION;
    let price = DECIMAL_PRECISION;
    let liquidation_vals = get_offset_and_redistribution_vals(starting_coll, starting_debt, amount_in_pool, price);
    let coll_liquidated = U128::from_u64(starting_debt) * U128::from_u64(ONE + STABILITY_POOL_FEE) / U128::from_u64(price);
    assert(liquidation_vals.entire_trove_coll == starting_coll);
    assert(liquidation_vals.entire_trove_debt == starting_debt);
    assert(liquidation_vals.is_partial_liquidation == false);
    assert(liquidation_vals.remaining_trove_coll == 0);
    assert(liquidation_vals.remaining_trove_debt == 0);
    assert(liquidation_vals.coll_surplus == starting_coll - coll_liquidated.as_u64().unwrap());
    assert(liquidation_vals.debt_to_offset == 1_000 * DECIMAL_PRECISION);
    assert(liquidation_vals.coll_to_send_to_sp == 1_050 * DECIMAL_PRECISION);
    assert(liquidation_vals.debt_to_redistribute == 0);
    assert(liquidation_vals.coll_to_redistribute == 0);
}

#[test]
fn test_get_offset_and_redistribution_vals_full_liquidation_partial_pool() {
    // Full liquidation, Stability Pool doesn't have enough USDF to offset the entire debt
    let starting_coll = 1_100 * DECIMAL_PRECISION;
    let starting_debt = 1_000 * DECIMAL_PRECISION;
    let amount_in_pool = 500 * DECIMAL_PRECISION;
    let price = DECIMAL_PRECISION;
    let liquidation_vals = get_offset_and_redistribution_vals(starting_coll, starting_debt, amount_in_pool, price);
    let coll_liquidated = U128::from_u64(starting_debt) * U128::from_u64(ONE + STABILITY_POOL_FEE) / U128::from_u64(price);

    assert(liquidation_vals.entire_trove_coll == starting_coll);
    assert(liquidation_vals.entire_trove_debt == starting_debt);
    assert(liquidation_vals.is_partial_liquidation == false);
    assert(liquidation_vals.remaining_trove_coll == 0);
    assert(liquidation_vals.remaining_trove_debt == 0);
    assert(liquidation_vals.coll_surplus == starting_coll - coll_liquidated.as_u64().unwrap());
    assert(liquidation_vals.debt_to_offset == 500 * DECIMAL_PRECISION);
    assert(liquidation_vals.coll_to_send_to_sp == 525 * DECIMAL_PRECISION);
    assert(liquidation_vals.debt_to_redistribute == 500 * DECIMAL_PRECISION);
    assert(liquidation_vals.coll_to_redistribute == 525 * DECIMAL_PRECISION);
}

#[test]
fn test_get_offset_and_redistribution_vals_partial_liquidation_empty_pool() {
     // Partial liquidation, Empty Stability Pool
    let starting_coll = 12_000 * DECIMAL_PRECISION;
    let starting_debt = 10_000 * DECIMAL_PRECISION;
    let price = DECIMAL_PRECISION;
    let liquidation_vals = get_offset_and_redistribution_vals(starting_coll, starting_debt, 0, price);

    let icr = fm_compute_cr(liquidation_vals.remaining_trove_coll, liquidation_vals.remaining_trove_debt, price);
    assert(liquidation_vals.entire_trove_coll == starting_coll);
    assert(liquidation_vals.entire_trove_debt == starting_debt);
    assert(liquidation_vals.is_partial_liquidation == true);
    assert_within_percent_tolerance(icr, POST_COLLATERAL_RATIO, DECIMAL_PRECISION / 100);
    assert(liquidation_vals.coll_surplus == 0);
    assert(liquidation_vals.debt_to_offset == 0);
    assert(liquidation_vals.coll_to_send_to_sp == 0);
    assert(liquidation_vals.debt_to_redistribute == starting_debt - liquidation_vals.remaining_trove_debt);
    assert(liquidation_vals.coll_to_redistribute == starting_coll - liquidation_vals.remaining_trove_coll);
}

#[test]
fn test_get_offset_and_redistribution_vals_partial_liquidation_enough_pool() {
    // Partial liquidation, Stability Pool has enough USDF to offset the entire debt
    let starting_coll = 12_000 * DECIMAL_PRECISION;
    let starting_debt = 10_000 * DECIMAL_PRECISION;
    let amount_in_pool = 20_000 * DECIMAL_PRECISION;
    let price = DECIMAL_PRECISION;
    let liquidation_vals = get_offset_and_redistribution_vals(starting_coll, starting_debt, amount_in_pool, price);
    let icr = fm_compute_cr(liquidation_vals.remaining_trove_coll, liquidation_vals.remaining_trove_debt, price);

    assert(liquidation_vals.entire_trove_coll == starting_coll);
    assert(liquidation_vals.entire_trove_debt == starting_debt);
    assert(liquidation_vals.is_partial_liquidation == true);
    assert(liquidation_vals.coll_surplus == 0);
    assert(liquidation_vals.debt_to_offset == starting_debt - liquidation_vals.remaining_trove_debt);
    assert(liquidation_vals.coll_to_send_to_sp == starting_coll - liquidation_vals.remaining_trove_coll);
    assert(liquidation_vals.debt_to_redistribute == 0);
    assert(liquidation_vals.coll_to_redistribute == 0);
    assert_within_percent_tolerance(icr, POST_COLLATERAL_RATIO, DECIMAL_PRECISION / 100);
}

#[test]
fn test_get_offset_and_redistribution_vals_partial_liquidation_partial_pool() {
    // Partial liquidation, Stability Pool doesn't have enough USDF to offset the entire debt
    let starting_coll = 12_000 * DECIMAL_PRECISION;
    let starting_debt = 10_000 * DECIMAL_PRECISION;
    let total_usdf = 1_000 * DECIMAL_PRECISION;
    let price = DECIMAL_PRECISION;

    let liquidation_vals = get_offset_and_redistribution_vals(starting_coll, starting_debt, total_usdf, price);
    let icr = fm_compute_cr(liquidation_vals.remaining_trove_coll, liquidation_vals.remaining_trove_debt, price);
    let coll_removed = starting_coll - liquidation_vals.remaining_trove_coll;

    let expected_coll_to_send_to_sp = U128::from_u64(total_usdf) * U128::from_u64(coll_removed) / U128::from_u64(starting_debt - liquidation_vals.remaining_trove_debt);

    assert(liquidation_vals.entire_trove_coll == starting_coll);
    assert(liquidation_vals.entire_trove_debt == starting_debt);
    assert(liquidation_vals.is_partial_liquidation == true);
    assert(liquidation_vals.coll_surplus == 0);
    assert(liquidation_vals.debt_to_offset == total_usdf);
    assert(liquidation_vals.coll_to_send_to_sp == expected_coll_to_send_to_sp.as_u64().unwrap());
    assert(liquidation_vals.debt_to_redistribute == starting_debt - liquidation_vals.remaining_trove_debt - total_usdf);
    assert(liquidation_vals.coll_to_redistribute == starting_coll - liquidation_vals.remaining_trove_coll - liquidation_vals.coll_to_send_to_sp);
    assert_within_percent_tolerance(icr, POST_COLLATERAL_RATIO, DECIMAL_PRECISION / 100);
}
