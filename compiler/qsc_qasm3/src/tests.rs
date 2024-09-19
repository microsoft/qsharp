// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    parse::{QasmParseResult, QasmSource},
    qasm_to_program, CompilerConfig, OutputSemantics, ProgramType, QasmCompileUnit, QubitSemantics,
};
use miette::Report;
use qsc::interpret::Error;
use qsc::{
    ast::{mut_visit::MutVisitor, Package, Stmt, TopLevelNode},
    target::Profile,
    PackageStore, SourceMap, Span,
};
use std::{path::Path, sync::Arc};

use crate::{
    io::{InMemorySourceResolver, SourceResolver},
    parse::parse_source,
};

pub(crate) mod assignment;
pub(crate) mod declaration;
pub(crate) mod expression;
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
    let mut store = PackageStore::new(qsc::compile::core());
    let mut dependencies = Vec::new();
    let capabilities = profile.into();
    dependencies.push((store.insert(qsc::compile::std(&store, capabilities)), None));

    qsc::codegen::qir::get_qir_from_ast(
        &mut store,
        &dependencies,
        ast_package,
        source_map,
        capabilities,
    )
}

fn compile_qasm_to_qir(source: &str, profile: Profile) -> Result<String, Vec<Report>> {
    let res = parse(source)?;
    assert!(!res.has_errors());

    let unit = qasm_to_program(
        res.source,
        res.source_map,
        CompilerConfig::new(
            QubitSemantics::Qiskit,
            OutputSemantics::Qiskit,
            ProgramType::File,
            Some("Test".into()),
            None,
        ),
    );
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

pub(crate) fn parse_all<P>(
    path: P,
    sources: impl IntoIterator<Item = (Arc<str>, Arc<str>)>,
) -> miette::Result<QasmParseResult, Vec<Report>>
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

pub fn qasm_to_program_fragments(source: QasmSource, source_map: SourceMap) -> QasmCompileUnit {
    qasm_to_program(
        source,
        source_map,
        CompilerConfig::new(
            QubitSemantics::Qiskit,
            OutputSemantics::OpenQasm,
            ProgramType::Fragments,
            None,
            None,
        ),
    )
}

pub fn compile_qasm_to_qsharp_file(source: &str) -> miette::Result<String, Vec<Report>> {
    let res = parse(source)?;
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
    let res = parse(source)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program(
        res.source,
        res.source_map,
        CompilerConfig::new(
            QubitSemantics::Qiskit,
            OutputSemantics::OpenQasm,
            ProgramType::Operation,
            Some("Test".into()),
            None,
        ),
    );
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
    let res = parse(source)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program(
        res.source,
        res.source_map,
        CompilerConfig::new(
            qubit_semantics,
            OutputSemantics::Qiskit,
            ProgramType::Fragments,
            None,
            None,
        ),
    );
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
    let res = parse(source)?;
    assert!(!res.has_errors());
    let unit = qasm_to_program(
        res.source,
        res.source_map,
        CompilerConfig::new(
            qubit_semantics,
            OutputSemantics::Qiskit,
            ProgramType::Fragments,
            None,
            None,
        ),
    );
    if unit.has_errors() {
        let errors = unit.errors.into_iter().map(Report::new).collect();
        return Err(errors);
    }
    let Some(package) = unit.package else {
        panic!("Expected package, got None");
    };
    let qsharp = get_first_statement_as_qsharp(&package);
    Ok(qsharp)
}

fn get_first_statement_as_qsharp(package: &Package) -> String {
    let qsharp = match package.nodes.first() {
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
