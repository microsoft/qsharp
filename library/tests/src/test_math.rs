// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::test_expression;
use core::f64::consts::E;
use num_bigint::BigInt;
use qsc::interpret::Value;
use std::f64::consts::PI;

//
// Sign, Abs, Min, Max, etc.
//

#[test]
fn check_sign_i() {
    test_expression("Microsoft.Quantum.Math.SignI(0)", &Value::Int(0));
    test_expression("Microsoft.Quantum.Math.SignI(1000)", &Value::Int(1));
    test_expression("Microsoft.Quantum.Math.SignI(-1000)", &Value::Int(-1));
}

#[test]
fn check_sign_d() {
    test_expression("Microsoft.Quantum.Math.SignD(0.0)", &Value::Int(0));
    test_expression("Microsoft.Quantum.Math.SignD(0.005)", &Value::Int(1));
    test_expression("Microsoft.Quantum.Math.SignD(-0.005)", &Value::Int(-1));
}

#[test]
fn check_sign_l() {
    test_expression("Microsoft.Quantum.Math.SignL(0L)", &Value::Int(0));
    test_expression(
        "Microsoft.Quantum.Math.SignL(9999999999999999999999999999999999999999L)",
        &Value::Int(1),
    );
    test_expression(
        "Microsoft.Quantum.Math.SignL(-9999999999999999999999999999999999999999L)",
        &Value::Int(-1),
    );
}

#[test]
fn check_abs_i() {
    test_expression("Microsoft.Quantum.Math.AbsI(0)", &Value::Int(0));
    test_expression("Microsoft.Quantum.Math.AbsI(1000)", &Value::Int(1000));
    test_expression("Microsoft.Quantum.Math.AbsI(-1000)", &Value::Int(1000));
}

#[test]
fn check_abs_d() {
    test_expression("Microsoft.Quantum.Math.AbsD(0.0)", &Value::Double(0.0));
    test_expression("Microsoft.Quantum.Math.AbsD(0.005)", &Value::Double(0.005));
    test_expression("Microsoft.Quantum.Math.AbsD(-0.005)", &Value::Double(0.005));
}

#[test]
fn check_abs_l() {
    test_expression(
        "Microsoft.Quantum.Math.AbsL(0L)",
        &Value::BigInt(BigInt::from(0)),
    );
    test_expression(
        "Microsoft.Quantum.Math.AbsL(9999L)",
        &Value::BigInt(BigInt::from(9999)),
    );
    test_expression(
        "Microsoft.Quantum.Math.AbsL(-9999L)",
        &Value::BigInt(BigInt::from(9999)),
    );
}

#[test]
fn check_max_i() {
    test_expression("Microsoft.Quantum.Math.MaxI(-5,7)", &Value::Int(7));
    test_expression("Microsoft.Quantum.Math.MaxI(-7,0)", &Value::Int(0));
}

#[test]
fn check_max_d() {
    test_expression("Microsoft.Quantum.Math.MaxD(-5.0,7.0)", &Value::Double(7.0));
    test_expression("Microsoft.Quantum.Math.MaxD(-7.0,0.0)", &Value::Double(0.0));
}

#[test]
fn check_max_l() {
    test_expression(
        "Microsoft.Quantum.Math.MaxL(-5L,7L)",
        &Value::BigInt(BigInt::from(7)),
    );
    test_expression(
        "Microsoft.Quantum.Math.MaxL(-7L,0L)",
        &Value::BigInt(BigInt::from(0)),
    );
}

#[test]
fn check_min_i() {
    test_expression("Microsoft.Quantum.Math.MinI(-5,7)", &Value::Int(-5));
    test_expression("Microsoft.Quantum.Math.MinI(-7,0)", &Value::Int(-7));
}

#[test]
fn check_min_d() {
    test_expression(
        "Microsoft.Quantum.Math.MinD(-5.0,7.0)",
        &Value::Double(-5.0),
    );
    test_expression(
        "Microsoft.Quantum.Math.MinD(-7.0,0.0)",
        &Value::Double(-7.0),
    );
}

#[test]
fn check_min_l() {
    test_expression(
        "Microsoft.Quantum.Math.MinL(-5L,7L)",
        &Value::BigInt(BigInt::from(-5)),
    );
    test_expression(
        "Microsoft.Quantum.Math.MinL(-7L,0L)",
        &Value::BigInt(BigInt::from(-7)),
    );
}

