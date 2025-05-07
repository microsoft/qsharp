// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::check_qasm_to_qsharp as check;
use expect_test::expect;

#[test]
fn float_to_bool() {
    let input = "
        float x;
        bool(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = 0.;
            if Std.Math.Truncate(x) == 0 {
                false
            } else {
                true
            };
        "#]],
    );
}

#[test]
fn float_to_int() {
    let input = "
        float x;
        int(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = 0.;
            Std.Math.Truncate(x);
        "#]],
    );
}

#[test]
fn float_to_uint() {
    let input = "
        float x;
        uint(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = 0.;
            Std.Math.Truncate(x);
        "#]],
    );
}

#[test]
fn float_to_float() {
    let input = "
        float x;
        float(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = 0.;
            x;
        "#]],
    );
}

#[test]
fn float_to_angle() {
    let input = "
        float x;
        angle(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = 0.;
            QasmStd.Angle.DoubleAsAngle(x, 53);
        "#]],
    );
}

#[test]
fn float_to_bit() {
    let input = "
        float x;
        bit(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = 0.;
            QasmStd.Convert.DoubleAsResult(x);
        "#]],
    );
}

#[test]
fn float_to_bitarray_fails() {
    let input = "
        float x;
        bit[8](x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Float(None, false) to type BitArray(8,
              | false)
               ,-[Test.qasm:3:16]
             2 |         float x;
             3 |         bit[8](x);
               :                ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn float_to_duration_fails() {
    let input = "
        float x;
        duration(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Float(None, false) to type Duration(false)
               ,-[Test.qasm:3:18]
             2 |         float x;
             3 |         duration(x);
               :                  ^
             4 |     
               `----
        "#]],
    );
}

/// Even though the spec doesn't say it, we need to allow
/// casting from float to complex, else this kind of expression
/// would be invalid: 2.0 + sin(pi) + 1.0i.
#[test]
fn float_to_complex() {
    let input = "
        float x;
        complex(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = 0.;
            Std.Math.Complex(x, 0.);
        "#]],
    );
}
