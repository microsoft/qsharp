// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::path::Path;
use std::sync::Arc;

use crate::io::InMemorySourceResolver;
use crate::io::SourceResolver;

use super::parse_source;
use super::QasmParseResult;
use miette::Report;

pub(crate) fn parse_all<P>(
    path: P,
    sources: impl IntoIterator<Item = (Arc<str>, Arc<str>)>,
) -> miette::Result<QasmParseResult, Vec<Report>>
where
    P: AsRef<Path>,
{
    let resolver = InMemorySourceResolver::from_iter(sources);
    let source = resolver.resolve(path.as_ref()).map_err(|e| vec![e])?.1;
    let res = crate::parser::parse_source(source, path, &resolver).map_err(|e| vec![e])?;
    if res.source.has_errors() {
        let errors = res
            .errors()
            .into_iter()
            .map(|e| Report::new(e.clone()))
            .collect();
        Err(errors)
    } else {
        Ok(res)
    }
}

pub(crate) fn parse<S>(source: S) -> miette::Result<QasmParseResult, Vec<Report>>
where
    S: AsRef<str>,
{
    let resolver = InMemorySourceResolver::from_iter([("test".into(), source.as_ref().into())]);
    let res = parse_source(source, "test", &resolver).map_err(|e| vec![e])?;
    if res.source.has_errors() {
        let errors = res
            .errors()
            .into_iter()
            .map(|e| Report::new(e.clone()))
            .collect();
        return Err(errors);
    }
    Ok(res)
}

#[test]
fn simple_programs_can_be_parsed() -> miette::Result<(), Vec<Report>> {
    let source = r#"OPENQASM 3.0;
    include "stdgates.inc";"#;
    let res = parse(source)?;
    assert!(res.source.program.version.is_some());
    Ok(())
}

#[test]
fn programs_with_includes_can_be_parsed() -> miette::Result<(), Vec<Report>> {
    let source0 = r#"include "stdgates.inc";
    include "source1.qasm";"#;
    let source1 = "";
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
    assert!(res.source.includes().len() == 1);
    assert!(res.source.includes()[0].includes().len() == 1);
    Ok(())
}
