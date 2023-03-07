library utils;

dep data_structures;
use data_structures::{LiquidatedTroveValsInner};
use libraries::fluid_math::*;

pub fn calculate_liqudated_trove_values(
    coll: u64,
    debt: u64,
    price: u64,
) -> LiquidatedTroveValsInner {
    let debt_repaid = (debt * POST_COLLATERAL_RATIO - coll * price) / (POST_COLLATERAL_RATIO - (ONE + STABILITY_POOL_FEE));
    let mut coll_to_send_to_sp = (debt_repaid * (ONE + STABILITY_POOL_FEE)) / price;

    let remaining_debt = debt - debt_repaid;

    if remaining_debt < MIN_NET_DEBT {
        coll_to_send_to_sp = debt * (ONE + STABILITY_POOL_FEE) / price;
        return LiquidatedTroveValsInner {
            coll_to_send_to_sp,
            debt_repaid: debt,
            is_partial_liquidation: false,
        }
    }

    return LiquidatedTroveValsInner {
        coll_to_send_to_sp,
        debt_repaid,
        is_partial_liquidation: true,
    }
}

#[test]
fn test_calculate_liqudated_trove_values() {
    // Full liquidation
    let liquidation_vals = calculate_liqudated_trove_values(1_100_000_000, 1_000_000_000, 1_000_000);

    // value of debt + 5% stability fee
    assert(liquidation_vals.coll_to_send_to_sp == 1_050_000_000);
    assert(liquidation_vals.debt_repaid == 1_000_000_000);
    assert(liquidation_vals.is_partial_liquidation == false);

    // Partial liquidation
    let starting_coll = 12_000_000_000;
    let starting_debt = 10_000_000_000;
    let price = 1_000_000;
    let liquidation_vals = calculate_liqudated_trove_values(starting_coll, starting_debt, 1_000_000);

    let ending_coll = starting_coll - liquidation_vals.coll_to_send_to_sp;
    let ending_debt = starting_debt - liquidation_vals.debt_repaid;

    let pcr = fm_compute_cr(ending_coll, ending_debt, price);
    assert(pcr == POST_COLLATERAL_RATIO);
}
