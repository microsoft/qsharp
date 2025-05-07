// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::check_qasm_to_qsharp as check;
use expect_test::expect;

#[test]
fn int_to_bool() {
    let input = "
        int x;
        bool(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = 0;
            if x == 0 {
                false
            } else {
                true
            };
        "#]],
    );
}

#[test]
fn int_to_int() {
    let input = "
        int x;
        int(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = 0;
            x;
        "#]],
    );
}

#[test]
fn int_to_uint() {
    let input = "
        int x;
        uint(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = 0;
            x;
        "#]],
    );
}

#[test]
fn int_to_float() {
    let input = "
        int x;
        float(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = 0;
            Std.Convert.IntAsDouble(x);
        "#]],
    );
}

#[test]
fn int_to_angle_fails() {
    let input = "
        int x;
        angle(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Int(None, false) to type Angle(None, false)
               ,-[Test.qasm:3:15]
             2 |         int x;
             3 |         angle(x);
               :               ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn int_to_bit() {
    let input = "
        int x;
        bit(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = 0;
            if x == 0 {
                One
            } else {
                Zero
            };
        "#]],
    );
}

#[test]
fn int_to_bitarray() {
    let input = "
        int x;
        bit[8](x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = 0;
            QasmStd.Convert.IntAsResultArrayBE(x, 8);
        "#]],
    );
}

#[test]
fn int_to_duration_fails() {
    let input = "
        int x;
        duration(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Int(None, false) to type Duration(false)
               ,-[Test.qasm:3:18]
             2 |         int x;
             3 |         duration(x);
               :                  ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn int_to_complex_fails() {
    let input = "
        int x;
        complex(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = 0;
            Std.Math.Complex(Std.Convert.IntAsDouble(x), 0.);
        "#]],
    );
}
