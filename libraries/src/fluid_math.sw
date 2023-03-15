library fluid_math;

dep numbers;
use numbers::*;
use std::{logging::log, u128::U128};

const ZERO_B256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
// Using Precision 6 until u128 is available
pub const PCT_100: u64 = 1_000_000_000;

pub const DECIMAL_PRECISION: u64 = 1_000_000_000;

pub const MCR: u64 = 1_200_000;

pub const MAX_U64: u64 = 18_446_744_073_709_551_615;
// 10 USDF 
pub const USDF_GAS_COMPENSATION: u64 = 10_000_000;

// min debt is 500 USDF
pub const MIN_NET_DEBT: u64 = 500_000_000;

pub const PERCENT_DIVERSOR = 200;

pub const POST_COLLATERAL_RATIO: u64 = 1_300_000;

pub const STABILITY_POOL_FEE: u64 = 50_000;

pub const ONE: u64 = 1_000_000;

pub fn fm_compute_nominal_cr(coll: u64, debt: u64) -> u64 {
    if (debt > 0) {
        let ncr: U128 = U128::from_u64(coll) * U128::from_u64(DECIMAL_PRECISION) / U128::from_u64(debt);
        return ncr.as_u64().unwrap();
    } else {
        return MAX_U64;
    }
}

pub fn fm_compute_cr(coll: u64, debt: u64, price: u64) -> u64 {
    if (debt > 0) {
        let cr: U128 = U128::from_u64(coll) * U128::from_u64(price) / U128::from_u64(debt);
        return cr.as_u64().unwrap();
    } else {
        return MAX_U64;
    }
}

pub fn fm_min(a: u64, b: u64) -> u64 {
    if a < b { return a; } else { return b; }
}

pub fn fm_max(a: u64, b: u64) -> u64 {
    if a > b { return a; } else { return b; }
}

fn dec_mul(a: u64, b: u64) -> U128 {
    let prod = U128::from_u64(a) * U128::from_u64(b);
    let dec_prod = (prod + U128::from_u64(DECIMAL_PRECISION / 2)) / U128::from_u64(DECIMAL_PRECISION);
    return dec_prod;
}

fn dec_pow(base: u64, _minutes: u64) -> U128 {
    let mut minutes = _minutes;
    if minutes > 525600000 {
        minutes = 525600000;
    }

    let mut y = DECIMAL_PRECISION;
    let mut x = base;
    let mut n = minutes;

    while n > 1 {
        if n % 2 == 0 {
            x = dec_mul(x, x).as_u64().unwrap();
            n = n / 2;
        } else {
            y = dec_mul(x, y).as_u64().unwrap();
            x = dec_mul(x, x).as_u64().unwrap();
            n = (n - 1) / 2;
        }
    }

    return dec_mul(x, y);
}

pub fn null_address() -> Identity {
    return Identity::Address(Address::from(ZERO_B256))
}