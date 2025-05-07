// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::check_qasm_to_qsharp as check;
use expect_test::expect;

#[test]
fn bitarray_to_bool() {
    let input = "
        bit[8] x;
        bool(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
            QasmStd.Convert.ResultArrayAsBoolBE(x);
        "#]],
    );
}

#[test]
fn bitarray_to_int() {
    let input = "
        bit[8] x;
        int(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
            QasmStd.Convert.ResultArrayAsIntBE(x);
        "#]],
    );
}

#[test]
fn bitarray_to_uint() {
    let input = "
        bit[8] x;
        uint(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
            QasmStd.Convert.ResultArrayAsIntBE(x);
        "#]],
    );
}

#[test]
fn bitarray_to_float_fails() {
    let input = "
        bit[8] x;
        float[8](x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(8, false) to type Float(Some(8),
              | false)
               ,-[Test.qasm:3:18]
             2 |         bit[8] x;
             3 |         float[8](x);
               :                  ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_angle() {
    let input = "
        bit[8] x;
        angle[8](x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
            QasmStd.Convert.ResultArrayAsAngleBE(x, 8);
        "#]],
    );
}

#[test]
fn bitarray_to_bit() {
    let input = "
        bit[8] x;
        bit(x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
            QasmStd.Convert.ResultArrayAsResultBE(x);
        "#]],
    );
}

#[test]
fn bitarray_to_bitarray() {
    let input = "
        bit[8] x;
        bit[8](x);
    ";
    check(
        input,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable x = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
            x;
        "#]],
    );
}

#[test]
fn bitarray_to_duration_fails() {
    let input = "
        bit[8] x;
        duration(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(8, false) to type Duration(false)
               ,-[Test.qasm:3:18]
             2 |         bit[8] x;
             3 |         duration(x);
               :                  ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_complex_fails() {
    let input = "
        bit[8] x;
        complex(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(8, false) to type Complex(None,
              | false)
               ,-[Test.qasm:3:17]
             2 |         bit[8] x;
             3 |         complex(x);
               :                 ^
             4 |     
               `----
        "#]],
    );
}
