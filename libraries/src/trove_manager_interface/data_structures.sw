library;

pub struct SingleRedemptionValues {
    pub usdm_lot: u64,
    pub asset_lot: u64,
    pub cancelled_partial: bool,
}

impl SingleRedemptionValues {
    pub fn default() -> Self {
        SingleRedemptionValues {
            usdm_lot: 0,
            asset_lot: 0,
            cancelled_partial: false,
        }
    }
}

pub enum Status {
    NonExistent: (),
    Active: (),
    ClosedByOwner: (),
    ClosedByLiquidation: (),
    ClosedByRedemption: (),
}

impl Status {
    pub fn eq(self, other: Status) -> bool {
        match (self, other) {
            (Status::NonExistent, Status::NonExistent) => true,
            (Status::Active, Status::Active) => true,
            (Status::ClosedByOwner, Status::ClosedByOwner) => true,
            (Status::ClosedByLiquidation, Status::ClosedByLiquidation) => true,
            (Status::ClosedByRedemption, Status::ClosedByRedemption) => true,
            _ => false,
        }
    }

    pub fn neq(self, other: Status) -> bool {
        match (self, other) {
            (Status::NonExistent, Status::NonExistent) => false,
            (Status::Active, Status::Active) => false,
            (Status::ClosedByOwner, Status::ClosedByOwner) => false,
            (Status::ClosedByLiquidation, Status::ClosedByLiquidation) => false,
            (Status::ClosedByRedemption, Status::ClosedByRedemption) => false,
            _ => true,
        }
    }
}
pub struct RewardSnapshot {
    pub asset: u64,
    pub usdm_debt: u64,
}

impl RewardSnapshot {
    pub fn default() -> Self {
        RewardSnapshot {
            asset: 0,
            usdm_debt: 0,
        }
    }
}
