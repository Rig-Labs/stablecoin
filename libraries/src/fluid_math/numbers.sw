library;
use std::{u128::U128, u256::U256};

impl U256 {
    pub fn from_u64(value: u64) -> U256 {
        U256::from((0, 0, 0, value))
    }
}

impl U256 {
    pub fn ge(self, other: Self) -> bool {
        self > other || self == other
    }

    pub fn le(self, other: Self) -> bool {
        self < other || self == other
    }
}

impl U128 {
    pub fn from_u64(value: u64) -> U128 {
        U128::from((0, value))
    }
}

impl U128 {
    pub fn ge(self, other: Self) -> bool {
        self > other || self == other
    }

    pub fn le(self, other: Self) -> bool {
        self < other || self == other
    }
}

impl U128 {
    pub fn is_power_of_two(self) -> bool {
        self.lower != 0
            && (self
                & (self - U128 {
                        upper: 0,
                        lower: 1,
                    })) == U128 {
                upper: 0,
                lower: 0,
            }
    }
}

impl U256 {
    pub fn is_power_of_two(self) -> bool {
        self.d != 0 && (self & (self - U256::from((0, 0, 0, 1)))) == U256::from((0, 0, 0, 0))
    }
}

fn is_power_of_two(value: u64) -> bool {
    value != 0 && (value & (value - 1)) == 0
}

fn trailing_zeros(value: u64) -> u64 {
    if value == 0 {
        64
    } else {
        let mut count = 0;
        let mut value = value;
        while value & 1 == 0 {
            count += 1;
            value >>= 1;
        }
        count
    }
}

impl U128 {
    pub fn modulo(self, other: Self) -> Self {
        // Handle the special case where the divisor is a power of 2.
        // This can be optimized using bit-shifting.
        if is_power_of_two(other.lower) {
            let shift = trailing_zeros(other.lower);
            return U128 {
                // upper: self.upper % other.upper,
                // TODO fix this later breaks when upper is 0
                upper: 0,
                lower: self.lower & ((1 << shift) - 1),
            };
        }

        // Compute the remainder using the division algorithm.
        // Note that this implementation assumes that the
        // divisor is non-zero.
        let mut dividend = self;
        let divisor = other;
        while dividend >= divisor {
            dividend -= divisor;
        }
        if dividend < divisor {
            dividend
        } else {
            dividend - divisor
        }
    }
}

impl U256 {
    pub fn modulo(self, other: Self) -> Self {
        // Handle the special case where the divisor is a power of 2.
        // This can be optimized using bit-shifting.
        if is_power_of_two(other.d) {
            let shift = trailing_zeros(other.d);
            return U256 {
                // a: self.a % other.a,
                // b: self.b % other.b,
                // c: self.c % other.c,
                a: 0,
                b: 0,
                c: 0,
                d: self.d & ((1 << shift) - 1),
            };
        }

        // Compute the remainder using the division algorithm.
        // Note that this implementation assumes that the
        // divisor is non-zero.
        let mut dividend = self;
        let divisor = other;
        while dividend >= divisor {
            dividend -= divisor;
        }
        if dividend < divisor {
            dividend
        } else {
            dividend - divisor
        }
    }
}

#[test]
fn test_is_power_of_two() {
    assert(is_power_of_two(0) == false);
    assert(is_power_of_two(1) == true);
    assert(is_power_of_two(2) == true);
    assert(is_power_of_two(3) == false);
    assert(is_power_of_two(4) == true);
    assert(is_power_of_two(5) == false);
    assert(is_power_of_two(100) == false);
    assert(is_power_of_two(1024) == true);
}

#[test]
fn test_trailing_zeros() {
    assert(trailing_zeros(0) == 64);
    assert(trailing_zeros(1) == 0);
    assert(trailing_zeros(2) == 1);
    assert(trailing_zeros(3) == 0);
    assert(trailing_zeros(4) == 2);
    assert(trailing_zeros(5) == 0);
    assert(trailing_zeros(100) == 2);
    assert(trailing_zeros(1024) == 10);
}

