const DECIMAL_PRECISION: u64 = 1_000_000_000;

// 0.5% min borrow fee
pub fn with_min_borrow_fee(debt: u64) -> u64 {
    let net_debt = debt * 1_005 / 1_000;
    return net_debt;
}

pub fn calculate_icr(coll: u64, debt: u64) -> u64 {
    let icr = coll * DECIMAL_PRECISION / debt;
    return icr;
}

pub fn with_liquidation_penalty(amount: u64) -> u64 {
    let amount_with_penalty = amount * 1_05 / 1_00;
    return amount_with_penalty;
}
