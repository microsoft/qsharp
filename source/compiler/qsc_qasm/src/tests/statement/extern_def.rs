// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::{compile_qasm_to_qir, compile_qasm_to_qsharp};
use expect_test::expect;
use miette::Report;

// some types aren't supported for code gen
// angle
// duration
// bigint
// complex

#[test]
fn void_ty_is_mapped() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        extern fn();
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import Std.OpenQASM.Intrinsic.*;
        @SimulatableIntrinsic()
        operation fn() : Unit {
            fail "Extern `fn` cannot be used without a linked implementation.";
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bit_ty_is_mapped() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        extern fn(bit) -> bit;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import Std.OpenQASM.Intrinsic.*;
        @SimulatableIntrinsic()
        operation fn(param_0 : Result) : Result {
            fail "Extern `fn` cannot be used without a linked implementation.";
            Zero
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn bool_ty_is_mapped() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        extern fn(bool) -> bool;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import Std.OpenQASM.Intrinsic.*;
        @SimulatableIntrinsic()
        operation fn(param_0 : Bool) : Bool {
            fail "Extern `fn` cannot be used without a linked implementation.";
            false
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn complex_ty_is_mapped() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        extern fn(complex, complex[float[32]]) -> complex;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import Std.OpenQASM.Intrinsic.*;
        @SimulatableIntrinsic()
        operation fn(param_0 : Std.Math.Complex, param_1 : Std.Math.Complex) : Std.Math.Complex {
            fail "Extern `fn` cannot be used without a linked implementation.";
            Std.Math.Complex(0., 0.)
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn float_ty_is_mapped() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        extern fn(float, float[17]) -> float;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import Std.OpenQASM.Intrinsic.*;
        @SimulatableIntrinsic()
        operation fn(param_0 : Double, param_1 : Double) : Double {
            fail "Extern `fn` cannot be used without a linked implementation.";
            0.
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn int_ty_is_mapped() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        extern fn(int, int[17]) -> int;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import Std.OpenQASM.Intrinsic.*;
        @SimulatableIntrinsic()
        operation fn(param_0 : Int, param_1 : Int) : Int {
            fail "Extern `fn` cannot be used without a linked implementation.";
            0
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn uint_ty_is_mapped() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        extern fn(uint, uint[17]) -> uint;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import Std.OpenQASM.Intrinsic.*;
        @SimulatableIntrinsic()
        operation fn(param_0 : Int, param_1 : Int) : Int {
            fail "Extern `fn` cannot be used without a linked implementation.";
            0
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn big_uint_ty_is_mapped() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        extern fn(uint[71]) -> uint[99];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import Std.OpenQASM.Intrinsic.*;
        @SimulatableIntrinsic()
        operation fn(param_0 : BigInt) : BigInt {
            fail "Extern `fn` cannot be used without a linked implementation.";
            0L
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn extern_generates_qir() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        #pragma qdk.qir.profile Adaptive_RIF
        bit c;
        qubit q;
        c = measure q;
        
        extern void_fn();
        extern bit_fn(bit) -> bit;
        extern bool_fn(bool) -> bool;
        extern float_fn(float, float[17]) -> float;
        extern int_fn(int, int[17]) -> int;
        extern uint_fn(uint, uint[17]) -> uint;

        void_fn();
        //bit_fn(c); // use of dynamic result
        bool_fn(true);
        float_fn(pi, tau);
        int_fn(42, -17);
        uint_fn(74, 65);
    "#;

    let qsharp = compile_qasm_to_qir(source)?;
    expect![[r#"
        %Result = type opaque
        %Qubit = type opaque

        define void @ENTRYPOINT__main() #0 {
        block_0:
          call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
          call void @void_fn()
          %var_0 = call i1 @bool_fn(i1 true)
          %var_1 = call double @float_fn(double 3.141592653589793, double 6.283185307179586)
          %var_2 = call i64 @int_fn(i64 42, i64 -17)
          %var_3 = call i64 @uint_fn(i64 74, i64 65)
          call void @__quantum__rt__tuple_record_output(i64 0, i8* null)
          ret void
        }

        declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

        declare void @void_fn()

        declare i1 @bool_fn(i1)

        declare double @float_fn(double, double)

        declare i64 @int_fn(i64, i64)

        declare i64 @uint_fn(i64, i64)

        declare void @__quantum__rt__tuple_record_output(i64, i8*)

        attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="adaptive_profile" "required_num_qubits"="1" "required_num_results"="1" }
        attributes #1 = { "irreversible" }

        ; module flags

        !llvm.module.flags = !{!0, !1, !2, !3, !4, !5}

        !0 = !{i32 1, !"qir_major_version", i32 1}
        !1 = !{i32 7, !"qir_minor_version", i32 0}
        !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
        !3 = !{i32 1, !"dynamic_result_management", i1 false}
        !4 = !{i32 1, !"int_computations", !"i64"}
        !5 = !{i32 1, !"float_computations", !"f64"}
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}
