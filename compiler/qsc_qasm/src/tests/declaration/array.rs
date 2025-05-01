// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod bit;
mod qubit;

use crate::tests::{compile_fragments, compile_qasm_to_qsharp, fail_on_compilation_errors};
use expect_test::expect;
use miette::Report;

#[test]
fn arrays() -> miette::Result<(), Vec<Report>> {
    let source = "
        int y = 6;
        int z = 7;
        array[uint[16], 1] a;
        array[int[8], 4] b;
        array[float[64], 4, 2] c;
        array[angle[32], 4, 3, 2] d;
        array[complex[float[32]], 4] e;
        array[bool, 3] f;
        array[int[8], 4] g = {1, 2, 3, 4};
        array[int[8], 2] h = {y, y+y};
        array[uint[32], 2, 2] i = {{3, 4}, {2-3, y*5}};
        array[uint[32], 2, 2] j = {z, {2-3, y*5}};
        array[uint[32], 2, 2] k = {z*2, {1, 2}};
    ";

    let unit = compile_fragments(source)?;
    fail_on_compilation_errors(&unit);
    Ok(())
}

#[test]
fn default_simple_arrays() -> miette::Result<(), Vec<Report>> {
    let source = "
        array[bool, 3] a;
        array[int, 3] b;
        array[uint, 3] c;
        array[angle, 3] d;
        array[float, 3] e;
        array[complex, 3] f;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        mutable a = [false, false, false];
        mutable b = [0, 0, 0];
        mutable c = [0, 0, 0];
        mutable d = [new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }, new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }, new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }];
        mutable e = [0., 0., 0.];
        mutable f = [Std.Math.Complex(0., 0.), Std.Math.Complex(0., 0.), Std.Math.Complex(0., 0.)];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn default_multidimensional_arrays() -> miette::Result<(), Vec<Report>> {
    let source = "
        array[bool, 2, 3] a;
        array[int, 2, 3] b;
        array[uint, 2, 3] c;
        array[angle, 2, 3] d;
        array[float, 2, 3] e;
        array[complex, 2, 3] f;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        mutable a = [[false, false, false], [false, false, false]];
        mutable b = [[0, 0, 0], [0, 0, 0]];
        mutable c = [[0, 0, 0], [0, 0, 0]];
        mutable d = [[new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }, new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }, new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }], [new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }, new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }, new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }]];
        mutable e = [[0., 0., 0.], [0., 0., 0.]];
        mutable f = [[Std.Math.Complex(0., 0.), Std.Math.Complex(0., 0.), Std.Math.Complex(0., 0.)], [Std.Math.Complex(0., 0.), Std.Math.Complex(0., 0.), Std.Math.Complex(0., 0.)]];
    "#]].assert_eq(&qsharp);
    Ok(())
}