//
// Trigonometric functions
//

#[test]
fn check_pi() {
    test_expression(
        "Microsoft.Quantum.Math.PI()",
        &Value::Double(std::f64::consts::PI),
    );
}

#[test]
fn check_e() {
    test_expression(
        "Microsoft.Quantum.Math.E()",
        &Value::Double(std::f64::consts::E),
    );
}

#[test]
fn check_arccos() {
    test_expression(
        "Microsoft.Quantum.Math.ArcCos(0.43)",
        &Value::Double(0.43_f64.acos()),
    );
}

#[test]
fn check_arcsin() {
    test_expression(
        "Microsoft.Quantum.Math.ArcSin(0.43)",
        &Value::Double(0.43_f64.asin()),
    );
}

#[test]
fn check_arctan() {
    test_expression(
        "Microsoft.Quantum.Math.ArcTan(43.43)",
        &Value::Double(43.43_f64.atan()),
    );
}

#[test]
fn check_arctan2() {
    test_expression(
        "Microsoft.Quantum.Math.ArcTan2(43.0,17.0)",
        &Value::Double(43.0_f64.atan2(17.0)),
    );
}

#[test]
fn check_cos() {
    test_expression(
        "Microsoft.Quantum.Math.Cos(1.11)",
        &Value::Double(1.11_f64.cos()),
    );
}

#[test]
fn check_cosh() {
    test_expression(
        "Microsoft.Quantum.Math.Cosh(1.11)",
        &Value::Double(1.11_f64.cosh()),
    );
}

#[test]
fn check_sin() {
    test_expression(
        "Microsoft.Quantum.Math.Sin(2.22)",
        &Value::Double(2.22_f64.sin()),
    );
}

#[test]
fn check_sinh() {
    test_expression(
        "Microsoft.Quantum.Math.Sinh(2.22)",
        &Value::Double(2.22_f64.sinh()),
    );
}

#[test]
fn check_tan() {
    test_expression(
        "Microsoft.Quantum.Math.Tan(1.23)",
        &Value::Double(1.23_f64.tan()),
    );
}

#[test]
fn check_tanh() {
    test_expression(
        "Microsoft.Quantum.Math.Tanh(1.23)",
        &Value::Double(1.23_f64.tanh()),
    );
}

#[test]
fn check_arccosh() {
    test_expression(
        "Microsoft.Quantum.Math.ArcCosh(1.234)",
        &Value::Double(1.234_f64.acosh()),
    );
}

#[test]
fn check_arcsinh() {
    test_expression(
        "Microsoft.Quantum.Math.ArcSinh(10.0)",
        &Value::Double(10.0_f64.asinh()),
    );
}

#[test]
fn check_arctanh() {
    test_expression(
        "Microsoft.Quantum.Math.ArcTanh(0.5)",
        &Value::Double(0.5_f64.atanh()),
    );
}

//
// Sqrt, Log, exp, etc.
//

#[test]
fn check_sqrt() {
    test_expression(
        "Microsoft.Quantum.Math.Sqrt(57121.0)",
        &Value::Double(239.0),
    );
}

#[test]
fn check_log() {
    test_expression(
        "Microsoft.Quantum.Math.Log(57121.0)",
        &Value::Double(57121.0_f64.ln()),
    );
}

#[test]
fn check_log10() {
    test_expression("Microsoft.Quantum.Math.Log10(1.0)", &Value::Double(0.0));
    test_expression("Microsoft.Quantum.Math.Log10(10.0)", &Value::Double(1.0));
}

#[test]
fn check_lg() {
    test_expression("Microsoft.Quantum.Math.Lg(1.0)", &Value::Double(0.0));
    test_expression("Microsoft.Quantum.Math.Lg(2.0)", &Value::Double(1.0));
}

//
// Truncation and Rounding
//

#[test]
fn check_truncate() {
    test_expression("Microsoft.Quantum.Math.Truncate(3.1)", &Value::Int(3));
    test_expression("Microsoft.Quantum.Math.Truncate(-3.7)", &Value::Int(-3));
}

#[test]
fn check_ceiling() {
    test_expression("Microsoft.Quantum.Math.Ceiling(3.1)", &Value::Int(4));
    test_expression("Microsoft.Quantum.Math.Ceiling(-3.7)", &Value::Int(-3));
}

