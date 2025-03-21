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
    check_map(input, expect, |program, symbol_table| {
        let kind = program
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
        let symbol = &symbol_table[decl.symbol_id];
        value.push_str(&format!("[{}] {symbol}", decl.symbol_id));
        value
    });
}

pub(super) fn check_classical_decls(input: &str, expect: &Expect) {
    check_map(input, expect, |program, symbol_table| {
        let kinds = program
            .statements
            .iter()
            .map(|stmt| stmt.kind.as_ref().clone())
            .collect::<Vec<_>>();
        let mut value = String::new();
        for kind in &kinds {
            let (symbol_id, str) = match kind {
                super::ast::StmtKind::ClassicalDecl(decl) => (decl.symbol_id, decl.to_string()),
                super::ast::StmtKind::OutputDeclaration(decl) => (decl.symbol_id, decl.to_string()),
                super::ast::StmtKind::Assign(stmt) => (stmt.symbol_id, stmt.to_string()),
                super::ast::StmtKind::AssignOp(stmt) => (stmt.symbol_id, stmt.to_string()),
                _ => panic!("unsupported stmt type {kind}"),
            };

            value.push_str(&str);
            value.push('\n');
            let symbol = &symbol_table[symbol_id];
            value.push_str(&format!("[{symbol_id}] {symbol}"));
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
    selector: impl FnOnce(&super::ast::Program, &super::symbols::SymbolTable) -> String,
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
        res.sytax_errors()
    );

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
    selector: impl FnOnce(&super::ast::Program, &super::symbols::SymbolTable) -> String,
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
        res.sytax_errors()
    );

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

#[test]
#[allow(clippy::too_many_lines)]
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
                                kind: Lit: Bit(1)
                    Stmt [211-227]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [211-227]:
                            symbol_id: 24
                            ty_span: [211-215]
                            init_expr: Expr [220-226]:
                                ty: Bool(false)
                                kind: BinaryOpExpr:
                                    op: AndL
                                    lhs: Expr [220-221]:
                                        ty: Err
                                        kind: SymbolId(25)
                                    rhs: Expr [225-226]:
                                        ty: Bool(false)
                                        kind: Cast [0-0]:
                                            ty: Bool(false)
                                            expr: Expr [225-226]:
                                                ty: Bit(false)
                                                kind: SymbolId(24)
                    Stmt [140-154]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [140-154]:
                            symbol_id: 26
                            ty_span: [140-145]
                            init_expr: Expr [150-153]:
                                ty: Angle(None, true)
                                kind: Lit: Float(7.0)
                    Stmt [159-179]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [159-179]:
                            symbol_id: 27
                            ty_span: [159-164]
                            init_expr: Expr [169-178]:
                                ty: Float(None, false)
                                kind: BinaryOpExpr:
                                    op: Add
                                    lhs: Expr [169-170]:
                                        ty: Angle(None, false)
                                        kind: SymbolId(26)
                                    rhs: Expr [173-178]:
                                        ty: Float(None, false)
                                        kind: Cast [0-0]:
                                            ty: Float(None, false)
                                            expr: Expr [173-178]:
                                                ty: Bool(true)
                                                kind: Lit: Bool(false)
                    Stmt [74-84]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [74-84]:
                            symbol_id: 29
                            ty_span: [74-77]
                            init_expr: Expr [82-83]:
                                ty: Err
                                kind: SymbolId(28)

            [Qsc.Qasm3.Compile.UndefinedSymbol

              x Undefined symbol: y.
               ,-[source2.qasm:2:14]
             1 | bit x = 1;
             2 |     bool x = y && x; // undefined y, redefine x
               :              ^
               `----
            , Qsc.Qasm3.Compile.CannotCast

              x Cannot cast expression of type Err to type Bool(false)
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
            , Qsc.Qasm3.Compile.CannotCast

              x Cannot cast expression of type Err to type Bit(false)
               ,-[source0.qasm:4:13]
             3 |     include "source1.qasm";
             4 |     bit c = r; // undefined symbol r
               :             ^
             5 |     
               `----
            ]"#]],
    );
}
