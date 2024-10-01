library;
use std::u128::U128;

impl U128 {
    pub fn is_power_of_two(self) -> bool {
        self.lower() != 0 && (self & (self - U128::from(1u64))) == U128::zero()
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
        if is_power_of_two(other.lower()) {
            let shift = trailing_zeros(other.lower());
            // upper: self.upper % other.upper,
            // TODO fix this later breaks when upper is 0
            return U128::from(self.lower() & ((1 << shift) - 1));
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
    assert(U128::from(100u64) % U128::from(3u64) == U128::from(1u64));
    assert(U128::from(101u64) % U128::from(3u64) == U128::from(2u64));
    assert(U128::from(102u64) % U128::from(3u64) == U128::from(0u64));
    assert(U128::from(103u64) % U128::from(5u64) == U128::from(3u64));
    assert(U128::from(104u64) % U128::from(5u64) == U128::from(4u64));
    assert(U128::from(8u64) % U128::from(10u64) == U128::from(8u64));
}
#[test]
fn test_u128_modulo_pow_2_divisor() {
    assert(U128::from(1u64) % U128::from(1u64) == U128::from(0u64));
    assert(U128::from(2u64) % U128::from(1u64) == U128::from(0u64));
    assert(U128::from(3u64) % U128::from(1u64) == U128::from(0u64));
    assert(U128::from(0u64) % U128::from(2u64) == U128::from(0u64));
    assert(U128::from(1u64) % U128::from(2u64) == U128::from(1u64));
    assert(U128::from(2u64) % U128::from(2u64) == U128::from(0u64));
    assert(U128::from(3u64) % U128::from(2u64) == U128::from(1u64));
    assert(U128::from(100u64) % U128::from(2u64) == U128::from(0u64));
    assert(U128::from(101u64) % U128::from(2u64) == U128::from(1u64));
}
