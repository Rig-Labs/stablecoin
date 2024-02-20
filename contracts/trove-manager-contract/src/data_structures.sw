library;

use libraries::trove_manager_interface::data_structures::{Status};

pub struct Trove {
    debt: u64,
    coll: u64,
    stake: u64,
    array_index: u64,
    status: Status,
}

impl Trove {
    pub fn default() -> Self {
        Trove {
            debt: 0,
            coll: 0,
            stake: 0,
            array_index: 0,
            status: Status::NonExistent,
        }
    }
}



pub struct LocalVariablesOuterLiquidationFunction {
    price: u64,
    usdf_in_stability_pool: u64,
    liquidated_debt: u64,
    liquidated_coll: u64,
}

impl LocalVariablesOuterLiquidationFunction {
    pub fn default() -> Self {
        LocalVariablesOuterLiquidationFunction {
            price: 0,
            usdf_in_stability_pool: 0,
            liquidated_debt: 0,
            liquidated_coll: 0,
        }
    }
}

pub struct LocalVariablesLiquidationSequence {
    remaining_usdf_in_stability_pool: u64,
    i: u64,
    icr: u64,
    borrower: Identity,
}

impl LocalVariablesLiquidationSequence {
    pub fn default() -> Self {
        LocalVariablesLiquidationSequence {
            remaining_usdf_in_stability_pool: 0,
            i: 0,
            icr: 0,
            borrower: Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000000)),
        }
    }
}

pub struct LiquidationValues {
    entire_trove_debt: u64,
    entire_trove_coll: u64,
    debt_to_offset: u64,
    coll_to_send_to_sp: u64,
    debt_to_redistribute: u64,
    coll_to_redistribute: u64,
    coll_surplus: u64,
    coll_gas_compensation: u64,
    is_partial_liquidation: bool,
    remaining_trove_coll: u64,
    remaining_trove_debt: u64,
}

impl LiquidationValues {
    pub fn default() -> Self {
        LiquidationValues {
            entire_trove_debt: 0,
            entire_trove_coll: 0,
            debt_to_offset: 0,
            coll_to_send_to_sp: 0,
            debt_to_redistribute: 0,
            coll_to_redistribute: 0,
            coll_surplus: 0,
            coll_gas_compensation: 0,
            is_partial_liquidation: false,
            remaining_trove_coll: 0,
            remaining_trove_debt: 0,
        }
    }
}

pub struct LiquidationTotals {
    total_debt_to_offset: u64,
    total_coll_to_send_to_sp: u64,
    total_debt_to_redistribute: u64,
    total_coll_to_redistribute: u64,
    total_coll_gas_compensation: u64,
    total_coll_surplus: u64,
    total_debt_in_sequence: u64,
    total_coll_in_sequence: u64,
}

impl LiquidationTotals {
    pub fn default() -> Self {
        LiquidationTotals {
            total_debt_to_offset: 0,
            total_coll_to_send_to_sp: 0,
            total_debt_to_redistribute: 0,
            total_coll_to_redistribute: 0,
            total_coll_gas_compensation: 0,
            total_coll_surplus: 0,
            total_debt_in_sequence: 0,
            total_coll_in_sequence: 0,
        }
    }
}

pub struct LiquidatedTroveValsInner {
    trove_debt_to_repay: u64,
    trove_coll_liquidated: u64,
    is_partial_liquidation: bool,
}

pub struct EntireTroveDebtAndColl {
    entire_trove_debt: u64,
    entire_trove_coll: u64,
    pending_debt_rewards: u64,
    pending_coll_rewards: u64,
}

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