#[test]
fn initialized_simple_arrays() -> miette::Result<(), Vec<Report>> {
    let source = "
        array[bool, 3] a = { false, true, true };
        array[int, 3] b = { -2, 0, 3 };
        array[uint, 3] c = { 1, 2, 3 };
        array[angle, 3] d = { -1.0, 2.0, 4.0 };
        array[float, 3] e = { -2, 0, 3.0 };
        array[complex, 3] f = { 2, 3.0, 5.0 im };
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        mutable a = [false, true, true];
        mutable b = [-2, 0, 3];
        mutable c = [1, 2, 3];
        mutable d = [QasmStd.Angle.DoubleAsAngle(-1., 53), QasmStd.Angle.DoubleAsAngle(2., 53), QasmStd.Angle.DoubleAsAngle(4., 53)];
        mutable e = [Std.Convert.IntAsDouble(-2), Std.Convert.IntAsDouble(0), 3.];
        mutable f = [Std.Math.Complex(Std.Convert.IntAsDouble(2), 0.), Std.Math.Complex(3., 0.), Std.Math.Complex(0., 5.)];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn initialized_multidimensional_arrays() -> miette::Result<(), Vec<Report>> {
    let source = "
        array[bool, 2, 2] a = {{ 0, 1 }, { false, true }};
        array[int, 2, 2] b = {{ -2, 0 }, { 1, 2 }};
        array[uint, 2, 2] c = {{ 0, 1 }, { false, true }};
        array[angle, 2, 2] d = {{ -1.0, 0.0 }, { 1.0, 5.0 }};
        array[float, 2, 2] e = {{ -1, 0.0 }, { 1.0, 5.0 }};
        array[complex, 2, 2] f = {{ -2, 0 }, { 3 im, 1 - 2 im }};
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        mutable a = [[if 0 == 0 {
            false
        } else {
            true
        }, if 1 == 0 {
            false
        } else {
            true
        }], [false, true]];
        mutable b = [[-2, 0], [1, 2]];
        mutable c = [[0, 1], [QasmStd.Convert.BoolAsInt(false), QasmStd.Convert.BoolAsInt(true)]];
        mutable d = [[QasmStd.Angle.DoubleAsAngle(-1., 53), QasmStd.Angle.DoubleAsAngle(0., 53)], [QasmStd.Angle.DoubleAsAngle(1., 53), QasmStd.Angle.DoubleAsAngle(5., 53)]];
        mutable e = [[Std.Convert.IntAsDouble(-1), 0.], [1., 5.]];
        mutable f = [[Std.Math.Complex(Std.Convert.IntAsDouble(-2), 0.), Std.Math.Complex(Std.Convert.IntAsDouble(0), 0.)], [Std.Math.Complex(0., 3.), Std.Math.MinusC(Std.Math.Complex(1., 0.), Std.Math.Complex(0., 2.))]];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn assign_to_simple_arrays() -> miette::Result<(), Vec<Report>> {
    let source = "
        array[bool, 3] a;
        array[int, 3] b;
        array[uint, 3] c;
        array[angle, 3] d;
        array[float, 3] e;
        array[complex, 3] f;

        a[1] = true;
        b[1] = 4;
        c[1] = 4;
        d[1] = 4.0;
        e[1] = 4;
        f[1] = 4;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        mutable a = [false, false, false];
        mutable b = [0, 0, 0];
        mutable c = [0, 0, 0];
        mutable d = [new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }, new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }, new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }];
        mutable e = [0., 0., 0.];
        mutable f = [Std.Math.Complex(0., 0.), Std.Math.Complex(0., 0.), Std.Math.Complex(0., 0.)];
        set a w/= 1 <- true;
        set b w/= 1 <- 4;
        set c w/= 1 <- 4;
        set d w/= 1 <- new QasmStd.Angle.Angle {
            Value = 5734161139222659,
            Size = 53
        };
        set e w/= 1 <- 4.;
        set f w/= 1 <- Std.Math.Complex(4., 0.);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn assign_to_multidimensional_arrays() -> miette::Result<(), Vec<Report>> {
    let source = "
        array[bool, 3, 2] a;
        array[int, 3, 2] b;
        array[uint, 3, 2] c;
        array[angle, 3, 2] d;
        array[float, 3, 2] e;
        array[complex, 3, 2] f;

        a[2, 1] = true;
        b[2, 1] = 4;
        c[2, 1] = 4;
        d[2, 1] = 4.0;
        e[2, 1] = 4;
        f[2, 1] = 4;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        mutable a = [[false, false], [false, false], [false, false]];
        mutable b = [[0, 0], [0, 0], [0, 0]];
        mutable c = [[0, 0], [0, 0], [0, 0]];
        mutable d = [[new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }, new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }], [new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }, new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }], [new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }, new QasmStd.Angle.Angle {
            Value = 0,
            Size = 53
        }]];
        mutable e = [[0., 0.], [0., 0.], [0., 0.]];
        mutable f = [[Std.Math.Complex(0., 0.), Std.Math.Complex(0., 0.)], [Std.Math.Complex(0., 0.), Std.Math.Complex(0., 0.)], [Std.Math.Complex(0., 0.), Std.Math.Complex(0., 0.)]];
        set a w/= 2 <- (a[2] w/ 1 <- true);
        set b w/= 2 <- (b[2] w/ 1 <- 4);
        set c w/= 2 <- (c[2] w/ 1 <- 4);
        set d w/= 2 <- (d[2] w/ 1 <- new QasmStd.Angle.Angle {
            Value = 5734161139222659,
            Size = 53
        });
        set e w/= 2 <- (e[2] w/ 1 <- 4.);
        set f w/= 2 <- (f[2] w/ 1 <- Std.Math.Complex(4., 0.));
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn assign_slice() -> miette::Result<(), Vec<Report>> {
    let source = "
        array[int, 3] a;
        array[int, 2] b = {5, 6};
        a[1:2] = b;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        mutable a = [0, 0, 0];
        mutable b = [5, 6];
        set a w/= 1..2 <- b;
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bitarray_indexing() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit[5] a = "10101";
        const bit b = a[2];

        def f() {
            bit c = b;
        }
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        let a = [One, Zero, One, Zero, One];
        let b = a[2];
        function f() : Unit {
            mutable c = One;
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bitarray_slicing() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        const bit[5] a = "10101";
        const bit[3] b = a[1:3];

        def f() {
            bit[3] c = b;
        }
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        let a = [One, Zero, One, Zero, One];
        let b = a[1..3];
        function f() : Unit {
            mutable c = [Zero, One, Zero];
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bitarray_const_evaluation() -> miette::Result<(), Vec<Report>> {
    let source = "
        const bit[5] a = 10;

        def f() {
            bit b = a[1];
        }
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        let a = [Zero, One, Zero, One, Zero];
        function f() : Unit {
            mutable b = [Zero, One, Zero, One, Zero][1];
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn array_with_size_zero() -> miette::Result<(), Vec<Report>> {
    let source = "
        array[int, 0] a;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        mutable a = [];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn array_with_size_one() -> miette::Result<(), Vec<Report>> {
    let source = "
        array[int, 1] a;
    ";

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Intrinsic.*;
        mutable a = [0];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}
