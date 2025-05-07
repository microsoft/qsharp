// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::check_qasm_to_qsharp as check;
use expect_test::expect;

#[test]
fn bit_to_bool() {
    let input = "
        bit x;
        bool(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = Zero;
            QasmStd.Convert.ResultAsBool(x);
        "#]],
    );
}

#[test]
fn bit_to_int() {
    let input = "
        bit x;
        int(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = Zero;
            QasmStd.Convert.ResultAsInt(x);
        "#]],
    );
}

#[test]
fn bit_to_uint() {
    let input = "
        bit x;
        uint(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = Zero;
            QasmStd.Convert.ResultAsInt(x);
        "#]],
    );
}

#[test]
fn bit_to_float() {
    let input = "
        bit x;
        float(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = Zero;
            QasmStd.Convert.ResultAsDouble(x);
        "#]],
    );
}

#[test]
fn bit_to_angle_fails() {
    let input = "
        bit x;
        angle(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Bit(false) to type Angle(None, false)
               ,-[Test.qasm:3:15]
             2 |         bit x;
             3 |         angle(x);
               :               ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bit_to_bit() {
    let input = "
        bit x;
        bit(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = Zero;
            x;
        "#]],
    );
}

#[test]
fn bit_to_bitarray() {
    let input = "
        bit x;
        bit[8](x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = Zero;
            QasmStd.Convert.ResultAsResultArrayBE(x, 8);
        "#]],
    );
}

#[test]
fn bit_to_duration_fails() {
    let input = "
        bit x;
        duration(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Bit(false) to type Duration(false)
               ,-[Test.qasm:3:18]
             2 |         bit x;
             3 |         duration(x);
               :                  ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bit_to_complex_fails() {
    let input = "
        bit x;
        complex(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Bit(false) to type Complex(None, false)
               ,-[Test.qasm:3:17]
             2 |         bit x;
             3 |         complex(x);
               :                 ^
             4 |     
               `----
        "#]],
    );
}
