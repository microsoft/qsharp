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
fn angle_to_bool() {
    let source = "
        angle a;
        bool(a);
    ";
    check(
        source,
        &expect![[r#"
            import Std.OpenQASM.Intrinsic.*;
            mutable a = new Std.OpenQASM.Angle.Angle {
                Value = 0,
                Size = 53
            };
            Std.OpenQASM.Angle.AngleAsBool(a);
        "#]],
    );
}

#[test]
fn sized_angle_to_bool() {
    let source = "
        angle[32] a;
        bool(a);
    ";
    check(
        source,
        &expect![[r#"
            import Std.OpenQASM.Intrinsic.*;
            mutable a = new Std.OpenQASM.Angle.Angle {
                Value = 0,
                Size = 53
            };
            Std.OpenQASM.Angle.AngleAsBool(a);
        "#]],
    );
}

//===================
// Casts to duration
//===================

#[test]
fn angle_to_duration_fails() {
    let source = "
        angle a;
        duration(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(None, false) to type Duration(false)
               ,-[Test.qasm:3:9]
             2 |         angle a;
             3 |         duration(a);
               :         ^^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_duration_fails() {
    let source = "
        angle[32] a;
        duration(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type
              | Duration(false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         duration(a);
               :         ^^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

//=========================
// Casts to int and int[n]
//=========================

#[test]
fn angle_to_int_fails() {
    let source = "
        angle a;
        int(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(None, false) to type Int(None, false)
               ,-[Test.qasm:3:9]
             2 |         angle a;
             3 |         int(a);
               :         ^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn angle_to_sized_int_fails() {
    let source = "
        angle a;
        int[32](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(None, false) to type Int(Some(32),
              | false)
               ,-[Test.qasm:3:9]
             2 |         angle a;
             3 |         int[32](a);
               :         ^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_int_fails() {
    let source = "
        angle[32] a;
        int(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type Int(None,
              | false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         int(a);
               :         ^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_sized_int_fails() {
    let source = "
        angle[32] a;
        int[32](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type
              | Int(Some(32), false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         int[32](a);
               :         ^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_sized_int_truncating_fails() {
    let source = "
        angle[32] a;
        int[16](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type
              | Int(Some(16), false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         int[16](a);
               :         ^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_sized_int_expanding_fails() {
    let source = "
        angle[32] a;
        int[64](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type
              | Int(Some(64), false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         int[64](a);
               :         ^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

//===========================
// Casts to uint and uint[n]
//===========================

#[test]
fn angle_to_uint_fails() {
    let source = "
        angle a;
        uint(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(None, false) to type UInt(None,
              | false)
               ,-[Test.qasm:3:9]
             2 |         angle a;
             3 |         uint(a);
               :         ^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn angle_to_sized_uint_fails() {
    let source = "
        angle a;
        uint[32](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(None, false) to type UInt(Some(32),
              | false)
               ,-[Test.qasm:3:9]
             2 |         angle a;
             3 |         uint[32](a);
               :         ^^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_uint_fails() {
    let source = "
        angle[32] a;
        uint(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type UInt(None,
              | false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         uint(a);
               :         ^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_sized_uint_fails() {
    let source = "
        angle[32] a;
        uint[32](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type
              | UInt(Some(32), false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         uint[32](a);
               :         ^^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_sized_uint_truncating_fails() {
    let source = "
        angle[32] a;
        uint[16](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type
              | UInt(Some(16), false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         uint[16](a);
               :         ^^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_sized_uint_expanding_fails() {
    let source = "
        angle[32] a;
        uint[64](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type
              | UInt(Some(64), false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         uint[64](a);
               :         ^^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

//=============================
// Casts to float and float[n]
//=============================

#[test]
fn angle_to_float_fails() {
    let source = "
        angle a;
        float(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(None, false) to type Float(None,
              | false)
               ,-[Test.qasm:3:9]
             2 |         angle a;
             3 |         float(a);
               :         ^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn angle_to_sized_float_fails() {
    let source = "
        angle a;
        float[32](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(None, false) to type Float(Some(32),
              | false)
               ,-[Test.qasm:3:9]
             2 |         angle a;
             3 |         float[32](a);
               :         ^^^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_float_fails() {
    let source = "
        angle[32] a;
        float(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type Float(None,
              | false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         float(a);
               :         ^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_sized_float_fails() {
    let source = "
        angle[32] a;
        float[32](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type
              | Float(Some(32), false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         float[32](a);
               :         ^^^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_sized_float_truncating_fails() {
    let source = "
        angle[32] a;
        float[16](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type
              | Float(Some(16), false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         float[16](a);
               :         ^^^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_sized_float_expanding_fails() {
    let source = "
        angle[32] a;
        float[64](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type
              | Float(Some(64), false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         float[64](a);
               :         ^^^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

//=============================
// Casts to angle and angle[n]
//=============================

#[test]
fn angle_to_angle() {
    let source = "
        angle a;
        angle(a);
    ";
    check(
        source,
        &expect![[r#"
            import Std.OpenQASM.Intrinsic.*;
            mutable a = new Std.OpenQASM.Angle.Angle {
                Value = 0,
                Size = 53
            };
            a;
        "#]],
    );
}

#[test]
fn angle_to_sized_angle() {
    let source = "
        angle a;
        angle[32](a);
    ";
    check(
        source,
        &expect![[r#"
            import Std.OpenQASM.Intrinsic.*;
            mutable a = new Std.OpenQASM.Angle.Angle {
                Value = 0,
                Size = 53
            };
            a;
        "#]],
    );
}

#[test]
fn sized_angle_to_angle() {
    let source = "
        angle[32] a;
        angle(a);
    ";
    check(
        source,
        &expect![[r#"
            import Std.OpenQASM.Intrinsic.*;
            mutable a = new Std.OpenQASM.Angle.Angle {
                Value = 0,
                Size = 53
            };
            a;
        "#]],
    );
}

#[test]
fn sized_angle_to_sized_angle() {
    let source = "
        angle[32] a;
        angle[32](a);
    ";
    check(
        source,
        &expect![[r#"
            import Std.OpenQASM.Intrinsic.*;
            mutable a = new Std.OpenQASM.Angle.Angle {
                Value = 0,
                Size = 53
            };
            a;
        "#]],
    );
}

#[test]
fn sized_angle_to_sized_angle_truncating() {
    let source = "
        angle[32] a;
        angle[16](a);
    ";
    check(
        source,
        &expect![[r#"
            import Std.OpenQASM.Intrinsic.*;
            mutable a = new Std.OpenQASM.Angle.Angle {
                Value = 0,
                Size = 53
            };
            a;
        "#]],
    );
}

#[test]
fn sized_angle_to_sized_angle_expanding() {
    let source = "
        angle[32] a;
        angle[64](a);
    ";
    check(
        source,
        &expect![[r#"
            import Std.OpenQASM.Intrinsic.*;
            mutable a = new Std.OpenQASM.Angle.Angle {
                Value = 0,
                Size = 53
            };
            Std.OpenQASM.Angle.AdjustAngleSizeNoTruncation(a, 64);
        "#]],
    );
}

//=================================
// Casts to complex and complex[n]
//=================================

#[test]
fn angle_to_complex_fails() {
    let source = "
        angle a;
        complex(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(None, false) to type Complex(None,
              | false)
               ,-[Test.qasm:3:9]
             2 |         angle a;
             3 |         complex(a);
               :         ^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn angle_to_sized_complex_fails() {
    let source = "
        angle a;
        complex[float[32]](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(None, false) to type
              | Complex(Some(32), false)
               ,-[Test.qasm:3:9]
             2 |         angle a;
             3 |         complex[float[32]](a);
               :         ^^^^^^^^^^^^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_complex_fails() {
    let source = "
        angle[32] a;
        complex(a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type
              | Complex(None, false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         complex(a);
               :         ^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_sized_complex_fails() {
    let source = "
        angle[32] a;
        complex[float[32]](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type
              | Complex(Some(32), false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         complex[float[32]](a);
               :         ^^^^^^^^^^^^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_sized_complex_truncating_fails() {
    let source = "
        angle[32] a;
        complex[float[16]](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type
              | Complex(Some(16), false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         complex[float[16]](a);
               :         ^^^^^^^^^^^^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_sized_complex_expanding_fails() {
    let source = "
        angle[32] a;
        complex[float[64]](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type
              | Complex(Some(64), false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         complex[float[64]](a);
               :         ^^^^^^^^^^^^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

//=================================
// Casts to bit and bit[n]
//=================================

#[test]
fn angle_to_bit() {
    let source = "
        angle a;
        bit(a);
    ";
    check(
        source,
        &expect![[r#"
            import Std.OpenQASM.Intrinsic.*;
            mutable a = new Std.OpenQASM.Angle.Angle {
                Value = 0,
                Size = 53
            };
            Std.OpenQASM.Angle.AngleAsResult(a);
        "#]],
    );
}

#[test]
fn angle_to_bitarray_fails() {
    let source = "
        angle a;
        bit[32](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(None, false) to type BitArray(32,
              | false)
               ,-[Test.qasm:3:9]
             2 |         angle a;
             3 |         bit[32](a);
               :         ^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_bit() {
    let source = "
        angle[32] a;
        bit(a);
    ";
    check(
        source,
        &expect![[r#"
            import Std.OpenQASM.Intrinsic.*;
            mutable a = new Std.OpenQASM.Angle.Angle {
                Value = 0,
                Size = 53
            };
            Std.OpenQASM.Angle.AngleAsResult(a);
        "#]],
    );
}

#[test]
fn sized_angle_to_bitarray() {
    let source = "
        angle[32] a;
        bit[32](a);
    ";
    check(
        source,
        &expect![[r#"
            import Std.OpenQASM.Intrinsic.*;
            mutable a = new Std.OpenQASM.Angle.Angle {
                Value = 0,
                Size = 53
            };
            Std.OpenQASM.Angle.AngleAsResultArrayBE(a);
        "#]],
    );
}

#[test]
fn sized_angle_to_bitarray_truncating_fails() {
    let source = "
        angle[32] a;
        bit[16](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type BitArray(16,
              | false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         bit[16](a);
               :         ^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}

#[test]
fn sized_angle_to_bitarray_expanding_fails() {
    let source = "
        angle[32] a;
        bit[64](a);
    ";
    check(
        source,
        &expect![[r#"
            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type BitArray(64,
              | false)
               ,-[Test.qasm:3:9]
             2 |         angle[32] a;
             3 |         bit[64](a);
               :         ^^^^^^^^^^
             4 |     
               `----
        "#]],
    );
}
