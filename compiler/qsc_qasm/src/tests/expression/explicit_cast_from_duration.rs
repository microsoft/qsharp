// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::check_qasm_to_qsharp as check;
use expect_test::expect;

#[test]
fn duration_to_bool_fails() {
    let input = "
        duration x;
        bool(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.NotSupported

              x duration type values are not supported
               ,-[Test.qasm:2:9]
             1 | 
             2 |         duration x;
               :         ^^^^^^^^
             3 |         bool(x);
               `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Duration(false) to type Bool(false)
               ,-[Test.qasm:3:14]
             2 |         duration x;
             3 |         bool(x);
               :              ^
             4 |     
               `----

            Qasm.Compiler.NotSupported

              x timing literals are not supported
               ,-[Test.qasm:1:1]
             1 | 
               : ^
             2 |         duration x;
             3 |         bool(x);
               `----
        "#]],
    );
}

#[test]
fn duration_to_int_fails() {
    let input = "
        duration x;
        int(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.NotSupported

              x duration type values are not supported
               ,-[Test.qasm:2:9]
             1 | 
             2 |         duration x;
               :         ^^^^^^^^
             3 |         int(x);
               `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Duration(false) to type Int(None, false)
               ,-[Test.qasm:3:13]
             2 |         duration x;
             3 |         int(x);
               :             ^
             4 |     
               `----

            Qasm.Compiler.NotSupported

              x timing literals are not supported
               ,-[Test.qasm:1:1]
             1 | 
               : ^
             2 |         duration x;
             3 |         int(x);
               `----
        "#]],
    );
}

#[test]
fn duration_to_uint_fails() {
    let input = "
        duration x;
        uint(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.NotSupported

              x duration type values are not supported
               ,-[Test.qasm:2:9]
             1 | 
             2 |         duration x;
               :         ^^^^^^^^
             3 |         uint(x);
               `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Duration(false) to type UInt(None, false)
               ,-[Test.qasm:3:14]
             2 |         duration x;
             3 |         uint(x);
               :              ^
             4 |     
               `----

            Qasm.Compiler.NotSupported

              x timing literals are not supported
               ,-[Test.qasm:1:1]
             1 | 
               : ^
             2 |         duration x;
             3 |         uint(x);
               `----
        "#]],
    );
}

#[test]
fn duration_to_float_fails() {
    let input = "
        duration x;
        float(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.NotSupported

              x duration type values are not supported
               ,-[Test.qasm:2:9]
             1 | 
             2 |         duration x;
               :         ^^^^^^^^
             3 |         float(x);
               `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Duration(false) to type Float(None, false)
               ,-[Test.qasm:3:15]
             2 |         duration x;
             3 |         float(x);
               :               ^
             4 |     
               `----

            Qasm.Compiler.NotSupported

              x timing literals are not supported
               ,-[Test.qasm:1:1]
             1 | 
               : ^
             2 |         duration x;
             3 |         float(x);
               `----
        "#]],
    );
}

#[test]
fn duration_to_angle_fails() {
    let input = "
        duration x;
        angle(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.NotSupported

              x duration type values are not supported
               ,-[Test.qasm:2:9]
             1 | 
             2 |         duration x;
               :         ^^^^^^^^
             3 |         angle(x);
               `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Duration(false) to type Angle(None, false)
               ,-[Test.qasm:3:15]
             2 |         duration x;
             3 |         angle(x);
               :               ^
             4 |     
               `----

            Qasm.Compiler.NotSupported

              x timing literals are not supported
               ,-[Test.qasm:1:1]
             1 | 
               : ^
             2 |         duration x;
             3 |         angle(x);
               `----
        "#]],
    );
}

#[test]
fn duration_to_bit_fails() {
    let input = "
        duration x;
        bit(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.NotSupported

              x duration type values are not supported
               ,-[Test.qasm:2:9]
             1 | 
             2 |         duration x;
               :         ^^^^^^^^
             3 |         bit(x);
               `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Duration(false) to type Bit(false)
               ,-[Test.qasm:3:13]
             2 |         duration x;
             3 |         bit(x);
               :             ^
             4 |     
               `----

            Qasm.Compiler.NotSupported

              x timing literals are not supported
               ,-[Test.qasm:1:1]
             1 | 
               : ^
             2 |         duration x;
             3 |         bit(x);
               `----
        "#]],
    );
}

#[test]
fn duration_to_bitarray_fails() {
    let input = "
        duration x;
        bit[8](x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.NotSupported

              x duration type values are not supported
               ,-[Test.qasm:2:9]
             1 | 
             2 |         duration x;
               :         ^^^^^^^^
             3 |         bit[8](x);
               `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Duration(false) to type BitArray(8, false)
               ,-[Test.qasm:3:16]
             2 |         duration x;
             3 |         bit[8](x);
               :                ^
             4 |     
               `----

            Qasm.Compiler.NotSupported

              x timing literals are not supported
               ,-[Test.qasm:1:1]
             1 | 
               : ^
             2 |         duration x;
             3 |         bit[8](x);
               `----
        "#]],
    );
}

#[test]
fn duration_to_duration() {
    let input = "
        duration x;
        duration(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.NotSupported

              x duration type values are not supported
               ,-[Test.qasm:2:9]
             1 | 
             2 |         duration x;
               :         ^^^^^^^^
             3 |         duration(x);
               `----

            Qasm.Compiler.NotSupported

              x timing literals are not supported
               ,-[Test.qasm:1:1]
             1 | 
               : ^
             2 |         duration x;
             3 |         duration(x);
               `----
        "#]],
    );
}

#[test]
fn duration_to_complex_fails() {
    let input = "
        duration x;
        complex(x);
    ";
    check(
        input,
        &expect![[r#"
            Qasm.Lowerer.NotSupported

              x duration type values are not supported
               ,-[Test.qasm:2:9]
             1 | 
             2 |         duration x;
               :         ^^^^^^^^
             3 |         complex(x);
               `----

            Qasm.Lowerer.CannotCast

              x cannot cast expression of type Duration(false) to type Complex(None,
              | false)
               ,-[Test.qasm:3:17]
             2 |         duration x;
             3 |         complex(x);
               :                 ^
             4 |     
               `----

            Qasm.Compiler.NotSupported

              x timing literals are not supported
               ,-[Test.qasm:1:1]
             1 | 
               : ^
             2 |         duration x;
             3 |         complex(x);
               `----
        "#]],
    );
}
