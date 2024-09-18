// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use crate::{
    qasm_to_program,
    tests::{fail_on_compilation_errors, gen_qsharp, parse},
    CompilerConfig, OutputSemantics, ProgramType, QubitSemantics,
};
use expect_test::expect;
use miette::Report;
use qsc::target::Profile;

use super::compile_qasm_to_qir;

#[test]
fn using_re_semantics_removes_output() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        OPENQASM 3.0;
        include "stdgates.inc";
        output bit[2] c;
        qubit[2] q;
        input float[64] theta;
        input int[64] beta;
        output float[64] gamma;
        output float[64] delta;
        rz(theta) q[0];
        h q[0];
        cx q[0], q[1];
        c[0] = measure q[0];
        c[1] = measure q[1];
    "#;

    let res = parse(source)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program(
        res.source,
        res.source_map,
        CompilerConfig::new(
            QubitSemantics::Qiskit,
            OutputSemantics::ResourceEstimation,
            ProgramType::File,
            Some("Test".into()),
            None,
        ),
    );
    fail_on_compilation_errors(&unit);
    let qsharp = gen_qsharp(&unit.package.expect("no package found"));
    expect![
        r#"
        namespace qasm3_import {
            operation Test(theta : Double, beta : Int) : Unit {
                mutable c = [Zero, Zero];
                let q = QIR.Runtime.AllocateQubitArray(2);
                mutable gamma = 0.;
                mutable delta = 0.;
                Rz(theta, q[0]);
                H(q[0]);
                CNOT(q[0], q[1]);
                set c w/= 0 <- M(q[0]);
                set c w/= 1 <- M(q[1]);
            }
        }"#
    ]
    .assert_eq(&qsharp);

    Ok(())
}

#[test]
fn using_qasm_semantics_captures_all_classical_decls_as_output() -> miette::Result<(), Vec<Report>>
{
    let source = r#"
        OPENQASM 3.0;
        include "stdgates.inc";
        output bit[2] c;
        qubit[2] q;
        input float[64] theta;
        input int[64] beta;
        output float[64] gamma;
        output float[64] delta;
        rz(theta) q[0];
        h q[0];
        cx q[0], q[1];
        c[0] = measure q[0];
        c[1] = measure q[1];
    "#;

    let res = parse(source)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program(
        res.source,
        res.source_map,
        CompilerConfig::new(
            QubitSemantics::Qiskit,
            OutputSemantics::OpenQasm,
            ProgramType::File,
            Some("Test".into()),
            None,
        ),
    );
    fail_on_compilation_errors(&unit);
    let qsharp = gen_qsharp(&unit.package.expect("no package found"));
    expect![
        r#"
        namespace qasm3_import {
            operation Test(theta : Double, beta : Int) : (Result[], Double, Double) {
                mutable c = [Zero, Zero];
                let q = QIR.Runtime.AllocateQubitArray(2);
                mutable gamma = 0.;
                mutable delta = 0.;
                Rz(theta, q[0]);
                H(q[0]);
                CNOT(q[0], q[1]);
                set c w/= 0 <- M(q[0]);
                set c w/= 1 <- M(q[1]);
                (c, gamma, delta)
            }
        }"#
    ]
    .assert_eq(&qsharp);

    Ok(())
}

#[test]
fn using_qiskit_semantics_only_bit_array_is_captured_and_reversed(
) -> miette::Result<(), Vec<Report>> {
    let source = r#"
        OPENQASM 3.0;
        include "stdgates.inc";
        output bit[2] c;
        qubit[2] q;
        input float[64] theta;
        input int[64] beta;
        output float[64] gamma;
        output float[64] delta;
        rz(theta) q[0];
        h q[0];
        cx q[0], q[1];
        c[0] = measure q[0];
        c[1] = measure q[1];
    "#;

    let res = parse(source)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program(
        res.source,
        res.source_map,
        CompilerConfig::new(
            QubitSemantics::Qiskit,
            OutputSemantics::Qiskit,
            ProgramType::File,
            Some("Test".into()),
            None,
        ),
    );
    fail_on_compilation_errors(&unit);
    let qsharp = gen_qsharp(&unit.package.expect("no package found"));
    expect![
        r#"
namespace qasm3_import {
    operation Test(theta : Double, beta : Int) : Result[] {
        mutable c = [Zero, Zero];
        let q = QIR.Runtime.AllocateQubitArray(2);
        mutable gamma = 0.;
        mutable delta = 0.;
        Rz(theta, q[0]);
        H(q[0]);
        CNOT(q[0], q[1]);
        set c w/= 0 <- M(q[0]);
        set c w/= 1 <- M(q[1]);
        Microsoft.Quantum.Arrays.Reversed(c)
    }
}"#
    ]
    .assert_eq(&qsharp);

    Ok(())
}

