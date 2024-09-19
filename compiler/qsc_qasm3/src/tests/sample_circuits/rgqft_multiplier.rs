// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    qasm_to_program,
    tests::{gen_qsharp, parse, print_compilation_errors},
    CompilerConfig, OutputSemantics, ProgramType, QubitSemantics,
};

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

const SOURCE: &str = r#"
OPENQASM 3.0;
include "stdgates.inc";
qubit[1] a;
qubit[1] b;
qubit[2] out;
h out[1];
rz(pi/4) out[1];
cx out[1], out[0];
rz(-pi/4) out[0];
cx out[1], out[0];
rz(pi/4) out[0];
h out[0];
cx a[0], out[0];
rz(-pi/8) out[0];
rx(pi/2) out[0];
rz(pi) out[0];
rx(pi/2) out[0];
rz(9.032078879070655) out[0];
cx b[0], out[0];
rz(-7*pi/8) out[0];
rx(pi/2) out[0];
rz(pi) out[0];
rx(pi/2) out[0];
rz(6.675884388878311) out[0];
cx a[0], out[0];
rz(-pi/8) out[0];
rx(pi/2) out[0];
rz(pi) out[0];
rx(pi/2) out[0];
rz(9.032078879070655) out[0];
cx b[0], out[0];
rz(pi/4) b[0];
rx(pi/2) b[0];
rz(pi) b[0];
rx(pi/2) b[0];
rz(3*pi) b[0];
cx a[0], b[0];
rz(-pi/4) b[0];
rx(pi/2) b[0];
rz(pi) b[0];
rx(pi/2) b[0];
rz(3*pi) b[0];
cx a[0], b[0];
rz(pi/4) a[0];
cx a[0], out[1];
rz(-7*pi/8) out[0];
rx(pi/2) out[0];
rz(pi) out[0];
rx(pi/2) out[0];
rz(6.675884388878311) out[0];
h out[0];
rz(-pi/16) out[1];
rx(pi/2) out[1];
rz(pi) out[1];
rx(pi/2) out[1];
rz(9.228428419920018) out[1];
cx b[0], out[1];
rz(-15*pi/16) out[1];
rx(pi/2) out[1];
rz(pi) out[1];
rx(pi/2) out[1];
rz(6.4795348480289485) out[1];
cx a[0], out[1];
rz(-pi/16) out[1];
rx(pi/2) out[1];
rz(pi) out[1];
rx(pi/2) out[1];
rz(9.228428419920018) out[1];
cx b[0], out[1];
rz(pi/8) b[0];
rx(pi/2) b[0];
rz(pi) b[0];
rx(pi/2) b[0];
rz(3*pi) b[0];
cx a[0], b[0];
rz(-pi/8) b[0];
rx(pi/2) b[0];
rz(pi) b[0];
rx(pi/2) b[0];
rz(3*pi) b[0];
cx a[0], b[0];
rz(pi/8) a[0];
rz(-15*pi/16) out[1];
rx(pi/2) out[1];
rz(pi) out[1];
rx(pi/2) out[1];
rz(6.4795348480289485) out[1];
rz(-pi/4) out[1];
cx out[1], out[0];
rz(pi/4) out[0];
cx out[1], out[0];
rz(-pi/4) out[0];
h out[1];"#;
