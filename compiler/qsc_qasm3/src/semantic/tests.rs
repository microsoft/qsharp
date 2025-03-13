// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod assignment;
pub mod decls;

pub mod expression;
pub mod statements;

use std::path::Path;
use std::sync::Arc;

use crate::io::InMemorySourceResolver;
use crate::io::SourceResolver;

use super::parse_source;

use super::QasmSemanticParseResult;

use expect_test::expect;
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
        for kind in &kinds {
            let (symbol_id, str) = match kind {
                super::ast::StmtKind::ClassicalDecl(decl) => (decl.symbol_id, decl.to_string()),
                super::ast::StmtKind::IODeclaration(decl) => (decl.symbol_id, decl.to_string()),
                super::ast::StmtKind::Assign(stmt) => (stmt.symbold_id, stmt.to_string()),
                super::ast::StmtKind::AssignOp(stmt) => (stmt.symbold_id, stmt.to_string()),
                _ => panic!("unsupported stmt type {kind}"),
            };

            value.push_str(&str);
            value.push('\n');
            let symbol = s.get_symbol_by_id(symbol_id).expect("getting symbol by id");
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

    assert!(
        !res.has_syntax_errors(),
        "syntax errors: {:?}",
        res.parse_errors()
    );

    let program = res.program.expect("no program");

    if errors.is_empty() {
        expect.assert_eq(&selector(&program, &res.symbols));
    } else {
        expect.assert_eq(&format!(
            "{}\n\n{:?}",
            program,
            errors
                .iter()
                .map(|e| Report::new(e.clone()))
                .collect::<Vec<_>>()
        ));
    }
}

pub(super) fn check_all<P>(
    path: P,
    sources: impl IntoIterator<Item = (Arc<str>, Arc<str>)>,
    expect: &Expect,
) where
    P: AsRef<Path>,
{
    check_map_all(path, sources, expect, |p, _| p.to_string());
}

fn check_map_all<P>(
    path: P,
    sources: impl IntoIterator<Item = (Arc<str>, Arc<str>)>,
    expect: &Expect,
    selector: impl FnOnce(&super::ast::Program, &super::SymbolTable) -> String,
) where
    P: AsRef<Path>,
{
    let resolver = InMemorySourceResolver::from_iter(sources);
    let source = resolver
        .resolve(path.as_ref())
        .map_err(|e| vec![e])
        .expect("could not load source")
        .1;
    let res = parse_source(source, path, &resolver)
        .map_err(|e| vec![e])
        .expect("failed to parse");

    let errors = res.all_errors();

    assert!(
        !res.has_syntax_errors(),
        "syntax errors: {:?}",
        res.parse_errors()
    );
    let program = res.program.expect("no program");

    if errors.is_empty() {
        expect.assert_eq(&selector(&program, &res.symbols));
    } else {
        expect.assert_eq(&format!(
            "{}\n\n{:?}",
            program,
            errors
                .iter()
                .map(|e| Report::new(e.clone()))
                .collect::<Vec<_>>()
        ));
    }
}

#[test]
fn semantic_errors_map_to_their_corresponding_file_specific_spans() {
    let source0 = r#"OPENQASM 3.0;
    include "stdgates.inc";
    include "source1.qasm";
    bit c = r; // undefined symbol r
    "#;
    let source1 = r#"include "source2.qasm";
    angle z = 7.0;
    float k = z + false; // invalid cast"#;
    let source2 = "bit x = 1;
    bool x = y && x; // undefined y, redefine x";
    let all_sources = [
        ("source0.qasm".into(), source0.into()),
        ("source1.qasm".into(), source1.into()),
        ("source2.qasm".into(), source2.into()),
    ];

    check_all(
        "source0.qasm",
        all_sources,
        &expect![[r#"
            Program:
                version: 3.0
                statements:
                    Stmt [196-206]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [196-206]:
                            symbol_id: 24
                            ty_span: [196-199]
                            init_expr: Expr [204-205]:
                                ty: Bit(true)
                                kind: Lit: Int(1)
                    Stmt [140-154]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [140-154]:
                            symbol_id: 26
                            ty_span: [140-145]
                            init_expr: Expr [150-153]:
                                ty: Angle(None, true)
                                kind: Lit: Float(7.0)

            [Qsc.Qasm3.Compile.UndefinedSymbol

              x Undefined symbol: y.
               ,-[source2.qasm:2:14]
             1 | bit x = 1;
             2 |     bool x = y && x; // undefined y, redefine x
               :              ^
               `----
            , Qsc.Qasm3.Compile.RedefinedSymbol

              x Redefined symbol: x.
               ,-[source2.qasm:2:10]
             1 | bit x = 1;
             2 |     bool x = y && x; // undefined y, redefine x
               :          ^
               `----
            , Qsc.Qasm3.Compile.CannotCast

              x Cannot cast expression of type Angle(None, false) to type Float(None,
              | false)
               ,-[source1.qasm:3:15]
             2 |     angle z = 7.0;
             3 |     float k = z + false; // invalid cast
               :               ^
               `----
            , Qsc.Qasm3.Compile.UndefinedSymbol

              x Undefined symbol: r.
               ,-[source0.qasm:4:13]
             3 |     include "source1.qasm";
             4 |     bit c = r; // undefined symbol r
               :             ^
             5 |     
               `----
            ]"#]],
    );
}
