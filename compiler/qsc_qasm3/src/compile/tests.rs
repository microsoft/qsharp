// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::{parse_all, print_compilation_errors, qasm_to_program_fragments};
use miette::Report;

#[test]
fn programs_with_includes_with_includes_can_be_compiled() -> miette::Result<(), Vec<Report>> {
    let source0 = r#"include "stdgates.inc";
    include "source1.qasm";
    "#;
    let source1 = r#"include "source2.qasm";
    "#;
    let source2 = "";
    let all_sources = [
        ("source0.qasm".into(), source0.into()),
        ("source1.qasm".into(), source1.into()),
        ("source2.qasm".into(), source2.into()),
    ];

    let res = parse_all("source0.qasm", all_sources)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program_fragments(res.source, res.source_map);
    print_compilation_errors(&unit);
    assert!(!unit.has_errors());
    Ok(())
}

#[test]
fn including_stdgates_multiple_times_causes_symbol_redifintion_errors(
) -> miette::Result<(), Vec<Report>> {
    let source0 = r#"include "stdgates.inc";
    include "source1.qasm";
    "#;
    let source1 = r#"include "source2.qasm";
    "#;
    let source2 = r#"include "stdgates.inc";"#;
    let all_sources = [
        ("source0.qasm".into(), source0.into()),
        ("source1.qasm".into(), source1.into()),
        ("source2.qasm".into(), source2.into()),
    ];

    let res = parse_all("source0.qasm", all_sources)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program_fragments(res.source, res.source_map);

    assert!(unit.has_errors());
    for error in unit.errors() {
        assert!(error.to_string().contains("Redefined symbol: "));
    }
    Ok(())
}