#[test]
fn check_floor() {
    test_expression("Microsoft.Quantum.Math.Floor(3.7)", &Value::Int(3));
    test_expression("Microsoft.Quantum.Math.Floor(-3.1)", &Value::Int(-4));
}

#[test]
fn check_round() {
    test_expression("Microsoft.Quantum.Math.Round(3.1)", &Value::Int(3));
    test_expression("Microsoft.Quantum.Math.Round(-3.7)", &Value::Int(-4));
}

//
// Modular arithmetic
//

#[test]
fn check_modulus_i() {
    test_expression("Microsoft.Quantum.Math.ModulusI(20, 3)", &Value::Int(2));
    test_expression("Microsoft.Quantum.Math.ModulusI(-20, 3)", &Value::Int(1));
}

#[test]
fn check_modulus_l() {
    test_expression(
        "Microsoft.Quantum.Math.ModulusL(20L, 3L)",
        &Value::BigInt(BigInt::from(2)),
    );
    test_expression(
        "Microsoft.Quantum.Math.ModulusL(-20L, 3L)",
        &Value::BigInt(BigInt::from(1)),
    );
}

#[test]
fn check_exp_mod_i() {
    test_expression("Microsoft.Quantum.Math.ExpModI(1,10,10)", &Value::Int(1));
    test_expression("Microsoft.Quantum.Math.ExpModI(10,0,10)", &Value::Int(1));
    test_expression("Microsoft.Quantum.Math.ExpModI(2,10,10)", &Value::Int(4));
}

#[test]
fn check_exp_mod_l() {
    test_expression(
        "Microsoft.Quantum.Math.ExpModL(1L,10L,10L)",
        &Value::BigInt(BigInt::from(1)),
    );
    test_expression(
        "Microsoft.Quantum.Math.ExpModL(10L,0L,10L)",
        &Value::BigInt(BigInt::from(1)),
    );
    test_expression(
        "Microsoft.Quantum.Math.ExpModL(2L,10L,10L)",
        &Value::BigInt(BigInt::from(4)),
    );
}

#[test]
fn check_inverse_mod_i() {
    test_expression("Microsoft.Quantum.Math.InverseModI(2,5)", &Value::Int(3));
    test_expression("Microsoft.Quantum.Math.InverseModI(3,10)", &Value::Int(7));
    test_expression("Microsoft.Quantum.Math.InverseModI(-1,5)", &Value::Int(4));
}

#[test]
fn check_inverse_mod_l() {
    test_expression(
        "Microsoft.Quantum.Math.InverseModL(2L,5L)",
        &Value::BigInt(BigInt::from(3)),
    );
    test_expression(
        "Microsoft.Quantum.Math.InverseModL(3L,10L)",
        &Value::BigInt(BigInt::from(7)),
    );
    test_expression(
        "Microsoft.Quantum.Math.InverseModL(-1L,5L)",
        &Value::BigInt(BigInt::from(4)),
    );
}

//
// GCD, etc.
//
#[test]
fn check_gcd_i() {
    test_expression(
        "Microsoft.Quantum.Math.GreatestCommonDivisorI(0,0)",
        &Value::Int(0),
    );
    test_expression(
        "Microsoft.Quantum.Math.GreatestCommonDivisorI(2*3*5,2*3*7)",
        &Value::Int(2 * 3),
    );
    test_expression(
        "Microsoft.Quantum.Math.GreatestCommonDivisorI(39088169,63245986)",
        &Value::Int(1),
    );
}

#[test]
fn check_gcd_l() {
    test_expression(
        "Microsoft.Quantum.Math.GreatestCommonDivisorL(0L,0L)",
        &Value::BigInt(BigInt::from(0)),
    );
    test_expression(
        "Microsoft.Quantum.Math.GreatestCommonDivisorL(2L*3L*5L,2L*3L*7L)",
        &Value::BigInt(BigInt::from(2 * 3)),
    );
    test_expression("Microsoft.Quantum.Math.GreatestCommonDivisorL(222232244629420445529739893461909967206666939096499764990979600L,359579325206583560961765665172189099052367214309267232255589801L)", &Value::BigInt(
        BigInt::from(1)));
}

