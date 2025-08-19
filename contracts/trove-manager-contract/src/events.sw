library;

pub struct TroveFullLiquidationEvent {
    pub borrower: Identity,
    pub debt: u64,
    pub collateral: u64,
}

pub struct TrovePartialLiquidationEvent {
    pub borrower: Identity,
    pub remaining_debt: u64,
    pub remaining_collateral: u64,
}

pub struct RedemptionEvent {
    pub borrower: Identity,
    pub usdm_amount: u64,
    pub collateral_amount: u64,
    pub collateral_price: u64,
}
