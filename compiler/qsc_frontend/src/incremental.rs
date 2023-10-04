// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::{
    compile::{
        self, preprocess, AstPackage, CompileUnit, Offsetter, PackageStore, SourceMap,
        TargetProfile,
    },
    error::WithSource,
    lower::Lowerer,
    resolve::{self, Resolver},
    typeck::{self, Checker},
};
use qsc_ast::{
    assigner::Assigner as AstAssigner,
    ast::{self, Stmt, TopLevelNode},
    mut_visit::MutVisitor,
    visit::Visitor,
};
use qsc_hir::{
    assigner::Assigner as HirAssigner,
    hir::{self, PackageId},
};

/// The frontend for an incremental compiler.
/// It is used to update a single `CompileUnit`
/// with additional sources.
pub struct Compiler {
    pub ast_assigner: AstAssigner,
    resolver: Resolver,
    checker: Checker,
    lowerer: Lowerer,
    target: TargetProfile,
}

pub type Error = WithSource<compile::Error>;

/// The result of an incremental compilation.
/// These packages can be merged into the original
/// `CompileUnit` that was used for the incremental compilation.
#[derive(Debug)]
pub struct Increment {
    pub ast: AstPackage,
    pub hir: hir::Package,
}

impl Compiler {
    /// Creates a new compiler.
    pub fn new(
        store: &PackageStore,
        dependencies: impl IntoIterator<Item = PackageId>,
        target: TargetProfile,
    ) -> Self {
        let mut resolve_globals = resolve::GlobalTable::new();
        let mut typeck_globals = typeck::GlobalTable::new();
        let mut dropped_names = Vec::new();
        if let Some(unit) = store.get(PackageId::CORE) {
            resolve_globals.add_external_package(PackageId::CORE, &unit.package);
            typeck_globals.add_external_package(PackageId::CORE, &unit.package);
            dropped_names.extend(unit.dropped_names.iter().cloned());
        }

        for id in dependencies {
            let unit = store
                .get(id)
                .expect("dependency should be added to package store before compilation");
            resolve_globals.add_external_package(id, &unit.package);
            typeck_globals.add_external_package(id, &unit.package);
            dropped_names.extend(unit.dropped_names.iter().cloned());
        }

        Self {
            ast_assigner: AstAssigner::new(),
            resolver: Resolver::with_persistent_local_scope(resolve_globals, dropped_names),
            checker: Checker::new(typeck_globals),
            lowerer: Lowerer::new(),
            target,
        }
    }

    /// Compiles Q# fragments.
    ///
    /// Uses the assigners and other mutable state from the passed in
    /// `CompileUnit` to guarantee uniqueness, however does not
    /// update the `CompileUnit` with the resulting AST and HIR packages.
    ///
    /// The caller can use the returned packages to perform passes,
    /// get information about the newly added items, or do other modifications.
    /// It is then the caller's responsibility to merge
    /// these packages into the current `CompileUnit`.
    pub fn compile_fragments(
        &mut self,
        unit: &mut CompileUnit,
        source_name: &str,
        source_contents: &str,
    ) -> Result<Increment, Vec<Error>> {
        let (mut ast, parse_errors) =
            Self::parse_fragments(&mut unit.sources, source_name, source_contents);

        if !parse_errors.is_empty() {
            return Err(parse_errors);
        }

        let (hir, lower_errors) = self.resolve_check_lower(unit, &mut ast);

        if lower_errors.is_empty() {
            Ok(Increment {
                ast: AstPackage {
                    package: ast,
                    names: self.resolver.names().clone(),
                    tys: self.checker.table().clone(),
                },
                hir,
            })
        } else {
            self.lowerer.clear_items();
            Err(lower_errors)
        }
    }

