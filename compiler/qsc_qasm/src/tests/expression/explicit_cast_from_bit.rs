// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! These unit tests check six properties for each target type.
//! Let's call the type we are casting from `T` and the type we are casting to `Q`.
//! We want to test that for each type `Q` we correctly:
//!   1. cast from T to Q.
//!   2. cast from T to Q[n].
//!   3. cast from T[n] to Q.
//!   4. cast from T[n] to Q[n].
//!   5. cast from T[n] to Q[m] when n > m; a truncating cast.
//!   6. cast from T[n] to Q[m] when n < m; an expanding cast.

use crate::tests::check_qasm_to_qsharp as check;
use expect_test::expect;

//===============
// Casts to bool
//===============

#[test]
fn bit_to_bool() {
    let source = "
        bit a;
        bool(a);
    ";
    check(
        source,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable a = Zero;
            QasmStd.Convert.ResultAsBool(a);
        "#]],
    );
}

#[test]
fn bitarray_to_bool() {
    let source = "
        bit[32] a;
        bool(a);
    ";
    check(
        source,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable a = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
            QasmStd.Convert.ResultArrayAsBool(a);
        "#]],
    );
}

//===================
// Casts to duration
//===================

#[test]
fn bit_to_duration_fails() {
    let source = "
        bit a;
        duration(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Bit(false) to type Duration(false)
               ,-[Test.qasm:3:18]
             2 |         bit a;
             3 |         duration(a);
               :                  ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_duration_fails() {
    let source = "
        bit[32] a;
        duration(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type Duration(false)
               ,-[Test.qasm:3:18]
             2 |         bit[32] a;
             3 |         duration(a);
               :                  ^
             4 |     
               `----
        "#]],
    );
}

//=========================
// Casts to int and int[n]
//=========================

#[test]
fn bit_to_int() {
    let source = "
        bit a;
        int(a);
    ";
    check(
        source,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable a = Zero;
            QasmStd.Convert.ResultAsInt(a);
        "#]],
    );
}

#[test]
fn bit_to_sized_int() {
    let source = "
        bit a;
        int[32](a);
    ";
    check(
        source,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable a = Zero;
            QasmStd.Convert.ResultAsInt(a);
        "#]],
    );
}

#[test]
#[ignore = "this should fail but we are using this cast for bitarray BinOps (we cast to int first)"]
fn bitarray_to_int_fails() {
    let source = "
        bit[32] a;
        int(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type Int(None,
              | false)
               ,-[Test.qasm:3:13]
             2 |         bit[32] a;
             3 |         int(a);
               :             ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_sized_int() {
    let source = "
        bit[32] a;
        int[32](a);
    ";
    check(
        source,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable a = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
            QasmStd.Convert.ResultArrayAsIntBE(a);
        "#]],
    );
}

#[test]
fn bitarray_to_sized_int_truncating_fails() {
    let source = "
        bit[32] a;
        int[16](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type Int(Some(16),
              | false)
               ,-[Test.qasm:3:17]
             2 |         bit[32] a;
             3 |         int[16](a);
               :                 ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_sized_int_expanding_fails() {
    let source = "
        bit[32] a;
        int[64](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type Int(Some(64),
              | false)
               ,-[Test.qasm:3:17]
             2 |         bit[32] a;
             3 |         int[64](a);
               :                 ^
             4 |     
               `----
        "#]],
    );
}

//===========================
// Casts to uint and uint[n]
//===========================

#[test]
fn bit_to_uint() {
    let source = "
        bit a;
        uint(a);
    ";
    check(
        source,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable a = Zero;
            QasmStd.Convert.ResultAsInt(a);
        "#]],
    );
}

#[test]
fn bit_to_sized_uint() {
    let source = "
        bit a;
        uint[32](a);
    ";
    check(
        source,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable a = Zero;
            QasmStd.Convert.ResultAsInt(a);
        "#]],
    );
}

#[test]
#[ignore = "this should fail but we are using this cast for bitarray bit shifts (we cast to uint first)"]
fn bitarray_to_uint_fails() {
    let source = "
        bit[32] a;
        uint(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type UInt(None,
              | false)
               ,-[Test.qasm:3:14]
             2 |         bit[32] a;
             3 |         uint(a);
               :              ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_sized_uint() {
    let source = "
        bit[32] a;
        uint[32](a);
    ";
    check(
        source,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable a = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
            QasmStd.Convert.ResultArrayAsIntBE(a);
        "#]],
    );
}

#[test]
fn bitarray_to_sized_uint_truncating_fails() {
    let source = "
        bit[32] a;
        uint[16](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type UInt(Some(16),
              | false)
               ,-[Test.qasm:3:18]
             2 |         bit[32] a;
             3 |         uint[16](a);
               :                  ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_sized_uint_expanding_fails() {
    let source = "
        bit[32] a;
        uint[64](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type UInt(Some(64),
              | false)
               ,-[Test.qasm:3:18]
             2 |         bit[32] a;
             3 |         uint[64](a);
               :                  ^
             4 |     
               `----
        "#]],
    );
}

//=============================
// Casts to float and float[n]
//=============================

#[test]
fn bit_to_float() {
    let source = "
        bit a;
        float(a);
    ";
    check(
        source,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable a = Zero;
            QasmStd.Convert.ResultAsDouble(a);
        "#]],
    );
}

#[test]
fn bit_to_sized_float() {
    let source = "
        bit a;
        float[32](a);
    ";
    check(
        source,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable a = Zero;
            QasmStd.Convert.ResultAsDouble(a);
        "#]],
    );
}

#[test]
fn bitarray_to_float_fails() {
    let source = "
        bit[32] a;
        float(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type Float(None,
              | false)
               ,-[Test.qasm:3:15]
             2 |         bit[32] a;
             3 |         float(a);
               :               ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_sized_float_fails() {
    let source = "
        bit[32] a;
        float[32](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type Float(Some(32),
              | false)
               ,-[Test.qasm:3:19]
             2 |         bit[32] a;
             3 |         float[32](a);
               :                   ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_sized_float_truncating_fails() {
    let source = "
        bit[32] a;
        float[16](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type Float(Some(16),
              | false)
               ,-[Test.qasm:3:19]
             2 |         bit[32] a;
             3 |         float[16](a);
               :                   ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_sized_float_expanding_fails() {
    let source = "
        bit[32] a;
        float[64](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type Float(Some(64),
              | false)
               ,-[Test.qasm:3:19]
             2 |         bit[32] a;
             3 |         float[64](a);
               :                   ^
             4 |     
               `----
        "#]],
    );
}

//=============================
// Casts to angle and angle[n]
//=============================

#[test]
fn bit_to_angle_fails() {
    let source = "
        bit a;
        angle(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Bit(false) to type Angle(None, false)
               ,-[Test.qasm:3:15]
             2 |         bit a;
             3 |         angle(a);
               :               ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bit_to_sized_angle_fails() {
    let source = "
        bit a;
        angle[32](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Bit(false) to type Angle(Some(32), false)
               ,-[Test.qasm:3:19]
             2 |         bit a;
             3 |         angle[32](a);
               :                   ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_angle_fails() {
    let source = "
        bit[32] a;
        angle(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type Angle(None,
              | false)
               ,-[Test.qasm:3:15]
             2 |         bit[32] a;
             3 |         angle(a);
               :               ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_sized_angle() {
    let source = "
        bit[32] a;
        angle[32](a);
    ";
    check(
        source,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable a = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
            QasmStd.Convert.ResultArrayAsAngleBE(a);
        "#]],
    );
}

#[test]
fn bitarray_to_sized_angle_truncating_fails() {
    let source = "
        bit[32] a;
        angle[16](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type Angle(Some(16),
              | false)
               ,-[Test.qasm:3:19]
             2 |         bit[32] a;
             3 |         angle[16](a);
               :                   ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_sized_angle_expanding_fails() {
    let source = "
        bit[32] a;
        angle[64](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type Angle(Some(64),
              | false)
               ,-[Test.qasm:3:19]
             2 |         bit[32] a;
             3 |         angle[64](a);
               :                   ^
             4 |     
               `----
        "#]],
    );
}

//=================================
// Casts to complex and complex[n]
//=================================

#[test]
fn bit_to_complex_fails() {
    let source = "
        bit a;
        complex(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Bit(false) to type Complex(None, false)
               ,-[Test.qasm:3:17]
             2 |         bit a;
             3 |         complex(a);
               :                 ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bit_to_sized_complex_fails() {
    let source = "
        bit a;
        complex[float[32]](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Bit(false) to type Complex(Some(32), false)
               ,-[Test.qasm:3:28]
             2 |         bit a;
             3 |         complex[float[32]](a);
               :                            ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_complex_fails() {
    let source = "
        bit[32] a;
        complex(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type Complex(None,
              | false)
               ,-[Test.qasm:3:17]
             2 |         bit[32] a;
             3 |         complex(a);
               :                 ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_sized_complex_fails() {
    let source = "
        bit[32] a;
        complex[float[32]](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type
              | Complex(Some(32), false)
               ,-[Test.qasm:3:28]
             2 |         bit[32] a;
             3 |         complex[float[32]](a);
               :                            ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_sized_complex_truncating_fails() {
    let source = "
        bit[32] a;
        complex[float[16]](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type
              | Complex(Some(16), false)
               ,-[Test.qasm:3:28]
             2 |         bit[32] a;
             3 |         complex[float[16]](a);
               :                            ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_sized_complex_expanding_fails() {
    let source = "
        bit[32] a;
        complex[float[64]](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type
              | Complex(Some(64), false)
               ,-[Test.qasm:3:28]
             2 |         bit[32] a;
             3 |         complex[float[64]](a);
               :                            ^
             4 |     
               `----
        "#]],
    );
}

//=================================
// Casts to bit and bit[n]
//=================================

#[test]
fn bit_to_bit() {
    let source = "
        bit a;
        bit(a);
    ";
    check(
        source,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable a = Zero;
            a;
        "#]],
    );
}

#[test]
fn bit_to_bitarray() {
    let source = "
        bit a;
        bit[32](a);
    ";
    check(
        source,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable a = Zero;
            QasmStd.Convert.ResultAsResultArrayBE(a, 32);
        "#]],
    );
}

#[test]
fn bitarray_to_bit() {
    let source = "
        bit[32] a;
        bit(a);
    ";
    check(
        source,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable a = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
            QasmStd.Convert.ResultArrayAsResultBE(a);
        "#]],
    );
}

#[test]
fn bitarray_to_bitarray() {
    let source = "
        bit[32] a;
        bit[32](a);
    ";
    check(
        source,
        &expect![[r#"
            import QasmStd.Intrinsic.*;
            mutable a = [Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero, Zero];
            a;
        "#]],
    );
}

#[test]
fn bitarray_to_bitarray_truncating_fails() {
    let source = "
        bit[32] a;
        bit[16](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type BitArray(16,
              | false)
               ,-[Test.qasm:3:17]
             2 |         bit[32] a;
             3 |         bit[16](a);
               :                 ^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn bitarray_to_bitarray_expanding_fails() {
    let source = "
        bit[32] a;
        bit[64](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(32, false) to type BitArray(64,
              | false)
               ,-[Test.qasm:3:17]
             2 |         bit[32] a;
             3 |         bit[64](a);
               :                 ^
             4 |     
               `----
        "#]],
    );
}
