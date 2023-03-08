library utils;

dep data_structures;
use data_structures::{LiquidatedTroveValsInner, LiquidationValues};
use libraries::fluid_math::*;

pub fn calculate_liqudated_trove_values(
    coll: u64,
    debt: u64,
    price: u64,
) -> LiquidatedTroveValsInner {
    let trove_debt_to_repay = (debt * POST_COLLATERAL_RATIO - coll * price) / (POST_COLLATERAL_RATIO - (ONE + STABILITY_POOL_FEE));
    let mut trove_coll_liquidated = (trove_debt_to_repay * (ONE + STABILITY_POOL_FEE)) / price;

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

fn get_offset_and_redistribution_vals(
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
        if (liquidated_position_vals.trove_debt_to_repay > usdf_in_stab_pool)
        {
            vars.debt_to_offset = usdf_in_stab_pool;
        } else {
            // If the Stability Pool has enough USDF to offset the entire debt, offset the entire debt
            vars.debt_to_offset = liquidated_position_vals.trove_debt_to_repay;
        }
        // Send collateral to the Stability Pool proportional to the amount of debt offset
        vars.coll_to_send_to_sp = liquidated_position_vals.trove_coll_liquidated * vars.debt_to_offset / liquidated_position_vals.trove_debt_to_repay;
        // If stability pool doesn't have enough USDF to offset the entire debt, redistribute the remaining debt and collateral
        vars.debt_to_redistribute = liquidated_position_vals.trove_debt_to_repay - vars.debt_to_offset;
        vars.coll_to_redistribute = liquidated_position_vals.trove_coll_liquidated - vars.coll_to_send_to_sp;
    } else {
        vars.debt_to_redistribute = liquidated_position_vals.trove_debt_to_repay;
        vars.coll_to_redistribute = liquidated_position_vals.trove_coll_liquidated;
    }

    return vars;
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
fn test_get_offset_and_redistribution_vals() {
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