#[test]
fn check_cfc_i() {
    // NOTE: It is not important if the function returns -3/-4 or 3/4,
    // we can ignore this implementation details or update a function
    // to return canonical result.
    test_expression(
        "Microsoft.Quantum.Math.ContinuedFractionConvergentI((72,100), 2)",
        &Value::Tuple(vec![Value::Int(-1), Value::Int(-1)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Math.ContinuedFractionConvergentI((72,100), 3)",
        &Value::Tuple(vec![Value::Int(2), Value::Int(3)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Math.ContinuedFractionConvergentI((72,100), 4)",
        &Value::Tuple(vec![Value::Int(-3), Value::Int(-4)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Math.ContinuedFractionConvergentI((72,100), 7)",
        &Value::Tuple(vec![Value::Int(5), Value::Int(7)].into()),
    );
    test_expression(
        "Microsoft.Quantum.Math.ContinuedFractionConvergentI((72,100), 25)",
        &Value::Tuple(vec![Value::Int(-18), Value::Int(-25)].into()),
    );
}

#[test]
fn check_bitsize_i() {
    test_expression("Microsoft.Quantum.Math.BitSizeI(0)", &Value::Int(0));
    test_expression("Microsoft.Quantum.Math.BitSizeI(1)", &Value::Int(1));
    test_expression("Microsoft.Quantum.Math.BitSizeI(2)", &Value::Int(2));
    test_expression("Microsoft.Quantum.Math.BitSizeI(3)", &Value::Int(2));
    test_expression(
        "Microsoft.Quantum.Math.BitSizeI(0x7FFFFFFFFFFFFFFF)",
        &Value::Int(63),
    );
}

#[test]
fn check_abs_squared_complex() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        AbsSquaredComplex(Complex(1.0,1.0))}",
        &Value::Double(2.0),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        AbsSquaredComplex(Complex(-3.0,4.0))}",
        &Value::Double(25.0),
    );
}

#[test]
fn check_abs_complex() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        AbsComplex(Complex(1.0,1.0))}",
        &Value::Double(2.0_f64.sqrt()),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        AbsComplex(Complex(-3.0,4.0))}",
        &Value::Double(5.0),
    );
}

#[test]
fn check_arg_complex() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        ArgComplex(Complex(100.0,0.0))}",
        &Value::Double(0.0),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        ArgComplex(Complex(1.0,1.0))}",
        &Value::Double(PI / 4.0),
    );
}

#[test]
fn check_abs_squared_complex_polar() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        AbsSquaredComplexPolar(ComplexPolar(1.0,2.0))}",
        &Value::Double(1.0),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        AbsSquaredComplexPolar(ComplexPolar(5.0,-1.0))}",
        &Value::Double(25.0),
    );
}

#[test]
fn check_abs_complex_polar() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        AbsComplexPolar(ComplexPolar(1.0,2.0))}",
        &Value::Double(1.0),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        AbsComplexPolar(ComplexPolar(5.0,-1.0))}",
        &Value::Double(5.0),
    );
}

#[test]
fn check_arg_complex_polar() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        ArgComplexPolar(ComplexPolar(1.0,2.0))}",
        &Value::Double(2.0),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        ArgComplexPolar(ComplexPolar(5.0,-1.0))}",
        &Value::Double(-1.0),
    );
}

#[test]
fn check_negation_c() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        NegationC(Complex(1.0,2.0))}",
        &Value::Tuple(vec![Value::Double(-1.0), Value::Double(-2.0)].into()),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        NegationC(Complex(5.0,-1.0))}",
        &Value::Tuple(vec![Value::Double(-5.0), Value::Double(1.0)].into()),
    );
}

#[test]
fn check_negation_cp() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        NegationCP(ComplexPolar(1.0,0.0))}",
        &Value::Tuple(vec![Value::Double(1.0), Value::Double(PI)].into()),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        NegationCP(ComplexPolar(5.0,-PI()/2.0))}",
        &Value::Tuple(vec![Value::Double(5.0), Value::Double(PI / 2.0)].into()),
    );
}

#[test]
fn check_plus_c() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        PlusC(Complex(1.0,0.0), Complex(0.0,1.0))}",
        &Value::Tuple(vec![Value::Double(1.0), Value::Double(1.0)].into()),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        PlusC(Complex(10.0,10.0), Complex(-10.0,10.0))}",
        &Value::Tuple(vec![Value::Double(0.0), Value::Double(20.0)].into()),
    );
}

