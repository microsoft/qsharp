// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::{fail_on_compilation_errors, parse, qasm_to_program_fragments};
use miette::Report;

#[test]
#[ignore = "unimplemented"]
fn classical() -> miette::Result<(), Vec<Report>> {
    let source = "
        bit[2] a;
        bit[2] b;
        let c = a[{0,1}] ++ b[1:2];
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
        qubit[5] q1;
        qubit[7] q2;
        let q = q1 ++ q2;
        let c = a[{0,1}] ++ b[1:2];
        let qq = q1[{1,3,4}];
        let qqq = qq ++ q2[1:2:6];
        let d = c;
        let e = d[1];
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
        creg a[2];
        creg b[2];
        let c = a[{0,1}] ++ b[1:2];
    ";

    let res = parse(source)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program_fragments(res.source, res.source_map);
    fail_on_compilation_errors(&unit);
    Ok(())
}

#[test]
#[ignore = "qasm3 parser does not support old-style decls yet"]
fn quantum_old_style_decls() -> miette::Result<(), Vec<Report>> {
    let source = "
        qreg q1[5];
        qreg q2[7];
        let q = q1 ++ q2;
        let c = a[{0,1}] ++ b[1:2];
        let qq = q1[{1,3,4}];
        let qqq = qq ++ q2[1:2:6];
        let d = c;
        let e = d[1];
    ";

    let res = parse(source)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program_fragments(res.source, res.source_map);
    fail_on_compilation_errors(&unit);
    Ok(())
}
