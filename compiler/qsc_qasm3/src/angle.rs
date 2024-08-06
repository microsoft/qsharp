// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::unreadable_literal)]
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_angle() {
        let angle1 = Angle::from_f64(PI, 4);
        let angle2 = Angle::from_f64(PI / 2.0, 6);
        let angle3 = Angle::from_f64(7.0 * (PI / 8.0), 8);

        assert_eq!(angle1.to_bitstring(), "1000");
        assert_eq!(angle2.to_bitstring(), "010000");
        assert_eq!(angle3.to_bitstring(), "01110000");
    }

    #[test]
    fn test_angle_creation() {
        let angle = Angle::from_f64(PI, 4);
        assert_eq!(angle.value, 8);
        assert_eq!(angle.size, 4);
    }

    #[test]
    fn test_angle_addition() {
        let angle1 = Angle::from_f64(PI / 2.0, 4);
        let angle2 = Angle::from_f64(PI / 2.0, 4);
        let result = angle1 + angle2;
        assert_eq!(result.value, 8);
        let angle: f64 = result.try_into().unwrap();
        assert!((angle - PI).abs() < f64::EPSILON);
    }

    #[test]
    fn test_angle_multiplication() {
        let angle = Angle::from_f64(PI / 4.0, 4);
        let result: Angle = angle * 2u64;
        assert_eq!(result.value, 4);
        let angle: f64 = result.try_into().unwrap();
        assert!((angle - PI / 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_angle_multiplication_bigint() {
        let angle = Angle::from_f64(PI / 4.0, 4);
        let result: Angle = angle * 18446744073709551616u128;
        assert_eq!(result.value, 0);
        let angle: f64 = result.try_into().unwrap();
        assert!((angle - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_angle_multiplication_bigint2() {
        let angle = Angle::from_f64(PI / 4.0, 4);
        let result: Angle = angle * 9223372036854775806u128;
        assert_eq!(result.value, 12);
        let angle: f64 = result.try_into().unwrap();
        assert!((angle - ((3. * PI) / 2.)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_angle_division_int() {
        let angle = Angle::from_f64(PI / 2.0, 4);
        let result = angle / 2;
        assert_eq!(result.value, 2);
        let angle: f64 = result.try_into().unwrap();
        assert!((angle - PI / 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_angle_division_by_angle() {
        let angle1 = Angle::from_f64(PI, 4);
        let angle2 = Angle::from_f64(PI / 4.0, 4);
        let result = angle1 / angle2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_angle_unary_negation() {
        let angle = Angle::from_f64(PI / 4.0, 4);
        let result = -angle; // "0010"
        assert_eq!(result.value, 14); // 7*(pi/4) │ "1110"
    }

    #[test]
    fn test_angle_compound_addition() {
        let mut angle1 = Angle::from_f64(PI / 2.0, 4);
        let angle2 = Angle::from_f64(PI / 2.0, 4);
        angle1 += angle2;
        assert_eq!(angle1.value, 8);
        let angle: f64 = angle1.try_into().unwrap();
        assert!((angle - PI).abs() < f64::EPSILON);
    }

    #[test]
    fn test_angle_compound_subtraction() {
        let mut angle1 = Angle::from_f64(PI, 4);
        let angle2 = Angle::from_f64(PI / 2.0, 4);
        angle1 -= angle2;
        assert_eq!(angle1.value, 4);
        let angle: f64 = angle1.try_into().unwrap();
        assert!((angle - PI / 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_angle_compound_multiplication() {
        let mut angle = Angle::from_f64(PI / 4.0, 4);
        angle *= 2;
        assert_eq!(angle.value, 4);
        let angle: f64 = angle.try_into().unwrap();
        assert!((angle - PI / 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_angle_compound_division() {
        let mut angle = Angle::from_f64(PI / 2.0, 4);
        angle /= 2;
        assert_eq!(angle.value, 2);
        let angle: f64 = angle.try_into().unwrap();
        assert!((angle - PI / 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_angle_bitstring() {
        let angle = Angle::from_f64(PI, 4);
        assert_eq!(angle.to_bitstring(), "1000");
    }

    #[test]
    fn test_angle_try_into_f64() {
        let angle: Angle = Angle::from_f64(PI, 4);
        let angle_f64: f64 = angle.try_into().unwrap();
        assert!((angle_f64 - PI).abs() < f64::EPSILON);
    }

    #[test]
    fn test_angle_display() {
        let angle = Angle::from_f64(PI, 4);
        assert_eq!(format!("{angle}"), format!("{PI}"));
    }

    #[test]
    fn from_f64_round_to_the_nearest_ties_to_even() {
        let angle = Angle::from_f64(2.0 * PI * (127. / 512.), 8);
        // 00111111 is equally close, but even rounds to 01000000
        assert_eq!(angle.to_bitstring(), "01000000");
    }

    #[test]
    fn test_angle_into_bool() {
        let angle = Angle {
            value: 10,
            size: 12,
        };
        let result: bool = angle.into();
        assert!(result);

        let angle_zero = Angle { value: 0, size: 12 };
        let result_zero: bool = angle_zero.into();
        assert!(!result_zero);
    }

    #[test]
    fn test_angle_cast_round_padding() {
        let angle = Angle {
            value: 0b1010,
            size: 4,
        };
        let new_angle = angle.cast(8, false);
        assert_eq!(new_angle.value, 0b10100000);
        assert_eq!(new_angle.size, 8);
        assert!(
            (TryInto::<f64>::try_into(angle).unwrap()
                - TryInto::<f64>::try_into(new_angle).unwrap())
            .abs()
                < f64::EPSILON
        );
    }

    #[test]
    fn test_angle_cast_rounding() {
        let angle = Angle {
            value: 0b101011,
            size: 6,
        };
        let new_angle = angle.cast(4, false);
        assert_eq!(new_angle.value, 0b1011);
        assert_eq!(new_angle.size, 4);
    }

    #[test]
    fn test_angle_cast_rounding_ties_to_even() {
        let angle = Angle {
            value: 0b101010,
            size: 6,
        };
        let new_angle = angle.cast(4, false);
        assert_eq!(new_angle.value, 0b1010);
        assert_eq!(new_angle.size, 4);
    }
    #[test]
    fn test_angle_cast_padding() {
        let angle = Angle {
            value: 0b1010,
            size: 4,
        };
        let new_angle = angle.cast(8, true);
        assert_eq!(new_angle.value, 0b10100000);
        assert_eq!(new_angle.size, 8);
    }

    #[test]
    fn test_angle_cast_truncation() {
        let angle = Angle {
            value: 0b101011,
            size: 6,
        };
        let new_angle = angle.cast(4, true);
        assert_eq!(new_angle.value, 0b1010);
        assert_eq!(new_angle.size, 4);
    }
}
