// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    tests::{compile_with_config, fail_on_compilation_errors, gen_qsharp},
    CompilerConfig, OutputSemantics, ProgramType, QubitSemantics,
};
use expect_test::expect;
use miette::Report;
use qsc::target::Profile;

use crate::tests::compile_qasm_to_qir;

#[test]
fn reset_calls_are_generated_from_qasm() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        OPENQASM 3.0;
        include "stdgates.inc";
        bit[1] meas;
        qubit[1] q;
        reset q[0];
        h q[0];
        meas[0] = measure q[0];
    "#;

    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::Qiskit,
        ProgramType::File,
        Some("Test".into()),
        None,
    );
    let unit = compile_with_config(source, config)?;
    fail_on_compilation_errors(&unit);
    let qsharp = gen_qsharp(&unit.package.expect("no package found"));
    expect![[r#"
        namespace qasm3_import {
            import QasmStd.Angle.*;
            import QasmStd.Convert.*;
            import QasmStd.Intrinsic.*;
            @EntryPoint()
            operation Test() : Result[] {
                mutable meas = [Zero];
                let q = QIR.Runtime.AllocateQubitArray(1);
                Reset(q[0]);
                h(q[0]);
                set meas w/= 0 <- QIR.Intrinsic.__quantum__qis__m__body(q[0]);
                QIR.Runtime.ReleaseQubitArray(q);
                Microsoft.Quantum.Arrays.Reversed(meas)
            }
        }"#]]
    .assert_eq(&qsharp);

    Ok(())
}

#[test]
fn reset_with_base_profile_is_rewritten_without_resets() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        OPENQASM 3.0;
        include "stdgates.inc";
        bit[1] meas;
        qubit[1] q;
        reset q[0];
        h q[0];
        meas[0] = measure q[0];
    "#;

    let qir = compile_qasm_to_qir(source, Profile::Base)?;
    expect![[r#"
        %Result = type opaque
        %Qubit = type opaque

        define void @ENTRYPOINT__main() #0 {
        block_0:
          call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
          call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
          call void @__quantum__rt__array_record_output(i64 1, i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
          ret void
        }

        declare void @__quantum__qis__h__body(%Qubit*)

        declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

        declare void @__quantum__rt__array_record_output(i64, i8*)

        declare void @__quantum__rt__result_record_output(%Result*, i8*)

        attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="1" }
        attributes #1 = { "irreversible" }

        ; module flags

        !llvm.module.flags = !{!0, !1, !2, !3}

        !0 = !{i32 1, !"qir_major_version", i32 1}
        !1 = !{i32 7, !"qir_minor_version", i32 0}
        !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
        !3 = !{i32 1, !"dynamic_result_management", i1 false}
    "#]]
    .assert_eq(&qir);

    Ok(())
}

#[test]
fn reset_with_adaptive_ri_profile_generates_reset_qir() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        OPENQASM 3.0;
        include "stdgates.inc";
        bit[1] meas;
        qubit[1] q;
        reset q[0];
        h q[0];
        meas[0] = measure q[0];
    "#;

    let qir = compile_qasm_to_qir(source, Profile::AdaptiveRI)?;
    expect![[r#"
        %Result = type opaque
        %Qubit = type opaque

        define void @ENTRYPOINT__main() #0 {
        block_0:
          call void @__quantum__qis__reset__body(%Qubit* inttoptr (i64 0 to %Qubit*))
          call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
          call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
          call void @__quantum__rt__array_record_output(i64 1, i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
          ret void
        }

        declare void @__quantum__qis__reset__body(%Qubit*) #1

        declare void @__quantum__qis__h__body(%Qubit*)

        declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

        declare void @__quantum__rt__array_record_output(i64, i8*)

        declare void @__quantum__rt__result_record_output(%Result*, i8*)

        attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="adaptive_profile" "required_num_qubits"="1" "required_num_results"="1" }
        attributes #1 = { "irreversible" }

        ; module flags

        !llvm.module.flags = !{!0, !1, !2, !3, !4}

        !0 = !{i32 1, !"qir_major_version", i32 1}
        !1 = !{i32 7, !"qir_minor_version", i32 0}
        !2 = !{i32 1, !"dynamic_qubit_management", i1 false}
        !3 = !{i32 1, !"dynamic_result_management", i1 false}
        !4 = !{i32 1, !"int_computations", !"i64"}
    "#]]
    .assert_eq(&qir);

    Ok(())
}
