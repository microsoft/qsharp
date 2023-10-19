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
    validate::Validator as AstValidator,
    visit::Visitor as AstVisitor,
};
use qsc_hir::{
    assigner::Assigner as HirAssigner,
    hir::{self, PackageId},
    validate::Validator as HirValidator,
    visit::Visitor as HirVisitor,
};
use std::mem::take;

/// The frontend for an incremental compiler.
/// It is used to update a single `CompileUnit`
/// with additional sources.
pub struct Compiler {
    ast_assigner: AstAssigner,
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
        self.compile_fragments_acc_errors(unit, source_name, source_contents, fail_on_error)
    }

    pub fn compile_fragments_acc_errors<F>(
        &mut self,
        unit: &mut CompileUnit,
        source_name: &str,
        source_contents: &str,
        mut accumulate_errors: F,
    ) -> Result<Increment, Vec<Error>>
    where
        F: FnMut(Vec<Error>) -> Result<(), Vec<Error>>,
    {
        let (mut ast, parse_errors) =
            Self::parse_fragments(&mut unit.sources, source_name, source_contents);

        accumulate_errors(parse_errors)?;

        let hir = self.resolve_check_lower(unit, &mut ast, accumulate_errors)?;

        Ok(Increment {
            ast: AstPackage {
                package: ast,
                names: self.resolver.names().clone(),
                tys: self.checker.table().clone(),
            },
            hir,
        })
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

        let package = self.resolve_check_lower(unit, &mut ast, fail_on_error)?;

        Ok(Increment {
            ast: AstPackage {
                package: ast,
                names: self.resolver.names().clone(),
                tys: self.checker.table().clone(),
            },
            hir: package,
        })
    }

    pub fn update(&mut self, unit: &mut CompileUnit, new: Increment) {
        // Update the AST
        unit.ast.package = self.concat_ast(take(&mut unit.ast.package), new.ast.package);

        // The new `Increment` will contain the names and tys
        // from the original package as well, so just
        // replace the current tables instead of extending.
        unit.ast.names = new.ast.names;
        unit.ast.tys = new.ast.tys;

        // Update the HIR
        extend_hir(&mut unit.package, new.hir);
    }

    fn resolve_check_lower<F>(
        &mut self,
        unit: &mut CompileUnit,
        ast: &mut ast::Package,
        mut check_errors: F,
    ) -> Result<hir::Package, Vec<Error>>
    where
        F: FnMut(Vec<Error>) -> Result<(), Vec<Error>>,
    {
        let mut cond_compile = preprocess::Conditional::new(self.target);
        cond_compile.visit_package(ast);

        self.ast_assigner.visit_package(ast);

        self.resolver
            .extend_dropped_names(cond_compile.into_names());
        self.resolver.bind_fragments(ast, &mut unit.assigner);
        self.resolver.with(&mut unit.assigner).visit_package(ast);

        let resolve_errors = self
            .resolver
            .drain_errors()
            .map(|e| WithSource::from_map(&unit.sources, compile::Error(e.into())))
            .collect();
        check_errors(resolve_errors)?;

        self.checker.check_package(self.resolver.names(), ast);
        self.checker.solve(self.resolver.names());

        let ty_errors = self
            .checker
            .drain_errors()
            .map(|e| WithSource::from_map(&unit.sources, compile::Error(e.into())))
            .collect();
        check_errors(ty_errors)?;

        let package = self.lower(&mut unit.assigner, &*ast);

        let lower_errors: Vec<Error> = self
            .lowerer
            .drain_errors()
            .map(|e| WithSource::from_map(&unit.sources, compile::Error(e.into())))
            .collect();

        if !lower_errors.is_empty() {
            self.lowerer.clear_items();
        }

        check_errors(lower_errors)?;

        Ok(package)
    }

    /// Creates a new `Package` by combining two packages.
    /// The two packages should not contain any conflicting `NodeId`s.
    /// Entry expressions are ignored.
    #[must_use]
    fn concat_ast(&mut self, mut left: ast::Package, right: ast::Package) -> ast::Package {
        assert!(right.entry.is_none(), "package should not have entry expr");
        assert!(left.entry.is_none(), "package should not have entry expr");

        let mut nodes = Vec::with_capacity(left.nodes.len() + right.nodes.len());
        nodes.extend(left.nodes.into_vec());
        nodes.extend(right.nodes.into_vec());
        left.id = self.ast_assigner.next_id();
        left.nodes = nodes.into_boxed_slice();

        AstValidator::default().visit_package(&left);
        left
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
}

/// Extends the `Package` with the contents of another `Package`.
/// `other` should not contain any `LocalItemId`s
/// that conflict with the current `Package`.
/// The entry expression from `other` will be ignored.
fn extend_hir(this: &mut hir::Package, mut other: hir::Package) {
    for (k, v) in other.items.drain() {
        this.items.insert(k, v);
    }

    this.stmts.extend(other.stmts);

    HirValidator::default().visit_package(this);
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

fn fail_on_error(errors: Vec<Error>) -> Result<(), Vec<Error>> {
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(())
}
