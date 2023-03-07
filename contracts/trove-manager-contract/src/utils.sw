library utils;

dep data_structures;
use data_structures::Trove;
use libraries::fluid_math::*;

pub fn calculate_liqudated_trove_values(coll: u64, debt: u64, price: u64) -> (u64, u64, bool) {
    let debt_repaid = (debt * POST_COLLATERAL_RATIO - coll * price) / (POST_COLLATERAL_RATIO - (ONE + STABILITY_POOL_FEE));
    let mut collateral_liquidated = (debt_repaid * (ONE + STABILITY_POOL_FEE)) / price;

    let remaining_debt = debt - debt_repaid;

    if remaining_debt < MIN_NET_DEBT {
        collateral_liquidated = debt * (ONE + STABILITY_POOL_FEE) / price;
        return (collateral_liquidated, debt, false)
    }

    return (collateral_liquidated, debt_repaid, true)
}