#[test]
fn test_u128_modulo() {
    assert(U128::from_u64(100) % U128::from_u64(3) == U128::from_u64(1));
    assert(U128::from_u64(101) % U128::from_u64(3) == U128::from_u64(2));
    assert(U128::from_u64(102) % U128::from_u64(3) == U128::from_u64(0));
    assert(U128::from_u64(103) % U128::from_u64(5) == U128::from_u64(3));
    assert(U128::from_u64(104) % U128::from_u64(5) == U128::from_u64(4));
    assert(U128::from_u64(8) % U128::from_u64(10) == U128::from_u64(8));
}

#[test]
fn test_u256_modulo() {
    assert(
        U256::from((0, 0, 0, 100)) % U256::from((0, 0, 0, 3)) == U256::from((0, 0, 0, 1)),
    );
    assert(
        U256::from((0, 0, 0, 101)) % U256::from((0, 0, 0, 3)) == U256::from((0, 0, 0, 2)),
    );
    assert(
        U256::from((0, 0, 0, 102)) % U256::from((0, 0, 0, 3)) == U256::from((0, 0, 0, 0)),
    );
    assert(
        U256::from((0, 0, 0, 103)) % U256::from((0, 0, 0, 5)) == U256::from((0, 0, 0, 3)),
    );
    assert(
        U256::from((0, 0, 0, 104)) % U256::from((0, 0, 0, 5)) == U256::from((0, 0, 0, 4)),
    );
    assert(
        U256::from((0, 0, 0, 8)) % U256::from((0, 0, 0, 10)) == U256::from((0, 0, 0, 8)),
    );
}

#[test]
fn test_u128_modulo_pow_2_divisor() {
    assert(U128::from_u64(1) % U128::from_u64(1) == U128::from_u64(0));
    assert(U128::from_u64(2) % U128::from_u64(1) == U128::from_u64(0));
    assert(U128::from_u64(3) % U128::from_u64(1) == U128::from_u64(0));
    assert(U128::from_u64(0) % U128::from_u64(2) == U128::from_u64(0));
    assert(U128::from_u64(1) % U128::from_u64(2) == U128::from_u64(1));
    assert(U128::from_u64(2) % U128::from_u64(2) == U128::from_u64(0));
    assert(U128::from_u64(3) % U128::from_u64(2) == U128::from_u64(1));
    assert(U128::from_u64(100) % U128::from_u64(2) == U128::from_u64(0));
    assert(U128::from_u64(101) % U128::from_u64(2) == U128::from_u64(1));
}

#[test]
fn test_u256_modulo_pow_2_divisor() {
    assert(U256::from((0, 0, 0, 1)) % U256::from((0, 0, 0, 1)) == U256::from((0, 0, 0, 0)));
    assert(U256::from((0, 0, 0, 2)) % U256::from((0, 0, 0, 1)) == U256::from((0, 0, 0, 0)));
    assert(U256::from((0, 0, 0, 3)) % U256::from((0, 0, 0, 1)) == U256::from((0, 0, 0, 0)));
    assert(U256::from((0, 0, 0, 0)) % U256::from((0, 0, 0, 2)) == U256::from((0, 0, 0, 0)));
    assert(U256::from((0, 0, 0, 1)) % U256::from((0, 0, 0, 2)) == U256::from((0, 0, 0, 1)));
    assert(U256::from((0, 0, 0, 2)) % U256::from((0, 0, 0, 2)) == U256::from((0, 0, 0, 0)));
    assert(U256::from((0, 0, 0, 3)) % U256::from((0, 0, 0, 2)) == U256::from((0, 0, 0, 1)));
    assert(
        U256::from((0, 0, 0, 100)) % U256::from((0, 0, 0, 2)) == U256::from((0, 0, 0, 0)),
    );
    assert(
        U256::from((0, 0, 0, 101)) % U256::from((0, 0, 0, 2)) == U256::from((0, 0, 0, 1)),
    );
}
