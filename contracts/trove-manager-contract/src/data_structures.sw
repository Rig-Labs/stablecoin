library data_structures;

use libraries::data_structures::{Status};

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

pub struct RewardSnapshot {
    asset: u64,
    usdf_debt: u64,
}

impl RewardSnapshot {
    pub fn default() -> Self {
        RewardSnapshot {
            asset: 0,
            usdf_debt: 0,
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
    usdf_gas_compensation: u64,
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
            usdf_gas_compensation: 0,
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
    total_usdf_gas_compensation: u64,
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
            total_usdf_gas_compensation: 0,
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