    /// Compiles an entry expression.
    ///
    /// Uses the assigners and other mutable state from the passed in
    /// `CompileUnit` to guarantee uniqueness, however does not
    /// update the `CompileUnit` with the resulting AST and HIR packages.
    ///
    /// The caller can use the returned packages to perform passes,
    /// get information about the newly added items, or do other modifications.
    /// It is then the caller's responsibility to merge
    /// these packages into the current `CompileUnit`.
    pub fn compile_expr(
        &mut self,
        unit: &mut CompileUnit,
        source_name: &str,
        source_contents: &str,
    ) -> Result<Increment, Vec<Error>> {
        let (mut ast, parse_errors) =
            Self::parse_expr(&mut unit.sources, source_name, source_contents);

        if !parse_errors.is_empty() {
            return Err(parse_errors);
        }

        let (package, errors) = self.resolve_check_lower(unit, &mut ast);

        if errors.is_empty() {
            Ok(Increment {
                ast: AstPackage {
                    package: ast,
                    names: self.resolver.names().clone(),
                    tys: self.checker.table().clone(),
                },
                hir: package,
            })
        } else {
            self.lowerer.clear_items();
            Err(errors)
        }
    }

    fn resolve_check_lower(
        &mut self,
        unit: &mut CompileUnit,
        ast: &mut ast::Package,
    ) -> (hir::Package, Vec<Error>) {
        let mut cond_compile = preprocess::Conditional::new(self.target);
        cond_compile.visit_package(ast);

        self.ast_assigner.visit_package(ast);

        self.resolver
            .extend_dropped_names(cond_compile.into_names());
        self.resolver.bind_fragments(ast, &mut unit.assigner);
        self.resolver.with(&mut unit.assigner).visit_package(ast);

        self.checker.check_package(self.resolver.names(), ast);
        self.checker.solve(self.resolver.names());

        let package = self.lower(&mut unit.assigner, &*ast);

        let errors: Vec<Error> = self
            .drain_errors()
            .into_iter()
            .map(|e| WithSource::from_map(&unit.sources, e))
            .collect();

        (package, errors)
    }

    fn parse_expr(
        sources: &mut SourceMap,
        source_name: &str,
        source_contents: &str,
    ) -> (ast::Package, Vec<Error>) {
        let offset = sources.push(source_name.into(), source_contents.into());

        let (expr, errors) = qsc_parse::expr(source_contents);
        let mut stmt = Box::new(Stmt {
            id: ast::NodeId::default(),
            span: expr.span,
            kind: Box::new(ast::StmtKind::Expr(expr)),
        });

        let mut offsetter = Offsetter(offset);
        offsetter.visit_stmt(&mut stmt);

        let top_level_nodes = Box::new([TopLevelNode::Stmt(stmt)]);

        let package = ast::Package {
            id: ast::NodeId::default(),
            nodes: top_level_nodes,
            entry: None,
        };

        (package, with_source(errors, sources, offset))
    }

    fn parse_fragments(
        sources: &mut SourceMap,
        source_name: &str,
        source_contents: &str,
    ) -> (ast::Package, Vec<Error>) {
        let offset = sources.push(source_name.into(), source_contents.into());

        let (mut top_level_nodes, errors) = qsc_parse::top_level_nodes(source_contents);
        let mut offsetter = Offsetter(offset);
        for node in &mut top_level_nodes {
            match node {
                ast::TopLevelNode::Namespace(ns) => offsetter.visit_namespace(ns),
                ast::TopLevelNode::Stmt(stmt) => offsetter.visit_stmt(stmt),
            }
        }

        let package = ast::Package {
            id: ast::NodeId::default(),
            nodes: top_level_nodes.into_boxed_slice(),
            entry: None,
        };

        (package, with_source(errors, sources, offset))
    }

    fn lower(&mut self, hir_assigner: &mut HirAssigner, package: &ast::Package) -> hir::Package {
        self.lowerer
            .with(hir_assigner, self.resolver.names(), self.checker.table())
            .lower_package(package)
    }

    fn drain_errors(&mut self) -> Vec<compile::Error> {
        self.resolver
            .drain_errors()
            .map(|e| compile::Error(e.into()))
            .chain(
                self.checker
                    .drain_errors()
                    .map(|e| compile::Error(e.into())),
            )
            .chain(
                self.lowerer
                    .drain_errors()
                    .map(|e| compile::Error(e.into())),
            )
            .collect()
    }
}

fn with_source(
    errors: Vec<qsc_parse::Error>,
    sources: &SourceMap,
    offset: u32,
) -> Vec<WithSource<compile::Error>> {
    errors
        .into_iter()
        .map(|e| {
            WithSource::from_map(
                sources,
                compile::Error(compile::ErrorKind::Parse(e.with_offset(offset))),
            )
        })
        .collect()
}
