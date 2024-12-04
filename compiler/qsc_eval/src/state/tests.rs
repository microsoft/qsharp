// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use super::{
    get_matrix_latex, get_state_latex, write_latex_for_algebraic_number,
    write_latex_for_cartesian_form, write_latex_for_complex_number, write_latex_for_decimal_number,
    write_latex_for_polar_form, write_latex_for_real_number, write_latex_for_term, AlgebraicNumber,
    CartesianForm, ComplexNumber, DecimalNumber, PolarForm, RationalNumber, RealNumber, Term,
};
use crate::state::{is_fractional_part_significant, is_significant};
use expect_test::{expect, Expect};
use num_complex::Complex64;
use std::{
    f64::consts::{FRAC_1_SQRT_2, PI},
    time::Instant,
};

#[test]
fn check_is_significant() {
    assert!(!is_significant(0.0));
    assert!(!is_significant(1e-10));
    assert!(!is_significant(-1e-10));
    assert!(is_significant(1.1e-9));
    assert!(is_significant(-1.1e-9));
    assert!(is_significant(1.0));
    assert!(is_significant(-1.0));
}

#[test]
fn check_is_fractional_part_significant() {
    assert!(!is_fractional_part_significant(0.0));
    assert!(!is_fractional_part_significant(1e-10));
    assert!(!is_fractional_part_significant(-1e-10));
    assert!(is_fractional_part_significant(1.1e-9));
    assert!(is_fractional_part_significant(-1.1e-9));
    assert!(!is_fractional_part_significant(1.000_000_000_1));
    assert!(!is_fractional_part_significant(-1.000_000_000_1));
    assert!(is_fractional_part_significant(1.000_000_001));
    assert!(is_fractional_part_significant(-1.000_000_001));
}

fn assert_rational_value(x: Option<RationalNumber>, expected: (i64, i64, i64)) {
    match x {
        None => panic!("Expected rational number."),
        Some(r) => assert!(
            r.sign == expected.0 && r.numerator == expected.1 && r.denominator == expected.2
        ),
    }
}

