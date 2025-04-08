// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;
use miette::Report;

use crate::tests::{
    compile_fragments, compile_with_config, fail_on_compilation_errors,
    qsharp_from_qasm_compilation,
};
use crate::{
    tests::{compile_qasm_stmt_to_qsharp, compile_qasm_stmt_to_qsharp_with_semantics},
    QubitSemantics,
};
use crate::{CompilerConfig, OutputSemantics, ProgramType};

#[test]
fn quantum() -> miette::Result<(), Vec<Report>> {
    let source = "
        qubit[6] q1;
        qubit q2;
    ";

    let unit = compile_fragments(source)?;
    fail_on_compilation_errors(&unit);
    Ok(())
}

#[test]
fn single_qubit_decl() -> miette::Result<(), Vec<Report>> {
    let source = "
        qubit my_qubit;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp(source)?;
    expect![
        r#"
        let my_qubit = QIR.Runtime.__quantum__rt__qubit_allocate();
    "#
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn single_qubit_decl_with_qsharp_semantics() -> miette::Result<(), Vec<Report>> {
    let source = "
        qubit my_qubit;
    ";

    let qsharp = compile_qasm_stmt_to_qsharp_with_semantics(source, QubitSemantics::QSharp)?;
    expect![
        "
        use my_qubit = Qubit();
    "
    ]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn fragment_does_not_generate_qubit_release_calls() -> miette::Result<(), Vec<Report>> {
    let source = "
        qubit q;
        qubit[3] qs;
    ";

    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::Fragments,
        None,
        None,
    );

    let unit = compile_with_config(source, config)?;
    let qsharp = qsharp_from_qasm_compilation(unit)?;

    expect![[r#"
        import QasmStd.Angle.*;
        import QasmStd.Convert.*;
        import QasmStd.Intrinsic.*;
        let q = QIR.Runtime.__quantum__rt__qubit_allocate();
        let qs = QIR.Runtime.AllocateQubitArray(3);
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn file_generates_qubit_release_calls() -> miette::Result<(), Vec<Report>> {
    let source = "
        qubit q;
        qubit[3] qs;
    ";

    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::File,
        None,
        None,
    );

    let unit = compile_with_config(source, config)?;
    let qsharp = qsharp_from_qasm_compilation(unit)?;

    expect![[r#"
        namespace qasm3_import {
            import QasmStd.Angle.*;
            import QasmStd.Convert.*;
            import QasmStd.Intrinsic.*;
            @EntryPoint()
            operation program() : Unit {
                let q = QIR.Runtime.__quantum__rt__qubit_allocate();
                let qs = QIR.Runtime.AllocateQubitArray(3);
                QIR.Runtime.__quantum__rt__qubit_release(q);
                QIR.Runtime.ReleaseQubitArray(qs);
            }
        }"#]]
    .assert_eq(&qsharp);
    Ok(())
}

#[test]
fn operation_generates_qubit_release_calls() -> miette::Result<(), Vec<Report>> {
    let source = "
        qubit q;
        qubit[3] qs;
    ";

    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::Operation,
        None,
        None,
    );

    let unit = compile_with_config(source, config)?;
    let qsharp = qsharp_from_qasm_compilation(unit)?;

    expect![[r#"
        operation program() : Unit {
            import QasmStd.Angle.*;
            import QasmStd.Convert.*;
            import QasmStd.Intrinsic.*;
            let q = QIR.Runtime.__quantum__rt__qubit_allocate();
            let qs = QIR.Runtime.AllocateQubitArray(3);
            QIR.Runtime.__quantum__rt__qubit_release(q);
            QIR.Runtime.ReleaseQubitArray(qs);
        }
    "#]]
    .assert_eq(&qsharp);
    Ok(())
}
