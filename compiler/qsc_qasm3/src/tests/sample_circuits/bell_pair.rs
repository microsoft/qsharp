// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    qasm_to_program,
    tests::{gen_qsharp, parse, print_compilation_errors},
    CompilerConfig, OutputSemantics, ProgramType, QubitSemantics,
};

const SOURCE: &str = r#"
OPENQASM 3.0;
include "stdgates.inc";
bit[2] c;
qubit[2] q;
h q[0];
cx q[0], q[1];
barrier q[0], q[1];
c[0] = measure q[0];
c[1] = measure q[1];
"#;

#[test]
fn it_compiles() {
    let source = SOURCE;

    let res = parse(source).expect("should parse");
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
    print_compilation_errors(&unit);
    assert!(!unit.has_errors());
    let Some(package) = &unit.package else {
        panic!("no package found");
    };
    let qsharp = gen_qsharp(package);
    println!("{qsharp}");
}