#[test]
fn check_construct_rational() {
    assert_rational_value(Some(RationalNumber::new(1, 2)), (1, 1, 2));
    assert_rational_value(Some(RationalNumber::new(-1, 2)), (-1, 1, 2));
    assert_rational_value(Some(RationalNumber::new(1, -2)), (-1, 1, 2));
    assert_rational_value(Some(RationalNumber::new(-1, -2)), (1, 1, 2));
    // Although 0 is never used in the code we check it for completeness.
    assert_rational_value(Some(RationalNumber::new(0, 1)), (0, 0, 1));
    expect!([r"
        RationalNumber {
            sign: 1,
            numerator: 1,
            denominator: 2,
        }
    "])
    .assert_debug_eq(&RationalNumber::new(1, 2));
}

#[test]
fn check_abs_rational() {
    assert_rational_value(Some(RationalNumber::new(1, 2).abs()), (1, 1, 2));
    assert_rational_value(Some(RationalNumber::new(-1, 2).abs()), (1, 1, 2));
    assert_rational_value(Some(RationalNumber::new(1, -2).abs()), (1, 1, 2));
    assert_rational_value(Some(RationalNumber::new(-1, -2).abs()), (1, 1, 2));
    // Although 0 is never used in the code we check it for completeness.
    assert_rational_value(Some(RationalNumber::new(0, 1).abs()), (0, 0, 1));
}

#[test]
fn check_recognize_rational() {
    assert_rational_value(RationalNumber::recognize(1.0 / 1.0), (1, 1, 1));
    assert_rational_value(RationalNumber::recognize(1.0 / 2.0), (1, 1, 2));
    assert_rational_value(RationalNumber::recognize(1.0 / 3.0), (1, 1, 3));
    assert_rational_value(RationalNumber::recognize(-5.0 / 7.0), (-1, 5, 7));
    assert!(RationalNumber::recognize(1.0 / 1000.0).is_none());
    assert!(RationalNumber::recognize(1000.0 / 1.0).is_none());
    // Although 0 is never used in the code we check it for completeness.
    assert_rational_value(RationalNumber::recognize(0.0), (0, 0, 1));
}

fn assert_algebraic_value(x: Option<AlgebraicNumber>, expected: (i64, i64, i64, i64, i64)) {
    match x {
        None => panic!("Expected algebraic number."),
        Some(a) => assert!(
            a.sign == expected.0
                && a.fraction.sign == expected.1
                && a.fraction.numerator == expected.2
                && a.fraction.denominator == expected.3
                && a.root == expected.4
        ),
    }
}

#[test]
fn check_construct_algebraic() {
    assert_algebraic_value(
        Some(AlgebraicNumber::new(&RationalNumber::new(1, 2), 3)),
        (1, 1, 1, 2, 3),
    );
    assert_algebraic_value(
        Some(AlgebraicNumber::new(&RationalNumber::new(-1, 2), 3)),
        (-1, 1, 1, 2, 3),
    );
    assert_algebraic_value(
        Some(AlgebraicNumber::new(&RationalNumber::new(1, -2), 3)),
        (-1, 1, 1, 2, 3),
    );
    assert_algebraic_value(
        Some(AlgebraicNumber::new(&RationalNumber::new(-1, -2), 3)),
        (1, 1, 1, 2, 3),
    );
    expect!([r"
        AlgebraicNumber {
            sign: 1,
            fraction: RationalNumber {
                sign: 1,
                numerator: 1,
                denominator: 2,
            },
            root: 3,
        }
    "])
    .assert_debug_eq(&AlgebraicNumber::new(&RationalNumber::new(1, 2), 3));
}

#[test]
fn check_recognize_algebraic() {
    assert_algebraic_value(AlgebraicNumber::recognize(5.0), (1, 1, 5, 1, 1));
    assert_algebraic_value(AlgebraicNumber::recognize(1.0 / 7.0), (1, 1, 1, 7, 1));
    assert_algebraic_value(AlgebraicNumber::recognize(7.0 / 10.0), (1, 1, 7, 10, 1));
    assert_algebraic_value(
        AlgebraicNumber::recognize(2.0 * 2.0_f64.sqrt()),
        (1, 1, 2, 1, 2),
    );
    assert_algebraic_value(AlgebraicNumber::recognize(8.0_f64.sqrt()), (1, 1, 2, 1, 2));
    assert_algebraic_value(
        AlgebraicNumber::recognize(5.0_f64.sqrt() / 15.0),
        (1, 1, 1, 15, 5),
    );
    assert_algebraic_value(
        AlgebraicNumber::recognize(3.0 / 5.0 * 2.0_f64.sqrt()),
        (1, 1, 3, 5, 2),
    );
    assert_algebraic_value(
        AlgebraicNumber::recognize(-3.0 / 5.0 * 2.0_f64.sqrt()),
        (-1, 1, 3, 5, 2),
    );
}

fn assert_decimal_value(x: &DecimalNumber, expected: (i64, f64)) {
    assert!(x.sign == expected.0 && (x.value - expected.1).abs() < f64::EPSILON);
}

#[test]
fn check_construct_decimal() {
    assert_decimal_value(&DecimalNumber::new(0.777), (1, 0.777));
    assert_decimal_value(&DecimalNumber::new(-0.777), (-1, 0.777));
    expect!([r"
        DecimalNumber {
            sign: 1,
            value: 1.0,
        }
    "])
    .assert_debug_eq(&DecimalNumber::new(1.0));
}

#[test]
fn check_recognize_decimal() {
    assert_decimal_value(&DecimalNumber::recognize(0.777), (1, 0.777));
    assert_decimal_value(&DecimalNumber::recognize(-0.777), (-1, 0.777));
}

#[test]
fn check_recognize_real_number() {
    expect!([r"
        Zero
    "])
    .assert_debug_eq(&RealNumber::recognize(0.0));

    expect!([r"
        Algebraic(
            AlgebraicNumber {
                sign: 1,
                fraction: RationalNumber {
                    sign: 1,
                    numerator: 5,
                    denominator: 3,
                },
                root: 2,
            },
        )
    "])
    .assert_debug_eq(&RealNumber::recognize(5.0 * 2.0_f64.sqrt() / 3.0));

    expect!([r"
        Algebraic(
            AlgebraicNumber {
                sign: 1,
                fraction: RationalNumber {
                    sign: 1,
                    numerator: 7,
                    denominator: 10,
                },
                root: 1,
            },
        )
    "])
    .assert_debug_eq(&RealNumber::recognize(7.0 / 10.0));

    expect!([r"
        Decimal(
            DecimalNumber {
                sign: 1,
                value: 0.00558659217877095,
            },
        )
    "])
    .assert_debug_eq(&RealNumber::recognize(1.0 / 179.0));

    expect!([r"
        Algebraic(
            AlgebraicNumber {
                sign: -1,
                fraction: RationalNumber {
                    sign: 1,
                    numerator: 2,
                    denominator: 3,
                },
                root: 1,
            },
        )
    "])
    .assert_debug_eq(&RealNumber::recognize(-2.0 / 3.0));

    expect!([r"
        Algebraic(
            AlgebraicNumber {
                sign: -1,
                fraction: RationalNumber {
                    sign: 1,
                    numerator: 5,
                    denominator: 7,
                },
                root: 3,
            },
        )
    "])
    .assert_debug_eq(&RealNumber::recognize(-5.0 * 3.0_f64.sqrt() / 7.0));
}

#[test]
fn check_recognize_polar() {
    expect!([r"
        Some(
            PolarForm {
                sign: 1,
                magnitude: AlgebraicNumber {
                    sign: 1,
                    fraction: RationalNumber {
                        sign: 1,
                        numerator: 5,
                        denominator: 2,
                    },
                    root: 1,
                },
                phase_multiplier: RationalNumber {
                    sign: 1,
                    numerator: 1,
                    denominator: 3,
                },
            },
        )
    "])
    .assert_debug_eq(&PolarForm::recognize(
        5.0 / 2.0 * (PI / 3.0).cos(),
        5.0 / 2.0 * (PI / 3.0).sin(),
    ));
    expect!([r"
        Some(
            PolarForm {
                sign: 1,
                magnitude: AlgebraicNumber {
                    sign: 1,
                    fraction: RationalNumber {
                        sign: 1,
                        numerator: 5,
                        denominator: 2,
                    },
                    root: 1,
                },
                phase_multiplier: RationalNumber {
                    sign: -1,
                    numerator: 1,
                    denominator: 3,
                },
            },
        )
    "])
    .assert_debug_eq(&PolarForm::recognize(
        5.0 / 2.0 * (PI / 3.0).cos(),
        5.0 / 2.0 * (-PI / 3.0).sin(),
    ));
}

#[test]
fn check_recognize_cartesian() {
    expect!([r"
        CartesianForm {
            sign: -1,
            real_part: Zero,
            imaginary_part: Algebraic(
                AlgebraicNumber {
                    sign: 1,
                    fraction: RationalNumber {
                        sign: 1,
                        numerator: 5,
                        denominator: 3,
                    },
                    root: 2,
                },
            ),
        }
    "])
    .assert_debug_eq(&CartesianForm::recognize(0.0, -5.0 / 3.0 * 2.0_f64.sqrt()));
    expect!([r"
        CartesianForm {
            sign: -1,
            real_part: Algebraic(
                AlgebraicNumber {
                    sign: 1,
                    fraction: RationalNumber {
                        sign: 1,
                        numerator: 7,
                        denominator: 3,
                    },
                    root: 1,
                },
            ),
            imaginary_part: Algebraic(
                AlgebraicNumber {
                    sign: -1,
                    fraction: RationalNumber {
                        sign: 1,
                        numerator: 2,
                        denominator: 9,
                    },
                    root: 3,
                },
            ),
        }
    "])
    .assert_debug_eq(&CartesianForm::recognize(
        -7.0 / 3.0,
        2.0 / 9.0 * 3.0_f64.sqrt(),
    ));
}

#[test]
fn check_recognize_complex() {
    expect!([r"
        Cartesian(
            CartesianForm {
                sign: -1,
                real_part: Zero,
                imaginary_part: Algebraic(
                    AlgebraicNumber {
                        sign: 1,
                        fraction: RationalNumber {
                            sign: 1,
                            numerator: 5,
                            denominator: 3,
                        },
                        root: 2,
                    },
                ),
            },
        )
    "])
    .assert_debug_eq(&ComplexNumber::recognize(0.0, -5.0 / 3.0 * 2.0_f64.sqrt()));

    expect!([r"
        Cartesian(
            CartesianForm {
                sign: -1,
                real_part: Algebraic(
                    AlgebraicNumber {
                        sign: 1,
                        fraction: RationalNumber {
                            sign: 1,
                            numerator: 7,
                            denominator: 3,
                        },
                        root: 1,
                    },
                ),
                imaginary_part: Algebraic(
                    AlgebraicNumber {
                        sign: -1,
                        fraction: RationalNumber {
                            sign: 1,
                            numerator: 2,
                            denominator: 9,
                        },
                        root: 3,
                    },
                ),
            },
        )
    "])
    .assert_debug_eq(&ComplexNumber::recognize(
        -7.0 / 3.0,
        2.0 / 9.0 * 3.0_f64.sqrt(),
    ));

    expect!([r"
        Polar(
            PolarForm {
                sign: 1,
                magnitude: AlgebraicNumber {
                    sign: 1,
                    fraction: RationalNumber {
                        sign: 1,
                        numerator: 5,
                        denominator: 2,
                    },
                    root: 1,
                },
                phase_multiplier: RationalNumber {
                    sign: 1,
                    numerator: 1,
                    denominator: 3,
                },
            },
        )
    "])
    .assert_debug_eq(&ComplexNumber::recognize(
        5.0 / 2.0 * (PI / 3.0).cos(),
        5.0 / 2.0 * (PI / 3.0).sin(),
    ));
}

fn assert_latex_for_algebraic(
    expected: &Expect,
    numerator: i64,
    denominator: i64,
    root: i64,
    render_one: bool,
) {
    let number = AlgebraicNumber::new(&RationalNumber::new(numerator, denominator), root);
    let mut latex = String::with_capacity(50);
    write_latex_for_algebraic_number(&mut latex, &number, render_one);
    expected.assert_eq(&latex);
}

#[test]
fn check_get_latex_for_algebraic() {
    assert_latex_for_algebraic(&expect!([r"\frac{5 \sqrt{2}}{3}"]), 5, 3, 2, false);
    assert_latex_for_algebraic(&expect!([r"\frac{5 \sqrt{2}}{3}"]), -5, 3, 2, false);
    assert_latex_for_algebraic(&expect!([r"\frac{\sqrt{2}}{3}"]), 1, 3, 2, false);
    assert_latex_for_algebraic(&expect!([r"5 \sqrt{2}"]), 5, 1, 2, false);
    assert_latex_for_algebraic(&expect!([r"\frac{5}{3}"]), 5, 3, 1, false);
    assert_latex_for_algebraic(&expect!([r"\sqrt{2}"]), 1, 1, 2, false);
    assert_latex_for_algebraic(&expect!("5"), 5, 1, 1, false);
    assert_latex_for_algebraic(&expect!([r"\frac{1}{3}"]), 1, 3, 1, false);
    assert_latex_for_algebraic(&expect!(""), 1, 1, 1, false);
    assert_latex_for_algebraic(&expect!("1"), 1, 1, 1, true);
}

fn assert_latex_for_decimal(expected: &Expect, number: f64, render_one: bool) {
    let number = DecimalNumber::new(number);
    let mut latex = String::with_capacity(50);
    write_latex_for_decimal_number(&mut latex, &number, render_one);
    expected.assert_eq(&latex);
}

#[test]
fn check_get_latex_for_decimal() {
    assert_latex_for_decimal(&expect!("0.25"), 0.25, false);
    assert_latex_for_decimal(&expect!("0.25"), -0.25, false);
    assert_latex_for_decimal(&expect!(""), -1.0, false);
    assert_latex_for_decimal(&expect!(""), 1.0, false);
    assert_latex_for_decimal(&expect!("1"), 1.0, true);
}

fn assert_latex_for_real(expected: &Expect, x: f64, render_one: bool) {
    let number = RealNumber::recognize(x);
    let mut latex = String::with_capacity(50);
    write_latex_for_real_number(&mut latex, &number, render_one);
    expected.assert_eq(&latex);
}

#[test]
fn check_get_latex_for_real() {
    assert_latex_for_real(&expect!([r"\frac{1}{4}"]), 1.0 / 4.0, false);
    assert_latex_for_real(&expect!([r"\frac{1}{4}"]), -1.0 / 4.0, false);
    assert_latex_for_real(&expect!(""), 1.0, false);
    assert_latex_for_real(&expect!("1"), 1.0, true);
    assert_latex_for_real(&expect!("0"), 0.0, false);
    assert_latex_for_real(&expect!("0.0003"), 1.0 / 4000.0, false);
}

fn assert_latex_for_cartesian(expected: &Expect, re: f64, im: f64, render_plus: bool) {
    let number = CartesianForm::recognize(re, im);
    let mut latex = String::with_capacity(50);
    write_latex_for_cartesian_form(&mut latex, &number, render_plus, false);
    expected.assert_eq(&latex);
}

#[test]
fn check_get_latex_for_cartesian() {
    assert_latex_for_cartesian(
        &expect!([r"\left( \frac{1}{2}+\frac{1}{2}i \right)"]),
        0.5,
        0.5,
        false,
    );
    assert_latex_for_cartesian(
        &expect!([r"-\left( \frac{1}{2}-\frac{1}{2}i \right)"]),
        -0.5,
        0.5,
        false,
    );
    assert_latex_for_cartesian(
        &expect!([r"\left( \frac{1}{2}-\frac{1}{2}i \right)"]),
        0.5,
        -0.5,
        false,
    );
    assert_latex_for_cartesian(
        &expect!([r"-\left( \frac{1}{2}+\frac{1}{2}i \right)"]),
        -0.5,
        -0.5,
        false,
    );
    assert_latex_for_cartesian(&expect!([r"-\frac{1}{2}i"]), 0.0, -0.5, false);
    assert_latex_for_cartesian(&expect!([r"-\frac{1}{2}"]), -0.5, 0.0, false);
    assert_latex_for_cartesian(&expect!(""), 1.0, 0.0, false);
    assert_latex_for_cartesian(&expect!("+"), 1.0, 0.0, true);
}

fn assert_latex_for_polar(expected: &Expect, re: f64, im: f64, render_plus: bool) {
    let number = PolarForm::recognize(re, im).expect("Polar form not recognized.");
    let mut latex = String::with_capacity(50);
    write_latex_for_polar_form(&mut latex, &number, render_plus);
    expected.assert_eq(&latex);
}

#[test]
fn check_get_latex_for_polar() {
    assert_latex_for_polar(
        &expect!([r"+\frac{1}{2} e^{ i \pi / 3}"]),
        1.0 / 2.0 * (PI / 3.0).cos(),
        1.0 / 2.0 * (PI / 3.0).sin(),
        true,
    );
    assert_latex_for_polar(
        &expect!([r"+ e^{ i \pi / 3}"]),
        (PI / 3.0).cos(),
        (PI / 3.0).sin(),
        true,
    );
    assert_latex_for_polar(
        &expect!([r"+\frac{1}{2} e^{- i \pi / 3}"]),
        1.0 / 2.0 * (-PI / 3.0).cos(),
        1.0 / 2.0 * (-PI / 3.0).sin(),
        true,
    );
    assert_latex_for_polar(
        &expect!([r"+\frac{1}{2} e^{2 i \pi / 3}"]),
        1.0 / 2.0 * (2.0 * PI / 3.0).cos(),
        1.0 / 2.0 * (2.0 * PI / 3.0).sin(),
        true,
    );
    assert_latex_for_polar(
        &expect!([r"+\frac{1}{2} e^{-2 i \pi / 3}"]),
        1.0 / 2.0 * (-2.0 * PI / 3.0).cos(),
        1.0 / 2.0 * (-2.0 * PI / 3.0).sin(),
        true,
    );
    assert_latex_for_polar(
        &expect!([r"\frac{1}{2} e^{-2 i \pi / 3}"]),
        1.0 / 2.0 * (-2.0 * PI / 3.0).cos(),
        1.0 / 2.0 * (-2.0 * PI / 3.0).sin(),
        false,
    );
}

fn assert_latex_for_term(expected: &Expect, re: f64, im: f64, render_plus: bool) {
    let t: Term = Term {
        basis_vector: 0_u8.into(),
        coordinate: ComplexNumber::recognize(re, im),
    };
    let mut latex = String::with_capacity(50);
    write_latex_for_term(&mut latex, &t, render_plus);
    expected.assert_eq(&latex);
}

#[test]
fn check_get_latex_for_term() {
    assert_latex_for_term(
        &expect!([r"+\frac{1}{2} e^{ i \pi / 3}"]),
        1.0 / 2.0 * (PI / 3.0).cos(),
        1.0 / 2.0 * (PI / 3.0).sin(),
        true,
    );
    assert_latex_for_term(
        &expect!([r"+\left( \frac{1}{2}+\frac{1}{2}i \right)"]),
        1.0 / 2.0,
        1.0 / 2.0,
        true,
    );
    assert_latex_for_term(
        &expect!([r"\left( \frac{1}{2}+\frac{1}{2}i \right)"]),
        1.0 / 2.0,
        1.0 / 2.0,
        false,
    );
    assert_latex_for_term(
        &expect!([r"-\left( \frac{1}{2}-\frac{1}{2}i \right)"]),
        -1.0 / 2.0,
        1.0 / 2.0,
        true,
    );
    assert_latex_for_term(
        &expect!([r"-\left( \frac{1}{2}-\frac{1}{2}i \right)"]),
        -1.0 / 2.0,
        1.0 / 2.0,
        false,
    );
}

fn assert_latex_for_complex_number(expected: &Expect, re: f64, im: f64) {
    let n: ComplexNumber = ComplexNumber::recognize(re, im);
    let mut latex = String::with_capacity(50);
    write_latex_for_complex_number(&mut latex, &n);
    expected.assert_eq(&latex);
}

#[test]
fn check_get_latex_for_complex_number() {
    // Future work:
    // While rendering is correct, a better way may be the following:
    // -(1-i) -> -1+i remove brackets for standalone number
    // 1/2 i -> i/2
    // √2/2 -> 1/√2
    assert_latex_for_complex_number(&expect!([r"0"]), 0.0, 0.0);

    assert_latex_for_complex_number(&expect!([r"1"]), 1.0, 0.0);
    assert_latex_for_complex_number(&expect!([r"-1"]), -1.0, 0.0);
    assert_latex_for_complex_number(&expect!([r"i"]), 0.0, 1.0);
    assert_latex_for_complex_number(&expect!([r"-i"]), 0.0, -1.0);

    assert_latex_for_complex_number(&expect!([r"\frac{1}{2}"]), 0.5, 0.0);
    assert_latex_for_complex_number(&expect!([r"-\frac{1}{2}"]), -0.5, 0.0);
    assert_latex_for_complex_number(&expect!([r"\frac{1}{2}i"]), 0.0, 0.5);
    assert_latex_for_complex_number(&expect!([r"-\frac{1}{2}i"]), 0.0, -0.5);

    assert_latex_for_complex_number(
        &expect!([r#"\left( \frac{1}{2}+\frac{1}{2}i \right)"#]),
        0.5,
        0.5,
    );
    assert_latex_for_complex_number(
        &expect!([r#"-\left( \frac{1}{2}-\frac{1}{2}i \right)"#]),
        -0.5,
        0.5,
    );
    assert_latex_for_complex_number(
        &expect!([r#"\left( \frac{1}{2}-\frac{1}{2}i \right)"#]),
        0.5,
        -0.5,
    );
    assert_latex_for_complex_number(
        &expect!([r#"-\left( \frac{1}{2}+\frac{1}{2}i \right)"#]),
        -0.5,
        -0.5,
    );

    assert_latex_for_complex_number(&expect!([r#"\frac{\sqrt{2}}{2}"#]), FRAC_1_SQRT_2, 0.0);
    assert_latex_for_complex_number(&expect!([r#"-\frac{\sqrt{2}}{2}"#]), -FRAC_1_SQRT_2, 0.0);
    assert_latex_for_complex_number(&expect!([r#"\frac{\sqrt{2}}{2}i"#]), 0.0, FRAC_1_SQRT_2);
    assert_latex_for_complex_number(&expect!([r#"-\frac{\sqrt{2}}{2}i"#]), 0.0, -FRAC_1_SQRT_2);

    assert_latex_for_complex_number(
        &expect!([r"\frac{1}{2} e^{ i \pi / 3}"]),
        1.0 / 2.0 * (PI / 3.0).cos(),
        1.0 / 2.0 * (PI / 3.0).sin(),
    );
    assert_latex_for_complex_number(
        &expect!([r#"\left( \frac{1}{2}+\frac{1}{2}i \right)"#]),
        1.0 / 2.0,
        1.0 / 2.0,
    );
}

#[test]
fn check_get_latex() {
    expect!([r"$|\psi\rangle = \left( \frac{1}{2}+\frac{1}{2}i \right)|00\rangle$"]).assert_eq(
        &get_state_latex(&vec![(0_u8.into(), Complex64::new(0.5, 0.5))], 2)
            .expect("expected valid latex"),
    );
    expect!([r"$|\psi\rangle = -|00\rangle$"]).assert_eq(
        &get_state_latex(&vec![(0_u8.into(), Complex64::new(-1.0, 0.0))], 2)
            .expect("expected valid latex"),
    );
    expect!([r"$|\psi\rangle = -i|00\rangle$"]).assert_eq(
        &get_state_latex(&vec![(0_u8.into(), Complex64::new(0.0, -1.0))], 2)
            .expect("expected valid latex"),
    );
    expect!([r"$|\psi\rangle =  e^{-2 i \pi / 3}|00\rangle$"]).assert_eq(
        &get_state_latex(
            &vec![(
                0_u8.into(),
                Complex64::new((-2.0 * PI / 3.0).cos(), (-2.0 * PI / 3.0).sin()),
            )],
            2,
        )
        .expect("expected valid latex"),
    );
    expect!([r"$|\psi\rangle = \left( 1+\frac{\sqrt{2}}{2}i \right)|00\rangle+\left( 1+\frac{\sqrt{2}}{2}i \right)|10\rangle$"])
    .assert_eq(&get_state_latex(
        &vec![
            (0_u8.into(), Complex64::new(1.0, 1.0 / 2.0_f64.sqrt())),
            (2_u8.into(), Complex64::new(1.0, 1.0 / 2.0_f64.sqrt())),
        ],
        2,
    ).expect("expected valid latex"));
}

#[test]
fn check_get_matrix_latex() {
    expect!([r#"$ \begin{bmatrix} 0 & 1 \\ i & \left( 1+i \right) \\ \end{bmatrix} $"#]).assert_eq(
        &get_matrix_latex(&vec![
            vec![Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0)],
            vec![Complex64::new(0.0, 1.0), Complex64::new(1.0, 1.0)],
        ]),
    );
    expect!([r#"$ \begin{bmatrix} -\left( 1-i \right) & -1 \\ -i & -\left( 1+i \right) \\ \end{bmatrix} $"#]).assert_eq(
        &get_matrix_latex(&vec![
            vec![Complex64::new(-1.0, 1.0), Complex64::new(-1.0, 0.0)],
            vec![Complex64::new(0.0, -1.0), Complex64::new(-1.0, -1.0)],
        ]),
    );
    expect!([r#"$ \begin{bmatrix} \frac{1}{\sqrt{2}} & \frac{i}{\sqrt{2}} \\ -\frac{1}{\sqrt{2}} & -\frac{i}{\sqrt{2}} \\ \end{bmatrix} $"#]).assert_eq(&get_matrix_latex(&vec![
        vec![
            Complex64::new(FRAC_1_SQRT_2, 0.0),
            Complex64::new(0.0, FRAC_1_SQRT_2),
        ],
        vec![
            Complex64::new(-FRAC_1_SQRT_2, 0.0),
            Complex64::new(0.0, -FRAC_1_SQRT_2),
        ],
    ]));
    expect!([r#"$ \begin{bmatrix} \frac{1}{2} & \frac{i}{2} \\ -\frac{1}{2} & -\frac{i}{2} \\ \end{bmatrix} $"#]).assert_eq(&get_matrix_latex(&vec![
        vec![
            Complex64::new(0.5, 0.0),
            Complex64::new(0.0, 0.5),
        ],
        vec![
            Complex64::new(-0.5, 0.0),
            Complex64::new(0.0, -0.5),
        ],
    ]));
    expect!([r#"$ \begin{bmatrix} \frac{1}{2} + \frac{i}{2} & -\frac{1}{2} - \frac{i}{2} \\ -\frac{1}{2} + \frac{i}{2} & \frac{1}{2} - \frac{i}{2} \\ \end{bmatrix} $"#]).assert_eq(&get_matrix_latex(&vec![
        vec![
            Complex64::new(0.5, 0.5),
            Complex64::new(-0.5, -0.5),
        ],
        vec![
            Complex64::new(-0.5, 0.5),
            Complex64::new(0.5, -0.5),
        ],
    ]));
}

#[test]
fn check_get_latex_perf() {
    // This is not a CI gate for performance, just prints out data.
    let state = vec![
        (0_u8.into(), Complex64::new(1.0 / 2.0, 0.0)),
        (
            1_u8.into(),
            Complex64::new(0.353_553_390_593_273_8, 0.353_553_390_593_273_8),
        ),
        (2_u8.into(), Complex64::new(0.0, 1.0 / 2.0)),
        (
            3_u8.into(),
            Complex64::new(-0.353_553_390_593_273_8, 0.353_553_390_593_273_8),
        ),
    ];

    expect!([r"$|\psi\rangle = \frac{1}{2}|00\rangle+\frac{1}{2} e^{ i \pi / 4}|01\rangle+\frac{1}{2}i|10\rangle+\frac{1}{2} e^{3 i \pi / 4}|11\rangle$"])
    .assert_eq(&get_state_latex(
        &state,
        2,
    ).expect("expected valid latex"));

    print!("Start...");
    let start = Instant::now();
    let mut l: usize = 0;
    for _ in 0..1_000 {
        let s = get_state_latex(&state, 2);
        l += s.map_or(0, |s| s.len());
    }
    println!(
        "Done. {} bytes in {:?}.",
        l,
        Instant::now().duration_since(start)
    );
}
