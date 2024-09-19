// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::{parse, parse_all};
use miette::Report;

#[test]
fn simple_programs_can_be_parsed() -> miette::Result<(), Vec<Report>> {
    let source = r#"OPENQASM 3.0;
    include "stdgates.inc";
    qubit q;
    "#;
    let _ = parse(source)?;
    Ok(())
}

#[test]
fn programs_with_includes_can_be_parsed() -> miette::Result<(), Vec<Report>> {
    let source0 = r#"OPENQASM 3.0;
    include "stdgates.inc";
    include "source1.qasm";
    qubit q1;
    "#;
    let source1 = "qubit q2;
    ";
    let all_sources = [
        ("source0.qasm".into(), source0.into()),
        ("source1.qasm".into(), source1.into()),
    ];

    let res = parse_all("source0.qasm", all_sources)?;
    assert!(res.source.includes().len() == 1);
    Ok(())
}

#[test]
fn programs_with_includes_with_includes_can_be_parsed() -> miette::Result<(), Vec<Report>> {
    let source0 = r#"OPENQASM 3.0;
    include "stdgates.inc";
    include "source1.qasm";
    qubit q1;
    "#;
    let source1 = r#"include "source2.qasm";
    qubit q2;
    "#;
    let source2 = "qubit q3;
    ";
    let all_sources = [
        ("source0.qasm".into(), source0.into()),
        ("source1.qasm".into(), source1.into()),
        ("source2.qasm".into(), source2.into()),
    ];

    let res = parse_all("source0.qasm", all_sources)?;
    assert!(res.source.includes().len() == 1);
    assert!(res.source.includes()[0].includes().len() == 1);
    Ok(())
}
