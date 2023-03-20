library numbers;
use std::u128::U128;

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
        self.lower != 0 && (self & (self - U128 {
            upper: 0,
            lower: 1,
        })) == U128 {
            upper: 0,
            lower: 0,
        }
    }
}

fn is_power_of_two(value: u64) -> bool {
    value != 0 && (value & (value - 1)) == 0
}

fn trailing_zeros(value: u64) -> u32 {
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
                upper: self.upper % other.upper,
                lower: self.lower
                & ((1 << shift) - 1),
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
        dividend
    }
}
