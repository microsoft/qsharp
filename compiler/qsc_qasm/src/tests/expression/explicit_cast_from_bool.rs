// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::check_qasm_to_qsharp as check;
use expect_test::expect;

#[test]
fn bool_to_bool() {
    let input = "
        bool x;
        bool(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = false;
            x;
        "#]],
    );
}

#[test]
fn bool_to_int() {
    let input = "
        bool x;
        int(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = false;
            QasmStd.Convert.BoolAsInt(x);
        "#]],
    );
}

#[test]
fn bool_to_uint() {
    let input = "
        bool x;
        uint(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = false;
            QasmStd.Convert.BoolAsInt(x);
        "#]],
    );
}

#[test]
fn bool_to_float() {
    let input = "
        bool x;
        float(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = false;
            QasmStd.Convert.BoolAsDouble(x);
        "#]],
    );
}

#[test]
fn bool_to_angle_fails() {
    let input = "
        bool x;
        angle(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Bool(false) to type Angle(None, false)
               ,-[Test.qasm:3:15]
             2 |         bool x;
             3 |         angle(x);
               :               ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bool_to_bit() {
    let input = "
        bool x;
        bit(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = false;
            QasmStd.Convert.BoolAsResult(x);
        "#]],
    );
}

#[test]
fn bool_to_bitarray() {
    let input = "
        bool x;
        bit[8](x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = false;
            QasmStd.Convert.BoolAsResultArrayBE(x, 8);
        "#]],
    );
}

#[test]
fn bool_to_duration_fails() {
    let input = "
        bool x;
        duration(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Bool(false) to type Duration(false)
               ,-[Test.qasm:3:18]
             2 |         bool x;
             3 |         duration(x);
               :                  ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bool_to_complex_fails() {
    let input = "
        bool x;
        complex(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Bool(false) to type Complex(None, false)
               ,-[Test.qasm:3:17]
             2 |         bool x;
             3 |         complex(x);
               :                 ^
             4 |     
               `----
        "#]],
    );
}
