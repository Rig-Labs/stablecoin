library fluid_math;

pub const PCT_100: u64 = 1_000_000_000;

pub const DECIMAL_PRECISION: u64 = 1_000_000_000;

pub const MCR: u64 = 120_000_000_000;

// 10 USDF 
pub const USDF_GAS_COMPENSATION: u64 = 10_000_000_000;

// min debt is 500 USDF
pub const MIN_NET_DEBT: u64 = 500_000_000_000;

pub const PERCENT_DIVERSOR = 200;

pub fn fm_get_net_debt(_debt: u64) -> u64 {
    return _debt + USDF_GAS_COMPENSATION;
}
