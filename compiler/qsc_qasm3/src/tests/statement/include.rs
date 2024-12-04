// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use crate::{
    qasm_to_program,
    tests::{parse_all, qsharp_from_qasm_compilation},
    CompilerConfig, OutputSemantics, ProgramType, QubitSemantics,
};
use expect_test::expect;
use miette::Report;

#[test]
fn programs_with_includes_can_be_parsed() -> miette::Result<(), Vec<Report>> {
    let source = r#"
        OPENQASM 3.0;
        include "stdgates.inc";
        include "custom_intrinsics.inc";
        bit[1] c;
        qubit[1] q;
        my_gate q[0];
        c[0] = measure q[0];
    "#;
    let custom_intrinsics = r#"
        @SimulatableIntrinsic
        gate my_gate q {
            x q;
        }
    "#;
    let all_sources = [
        ("source0.qasm".into(), source.into()),
        ("custom_intrinsics.inc".into(), custom_intrinsics.into()),
    ];

    let res = parse_all("source0.qasm", all_sources)?;
    let r = qasm_to_program(
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
    let qsharp = qsharp_from_qasm_compilation(r)?;
    expect![[r#"
        namespace qasm3_import {
            @EntryPoint()
            operation Test() : Result[] {
                @SimulatableIntrinsic()
                operation my_gate(q : Qubit) : Unit {
                    X(q);
                }
                mutable c = [Zero];
                let q = QIR.Runtime.AllocateQubitArray(1);
                my_gate(q[0]);
                set c w/= 0 <- QIR.Intrinsic.__quantum__qis__m__body(q[0]);
                Microsoft.Quantum.Arrays.Reversed(c)
            }
        }"#]]
    .assert_eq(&qsharp);
    Ok(())
}
