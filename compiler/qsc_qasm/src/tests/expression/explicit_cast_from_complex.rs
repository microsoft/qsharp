// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::check_qasm_to_qsharp as check;
use expect_test::expect;

#[test]
fn complex_to_bool_fails() {
    let input = "
        complex x;
        bool(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Complex(None, false) to type Bool(false)
               ,-[Test.qasm:3:14]
             2 |         complex x;
             3 |         bool(x);
               :              ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn complex_to_int_fails() {
    let input = "
        complex x;
        int(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Complex(None, false) to type Int(None,
              | false)
               ,-[Test.qasm:3:13]
             2 |         complex x;
             3 |         int(x);
               :             ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn complex_to_uint_fails() {
    let input = "
        complex x;
        uint(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Complex(None, false) to type UInt(None,
              | false)
               ,-[Test.qasm:3:14]
             2 |         complex x;
             3 |         uint(x);
               :              ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn complex_to_float_fails() {
    let input = "
        complex x;
        float(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Complex(None, false) to type Float(None,
              | false)
               ,-[Test.qasm:3:15]
             2 |         complex x;
             3 |         float(x);
               :               ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn complex_to_angle_fails() {
    let input = "
        complex x;
        angle(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Complex(None, false) to type Angle(None,
              | false)
               ,-[Test.qasm:3:15]
             2 |         complex x;
             3 |         angle(x);
               :               ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn complex_to_bit_fails() {
    let input = "
        complex x;
        bit(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Complex(None, false) to type Bit(false)
               ,-[Test.qasm:3:13]
             2 |         complex x;
             3 |         bit(x);
               :             ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn complex_to_bitarray_fails() {
    let input = "
        complex x;
        bit[8](x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Complex(None, false) to type BitArray(8,
              | false)
               ,-[Test.qasm:3:16]
             2 |         complex x;
             3 |         bit[8](x);
               :                ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn complex_to_duration_fails() {
    let input = "
        complex x;
        duration(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Complex(None, false) to type
              | Duration(false)
               ,-[Test.qasm:3:18]
             2 |         complex x;
             3 |         duration(x);
               :                  ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn complex_to_complex() {
    let input = "
        complex x;
        complex(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = Std.Math.Complex(0., 0.);
            x;
        "#]],
    );
}
