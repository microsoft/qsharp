// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

#[cfg(test)]
mod stepping_tests;

use super::debug::format_call_stack;
use crate::{
    compile::{self, compile},
    error::{self, WithStack},
    hir,
};
use miette::Diagnostic;
use num_bigint::BigUint;
use num_complex::Complex;
use qsc_codegen::qir_base::BaseProfSim;
use qsc_data_structures::{index_map::IndexMap, span::Span};
use qsc_eval::{
    backend::{Backend, SparseSim},
    debug::{map_fir_package_to_hir, map_hir_package_to_fir, Frame},
    eval_expr, eval_stmt,
    output::{GenericReceiver, Receiver},
    val::{self, GlobalId, Value},
    Env, Global, NodeLookup, State, StepAction, StepResult, VariableInfo,
};
use qsc_fir::{
    fir::{
        Block, BlockId, CallableDecl, Expr, ExprId, ItemKind, LocalItemId, Package, PackageId, Pat,
        PatId, Stmt, StmtId,
    },
    visit::{self, Visitor},
};
use qsc_frontend::{
    compile::{CompileUnit, PackageStore, Source, SourceMap, TargetProfile},
    error::WithSource,
    incremental::{self, Compiler, Fragment},
};
use qsc_passes::{PackageType, PassContext};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub struct Error(#[from] ErrorKind);

impl Error {
    #[must_use]
    pub fn stack_trace(&self) -> &Option<String> {
        match &self.0 {
            ErrorKind::Eval(err) => err.stack_trace(),
            _ => &None,
        }
    }
}

#[derive(Clone, Debug, Diagnostic, Error)]
enum ErrorKind {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Compile(#[from] WithSource<compile::Error>),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Incremental(#[from] WithSource<incremental::Error>),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Pass(#[from] WithSource<qsc_passes::Error>),
    #[error("runtime error")]
    #[diagnostic(transparent)]
    Eval(#[from] WithStack<WithSource<qsc_eval::Error>>),
    #[error("entry point not found")]
    #[diagnostic(code("Qsc.Interpret.NoEntryPoint"))]
    NoEntryPoint,
    #[error("code generation target mismatch")]
    #[diagnostic(code("Qsc.Interpret.TargetMismatch"))]
    TargetMismatch,
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

impl<'a> NodeLookup for Lookup<'a> {
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
    // compilation state (ast, hir)
    store: PackageStore,
    compiler: Compiler,
    passes: PassContext,
    lines: u32,
    target: TargetProfile,
    // fir state
    fir_store: IndexMap<PackageId, qsc_fir::fir::Package>,
    udts: HashSet<LocalItemId>,
    callables: IndexMap<LocalItemId, CallableDecl>,
    lowerer: qsc_eval::lower::Lowerer,
    // package IDs (used to index into both ast/hir and fir package stores)
    package: PackageId,
    source_package: PackageId,
    // evaluator state
    env: Env,
    sim: SparseSim,
    state: State,
}

pub type InterpretResult = Result<Value, Vec<Error>>;

impl Interpreter {
    /// # Errors
    /// If the compilation of the standard library fails, an error is returned.
    /// If the compilation of the sources fails, an error is returned.
    pub fn new(
        std: bool,
        sources: SourceMap,
        package_type: PackageType,
        target: TargetProfile,
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
            let std = compile::std(&store, target);
            let std_fir = lowerer.lower_package(&std.package);
            let id = store.insert(std);
            fir_store.insert(map_hir_package_to_fir(id), std_fir);
            dependencies.push(id);
        }

        let (unit, errors) = compile(&store, &dependencies, sources, package_type, target);
        if !errors.is_empty() {
            return Err(errors
                .into_iter()
                .map(|error| Error(WithSource::from_map(&unit.sources, error).into()))
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
        let compiler = Compiler::new(&store, dependencies, target);
        Ok(Self {
            store,
            compiler,
            passes: PassContext::new(target),
            lines: 0,
            target,
            fir_store,
            udts: HashSet::new(),
            callables: IndexMap::new(),
            lowerer,
            env: Env::with_empty_scope(),
            sim: SparseSim::new(),
            state: State::new(map_hir_package_to_fir(package)),
            package: map_hir_package_to_fir(user_package),
            source_package: map_hir_package_to_fir(package),
        })
    }

    /// Loads the entry expression to the top of the evaluation stack.
    /// This is needed for debugging so that when begging to debug with
    /// a step action the system is already in the correct state.
    /// # Errors
    /// Returns a vector of errors if loading the entry point fails.
    pub fn set_entry(&mut self) -> Result<(), Vec<Error>> {
        let expr = self.get_entry_expr()?;
        qsc_eval::eval_push_expr(&mut self.state, expr);
        Ok(())
    }

    /// Resumes execution with specified `StepAction`.
    /// # Errors
    /// Returns a vector of errors if evaluating the entry point fails.
    /// # Panics
    ///
    /// This function will panic if compiler state is invalid or in out-of-memory conditions.
    pub fn eval_step(
        &mut self,
        receiver: &mut impl Receiver,
        breakpoints: &[StmtId],
        step: StepAction,
    ) -> Result<StepResult, Vec<Error>> {
        let globals = Lookup {
            fir_store: &self.fir_store,
            package: self.package,
            udts: &self.udts,
            callables: &self.callables,
        };

        self.state
            .eval(
                &globals,
                &mut self.env,
                &mut self.sim,
                receiver,
                breakpoints,
                step,
            )
            .map_err(|(error, call_stack)| {
                let stack_trace = if call_stack.is_empty() {
                    None
                } else {
                    Some(self.render_call_stack(call_stack, &error))
                };

                vec![Error(
                    error::from_eval(error, &self.store, stack_trace).into(),
                )]
            })
    }

    /// Executes the entry expression until the end of execution.
    /// # Errors
    /// Returns a vector of errors if evaluating the entry point fails.
    /// # Panics
    ///
    /// This function will panic if compiler state is invalid or in out-of-memory conditions.
    pub fn eval_entry(&mut self, receiver: &mut impl Receiver) -> Result<Value, Vec<Error>> {
        let expr = self.get_entry_expr()?;
        let globals = Lookup {
            fir_store: &self.fir_store,
            package: self.package,
            udts: &self.udts,
            callables: &self.callables,
        };

        eval_expr(
            &mut self.state,
            expr,
            &globals,
            &mut self.env,
            &mut self.sim,
            receiver,
        )
        .map_err(|(error, call_stack)| {
            let stack_trace = if call_stack.is_empty() {
                None
            } else {
                Some(self.render_call_stack(call_stack, &error))
            };

            vec![Error(
                error::from_eval(error, &self.store, stack_trace).into(),
            )]
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
        Err(vec![Error(ErrorKind::NoEntryPoint)])
    }

    /// # Errors
    /// If the parsing of the fragments fails, an error is returned.
    /// If the compilation of the fragments fails, an error is returned.
    /// If there is a runtime error when interpreting the fragments, an error is returned.
    pub fn eval_fragments(
        &mut self,
        receiver: &mut impl Receiver,
        fragments: &str,
    ) -> InterpretResult {
        let mut result = Value::unit();

        let label = &self.next_line_label();

        let (items, stmts) =
            self.compile_incremental(|c, p| c.compile_fragments(p, label, fragments))?;

        for item in items {
            match item.kind {
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
            }
        }

        for stmt in stmts {
            let stmt_id = self.lower_stmt(&stmt);
            let globals = Lookup {
                fir_store: &self.fir_store,
                package: self.package,
                udts: &self.udts,
                callables: &self.callables,
            };

            match eval_stmt(
                stmt_id,
                &globals,
                &mut self.env,
                &mut self.sim,
                self.package,
                receiver,
            ) {
                Ok(value) => result = value,
                Err((error, call_stack)) => {
                    let stack_trace = if call_stack.is_empty() {
                        None
                    } else {
                        Some(self.render_call_stack(call_stack, &error))
                    };

                    return Err(vec![Error(
                        error::from_eval(error, &self.store, stack_trace).into(),
                    )]);
                }
            }
        }

        Ok(result)
    }

    fn compile_incremental(
        &mut self,
        mut compile: impl FnMut(
            &mut Compiler,
            &mut CompileUnit,
        ) -> Result<Vec<Fragment>, Vec<incremental::Error>>,
    ) -> Result<(Vec<hir::Item>, Vec<hir::Stmt>), Vec<Error>> {
        let (core, package) = self.store.get_mut(map_fir_package_to_hir(self.package));

        let package = package.expect("expected to find package");

        let mut fragments = compile(&mut self.compiler, package).map_err(|errors| {
            errors
                .into_iter()
                .map(|error| Error(WithSource::from_map(&package.sources, error).into()))
                .collect::<Vec<_>>()
        })?;

        let pass_errors = fragments
            .iter_mut()
            .flat_map(|fragment| self.passes.run(core, &mut package.assigner, fragment))
            .collect::<Vec<_>>();
        if !pass_errors.is_empty() {
            return Err(pass_errors
                .into_iter()
                .map(|error| Error(WithSource::from_map(&package.sources, error).into()))
                .collect());
        }

        // Partition list into items and statements since items are always
        // meant to be processed before statements.
        let mut items = Vec::new();
        let mut stmts = Vec::new();

        for f in fragments {
            match f {
                Fragment::Item(item) => items.push(item),
                Fragment::Stmt(stmt) => stmts.push(stmt),
            }
        }

        Ok((items, stmts))
    }

    /// Runs the given entry expression on a new instance of the environment and simulator,
    /// but using the current compilation.
    pub fn run(
        &mut self,
        receiver: &mut impl Receiver,
        expr: &str,
        shots: u32,
    ) -> Result<Vec<InterpretResult>, Vec<Error>> {
        self.run_with_sim(&mut SparseSim::new(), receiver, expr, shots)
    }

    /// Performs QIR codegen using the given entry expression on a new instance of the environment
    /// and simulator but using the current compilation.
    pub fn qirgen(&mut self, expr: &str) -> Result<String, Vec<Error>> {
        if self.target != TargetProfile::Base {
            return Err(vec![Error(ErrorKind::TargetMismatch)]);
        }

        let mut sim = BaseProfSim::new();
        let mut stdout = std::io::sink();
        let mut out = GenericReceiver::new(&mut stdout);

        let val = self
            .run_with_sim(&mut sim, &mut out, expr, 1)?
            .into_iter()
            .last()
            .expect("execution should have at least one result")?;

        Ok(sim.finish(&val))
    }

    /// Runs the given entry expression on the given simulator with a new instance of the environment
    /// but using the current compilation.
    pub fn run_with_sim(
        &mut self,
        sim: &mut impl Backend<ResultType = impl Into<val::Result>>,
        receiver: &mut impl Receiver,
        expr: &str,
        shots: u32,
    ) -> Result<Vec<InterpretResult>, Vec<Error>> {
        let (items, stmts) = self.compile_incremental(|c, p| c.compile_expr(p, "<entry>", expr))?;

        Ok(self.run_internal(sim, receiver, items, stmts, shots))
    }

    fn run_internal(
        &mut self,
        sim: &mut impl Backend<ResultType = impl Into<val::Result>>,
        receiver: &mut impl Receiver,
        items: Vec<hir::Item>,
        stmts: Vec<hir::Stmt>,
        shots: u32,
    ) -> Vec<InterpretResult> {
        for item in items {
            match item.kind {
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
            }
        }

        let mut stmt_ids = Vec::new();
        for stmt in stmts {
            stmt_ids.push(self.lower_stmt(&stmt));
        }

        let globals = Lookup {
            fir_store: &self.fir_store,
            package: self.package,
            udts: &self.udts,
            callables: &self.callables,
        };

        let mut results: Vec<InterpretResult> = Vec::new();
        for i in 0..shots {
            for stmt_id in &stmt_ids {
                results.push(
                    match eval_stmt(
                        *stmt_id,
                        &globals,
                        &mut Env::with_empty_scope(),
                        sim,
                        self.package,
                        receiver,
                    ) {
                        Ok(value) => Ok(value),
                        Err((error, call_stack)) => {
                            let stack_trace = if call_stack.is_empty() {
                                None
                            } else {
                                Some(self.render_call_stack(call_stack, &error))
                            };

                            Err(vec![Error(
                                error::from_eval(error, &self.store, stack_trace).into(),
                            )])
                        }
                    },
                );
            }

            if i != 0 {
                // If running more than one shot, re-initialize the simulator to start the next shot
                // from a clean state.
                sim.reinit();
            }
        }

        results
    }

    fn package(&self) -> &CompileUnit {
        self.store
            .get(map_fir_package_to_hir(self.source_package))
            .expect("Could not load package")
    }

    fn next_line_label(&mut self) -> String {
        let label = format!("line_{}", self.lines);
        self.lines += 1;
        label
    }

    fn lower_callable_decl(&mut self, callable: &qsc_hir::hir::CallableDecl) -> CallableDecl {
        let callable = self.lowerer.lower_callable_decl(callable);
        self.update_fir_package();
        callable
    }

    fn lower_stmt(&mut self, stmt: &qsc_hir::hir::Stmt) -> StmtId {
        let stmt_id = self.lowerer.lower_stmt(stmt);
        self.update_fir_package();
        stmt_id
    }

    fn update_fir_package(&mut self) {
        let package = self
            .fir_store
            .get_mut(self.package)
            .expect("package should be in store");

        self.lowerer.update_package(package);
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

    /// # Panics
    ///
    /// This function will panic if compiler state is invalid or in out-of-memory conditions.
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
                    lo: frame.span.lo - source.offset,
                    hi: frame.span.hi - source.offset,
                }
            })
            .collect();
        stack_frames
    }

    pub fn capture_quantum_state(&mut self) -> (Vec<(BigUint, Complex<f64>)>, usize) {
        self.sim.capture_quantum_state()
    }

    /// # Panics
    ///
    /// This function will panic if compiler state is invalid or in out-of-memory conditions.
    #[must_use]
    pub fn get_breakpoints(&self, path: &str) -> Vec<BreakpointSpan> {
        let unit = self.package();

        if let Some(source) = unit.sources.find_by_name(path) {
            let package = self
                .fir_store
                .get(self.source_package)
                .expect("package should have been lowered");
            let mut collector = BreakpointCollector::new(&unit.sources, source.offset, package);
            collector.visit_package(package);
            let mut spans: Vec<_> = collector
                .statements
                .iter()
                .map(|bps| BreakpointSpan {
                    id: bps.id,
                    lo: bps.lo,
                    hi: bps.hi,
                })
                .collect();
            spans.sort_by_key(|s| s.lo);
            spans
        } else {
            Vec::new()
        }
    }

    #[must_use]
    pub fn get_locals(&self) -> Vec<VariableInfo> {
        self.env
            .get_variables_in_top_frame()
            .into_iter()
            .filter(|v| !v.name.starts_with('@'))
            .collect()
    }
}

/// Represents a stack frame for debugging.
pub struct StackFrame {
    /// The name of the callable.
    pub name: String,
    /// The functor of the callable.
    pub functor: String,
    /// The path of the source file.
    pub path: String,
    /// The start of the call site span in utf8 characters.
    pub lo: u32,
    /// The end of the call site span in utf8 characters.
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
        fir_store
            .get(id.package)
            .and_then(|package| match &package.items.get(id.item)?.kind {
                ItemKind::Callable(callable) => Some(Global::Callable(callable)),
                ItemKind::Namespace(..) => None,
                ItemKind::Ty(..) => Some(Global::Udt),
            })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BreakpointSpan {
    /// The id of the statement representing the breakpoint location.
    pub id: u32,
    /// The start of the call site span in utf8 characters.
    pub lo: u32,
    /// The end of the call site span in utf8 characters.
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
            let span = stmt.span - source.offset;
            let bps = BreakpointSpan {
                id: stmt.id.into(),
                lo: span.lo,
                hi: span.hi,
            };
            if span != Span::default() {
                self.statements.insert(bps);
            }
        }
    }
}

impl<'a> Visitor<'a> for BreakpointCollector<'a> {
    fn visit_stmt(&mut self, stmt: StmtId) {
        let stmt_res = self.get_stmt(stmt);
        match stmt_res.kind {
            qsc_fir::fir::StmtKind::Expr(expr) | qsc_fir::fir::StmtKind::Local(_, _, expr) => {
                self.add_stmt(stmt_res);
                visit::walk_expr(self, expr);
            }
            qsc_fir::fir::StmtKind::Qubit(_, _, _, block) => match block {
                Some(block) => visit::walk_block(self, block),
                None => self.add_stmt(stmt_res),
            },
            qsc_fir::fir::StmtKind::Item(_) | qsc_fir::fir::StmtKind::Semi(_) => {
                self.add_stmt(stmt_res);
            }
        };
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
