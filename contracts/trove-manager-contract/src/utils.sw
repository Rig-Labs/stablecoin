library utils;

dep data_structures;
use data_structures::Trove;
use libraries::fluid_math::*;

pub fn calculate_liqudated_trove_values(trove: Trove, price: u64) -> (u64, u64, bool) {
    let debt_repaid = (trove.debt*POST_COLLATERAL_RATIO - trove.coll*price)/(POST_COLLATERAL_RATIO - (ONE + STABILITY_POOL_FEE));
    let mut collateral_liquidated = (debt_repaid * (ONE + STABILITY_POOL_FEE))/price;

    let remaining_debt = trove.debt - debt_repaid;

    if remaining_debt < MIN_NET_DEBT {
        collateral_liquidated = trove.debt * (ONE + STABILITY_POOL_FEE)/price;
        return ( collateral_liquidated, trove.debt, true)
    }

    return (collateral_liquidated, debt_repaid, true)
}