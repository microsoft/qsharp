// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::check_qasm_to_qsharp as check;
use expect_test::expect;

#[test]
fn angle_to_bool() {
    let input = "
        angle x;
        bool(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = new QasmStd.Angle.Angle {
                Value = 0,
                Size = 53
            };
            QasmStd.Angle.AngleAsBool(x);
        "#]],
    );
}

#[test]
fn angle_to_int_fails() {
    let input = "
        angle x;
        int(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(None, false) to type Int(None, false)
               ,-[Test.qasm:3:13]
             2 |         angle x;
             3 |         int(x);
               :             ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn angle_to_uint_fails() {
    let input = "
        angle x;
        uint(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(None, false) to type UInt(None,
              | false)
               ,-[Test.qasm:3:14]
             2 |         angle x;
             3 |         uint(x);
               :              ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn angle_to_float_fails() {
    let input = "
        angle x;
        float(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(None, false) to type Float(None,
              | false)
               ,-[Test.qasm:3:15]
             2 |         angle x;
             3 |         float(x);
               :               ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn angle_to_angle() {
    let input = "
        angle x;
        angle(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = new QasmStd.Angle.Angle {
                Value = 0,
                Size = 53
            };
            x;
        "#]],
    );
}

#[test]
fn angle_to_bit() {
    let input = "
        angle x;
        bit(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = new QasmStd.Angle.Angle {
                Value = 0,
                Size = 53
            };
            QasmStd.Angle.AngleAsResult(x);
        "#]],
    );
}

#[test]
fn angle_to_bitarray() {
    let input = "
        angle x;
        bit[8](x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = new QasmStd.Angle.Angle {
                Value = 0,
                Size = 53
            };
            QasmStd.Angle.AngleAsResultArrayBE(x);
        "#]],
    );
}

#[test]
fn angle_to_duration_fails() {
    let input = "
        angle x;
        duration(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(None, false) to type Duration(false)
               ,-[Test.qasm:3:18]
             2 |         angle x;
             3 |         duration(x);
               :                  ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn angle_to_complex_fails() {
    let input = "
        angle x;
        complex(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(None, false) to type Complex(None,
              | false)
               ,-[Test.qasm:3:17]
             2 |         angle x;
             3 |         complex(x);
               :                 ^
             4 |     
               `----
        "#]],
    );
}
