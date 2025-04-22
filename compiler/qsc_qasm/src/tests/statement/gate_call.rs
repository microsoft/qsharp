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
fn custom_gates_can_be_called_bypassing_stdgates() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        gate h a { U(π/2, 0., π) a; gphase(-π/4);}
        gate x a { U(π, 0., π) a; gphase(-π/2);}
        gate cx a, b { ctrl @ x a, b; }
        gate rz(λ) a { gphase(-λ/2); U(0., 0., λ) a; }
        gate rxx(theta) a, b { h a; h b; cx a, b; rz(theta) b; cx a, b; h b; h a; }

        qubit a;
        qubit b;
        x a;
        rxx(π/2) a, b;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        operation h(a : Qubit) : Unit is Adj + Ctl {
            U(__DoubleAsAngle__(3.141592653589793 / 2., 53), __DoubleAsAngle__(0., 53), __DoubleAsAngle__(3.141592653589793, 53), a);
            gphase(__DoubleAsAngle__(-3.141592653589793 / 4., 53));
        }
        operation x(a : Qubit) : Unit is Adj + Ctl {
            U(__DoubleAsAngle__(3.141592653589793, 53), __DoubleAsAngle__(0., 53), __DoubleAsAngle__(3.141592653589793, 53), a);
            gphase(__DoubleAsAngle__(-3.141592653589793 / 2., 53));
        }
        operation cx(a : Qubit, b : Qubit) : Unit is Adj + Ctl {
            Controlled x([a], b);
        }
        operation rz(λ : __Angle__, a : Qubit) : Unit is Adj + Ctl {
            gphase(__DivideAngleByInt__(__NegAngle__(λ), 2));
            U(__DoubleAsAngle__(0., 53), __DoubleAsAngle__(0., 53), λ, a);
        }
        operation rxx(theta : __Angle__, a : Qubit, b : Qubit) : Unit is Adj + Ctl {
            h(a);
            h(b);
            cx(a, b);
            rz(theta, b);
            cx(a, b);
            h(b);
            h(a);
        }
        let a = QIR.Runtime.__quantum__rt__qubit_allocate();
        let b = QIR.Runtime.__quantum__rt__qubit_allocate();
        x(a);
        rxx(__DoubleAsAngle__(Microsoft.Quantum.Math.PI() / 2., 53), a, b);
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
    expect![[
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

        !llvm.module.flags = !{!0, !1, !2, !3, !4}

        !0 = !{i32 1, !"qir_major_version", i32 1}
        !1 = !{i32 7, !"qir_minor_version", i32 0}
        !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
        !3 = !{i32 1, !"dynamic_result_management", i1 false}
        !4 = !{i32 1, !"int_computations", !"i64"}
        "#
    ]]
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
        [Qasm.Lowerer.InvalidNumberOfQubitArgs

          x gate expects 2 qubit arguments, but 1 were provided
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
        [Qasm.Lowerer.InvalidNumberOfQubitArgs

          x gate expects 2 qubit arguments, but 3 were provided
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
        [Qasm.Lowerer.InvalidNumberOfClassicalArgs

          x gate expects 1 classical arguments, but 0 were provided
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
        [Qasm.Lowerer.InvalidNumberOfClassicalArgs

          x gate expects 1 classical arguments, but 2 were provided
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

#[test]
fn rxx_gate_with_one_angle_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        qubit[2] q;
        rxx(2.0) q[1], q[0];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.AllocateQubitArray(2);
        rxx(__DoubleAsAngle__(2., 53), q[1], q[0]);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn ryy_gate_with_one_angle_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        qubit[2] q;
        ryy(2.0) q[1], q[0];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.AllocateQubitArray(2);
        ryy(__DoubleAsAngle__(2., 53), q[1], q[0]);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn rzz_gate_with_one_angle_can_be_called() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        qubit[2] q;
        rzz(2.0) q[1], q[0];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.AllocateQubitArray(2);
        rzz(__DoubleAsAngle__(2., 53), q[1], q[0]);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn all_qiskit_stdgates_can_be_called_included() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        qubit[4] q;
        rxx(pi / 2.0) q[1], q[0];
        ryy(pi / 2.0) q[1], q[0];
        rzz(pi / 2.0) q[1], q[0];
        dcx q[0], q[1];
        ecr q[0], q[1];
        r(pi / 2.0, pi / 4.0) q[1];
        rzx(pi / 2.0) q[1], q[0];
        cs q[0], q[1];
        csdg q[0], q[1];
        sxdg q[0];
        csx q[0], q[1];
        cu1(pi / 2.0) q[1], q[0];
        cu3(pi / 2.0, pi / 4.0, pi / 8.0) q[1], q[0];
        rccx q[0], q[1], q[2];
        c3sqrtx q[0], q[1], q[2], q[3];
        c3x q[0], q[1], q[2], q[3];
        rc3x q[0], q[1], q[2], q[3];
        xx_minus_yy(pi / 2.0, pi / 4.0) q[1], q[0];
        xx_plus_yy(pi / 2.0, pi / 4.0) q[1], q[0];
        ccz q[0], q[1], q[2];
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.AllocateQubitArray(4);
        rxx(__DoubleAsAngle__(Microsoft.Quantum.Math.PI() / 2., 53), q[1], q[0]);
        ryy(__DoubleAsAngle__(Microsoft.Quantum.Math.PI() / 2., 53), q[1], q[0]);
        rzz(__DoubleAsAngle__(Microsoft.Quantum.Math.PI() / 2., 53), q[1], q[0]);
        dcx(q[0], q[1]);
        ecr(q[0], q[1]);
        r(__DoubleAsAngle__(Microsoft.Quantum.Math.PI() / 2., 53), __DoubleAsAngle__(Microsoft.Quantum.Math.PI() / 4., 53), q[1]);
        rzx(__DoubleAsAngle__(Microsoft.Quantum.Math.PI() / 2., 53), q[1], q[0]);
        cs(q[0], q[1]);
        csdg(q[0], q[1]);
        sxdg(q[0]);
        csx(q[0], q[1]);
        cu1(__DoubleAsAngle__(Microsoft.Quantum.Math.PI() / 2., 53), q[1], q[0]);
        cu3(__DoubleAsAngle__(Microsoft.Quantum.Math.PI() / 2., 53), __DoubleAsAngle__(Microsoft.Quantum.Math.PI() / 4., 53), __DoubleAsAngle__(Microsoft.Quantum.Math.PI() / 8., 53), q[1], q[0]);
        rccx(q[0], q[1], q[2]);
        c3sqrtx(q[0], q[1], q[2], q[3]);
        c3x(q[0], q[1], q[2], q[3]);
        rc3x(q[0], q[1], q[2], q[3]);
        xx_minus_yy(__DoubleAsAngle__(Microsoft.Quantum.Math.PI() / 2., 53), __DoubleAsAngle__(Microsoft.Quantum.Math.PI() / 4., 53), q[1], q[0]);
        xx_plus_yy(__DoubleAsAngle__(Microsoft.Quantum.Math.PI() / 2., 53), __DoubleAsAngle__(Microsoft.Quantum.Math.PI() / 4., 53), q[1], q[0]);
        ccz(q[0], q[1], q[2]);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn broadcast_one_qubit_gate() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[2] qs;
        h qs;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let qs = QIR.Runtime.AllocateQubitArray(2);
        h(qs[0]);
        h(qs[1]);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn broadcast_two_qubit_gate() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[3] ctrls;
        qubit[3] targets;
        cx ctrls, targets;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let ctrls = QIR.Runtime.AllocateQubitArray(3);
        let targets = QIR.Runtime.AllocateQubitArray(3);
        cx(ctrls[0], targets[0]);
        cx(ctrls[1], targets[1]);
        cx(ctrls[2], targets[2]);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn broadcast_controlled_two_qubit_gate() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[3] ctrls;
        qubit[3] targets;
        inv @ cx ctrls, targets;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let ctrls = QIR.Runtime.AllocateQubitArray(3);
        let targets = QIR.Runtime.AllocateQubitArray(3);
        Adjoint cx(ctrls[0], targets[0]);
        Adjoint cx(ctrls[1], targets[1]);
        Adjoint cx(ctrls[2], targets[2]);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn broadcast_explicitly_controlled_gate() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit[3] ctrls;
        qubit[3] targets;
        ctrl @ x ctrls, targets;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let ctrls = QIR.Runtime.AllocateQubitArray(3);
        let targets = QIR.Runtime.AllocateQubitArray(3);
        Controlled x([ctrls[0]], targets[0]);
        Controlled x([ctrls[1]], targets[1]);
        Controlled x([ctrls[2]], targets[2]);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn broadcast_with_qubit_and_register() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        include "stdgates.inc";
        qubit ctl;
        qubit[2] targets;
        ctrl @ x ctl, targets;
    "#;

    let qsharp = compile_qasm_to_qsharp(source)?;
    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let ctl = QIR.Runtime.__quantum__rt__qubit_allocate();
        let targets = QIR.Runtime.AllocateQubitArray(2);
        Controlled x([ctl], targets[0]);
        Controlled x([ctl], targets[1]);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn broadcast_with_different_register_sizes_fails() {
    let source = r#"
        include "stdgates.inc";
        qubit[3] ctrls;
        qubit[2] targets;
        ctrl @ x ctrls, targets;
    "#;

    let Err(errors) = compile_qasm_to_qsharp(source) else {
        panic!("Expected error");
    };

    expect![[r#"
        [Qasm.Lowerer.BroadcastCallQuantumArgsDisagreeInSize

          x first quantum register is of type QubitArray(3) but found an argument of
          | type QubitArray(2)
           ,-[Test.qasm:5:25]
         4 |         qubit[2] targets;
         5 |         ctrl @ x ctrls, targets;
           :                         ^^^^^^^
         6 |     
           `----
        ]"#]]
    .assert_eq(&format!("{errors:?}"));
}
