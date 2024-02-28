//use expect_test::expect;

use crate::state::{is_fractional_part_significant, is_significant, RationalNumber};

//use super::{get_latex_for_exponent, recognize_nice_exponent};

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
    assert_rational_value(Some(RationalNumber::construct(1, 2)), (1, 1, 2));
    assert_rational_value(Some(RationalNumber::construct(-1, 2)), (-1, 1, 2));
    assert_rational_value(Some(RationalNumber::construct(1, -2)), (-1, 1, 2));
    assert_rational_value(Some(RationalNumber::construct(-1, -2)), (1, 1, 2));
    // Although 0 is never used in the code we check it for completeness.
    assert_rational_value(Some(RationalNumber::construct(0, 1)), (0, 0, 1));
}

#[test]
fn check_abs_rational() {
    assert_rational_value(Some(RationalNumber::construct(1, 2).abs()), (1, 1, 2));
    assert_rational_value(Some(RationalNumber::construct(-1, 2).abs()), (1, 1, 2));
    assert_rational_value(Some(RationalNumber::construct(1, -2).abs()), (1, 1, 2));
    assert_rational_value(Some(RationalNumber::construct(-1, -2).abs()), (1, 1, 2));
    // Although 0 is never used in the code we check it for completeness.
    assert_rational_value(Some(RationalNumber::construct(0, 1).abs()), (0, 0, 1));
}

#[test]
fn check_recognize_rational() {
    assert_rational_value(RationalNumber::recognize(1.0 / 1.0), (1, 1, 1));
    assert_rational_value(RationalNumber::recognize(1.0 / 2.0), (1, 1, 2));
    assert_rational_value(RationalNumber::recognize(1.0 / 3.0), (1, 1, 3));
    assert_rational_value(RationalNumber::recognize(-5.0 / 7.0), (-1, 5, 7));
    assert_rational_value(RationalNumber::recognize(5.0 / -7.0), (-1, 5, 7));
    assert!(RationalNumber::recognize(1.0 / 1000.0).is_none());
    assert!(RationalNumber::recognize(1000.0 / 1.0).is_none());

    // Although 0 is never used in the code we check it for completeness.
    assert_rational_value(RationalNumber::recognize(0.0 / -7.0), (0, 0, 1));
}

// #[test]
// fn check_get_latex_for_algebraic() {
//     expect!([r#"\frac{5 \sqrt{2}}{3}"#]).assert_eq(&get_latex_for_algebraic(5, 3, 2, false));
//     expect!([r#"\frac{\sqrt{2}}{3}"#]).assert_eq(&get_latex_for_algebraic(1, 3, 2, false));
//     expect!([r#"5 \sqrt{2}"#]).assert_eq(&get_latex_for_algebraic(5, 1, 2, false));
//     expect!([r#"\frac{5}{3}"#]).assert_eq(&get_latex_for_algebraic(5, 3, 1, false));
//     expect!([r#"\sqrt{2}"#]).assert_eq(&get_latex_for_algebraic(1, 1, 2, false));
//     expect!("5").assert_eq(&get_latex_for_algebraic(5, 1, 1, false));
//     expect!([r#"\frac{1}{3}"#]).assert_eq(&get_latex_for_algebraic(1, 3, 1, false));
//     expect!("").assert_eq(&get_latex_for_algebraic(1, 1, 1, false));
//     expect!("1").assert_eq(&get_latex_for_algebraic(1, 1, 1, true));
// }

// #[test]
// fn check_recognize_nice_algebraic() {
//     let (p, s) = recognize_nice_algebraic(0.25, false);
//     assert!(p);
//     expect!([r#"\frac{1}{4}"#]).assert_eq(&s);
// }

// #[test]
// fn check_recognize_nice_exponent() {
//     assert!(
//         recognize_nice_exponent(1.0 / 2.0_f64.sqrt(), 1.0 / 2.0_f64.sqrt()) == (true, 1, 1, 1, 4)
//     );
// }

// #[test]
// fn check_get_latex_for_exponent() {
//     expect!([r#"e^{ i \pi }"#]).assert_eq(&get_latex_for_exponent(1, 1));
//     expect!([r#"e^{- i \pi }"#]).assert_eq(&get_latex_for_exponent(-1, 1));
//     expect!([r#"e^{ i \pi  / 2}"#]).assert_eq(&get_latex_for_exponent(1, 2));
//     expect!([r#"e^{- i \pi  / 2}"#]).assert_eq(&get_latex_for_exponent(-1, 2));
//     expect!([r#"e^{2 i \pi  / 3}"#]).assert_eq(&get_latex_for_exponent(2, 3));
//     expect!([r#"e^{-2 i \pi  / 3}"#]).assert_eq(&get_latex_for_exponent(-2, 3));
// }
