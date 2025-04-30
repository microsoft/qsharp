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
        array[complex[float[32]], 4] g;
        array[bool, 3] h;
        array[int[8], 4] i = {1, 2, 3, 4};
        array[int[8], 2] k = {y, y+y};
        array[uint[32], 2, 2] l = {{3, 4}, {2-3, y*5}};
        array[uint[32], 2, 2] m = {z, {2-3, y*5}};
        array[uint[32], 2, 2] n = {z*2, {1, 2}};
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
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        mutable a = [false, false, false];
        mutable b = [0, 0, 0];
        mutable c = [0, 0, 0];
        mutable d = [new __Angle__ {
            Value = 0,
            Size = 53
        }, new __Angle__ {
            Value = 0,
            Size = 53
        }, new __Angle__ {
            Value = 0,
            Size = 53
        }];
        mutable e = [0., 0., 0.];
        mutable f = [Microsoft.Quantum.Math.Complex(0., 0.), Microsoft.Quantum.Math.Complex(0., 0.), Microsoft.Quantum.Math.Complex(0., 0.)];
    "#]].assert_eq(&qsharp);
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
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        mutable a = [[false, false, false], [false, false, false]];
        mutable b = [[0, 0, 0], [0, 0, 0]];
        mutable c = [[0, 0, 0], [0, 0, 0]];
        mutable d = [[new __Angle__ {
            Value = 0,
            Size = 53
        }, new __Angle__ {
            Value = 0,
            Size = 53
        }, new __Angle__ {
            Value = 0,
            Size = 53
        }], [new __Angle__ {
            Value = 0,
            Size = 53
        }, new __Angle__ {
            Value = 0,
            Size = 53
        }, new __Angle__ {
            Value = 0,
            Size = 53
        }]];
        mutable e = [[0., 0., 0.], [0., 0., 0.]];
        mutable f = [[Microsoft.Quantum.Math.Complex(0., 0.), Microsoft.Quantum.Math.Complex(0., 0.), Microsoft.Quantum.Math.Complex(0., 0.)], [Microsoft.Quantum.Math.Complex(0., 0.), Microsoft.Quantum.Math.Complex(0., 0.), Microsoft.Quantum.Math.Complex(0., 0.)]];
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
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        mutable a = [false, true, true];
        mutable b = [-2, 0, 3];
        mutable c = [1, 2, 3];
        mutable d = [__DoubleAsAngle__(-1., 53), __DoubleAsAngle__(2., 53), __DoubleAsAngle__(4., 53)];
        mutable e = [Microsoft.Quantum.Convert.IntAsDouble(-2), Microsoft.Quantum.Convert.IntAsDouble(0), 3.];
        mutable f = [Microsoft.Quantum.Math.Complex(Microsoft.Quantum.Convert.IntAsDouble(2), 0.), Microsoft.Quantum.Math.Complex(3., 0.), Microsoft.Quantum.Math.Complex(0., 5.)];
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
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
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
        mutable c = [[0, 1], [__BoolAsInt__(false), __BoolAsInt__(true)]];
        mutable d = [[__DoubleAsAngle__(-1., 53), __DoubleAsAngle__(0., 53)], [__DoubleAsAngle__(1., 53), __DoubleAsAngle__(5., 53)]];
        mutable e = [[Microsoft.Quantum.Convert.IntAsDouble(-1), 0.], [1., 5.]];
        mutable f = [[Microsoft.Quantum.Math.Complex(Microsoft.Quantum.Convert.IntAsDouble(-2), 0.), Microsoft.Quantum.Math.Complex(Microsoft.Quantum.Convert.IntAsDouble(0), 0.)], [Microsoft.Quantum.Math.Complex(0., 3.), Microsoft.Quantum.Math.MinusC(Microsoft.Quantum.Math.Complex(1., 0.), Microsoft.Quantum.Math.Complex(0., 2.))]];
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}
