library;

pub struct RedemptionTotals {
    remaining_usdf: u64,
    total_usdf_to_redeem: u64,
    total_asset_drawn: u64,
    asset_fee: u64,
    asset_to_send_to_redeemer: u64,
    decayed_base_rate: u64,
    price: u64,
    total_usdf_supply_at_start: u64,
}

impl RedemptionTotals {
    pub fn default() -> Self {
        RedemptionTotals {
            remaining_usdf: 0,
            total_usdf_to_redeem: 0,
            total_asset_drawn: 0,
            asset_fee: 0,
            asset_to_send_to_redeemer: 0,
            decayed_base_rate: 0,
            price: 0,
            total_usdf_supply_at_start: 0,
        }
    }
}

pub struct SingleRedemptionValues {
    usdf_lot: u64,
    asset_lot: u64,
    cancelled_partial: bool,
}

impl SingleRedemptionValues {
    pub fn default() -> Self {
        SingleRedemptionValues {
            usdf_lot: 0,
            asset_lot: 0,
            cancelled_partial: false,
        }
    }
}

pub struct AssetInfo {
    assets: Vec<AssetId>,
    asset_contracts: Vec<AssetContracts>,
    prices: Vec<u64>,
    system_debts: Vec<u64>,
    redemption_totals: Vec<RedemptionTotals>,
    current_borrowers: Vec<Identity>,
    current_crs: Vec<u64>,
}

pub struct AssetContracts {
    trove_manager: ContractId,
    oracle: ContractId,
    asset_address: AssetId,
}
