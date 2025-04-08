// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::{compile_qasm_to_qir, compile_qasm_to_qsharp};
use expect_test::expect;
use miette::Report;
use qsc::target::Profile;

#[test]
fn u_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        qubit q;
        U(1.0, 2.0, 3.0) q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        U(__DoubleAsAngle__(1., 53), __DoubleAsAngle__(2., 53), __DoubleAsAngle__(3., 53), q);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn gphase_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        gphase(2.0);
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        gphase(__DoubleAsAngle__(2., 53));
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn x_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit q;
        x q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        x(q);
    "#]]
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
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        __quantum__qis__barrier__body();
    "#]]
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
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        __quantum__qis__barrier__body();
    "#]]
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
    expect![[r#"
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

        !llvm.module.flags = !{!0, !1, !2, !3, !4}

        !0 = !{i32 1, !"qir_major_version", i32 1}
        !1 = !{i32 7, !"qir_minor_version", i32 0}
        !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
        !3 = !{i32 1, !"dynamic_result_management", i1 false}
        !4 = !{i32 1, !"int_computations", !"i64"}
    "#]]
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
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.AllocateQubitArray(2);
        __quantum__qis__barrier__body();
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn cx_called_with_one_qubit_generates_error() {
    let source = r#"
        include "stdgates.inc";
        qubit[2] q;
        cx q[0];
    "#;

    let Err(errors) = compile_qasm_to_qsharp(source) else {
        panic!("Expected error");
    };

    expect![[r#"
        [Qsc.Qasm3.Compile.InvalidNumberOfQubitArgs

          x Gate expects 2 qubit arguments, but 1 were provided.
           ,-[Test.qasm:4:9]
         3 |         qubit[2] q;
         4 |         cx q[0];
           :         ^^^^^^^^
         5 |     
           `----
        ]"#]]
    .assert_eq(&format!("{errors:?}"));
}

#[test]
fn cx_called_with_too_many_qubits_generates_error() {
    let source = r#"
        include "stdgates.inc";
        qubit[3] q;
        cx q[0], q[1], q[2];
    "#;

    let Err(errors) = compile_qasm_to_qsharp(source) else {
        panic!("Expected error");
    };

    expect![[r#"
        [Qsc.Qasm3.Compile.InvalidNumberOfQubitArgs

          x Gate expects 2 qubit arguments, but 3 were provided.
           ,-[Test.qasm:4:9]
         3 |         qubit[3] q;
         4 |         cx q[0], q[1], q[2];
           :         ^^^^^^^^^^^^^^^^^^^^
         5 |     
           `----
        ]"#]]
    .assert_eq(&format!("{errors:?}"));
}

#[test]
fn rx_gate_with_no_angles_generates_error() {
    let source = r#"
        include "stdgates.inc";
        qubit q;
        rx q;
    "#;

    let Err(errors) = compile_qasm_to_qsharp(source) else {
        panic!("Expected error");
    };

    expect![[r#"
        [Qsc.Qasm3.Compile.InvalidNumberOfClassicalArgs

          x Gate expects 1 classical arguments, but 0 were provided.
           ,-[Test.qasm:4:9]
         3 |         qubit q;
         4 |         rx q;
           :         ^^^^^
         5 |     
           `----
        ]"#]]
    .assert_eq(&format!("{errors:?}"));
}

#[test]
fn rx_gate_with_one_angle_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit q;
        rx(2.0) q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        rx(__DoubleAsAngle__(2., 53), q);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn rx_gate_with_too_many_angles_generates_error() {
    let source = r#"
        include "stdgates.inc";
        qubit q;
        rx(2.0, 3.0) q;
    "#;

    let Err(errors) = compile_qasm_to_qsharp(source) else {
        panic!("Expected error");
    };

    expect![[r#"
        [Qsc.Qasm3.Compile.InvalidNumberOfClassicalArgs

          x Gate expects 1 classical arguments, but 2 were provided.
           ,-[Test.qasm:4:9]
         3 |         qubit q;
         4 |         rx(2.0, 3.0) q;
           :         ^^^^^^^^^^^^^^^
         5 |     
           `----
        ]"#]]
    .assert_eq(&format!("{errors:?}"));
}

#[test]
fn implicit_cast_to_angle_works() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit q;
        float a = 2.0;
        rx(a) q;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        mutable a = 2.;
        rx(__DoubleAsAngle__(a, 53), q);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn custom_gate_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        gate my_gate q1, q2 {
            h q1;
            h q2;
        }

        qubit[2] q;
        my_gate q[0], q[1];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        operation my_gate(q1 : Qubit, q2 : Qubit) : Unit is Adj + Ctl {
            h(q1);
            h(q2);
        }
        let q = QIR.Runtime.AllocateQubitArray(2);
        my_gate(q[0], q[1]);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn custom_gate_can_be_called_with_inv_modifier() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        gate my_gate q1, q2 {
            h q1;
            h q2;
        }

        qubit[2] q;
        inv @ my_gate q[0], q[1];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        operation my_gate(q1 : Qubit, q2 : Qubit) : Unit is Adj + Ctl {
            h(q1);
            h(q2);
        }
        let q = QIR.Runtime.AllocateQubitArray(2);
        Adjoint my_gate(q[0], q[1]);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn custom_gate_can_be_called_with_ctrl_modifier() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        gate my_gate q1, q2 {
            h q1;
            h q2;
        }

        qubit[2] ctl;
        qubit[2] q;
        ctrl(2) @ my_gate ctl[0], ctl[1], q[0], q[1];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        operation my_gate(q1 : Qubit, q2 : Qubit) : Unit is Adj + Ctl {
            h(q1);
            h(q2);
        }
        let ctl = QIR.Runtime.AllocateQubitArray(2);
        let q = QIR.Runtime.AllocateQubitArray(2);
        Controlled my_gate([ctl[0], ctl[1]], (q[0], q[1]));
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn custom_gate_can_be_called_with_negctrl_modifier() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        gate my_gate q1, q2 {
            h q1;
            h q2;
        }

        qubit[2] ctl;
        qubit[2] q;
        negctrl(2) @ my_gate ctl[0], ctl[1], q[0], q[1];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        operation my_gate(q1 : Qubit, q2 : Qubit) : Unit is Adj + Ctl {
            h(q1);
            h(q2);
        }
        let ctl = QIR.Runtime.AllocateQubitArray(2);
        let q = QIR.Runtime.AllocateQubitArray(2);
        ApplyControlledOnInt(0, my_gate, [ctl[0], ctl[1]], (q[0], q[1]));
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn custom_gate_can_be_called_with_pow_modifier() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        gate my_gate q1, q2 {
            h q1;
            h q2;
        }

        qubit[2] q;
        pow(2) @ my_gate q[0], q[1];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        operation my_gate(q1 : Qubit, q2 : Qubit) : Unit is Adj + Ctl {
            h(q1);
            h(q2);
        }
        let q = QIR.Runtime.AllocateQubitArray(2);
        __Pow__(2, my_gate, (q[0], q[1]));
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn simulatable_intrinsic_on_gate_stmt_generates_correct_qir() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";

        @SimulatableIntrinsic
        gate my_gate q {
            x q;
        }

        qubit q;
        my_gate q;
        bit result = measure q;
    "#;

    let qsharp = compile_qasm_to_qir(source, Profile::AdaptiveRI)?;
    expect![[r#"
        %Result = type opaque
        %Qubit = type opaque

        define void @ENTRYPOINT__main() #0 {
        block_0:
          call void @my_gate(%Qubit* inttoptr (i64 0 to %Qubit*))
          call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
          call void @__quantum__rt__tuple_record_output(i64 0, i8* null)
          ret void
        }

        declare void @my_gate(%Qubit*)

        declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

        declare void @__quantum__rt__tuple_record_output(i64, i8*)

        attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="adaptive_profile" "required_num_qubits"="1" "required_num_results"="1" }
        attributes #1 = { "irreversible" }

        ; module flags

        !llvm.module.flags = !{!0, !1, !2, !3, !4}

        !0 = !{i32 1, !"qir_major_version", i32 1}
        !1 = !{i32 7, !"qir_minor_version", i32 0}
        !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
        !3 = !{i32 1, !"dynamic_result_management", i1 false}
        !4 = !{i32 1, !"int_computations", !"i64"}
    "#]].assert_eq(&qsharp);
    Ok(())
}
