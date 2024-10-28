library;

pub struct OpenTroveEvent {
    pub timestamp: u64,
    pub user: Identity,
    pub asset_id: AssetId,
    pub collateral: u64,
    pub debt: u64,
}

pub struct AdjustTroveEvent {
    pub timestamp: u64,
    pub user: Identity,
    pub asset_id: AssetId,
    pub collateral_change: u64,
    pub debt_change: u64,
    pub is_collateral_increase: bool,
    pub is_debt_increase: bool,
    pub total_collateral: u64,
    pub total_debt: u64,
}

pub struct CloseTroveEvent {
    pub timestamp: u64,
    pub user: Identity,
    pub asset_id: AssetId,
    pub collateral: u64,
    pub debt: u64,
}
