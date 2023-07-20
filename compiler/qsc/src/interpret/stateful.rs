// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::{
    compile::{self, compile},
    error::WithSource,
};
use miette::Diagnostic;
use qsc_data_structures::index_map::IndexMap;
use qsc_eval::{
    backend::SparseSim,
    debug::{map_fir_package_to_hir, map_hir_package_to_fir, Frame},
    eval_stmt,
    output::Receiver,
    val::{GlobalId, Value},
    Env, Global, GlobalLookup, State,
};

use qsc_fir::{
    fir::{
        Block, BlockId, CallableDecl, Expr, ExprId, LocalItemId, NodeId, Package, PackageId, Pat,
        PatId, Stmt, StmtId,
    },
    visit::{self, Visitor},
};
use qsc_frontend::{
    compile::{CompileUnit, PackageStore, Source, SourceMap},
    incremental::{self, Compiler, Fragment},
};
use qsc_passes::{PackageType, PassContext};
use std::{collections::HashSet, sync::Arc};
use thiserror::Error;

use super::{debug::format_call_stack, stateless};

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub struct Error(WithSource<Source, ErrorKind>);

impl Error {
    #[must_use]
    pub fn stack_trace(&self) -> &Option<String> {
        self.0.stack_trace()
    }
}

#[derive(Clone, Debug, Diagnostic, Error)]
enum ErrorKind {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Compile(#[from] compile::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Pass(#[from] qsc_passes::Error),
    #[error("runtime error")]
    #[diagnostic(transparent)]
    Eval(#[from] qsc_eval::Error),
    #[error("entry point not found")]
    #[diagnostic(code("Qsc.Interpret.NoEntryPoint"))]
    NoEntryPoint,
}

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub struct CompileError(WithSource<Source, compile::Error>);

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub struct LineError(WithSource<Arc<str>, LineErrorKind>);

impl LineError {
    #[must_use]
    pub fn kind(&self) -> &LineErrorKind {
        self.0.error()
    }

    #[must_use]
    pub fn stack_trace(&self) -> &Option<String> {
        self.0.stack_trace()
    }
}

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
pub enum LineErrorKind {
    #[error(transparent)]
    Compile(#[from] incremental::Error),
    #[error(transparent)]
    Pass(#[from] qsc_passes::Error),
    #[error("runtime error")]
    Eval(#[from] qsc_eval::Error),
}

struct Lookup<'a> {
    fir_store: &'a IndexMap<PackageId, qsc_fir::fir::Package>,
    package: PackageId,
    udts: &'a HashSet<LocalItemId>,
    callables: &'a IndexMap<LocalItemId, CallableDecl>,
}

impl<'a> Lookup<'a> {
    fn get_package(&self, package: PackageId) -> &qsc_fir::fir::Package {
        self.fir_store
            .get(package)
            .expect("Package should be in FIR store")
    }
}

impl<'a> GlobalLookup for Lookup<'a> {
    fn get(&self, id: GlobalId) -> Option<Global<'a>> {
        get_global(self.fir_store, self.udts, self.callables, self.package, id)
    }
    fn get_block(&self, package: PackageId, id: BlockId) -> &qsc_fir::fir::Block {
        self.get_package(package)
            .blocks
            .get(id)
            .expect("BlockId should have been lowered")
    }
    fn get_expr(&self, package: PackageId, id: ExprId) -> &qsc_fir::fir::Expr {
        self.get_package(package)
            .exprs
            .get(id)
            .expect("ExprId should have been lowered")
    }
    fn get_pat(&self, package: PackageId, id: PatId) -> &qsc_fir::fir::Pat {
        self.get_package(package)
            .pats
            .get(id)
            .expect("PatId should have been lowered")
    }
    fn get_stmt(&self, package: PackageId, id: StmtId) -> &qsc_fir::fir::Stmt {
        self.get_package(package)
            .stmts
            .get(id)
            .expect("StmtId should have been lowered")
    }
}

pub struct Interpreter {
    store: PackageStore,
    package: PackageId,
    compiler: Compiler,
    udts: HashSet<LocalItemId>,
    callables: IndexMap<LocalItemId, CallableDecl>,
    passes: PassContext,
    env: Env,
    sim: SparseSim,
    lowerer: qsc_eval::lower::Lowerer,
    fir_store: IndexMap<PackageId, qsc_fir::fir::Package>,
    state: State,
    source_package: PackageId,
}

impl Interpreter {
    /// # Errors
    /// If the compilation of the standard library fails, an error is returned.
    /// If the compilation of the sources fails, an error is returned.
    pub fn new(std: bool) -> Result<Self, Vec<Error>> {
        Self::new_with_context(std, SourceMap::default(), PackageType::Lib)
    }

    /// # Errors
    /// If the compilation of the standard library fails, an error is returned.
    /// If the compilation of the sources fails, an error is returned.
    pub fn new_with_context(
        std: bool,
        sources: SourceMap,
        package_type: PackageType,
    ) -> Result<Self, Vec<Error>> {
        let mut lowerer = qsc_eval::lower::Lowerer::new();
        let core = compile::core();
        let core_fir = lowerer.lower_package(&core.package);
        let mut fir_store = IndexMap::new();
        fir_store.insert(
            map_hir_package_to_fir(qsc_hir::hir::PackageId::CORE),
            core_fir,
        );
        let mut store = PackageStore::new(core);
        let mut dependencies = Vec::new();
        if std {
            let std = compile::std(&store);
            let std_fir = lowerer.lower_package(&std.package);
            let id = store.insert(std);
            fir_store.insert(map_hir_package_to_fir(id), std_fir);
            dependencies.push(id);
        }

        let (unit, errors) = compile(&store, &dependencies, sources, package_type);
        if !errors.is_empty() {
            return Err(errors
                .into_iter()
                .map(|error| Error(WithSource::from_map(&unit.sources, error.into(), None)))
                .collect());
        }

        let user_fir = lowerer.lower_package(&unit.package);
        let package = store.insert(unit);
        fir_store.insert(map_hir_package_to_fir(package), user_fir);

        dependencies.push(package);
        let unit = CompileUnit::default();
        let user_fir = lowerer.lower_package(&unit.package);
        let user_package = store.insert(unit);
        fir_store.insert(map_hir_package_to_fir(user_package), user_fir);
        let compiler = Compiler::new(&store, dependencies);
        Ok(Self {
            store,
            package: map_hir_package_to_fir(user_package),
            source_package: map_hir_package_to_fir(package),
            compiler,
            udts: HashSet::new(),
            callables: IndexMap::new(),
            passes: PassContext::default(),
            env: Env::with_empty_scope(),
            sim: SparseSim::new(),
            state: State::new(map_hir_package_to_fir(package)),
            lowerer,
            fir_store,
        })
    }

    /// # Errors
    ///
    /// Returns a vector of errors if loading the entry point fails.
    pub fn set_entry(&mut self) -> Result<(), Vec<Error>> {
        let expr = self.get_entry_expr()?;
        qsc_eval::eval_push_expr(&mut self.state, expr);
        Ok(())
    }

    pub fn get_result(&mut self) -> Value {
        self.state.get_result()
    }

    /// # Errors
    ///
    /// Returns a vector of errors if evaluating the entry point fails.
    pub fn eval_continue(
        &mut self,
        receiver: &mut impl Receiver,
        breakpoints: &[NodeId],
    ) -> Result<Option<NodeId>, Vec<Error>> {
        let globals = Lookup {
            fir_store: &self.fir_store,
            package: self.package,
            udts: &self.udts,
            callables: &self.callables,
        };

        qsc_eval::eval_continue(
            &mut self.state,
            &globals,
            &mut self.env,
            &mut self.sim,
            receiver,
            breakpoints,
        )
        .map_err(|(error, call_stack)| {
            let package = self
                .store
                .get(map_fir_package_to_hir(self.package))
                .expect("package should be in store");

            let stack_trace = if call_stack.is_empty() {
                None
            } else {
                Some(self.render_call_stack(call_stack, &error))
            };

            vec![Error(WithSource::from_map(
                &package.sources,
                error.into(),
                stack_trace,
            ))]
        })
    }

    /// # Errors
    ///
    /// Returns a vector of errors if evaluating the entry point fails.
    pub fn eval_entry(&mut self, receiver: &mut impl Receiver) -> Result<Value, Vec<Error>> {
        let expr = self.get_entry_expr()?;
        let globals = Lookup {
            fir_store: &self.fir_store,
            package: self.package,
            udts: &self.udts,
            callables: &self.callables,
        };

        qsc_eval::eval_expr(
            &mut self.state,
            expr,
            &globals,
            &mut self.env,
            &mut self.sim,
            receiver,
        )
        .map_err(|(error, call_stack)| {
            let package = self
                .store
                .get(map_fir_package_to_hir(self.package))
                .expect("package should be in store");

            let stack_trace = if call_stack.is_empty() {
                None
            } else {
                Some(self.render_call_stack(call_stack, &error))
            };

            vec![Error(WithSource::from_map(
                &package.sources,
                error.into(),
                stack_trace,
            ))]
        })
    }
    fn get_entry_expr(&self) -> Result<ExprId, Vec<Error>> {
        let unit = self
            .fir_store
            .get(self.source_package)
            .expect("store should have package");
        if let Some(entry) = unit.entry {
            return Ok(entry);
        };
        let unit = self
            .store
            .get(map_fir_package_to_hir(self.source_package))
            .expect("store should have package");
        Err(vec![Error(WithSource::from_map(
            &unit.sources,
            ErrorKind::NoEntryPoint,
            None,
        ))])
    }

    /// # Errors
    /// If the parsing of the line fails, an error is returned.
    /// If the compilation of the line fails, an error is returned.
    /// If there is a runtime error when interpreting the line, an error is returned.
    pub fn interpret_line(
        &mut self,
        receiver: &mut impl Receiver,
        line: &str,
    ) -> Result<Value, Vec<LineError>> {
        let mut result = Value::unit();

        let mut fragments = self.compiler.compile_fragments(line).map_err(|errors| {
            let source = line.into();
            errors
                .into_iter()
                .map(|error| LineError(WithSource::new(Arc::clone(&source), error.into(), None)))
                .collect::<Vec<_>>()
        })?;

        let pass_errors = fragments
            .iter_mut()
            .flat_map(|fragment| {
                self.passes
                    .run(self.store.core(), self.compiler.assigner_mut(), fragment)
            })
            .collect::<Vec<_>>();
        if !pass_errors.is_empty() {
            let source = line.into();
            return Err(pass_errors
                .into_iter()
                .map(|error| LineError(WithSource::new(Arc::clone(&source), error.into(), None)))
                .collect());
        }

        for fragment in fragments {
            match fragment {
                Fragment::Item(item) => match item.kind {
                    qsc_hir::hir::ItemKind::Callable(callable) => {
                        let callable = self.lower_callable_decl(&callable);

                        self.callables
                            .insert(qsc_eval::lower::lower_local_item_id(item.id), callable);
                    }
                    qsc_hir::hir::ItemKind::Namespace(..) => {}
                    qsc_hir::hir::ItemKind::Ty(..) => {
                        self.udts
                            .insert(qsc_eval::lower::lower_local_item_id(item.id));
                    }
                },
                Fragment::Stmt(stmt) => {
                    let stmt_id = self.lower_stmt(&stmt);

                    match self.eval_stmt(receiver, stmt_id) {
                        Ok(value) => result = value,
                        Err((error, call_stack)) => {
                            let stack_trace = if call_stack.is_empty() {
                                None
                            } else {
                                Some(self.render_call_stack(call_stack, &error))
                            };

                            return Err(vec![LineError(WithSource::new(
                                line.into(),
                                error.into(),
                                stack_trace,
                            ))]);
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    fn lower_callable_decl(&mut self, callable: &qsc_hir::hir::CallableDecl) -> CallableDecl {
        let callable = self.lowerer.lower_callable_decl(callable);
        self.update_fir();
        callable
    }

    fn lower_stmt(&mut self, stmt: &qsc_hir::hir::Stmt) -> StmtId {
        let stmt_id = self.lowerer.lower_stmt(stmt);
        self.update_fir();
        stmt_id
    }

    fn update_fir(&mut self) {
        let package = self.fir_store.get_mut(self.package).expect("msg");

        for (id, value) in self.lowerer.blocks.iter() {
            if !package.blocks.contains_key(id) {
                package.blocks.insert(id, value.clone());
            }
        }
        for (id, value) in self.lowerer.exprs.iter() {
            if !package.exprs.contains_key(id) {
                package.exprs.insert(id, value.clone());
            }
        }
        for (id, value) in self.lowerer.pats.iter() {
            if !package.pats.contains_key(id) {
                package.pats.insert(id, value.clone());
            }
        }
        for (id, value) in self.lowerer.stmts.iter() {
            if !package.stmts.contains_key(id) {
                package.stmts.insert(id, value.clone());
            }
        }
        self.lowerer.blocks.clear();
        self.lowerer.exprs.clear();
        self.lowerer.pats.clear();
        self.lowerer.stmts.clear();
    }

    fn eval_stmt(
        &mut self,
        receiver: &mut impl Receiver,
        stmt: StmtId,
    ) -> Result<Value, (qsc_eval::Error, Vec<Frame>)> {
        let globals = Lookup {
            fir_store: &self.fir_store,
            package: self.package,
            udts: &self.udts,
            callables: &self.callables,
        };

        eval_stmt(
            stmt,
            &globals,
            &mut self.env,
            &mut self.sim,
            self.package,
            receiver,
        )
    }

    fn render_call_stack(&self, call_stack: Vec<Frame>, error: &dyn std::error::Error) -> String {
        let globals = Lookup {
            fir_store: &self.fir_store,
            package: self.package,
            udts: &self.udts,
            callables: &self.callables,
        };
        format_call_stack(&self.store, &globals, call_stack, error)
    }

    #[must_use]
    pub fn get_stack_frames(&self) -> Vec<StackFrame> {
        let globals = Lookup {
            fir_store: &self.fir_store,
            package: self.package,
            udts: &self.udts,
            callables: &self.callables,
        };
        let frames = self.state.get_stack_frames();
        let stack_frames = frames
            .iter()
            .map(|frame| {
                let callable = globals.get(frame.id).expect("frame should exist");
                let functor = format!("{}", frame.functor);
                let name = match callable {
                    Global::Callable(decl) => decl.name.name.to_string(),
                    Global::Udt => "udt".into(),
                };

                let hir_package = self
                    .store
                    .get(map_fir_package_to_hir(frame.id.package))
                    .expect("package should exist");
                let source = hir_package
                    .sources
                    .find_by_offset(frame.span.lo)
                    .expect("frame should have a source");
                let path = source.name.to_string();
                StackFrame {
                    name,
                    functor,
                    path,
                    lo: frame.span.lo,
                    hi: frame.span.hi,
                }
            })
            .collect();
        stack_frames
    }

    #[must_use]
    pub fn get_breakpoints(&self, path: &str) -> Vec<BreakpointSpan> {
        let unit = self
            .store
            .get(map_fir_package_to_hir(self.source_package))
            .expect("Could not load package");

        if let Some(source) = unit.sources.find_by_name(path) {
            let package = self
                .fir_store
                .get(self.source_package)
                .expect("package should have been lowered");
            let mut colllector = BreakpointCollector::new(&unit.sources, source.offset, package);
            colllector.visit_package(package);
            colllector
                .statements
                .iter()
                .map(|bps| BreakpointSpan {
                    id: bps.id,
                    lo: bps.lo - source.offset,
                    hi: bps.hi - source.offset,
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

pub struct StackFrame {
    pub name: String,
    pub functor: String,
    pub path: String,
    pub lo: u32,
    pub hi: u32,
}

fn get_global<'a>(
    fir_store: &'a IndexMap<PackageId, qsc_fir::fir::Package>,
    udts: &'a HashSet<LocalItemId>,
    callables: &'a IndexMap<LocalItemId, CallableDecl>,
    package: PackageId,
    id: GlobalId,
) -> Option<Global<'a>> {
    if id.package == package {
        udts.contains(&id.item)
            .then_some(Global::Udt)
            .or_else(|| callables.get(id.item).map(Global::Callable))
    } else {
        stateless::get_global(fir_store, id)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BreakpointSpan {
    pub id: u32,
    pub lo: u32,
    pub hi: u32,
}

struct BreakpointCollector<'a> {
    statements: HashSet<BreakpointSpan>,
    sources: &'a SourceMap,
    offset: u32,
    package: &'a Package,
}

impl<'a> BreakpointCollector<'a> {
    fn new(sources: &'a SourceMap, offset: u32, package: &'a Package) -> Self {
        Self {
            statements: HashSet::new(),
            sources,
            offset,
            package,
        }
    }

    fn get_source(&self, offset: u32) -> &Source {
        self.sources
            .find_by_offset(offset)
            .expect("Couldn't find source file")
    }

    fn add_stmt(&mut self, stmt: &qsc_fir::fir::Stmt) {
        let source: &Source = self.get_source(self.offset);
        if source.offset == self.offset {
            self.statements.insert(BreakpointSpan {
                id: stmt.id.into(),
                lo: stmt.span.lo,
                hi: stmt.span.hi,
            });
        }
    }
}

impl<'a> Visitor<'a> for BreakpointCollector<'a> {
    fn visit_stmt(&mut self, stmt: StmtId) {
        let stmt_res = self.get_stmt(stmt);
        self.add_stmt(stmt_res);

        visit::walk_stmt(self, stmt);
    }

    fn get_block(&mut self, id: BlockId) -> &'a Block {
        self.package
            .blocks
            .get(id)
            .expect("couldn't find block in FIR")
    }

    fn get_expr(&mut self, id: ExprId) -> &'a Expr {
        self.package
            .exprs
            .get(id)
            .expect("couldn't find expr in FIR")
    }

    fn get_pat(&mut self, id: PatId) -> &'a Pat {
        self.package.pats.get(id).expect("couldn't find pat in FIR")
    }

    fn get_stmt(&mut self, id: StmtId) -> &'a Stmt {
        self.package
            .stmts
            .get(id)
            .expect("couldn't find stmt in FIR")
    }
}
