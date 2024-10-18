// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::{compile_qasm_to_qir, compile_qasm_to_qsharp};
use expect_test::expect;
use miette::Report;
use qsc::target::Profile;

#[test]
fn x_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit q;
        x q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        X(q);
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn barrier_can_be_called_on_single_qubit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit q;
        barrier q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        @SimulatableIntrinsic()
        operation __quantum__qis__barrier__body() : Unit {}
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        __quantum__qis__barrier__body();
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn barrier_can_be_called_without_qubits() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit q;
        barrier;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        @SimulatableIntrinsic()
        operation __quantum__qis__barrier__body() : Unit {}
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        __quantum__qis__barrier__body();
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn barrier_generates_qir() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        bit[1] c;
        qubit[2] q;
        barrier q[0], q[1];
        barrier q[0];
        barrier;
        barrier q[0], q[1], q[0];
        c[0] = measure q[0];
    "#;

    let qsharp = compile_qasm_to_qir(source, Profile::AdaptiveRI)?;
    expect![
        r#"
        %Result = type opaque
        %Qubit = type opaque

        define void @ENTRYPOINT__main() #0 {
        block_0:
          call void @__quantum__qis__barrier__body()
          call void @__quantum__qis__barrier__body()
          call void @__quantum__qis__barrier__body()
          call void @__quantum__qis__barrier__body()
          call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
          call void @__quantum__rt__array_record_output(i64 1, i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
          ret void
        }

        declare void @__quantum__qis__barrier__body()

        declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

        declare void @__quantum__rt__array_record_output(i64, i8*)

        declare void @__quantum__rt__result_record_output(%Result*, i8*)

        attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="adaptive_profile" "required_num_qubits"="2" "required_num_results"="1" }
        attributes #1 = { "irreversible" }

        ; module flags

        !llvm.module.flags = !{!0, !1, !2, !3, !4, !5, !6, !7, !8, !9, !10}

        !0 = !{i32 1, !"qir_major_version", i32 1}
        !1 = !{i32 7, !"qir_minor_version", i32 0}
        !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
        !3 = !{i32 1, !"dynamic_result_management", i1 false}
        !4 = !{i32 1, !"classical_ints", i1 true}
        !5 = !{i32 1, !"qubit_resetting", i1 true}
        !6 = !{i32 1, !"classical_floats", i1 false}
        !7 = !{i32 1, !"backwards_branching", i1 false}
        !8 = !{i32 1, !"classical_fixed_points", i1 false}
        !9 = !{i32 1, !"user_functions", i1 false}
        !10 = !{i32 1, !"multiple_target_branching", i1 false}
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn barrier_can_be_called_on_two_qubit() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[2] q;
        barrier q[0], q[1];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![
        r#"
        @SimulatableIntrinsic()
        operation __quantum__qis__barrier__body() : Unit {}
        let q = QIR.Runtime.AllocateQubitArray(2);
        __quantum__qis__barrier__body();
        "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}
