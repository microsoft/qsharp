// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::io::{InMemorySourceResolver, SourceResolver};
use crate::semantic::{parse_source, QasmSemanticParseResult};
use crate::stdlib::compile::package_store_with_qasm;
use crate::{
    compile_to_qsharp_ast_with_config, CompilerConfig, OutputSemantics, ProgramType,
    QasmCompileUnit, QubitSemantics,
};
use miette::Report;
use qsc::compile::compile_ast;
use qsc::interpret::Error;
use qsc::target::Profile;
use qsc::{
    ast::{mut_visit::MutVisitor, Package, Stmt, TopLevelNode},
    SourceMap, Span,
};
use qsc_hir::hir::PackageId;
use qsc_passes::PackageType;
use std::{path::Path, sync::Arc};

pub(crate) mod assignment;
pub(crate) mod declaration;
pub(crate) mod expression;
pub(crate) mod fuzz;
pub(crate) mod output;
pub(crate) mod sample_circuits;
pub(crate) mod scopes;
pub(crate) mod statement;

pub(crate) fn fail_on_compilation_errors(unit: &QasmCompileUnit) {
    if unit.has_errors() {
        print_compilation_errors(unit);
        panic!("Errors found in compilation");
    }
}

pub(crate) fn print_compilation_errors(unit: &QasmCompileUnit) {
    if unit.has_errors() {
        for e in unit.errors() {
            println!("{:?}", Report::new(e.clone()));
        }
    }
}

pub(crate) fn gen_qsharp(package: &Package) -> String {
    qsc::codegen::qsharp::write_package_string(package)
}

/// Generates QIR from an AST package.
/// This function is used for testing purposes only.
/// The interactive environment uses a different mechanism to generate QIR.
/// As we need an entry expression to generate QIR in those cases.
///
/// This function assumes that the AST package was designed as an entry point.
pub(crate) fn generate_qir_from_ast(
    ast_package: Package,
    source_map: SourceMap,
    profile: Profile,
) -> Result<String, Vec<Error>> {
    let capabilities = profile.into();
    let (stdid, qasmid, mut store) = package_store_with_qasm(capabilities);
    let dependencies = vec![
        (PackageId::CORE, None),
        (stdid, None),
        (qasmid, Some("QasmStd".into())),
    ];
    qsc::codegen::qir::get_qir_from_ast(
        &mut store,
        &dependencies,
        ast_package,
        source_map,
        capabilities,
    )
}

fn compile<S>(source: S) -> miette::Result<QasmCompileUnit, Vec<Report>>
where
    S: AsRef<str>,
{
    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::Qiskit,
        ProgramType::File,
        Some("Test".into()),
        None,
    );
    compile_with_config(source, config)
}

fn compile_with_config<S>(
    source: S,
    config: CompilerConfig,
) -> miette::Result<QasmCompileUnit, Vec<Report>>
where
    S: AsRef<str>,
{
    let res = parse(source)?;
    if res.has_syntax_errors() {
        for e in res.sytax_errors() {
            println!("{:?}", Report::new(e.clone()));
        }
    }
    assert!(!res.has_syntax_errors());
    let program = res.program;

    let compiler = crate::compiler::QasmCompiler {
        source_map: res.source_map,
        config,
        stmts: vec![],
        symbols: res.symbols,
        errors: res.errors,
    };

    let unit = compiler.compile(&program);
    Ok(unit)
}

pub fn compile_all<P>(
    path: P,
    sources: impl IntoIterator<Item = (Arc<str>, Arc<str>)>,
) -> miette::Result<QasmCompileUnit, Vec<Report>>
where
    P: AsRef<Path>,
{
    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::Qiskit,
        ProgramType::File,
        Some("Test".into()),
        None,
    );
    compile_all_with_config(path, sources, config)
}

pub fn compile_all_fragments<P>(
    path: P,
    sources: impl IntoIterator<Item = (Arc<str>, Arc<str>)>,
) -> miette::Result<QasmCompileUnit, Vec<Report>>
where
    P: AsRef<Path>,
{
    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::Fragments,
        None,
        None,
    );
    compile_all_with_config(path, sources, config)
}

