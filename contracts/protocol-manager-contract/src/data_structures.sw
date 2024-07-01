library;

pub struct RedemptionTotals {
    pub remaining_usdf: u64,
    pub total_usdf_to_redeem: u64,
    pub total_asset_drawn: u64,
    pub asset_fee: u64,
    pub asset_to_send_to_redeemer: u64,
    pub decayed_base_rate: u64,
    pub price: u64,
    pub total_usdf_supply_at_start: u64,
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
    pub usdf_lot: u64,
    pub asset_lot: u64,
    pub cancelled_partial: bool,
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

// TODO: compiler says there is no impl when commented but it also says there is one when uncommented
impl AbiDecode for SingleRedemptionValues {
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        // buffer.read::<b256>()
        SingleRedemptionValues::default()
    }
}

pub struct AssetInfo {
    pub assets: Vec<AssetId>,
    pub asset_contracts: Vec<AssetContracts>,
    pub prices: Vec<u64>,
    pub system_debts: Vec<u64>,
    pub redemption_totals: Vec<RedemptionTotals>,
    pub current_borrowers: Vec<Identity>,
    pub current_crs: Vec<u64>,
}

pub struct AssetContracts {
    pub trove_manager: ContractId,
    pub oracle: ContractId,
    pub asset_address: AssetId,
}
