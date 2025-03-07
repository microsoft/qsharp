// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod decls;

use std::path::Path;
use std::sync::Arc;

use crate::io::InMemorySourceResolver;
use crate::io::SourceResolver;

use super::parse_source;

use super::QasmSemanticParseResult;

use miette::Report;

use expect_test::Expect;

pub(crate) fn parse_all<P>(
    path: P,
    sources: impl IntoIterator<Item = (Arc<str>, Arc<str>)>,
) -> miette::Result<QasmSemanticParseResult, Vec<Report>>
where
    P: AsRef<Path>,
{
    let resolver = InMemorySourceResolver::from_iter(sources);
    let source = resolver.resolve(path.as_ref()).map_err(|e| vec![e])?.1;
    let res = parse_source(source, path, &resolver).map_err(|e| vec![e])?;
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

pub(crate) fn parse<S>(source: S) -> miette::Result<QasmSemanticParseResult, Vec<Report>>
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

pub(super) fn check(input: &str, expect: &Expect) {
    check_map(input, expect, |p, _| p.to_string());
}

pub(super) fn check_classical_decl(input: &str, expect: &Expect) {
    check_map(input, expect, |p, s| {
        let kind = p
            .statements
            .first()
            .expect("reading first statement")
            .kind
            .clone();
        let super::ast::StmtKind::ClassicalDecl(decl) = kind.as_ref() else {
            panic!("expected classical declaration statement");
        };
        let mut value = decl.to_string();
        value.push('\n');
        let symbol = s
            .get_symbol_by_id(decl.symbol_id)
            .expect("getting symbol by id");
        value.push_str(&format!("[{}] {}", symbol.0, symbol.1));
        value
    });
}

pub(super) fn check_classical_decls(input: &str, expect: &Expect) {
    check_map(input, expect, |p, s| {
        let kinds = p
            .statements
            .iter()
            .map(|stmt| stmt.kind.as_ref().clone())
            .collect::<Vec<_>>();
        let mut value = String::new();
        for kind in kinds {
            let super::ast::StmtKind::ClassicalDecl(decl) = kind else {
                panic!("expected classical declaration statement");
            };
            value.push_str(&decl.to_string());
            value.push('\n');
            let symbol = s
                .get_symbol_by_id(decl.symbol_id)
                .expect("getting symbol by id");
            value.push_str(&format!("[{}] {}", symbol.0, symbol.1));
            value.push('\n');
        }

        value
    });
}

pub(super) fn check_stmt_kind(input: &str, expect: &Expect) {
    check_map(input, expect, |p, _| {
        p.statements
            .first()
            .expect("reading first statement")
            .kind
            .to_string()
    });
}

pub(super) fn check_stmt_kinds(input: &str, expect: &Expect) {
    check_map(input, expect, |p, _| {
        p.statements
            .iter()
            .fold(String::new(), |acc, x| format!("{acc}{}\n", x.kind))
    });
}

fn check_map<S>(
    input: S,
    expect: &Expect,
    selector: impl FnOnce(&super::ast::Program, &super::SymbolTable) -> String,
) where
    S: AsRef<str>,
{
    let resolver = InMemorySourceResolver::from_iter([("test".into(), input.as_ref().into())]);
    let res = parse_source(input, "test", &resolver)
        .map_err(|e| vec![e])
        .expect("failed to parse");
    let errors = res.all_errors();

    if errors.is_empty() {
        expect.assert_eq(&selector(&res.program, &res.symbols));
    } else {
        expect.assert_eq(&format!(
            "{}\n\n{:?}",
            res.program,
            errors
                .iter()
                .map(|e| Report::new(e.clone()))
                .collect::<Vec<_>>()
        ));
    }
}