fn compile_fragments<S>(source: S) -> miette::Result<QasmCompileUnit, Vec<Report>>
where
    S: AsRef<str>,
{
    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::Fragments,
        None,
        None,
    );
    compile_with_config(source, config)
}

pub fn compile_all_with_config<P>(
    path: P,
    sources: impl IntoIterator<Item = (Arc<str>, Arc<str>)>,
    config: CompilerConfig,
) -> miette::Result<QasmCompileUnit, Vec<Report>>
where
    P: AsRef<Path>,
{
    let res = parse_all(path, sources)?;
    assert!(!res.has_syntax_errors());
    let program = res.program;

    let compiler = crate::compiler::QasmCompiler {
        source_map: res.source_map,
        config,
        stmts: vec![],
        symbols: res.symbols,
        errors: res.errors,
    };

    let unit = compiler.compile(&program);
    Ok(unit)
}

fn compile_qasm_to_qir(source: &str, profile: Profile) -> Result<String, Vec<Report>> {
    let unit = compile(source)?;
    fail_on_compilation_errors(&unit);
    let package = unit.package.expect("no package found");
    let qir = generate_qir_from_ast(package, unit.source_map, profile).map_err(|errors| {
        errors
            .iter()
            .map(|e| Report::new(e.clone()))
            .collect::<Vec<_>>()
    })?;
    Ok(qir)
}

/// used to do full compilation with best effort of the input.
/// This is useful for fuzz testing.
fn compile_qasm_best_effort(source: &str, profile: Profile) {
    let (stdid, qasmid, store) = package_store_with_qasm(profile.into());

    let mut resolver = InMemorySourceResolver::from_iter([]);
    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::File,
        Some("Fuzz".into()),
        None,
    );

    let unit =
        compile_to_qsharp_ast_with_config(source, "source.qasm", Some(&mut resolver), config);
    let (sources, _, package, _) = unit.into_tuple();

    let dependencies = vec![
        (PackageId::CORE, None),
        (stdid, None),
        (qasmid, Some("QasmStd".into())),
    ];

    let (mut _unit, _errors) = compile_ast(
        &store,
        &dependencies,
        package.expect("package must exist"),
        sources,
        PackageType::Lib,
        profile.into(),
    );
}

pub(crate) fn gen_qsharp_stmt(stmt: &Stmt) -> String {
    qsc::codegen::qsharp::write_stmt_string(stmt)
}

#[allow(dead_code)]
pub(crate) fn compare_compilation_to_qsharp(unit: &QasmCompileUnit, expected: &str) {
    let package = unit.package.as_ref().expect("package must exist");
    let despanned_ast = AstDespanner.despan(package);
    let qsharp = gen_qsharp(&despanned_ast);
    difference::assert_diff!(&qsharp, expected, "\n", 0);
}

pub(crate) fn parse<S>(source: S) -> miette::Result<QasmSemanticParseResult, Vec<Report>>
where
    S: AsRef<str>,
{
    let mut resolver =
        InMemorySourceResolver::from_iter([("Test.qasm".into(), source.as_ref().into())]);
    let res = parse_source(source, "Test.qasm", &mut resolver);
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

pub(crate) fn parse_all<P>(
    path: P,
    sources: impl IntoIterator<Item = (Arc<str>, Arc<str>)>,
) -> miette::Result<QasmSemanticParseResult, Vec<Report>>
where
    P: AsRef<Path>,
{
    let mut resolver = InMemorySourceResolver::from_iter(sources);
    let source = resolver
        .resolve(path.as_ref())
        .map_err(|e| vec![Report::new(e)])?
        .1;
    let res = parse_source(source, path, &mut resolver);
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

pub fn compile_qasm_to_qsharp_file(source: &str) -> miette::Result<String, Vec<Report>> {
    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::File,
        Some("Test".into()),
        None,
    );
    let unit = compile_with_config(source, config)?;
    if unit.has_errors() {
        let errors = unit.errors.into_iter().map(Report::new).collect();
        return Err(errors);
    }
    let Some(package) = unit.package else {
        panic!("Expected package, got None");
    };
    let qsharp = gen_qsharp(&package);
    Ok(qsharp)
}

pub fn compile_qasm_to_qsharp_operation(source: &str) -> miette::Result<String, Vec<Report>> {
    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::Operation,
        Some("Test".into()),
        None,
    );
    let unit = compile_with_config(source, config)?;
    if unit.has_errors() {
        let errors = unit.errors.into_iter().map(Report::new).collect();
        return Err(errors);
    }
    let Some(package) = unit.package else {
        panic!("Expected package, got None");
    };
    let qsharp = gen_qsharp(&package);
    Ok(qsharp)
}

