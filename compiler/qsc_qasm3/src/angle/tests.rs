// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::unwrap_used)]
#![allow(clippy::unreadable_literal)]

use std::convert::TryInto;
use std::f64::consts::PI;

use super::Angle;

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
        (TryInto::<f64>::try_into(angle).unwrap() - TryInto::<f64>::try_into(new_angle).unwrap())
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