#[test]
fn using_qiskit_semantics_multiple_bit_arrays_are_reversed_in_order_and_reversed_in_content(
) -> miette::Result<(), Vec<Report>> {
    let source = r#"
OPENQASM 3.0;
include "stdgates.inc";
output bit[2] c;
output bit[3] c2;
qubit[5] q;
input float[64] theta;
input int[64] beta;
output float[64] gamma;
output float[64] delta;
rz(theta) q[0];
h q[0];
cx q[0], q[1];
x q[2];
id q[3];
x q[4];
c[0] = measure q[0];
c[1] = measure q[1];
c2[0] = measure q[2];
c2[1] = measure q[3];
c2[2] = measure q[4];
    "#;

    let res = parse(source)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program(
        res.source,
        res.source_map,
        CompilerConfig::new(
            QubitSemantics::Qiskit,
            OutputSemantics::Qiskit,
            ProgramType::File,
            Some("Test".into()),
            None,
        ),
    );
    fail_on_compilation_errors(&unit);
    let package = unit.package.expect("no package found");
    let qsharp = gen_qsharp(&package.clone());
    expect![
        r#"
namespace qasm3_import {
    operation Test(theta : Double, beta : Int) : (Result[], Result[]) {
        mutable c = [Zero, Zero];
        mutable c2 = [Zero, Zero, Zero];
        let q = QIR.Runtime.AllocateQubitArray(5);
        mutable gamma = 0.;
        mutable delta = 0.;
        Rz(theta, q[0]);
        H(q[0]);
        CNOT(q[0], q[1]);
        X(q[2]);
        I(q[3]);
        X(q[4]);
        set c w/= 0 <- M(q[0]);
        set c w/= 1 <- M(q[1]);
        set c2 w/= 0 <- M(q[2]);
        set c2 w/= 1 <- M(q[3]);
        set c2 w/= 2 <- M(q[4]);
        (Microsoft.Quantum.Arrays.Reversed(c2), Microsoft.Quantum.Arrays.Reversed(c))
    }
}"#
    ]
    .assert_eq(&qsharp);

    Ok(())
}

#[test]
fn qir_generation_using_qiskit_semantics_multiple_bit_arrays_are_reversed_in_order_and_reversed_in_content(
) -> miette::Result<(), Vec<Report>> {
    let source = r#"
OPENQASM 3.0;
include "stdgates.inc";
output bit[2] c;
output bit[3] c2;
qubit[5] q;
float[64] theta = 0.5;
int[64] beta = 4;
output float[64] gamma;
output float[64] delta;
rz(theta) q[0];
h q[0];
cx q[0], q[1];
x q[2];
id q[3];
x q[4];
barrier q[0], q[1];
c[0] = measure q[0];
c[1] = measure q[1];
c2[0] = measure q[2];
c2[1] = measure q[3];
c2[2] = measure q[4];
    "#;

    let qir = compile_qasm_to_qir(source, Profile::AdaptiveRI)?;
    expect![
        r#"
%Result = type opaque
%Qubit = type opaque

define void @ENTRYPOINT__main() #0 {
block_0:
  call void @__quantum__qis__rz__body(double 0.5, %Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__cx__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 2 to %Qubit*))
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 4 to %Qubit*))
  call void @__quantum__qis__barrier__body()
  call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
  call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
  call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 2 to %Qubit*), %Result* inttoptr (i64 2 to %Result*))
  call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 3 to %Qubit*), %Result* inttoptr (i64 3 to %Result*))
  call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 4 to %Qubit*), %Result* inttoptr (i64 4 to %Result*))
  call void @__quantum__rt__tuple_record_output(i64 2, i8* null)
  call void @__quantum__rt__array_record_output(i64 3, i8* null)
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 4 to %Result*), i8* null)
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 3 to %Result*), i8* null)
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 2 to %Result*), i8* null)
  call void @__quantum__rt__array_record_output(i64 2, i8* null)
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
  ret void
}

declare void @__quantum__qis__rz__body(double, %Qubit*)

declare void @__quantum__qis__h__body(%Qubit*)

declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)

declare void @__quantum__qis__x__body(%Qubit*)

declare void @__quantum__qis__barrier__body()

declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

declare void @__quantum__rt__tuple_record_output(i64, i8*)

declare void @__quantum__rt__array_record_output(i64, i8*)

declare void @__quantum__rt__result_record_output(%Result*, i8*)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="adaptive_profile" "required_num_qubits"="5" "required_num_results"="5" }
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
    .assert_eq(&qir);

    Ok(())
}