#[test]
fn check_plus_cp() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        PlusCP(ComplexPolar(1.0,0.0), ComplexPolar(1.0,PI()/2.0))}",
        &Value::Tuple(vec![Value::Double(2.0_f64.sqrt()), Value::Double(PI / 4.0)].into()),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        PlusCP(ComplexPolar(10.0,PI()/4.0), ComplexPolar(10.0,3.0*PI()/4.0))}",
        &Value::Tuple(vec![Value::Double(200.0_f64.sqrt()), Value::Double(PI / 2.0)].into()),
    );
}

#[test]
fn check_minus_c() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        MinusC(Complex(1.0,0.0), Complex(0.0,1.0))}",
        &Value::Tuple(vec![Value::Double(1.0), Value::Double(-1.0)].into()),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        MinusC(Complex(10.0,10.0), Complex(-10.0,10.0))}",
        &Value::Tuple(vec![Value::Double(20.0), Value::Double(0.0)].into()),
    );
}

#[test]
fn check_minus_cp() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        MinusCP(ComplexPolar(4.0,0.0), ComplexPolar(1.0,-PI()))}",
        &Value::Tuple(vec![Value::Double(5.0), Value::Double(0.0)].into()),
    );
}

#[test]
fn check_times_c() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        TimesC(Complex(2.0,0.0), Complex(3.0,0.0))}",
        &Value::Tuple(vec![Value::Double(6.0), Value::Double(0.0)].into()),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        TimesC(Complex(3.0,0.0), Complex(0.0,1.0))}",
        &Value::Tuple(vec![Value::Double(0.0), Value::Double(3.0)].into()),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        TimesC(Complex(1.0,2.0), Complex(3.0,4.0))}",
        &Value::Tuple(vec![Value::Double(-5.0), Value::Double(10.0)].into()),
    );
}

#[test]
fn check_times_cp() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        TimesCP(ComplexPolar(1.0,0.0), ComplexPolar(1.0,PI()/2.0))}",
        &Value::Tuple(vec![Value::Double(1.0), Value::Double(PI / 2.0)].into()),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        TimesCP(ComplexPolar(1.0,PI()/4.0), ComplexPolar(2.0,3.0*PI()/4.0))}",
        &Value::Tuple(vec![Value::Double(2.0), Value::Double(PI)].into()),
    );
}

#[test]
fn check_pow_c() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        PowC(Complex(2.0,0.0), Complex(3.0,0.0))}",
        &Value::Tuple(vec![Value::Double(8.0), Value::Double(0.0)].into()),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        PowC(Complex(0.0,1.0), Complex(0.0,1.0))}",
        &Value::Tuple(vec![Value::Double(E.powf(-PI / 2.0)), Value::Double(0.0)].into()),
    );
}

#[test]
fn check_pow_cp() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        PowCP(ComplexPolar(2.0,0.0), ComplexPolar(3.0,0.0))}",
        &Value::Tuple(vec![Value::Double(8.0), Value::Double(0.0)].into()),
    );
}

#[test]
fn check_divide_by_c() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        DividedByC(Complex(1.0,0.0), Complex(2.0,0.0))}",
        &Value::Tuple(vec![Value::Double(0.5), Value::Double(0.0)].into()),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        DividedByC(Complex(3.0,0.0), Complex(0.0,1.0))}",
        &Value::Tuple(vec![Value::Double(0.0), Value::Double(-3.0)].into()),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        DividedByC(Complex(1.0,2.0), Complex(3.0,4.0))}",
        &Value::Tuple(vec![Value::Double(0.44), Value::Double(0.08)].into()),
    );
}

#[test]
fn check_devide_by_cp() {
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        DividedByCP(ComplexPolar(1.0,0.0), ComplexPolar(1.0,PI()/2.0))}",
        &Value::Tuple(vec![Value::Double(1.0), Value::Double(-PI / 2.0)].into()),
    );
    test_expression(
        "{
        open Microsoft.Quantum.Math;
        DividedByCP(ComplexPolar(1.0,PI()/4.0), ComplexPolar(2.0,3.0*PI()/4.0))}",
        &Value::Tuple(vec![Value::Double(0.5), Value::Double(-PI / 2.0)].into()),
    );
}
