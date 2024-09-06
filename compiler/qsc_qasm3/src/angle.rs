// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
pub(crate) mod tests;

use num_bigint::BigInt;

use crate::oqasm_helpers::safe_u64_to_f64;
use std::convert::TryInto;
use std::f64::consts::PI;
use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// A fixed-point angle type with a specified number of bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Angle {
    value: u64,
    size: u32,
}

#[allow(dead_code)]
impl Angle {
    fn new(value: u64, size: u32) -> Self {
        Angle { value, size }
    }

    fn from_f64(val: f64, size: u32) -> Self {
        #[allow(clippy::cast_precision_loss)]
        let factor = (2.0 * PI) / (1u64 << size) as f64;
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let value = (val / factor).round() as u64;
        Angle { value, size }
    }

    fn to_bitstring(self) -> String {
        format!("{:0width$b}", self.value, width = self.size as usize)
    }

    fn cast(&self, new_size: u32, truncate: bool) -> Self {
        match new_size.cmp(&self.size) {
            std::cmp::Ordering::Less => {
                let value = if truncate {
                    let shift_amount = self.size - new_size;
                    self.value >> shift_amount
                } else {
                    // Rounding
                    let shift_amount = self.size - new_size;
                    let half = 1u64 << (shift_amount - 1);
                    let mask = (1u64 << shift_amount) - 1;
                    let lower_bits = self.value & mask;
                    let upper_bits = self.value >> shift_amount;

                    if lower_bits > half || (lower_bits == half && (upper_bits & 1) == 1) {
                        upper_bits + 1
                    } else {
                        upper_bits
                    }
                };
                Angle {
                    value,
                    size: new_size,
                }
            }
            std::cmp::Ordering::Equal => {
                // Same size, no change
                *self
            }
            std::cmp::Ordering::Greater => {
                // Padding with zeros
                let new_value = self.value << (new_size - self.size);
                Angle {
                    value: new_value,
                    size: new_size,
                }
            }
        }
    }
}

impl Add for Angle {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        assert_eq!(self.size, other.size, "Sizes must be the same");
        Angle {
            value: (self.value + other.value) % (1u64 << self.size),
            size: self.size,
        }
    }
}

impl Sub for Angle {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        assert_eq!(self.size, other.size, "Sizes must be the same");
        Angle {
            value: (self.value + (1u64 << self.size) - other.value) % (1u64 << self.size),
            size: self.size,
        }
    }
}

impl Mul<u64> for Angle {
    type Output = Self;

    fn mul(self, factor: u64) -> Self {
        Angle {
            value: (self.value * factor) % (1u64 << self.size),
            size: self.size,
        }
    }
}

impl Mul<u128> for Angle {
    type Output = Self;

    fn mul(self, factor: u128) -> Self {
        let r = BigInt::from(self.value) * BigInt::from(factor);
        let r = r % BigInt::from(1u128 << self.size);
        Angle {
            value: r.try_into().expect("Value is too large"),
            size: self.size,
        }
    }
}

impl Div<u64> for Angle {
    type Output = Self;

    fn div(self, divisor: u64) -> Self {
        Angle {
            value: self.value / divisor,
            size: self.size,
        }
    }
}

impl Div for Angle {
    type Output = u64;

    fn div(self, other: Self) -> u64 {
        assert_eq!(self.size, other.size, "Sizes must be the same");
        self.value / other.value
    }
}

impl Neg for Angle {
    type Output = Self;

    fn neg(self) -> Self {
        Angle {
            value: (1u64 << self.size) - self.value,
            size: self.size,
        }
    }
}

impl AddAssign for Angle {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl SubAssign for Angle {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl MulAssign<u64> for Angle {
    fn mul_assign(&mut self, factor: u64) {
        *self = *self * factor;
    }
}

impl DivAssign<u64> for Angle {
    fn div_assign(&mut self, divisor: u64) {
        *self = *self / divisor;
    }
}

impl TryInto<f64> for Angle {
    type Error = &'static str;

    fn try_into(self) -> Result<f64, Self::Error> {
        if self.size > 64 {
            return Err("Size exceeds 64 bits");
        }
        let Some(denom) = safe_u64_to_f64(1u64 << self.size) else {
            return Err("Denominator is too large");
        };
        let Some(value) = safe_u64_to_f64(self.value) else {
            return Err("Value is too large");
        };
        let factor = (2.0 * PI) / denom;
        Ok(value * factor)
    }
}

impl From<Angle> for bool {
    fn from(val: Angle) -> Self {
        val.value != 0
    }
}

impl fmt::Display for Angle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", (*self).try_into().unwrap_or(f64::NAN))
    }
}
