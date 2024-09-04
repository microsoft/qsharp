// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod alias;

use crate::tests::{fail_on_compilation_errors, parse, qasm_to_program_fragments};
use miette::Report;

#[test]
#[ignore = "unimplemented"]
fn classical() -> miette::Result<(), Vec<Report>> {
    let source = "
        bit[2] a;
        creg b[2];
        qubit[3] q;
        int[10] x = 12;
        a[0] = b[1];
        x += int[10](a[1]);
        measure q[1] -> a[0];
        a = measure q[1:2];
        measure q[0];
        b = a == 0;
    ";

    let res = parse(source)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program_fragments(res.source, res.source_map);
    fail_on_compilation_errors(&unit);
    Ok(())
}

#[test]
#[ignore = "unimplemented"]
fn quantum() -> miette::Result<(), Vec<Report>> {
    let source = "
        bit[2] a;
        creg b[2];
        qubit[3] q;
        int[10] x = 12;
        a[0] = b[1];
        x += int[10](a[1]);
        measure q[1] -> a[0];
        a = measure q[1:2];
        measure q[0];
        b = a == 0;
    ";

    let res = parse(source)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program_fragments(res.source, res.source_map);
    fail_on_compilation_errors(&unit);
    Ok(())
}

#[test]
#[ignore = "qasm3 parser does not support old-style decls yet"]
fn classical_old_style_decls() -> miette::Result<(), Vec<Report>> {
    let source = "
        bit[2] a;
        creg b[2];
        qubit[3] q;
        int[10] x = 12;
        a[0] = b[1];
        x += int[10](a[1]);
        measure q[1] -> a[0];
        a = measure q[1:2];
        measure q[0];
        b = a == 0;
    ";

    let res = parse(source)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program_fragments(res.source, res.source_map);
    fail_on_compilation_errors(&unit);
    Ok(())
}
