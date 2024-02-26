//use expect_test::expect;

use crate::state::{is_fractional_part_significant, is_significant};

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

// #[test]
// fn check_recognize_nice_rational() {
//     assert!(recognize_nice_rational(1.0 / 1.0) == (true, 1, 1));
//     assert!(recognize_nice_rational(1.0 / 2.0) == (true, 1, 2));
//     assert!(recognize_nice_rational(1.0 / 3.0) == (true, 1, 3));
//     assert!(recognize_nice_rational(-5.0 / 7.0) == (true, -5, 7));
//     assert!(recognize_nice_rational(5.0 / -7.0) == (true, -5, 7));
//     assert!(recognize_nice_rational(1.0 / 100.0) == (false, 0, 1));
// }

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