pub fn compile_qasm_to_qsharp(source: &str) -> miette::Result<String, Vec<Report>> {
    compile_qasm_to_qsharp_with_semantics(source, QubitSemantics::Qiskit)
}

pub fn compile_qasm_to_qsharp_with_semantics(
    source: &str,
    qubit_semantics: QubitSemantics,
) -> miette::Result<String, Vec<Report>> {
    let config = CompilerConfig::new(
        qubit_semantics,
        OutputSemantics::Qiskit,
        ProgramType::Fragments,
        None,
        None,
    );
    let unit = compile_with_config(source, config)?;
    qsharp_from_qasm_compilation(unit)
}

pub fn qsharp_from_qasm_compilation(unit: QasmCompileUnit) -> miette::Result<String, Vec<Report>> {
    if unit.has_errors() {
        let errors = unit.errors.into_iter().map(Report::new).collect();
        return Err(errors);
    }
    let Some(package) = unit.package else {
        panic!("Expected package, got None");
    };
    let qsharp = gen_qsharp(&package);
    Ok(qsharp)
}

pub fn compile_qasm_stmt_to_qsharp(source: &str) -> miette::Result<String, Vec<Report>> {
    compile_qasm_stmt_to_qsharp_with_semantics(source, QubitSemantics::Qiskit)
}

pub fn compile_qasm_stmt_to_qsharp_with_semantics(
    source: &str,
    qubit_semantics: QubitSemantics,
) -> miette::Result<String, Vec<Report>> {
    let config = CompilerConfig::new(
        qubit_semantics,
        OutputSemantics::Qiskit,
        ProgramType::Fragments,
        None,
        None,
    );
    let unit = compile_with_config(source, config)?;
    if unit.has_errors() {
        let errors = unit.errors.into_iter().map(Report::new).collect();
        return Err(errors);
    }
    let Some(package) = unit.package else {
        panic!("Expected package, got None");
    };
    let qsharp = get_last_statement_as_qsharp(&package);
    Ok(qsharp)
}

fn get_last_statement_as_qsharp(package: &Package) -> String {
    let qsharp = match package.nodes.iter().last() {
        Some(i) => match i {
            TopLevelNode::Namespace(_) => panic!("Expected Stmt, got Namespace"),
            TopLevelNode::Stmt(stmt) => gen_qsharp_stmt(stmt.as_ref()),
        },
        None => panic!("Expected Stmt, got None"),
    };
    qsharp
}

fn get_first_statement_as_qsharp(package: &Package) -> String {
    let qsharp = match package.nodes.get(1) {
        Some(i) => match i {
            TopLevelNode::Namespace(_) => panic!("Expected Stmt, got Namespace"),
            TopLevelNode::Stmt(stmt) => gen_qsharp_stmt(stmt.as_ref()),
        },
        None => panic!("Expected Stmt, got None"),
    };
    qsharp
}

pub struct AstDespanner;
impl AstDespanner {
    #[allow(dead_code)] // false positive lint
    pub fn despan(&mut self, package: &Package) -> Package {
        let mut p = package.clone();
        self.visit_package(&mut p);
        p
    }
}

impl MutVisitor for AstDespanner {
    fn visit_span(&mut self, span: &mut Span) {
        span.hi = 0;
        span.lo = 0;
    }
}

#[allow(dead_code)]
struct HirDespanner;
impl HirDespanner {
    #[allow(dead_code)]
    fn despan(&mut self, package: &qsc::hir::Package) -> qsc::hir::Package {
        let mut p = package.clone();
        qsc::hir::mut_visit::MutVisitor::visit_package(self, &mut p);
        p
    }
}

impl qsc::hir::mut_visit::MutVisitor for HirDespanner {
    fn visit_span(&mut self, span: &mut Span) {
        span.hi = 0;
        span.lo = 0;
    }
}
