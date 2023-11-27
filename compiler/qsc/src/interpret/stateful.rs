// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

#[cfg(test)]
mod stepping_tests;

use super::debug::format_call_stack;
use crate::{
    error::{self, WithStack},
    incremental::Compiler,
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
    fir::{Block, BlockId, Expr, ExprId, ItemKind, Package, PackageId, Pat, PatId, Stmt, StmtId},
    visit::{self, Visitor},
};
use qsc_frontend::{
    compile::{CompileUnit, PackageStore, Source, SourceMap, TargetProfile},
    error::WithSource,
};
use qsc_passes::PackageType;
use rustc_hash::FxHashSet;
use thiserror::Error;

impl Error {
    #[must_use]
    pub fn stack_trace(&self) -> &Option<String> {
        match &self {
            Error::Eval(err) => err.stack_trace(),
            _ => &None,
        }
    }
}

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Compile(#[from] crate::compile::Error),
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
        get_global(self.fir_store, id)
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

/// A Q# interpreter.
pub struct Interpreter {
    /// The incremental Q# compiler.
    compiler: Compiler,
    /// The `TargetProfile` used for compilation.
    target: TargetProfile,
    /// The number of lines that have so far been compiled.
    /// This field is used to generate a unique label
    /// for each line evaluated with `eval_fragments`.
    lines: u32,
    // The FIR store
    fir_store: IndexMap<PackageId, qsc_fir::fir::Package>,
    /// FIR lowerer
    lowerer: qsc_eval::lower::Lowerer,
    /// The ID of the current package.
    /// This ID is valid both for the FIR store and the `PackageStore`.
    package: PackageId,
    /// The ID of the source package. The source package
    /// is made up of the initial sources passed in when creating the interpreter.
    /// This ID is valid both for the FIR store and the `PackageStore`.
    source_package: PackageId,
    /// The default simulator backend.
    sim: SparseSim,
    /// The evaluator environment.
    env: Env,
    /// The current state of the evaluator.
    state: State,
}

pub type InterpretResult = Result<Value, Vec<Error>>;

impl Interpreter {
    /// Creates a new incremental compiler, compiling the passed in sources.
    /// # Errors
    /// If compiling the sources fails, compiler errors are returned.
    pub fn new(
        std: bool,
        sources: SourceMap,
        package_type: PackageType,
        target: TargetProfile,
    ) -> Result<Self, Vec<Error>> {
        let mut lowerer = qsc_eval::lower::Lowerer::new();
        let mut fir_store = IndexMap::new();

        let compiler = Compiler::new(std, sources, package_type, target).map_err(into_errors)?;

        for (id, unit) in compiler.package_store().iter() {
            fir_store.insert(
                map_hir_package_to_fir(id),
                lowerer.lower_package(&unit.package),
            );
        }

        let source_package_id = compiler.source_package_id();
        let package_id = compiler.package_id();

        Ok(Self {
            compiler,
            lines: 0,
            target,
            fir_store,
            lowerer,
            env: Env::with_empty_scope(),
            sim: SparseSim::new(),
            state: State::new(map_hir_package_to_fir(source_package_id)),
            package: map_hir_package_to_fir(package_id),
            source_package: map_hir_package_to_fir(source_package_id),
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
    pub fn eval_step(
        &mut self,
        receiver: &mut impl Receiver,
        breakpoints: &[StmtId],
        step: StepAction,
    ) -> Result<StepResult, Vec<Error>> {
        let globals = Lookup {
            fir_store: &self.fir_store,
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
                eval_error(
                    self.compiler.package_store(),
                    &self.fir_store,
                    call_stack,
                    error,
                )
            })
    }

    /// Executes the entry expression until the end of execution.
    /// # Errors
    /// Returns a vector of errors if evaluating the entry point fails.
    pub fn eval_entry(&mut self, receiver: &mut impl Receiver) -> Result<Value, Vec<Error>> {
        let expr = self.get_entry_expr()?;
        let globals = Lookup {
            fir_store: &self.fir_store,
        };

        eval_expr(
            &mut State::new(self.source_package),
            expr,
            &globals,
            &mut Env::with_empty_scope(),
            &mut self.sim,
            receiver,
        )
        .map_err(|(error, call_stack)| {
            eval_error(
                self.compiler.package_store(),
                &self.fir_store,
                call_stack,
                error,
            )
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
        Err(vec![Error::NoEntryPoint])
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
        let label = self.next_line_label();

        let increment = self
            .compiler
            .compile_fragments_fail_fast(&label, fragments)
            .map_err(into_errors)?;

        let stmts = self.lower(&increment);

        // Updating the compiler state with the new AST/HIR nodes
        // is not necessary for the interpreter to function, as all
        // the state required for evaluation already exists in the
        // FIR store. It could potentially save some memory
        // *not* to do hold on to the AST/HIR, but it is done
        // here to keep the package stores consistent.
        self.compiler.update(increment);

        let mut result = Value::unit();

        for stmt_id in stmts {
            let globals = Lookup {
                fir_store: &self.fir_store,
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
                    return Err(eval_error(
                        self.compiler.package_store(),
                        &self.fir_store,
                        call_stack,
                        error,
                    ))
                }
            }
        }

        Ok(result)
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

    /// Gets the current quantum state of the simulator.
    pub fn get_quantum_state(&mut self) -> (Vec<(BigUint, Complex<f64>)>, usize) {
        self.sim.capture_quantum_state()
    }

    /// Performs QIR codegen using the given entry expression on a new instance of the environment
    /// and simulator but using the current compilation.
    pub fn qirgen(&mut self, expr: &str) -> Result<String, Vec<Error>> {
        if self.target != TargetProfile::Base {
            return Err(vec![Error::TargetMismatch]);
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
        let stmt_id = self.compile_expr_to_stmt(expr)?;

        Ok(self.run_internal(sim, receiver, stmt_id, shots))
    }

    fn run_internal(
        &mut self,
        sim: &mut impl Backend<ResultType = impl Into<val::Result>>,
        receiver: &mut impl Receiver,
        stmt_id: StmtId,
        shots: u32,
    ) -> Vec<InterpretResult> {
        let globals = Lookup {
            fir_store: &self.fir_store,
        };

        let mut results: Vec<InterpretResult> = Vec::new();
        for i in 0..shots {
            results.push(
                match eval_stmt(
                    stmt_id,
                    &globals,
                    &mut Env::with_empty_scope(),
                    sim,
                    self.package,
                    receiver,
                ) {
                    Ok(value) => Ok(value),
                    Err((error, call_stack)) => Err(eval_error(
                        self.compiler.package_store(),
                        &self.fir_store,
                        call_stack,
                        error,
                    )),
                },
            );

            if i != 0 {
                // If running more than one shot, re-initialize the simulator to start the next shot
                // from a clean state.
                sim.reinit();
            }
        }

        results
    }

    fn compile_expr_to_stmt(&mut self, expr: &str) -> Result<StmtId, Vec<Error>> {
        let increment = self.compiler.compile_expr(expr).map_err(into_errors)?;

        let stmts = self.lower(&increment);

        // Updating the compiler state with the new AST/HIR nodes
        // is not necessary for the interpreter to function, as all
        // the state required for evaluation already exists in the
        // FIR store. It could potentially save some memory
        // *not* to do hold on to the AST/HIR, but it is done
        // here to keep the package stores consistent.
        self.compiler.update(increment);

        assert!(stmts.len() == 1, "expected exactly one statement");
        let stmt_id = stmts.get(0).expect("expected exactly one statement");

        Ok(*stmt_id)
    }

    fn lower(&mut self, unit_addition: &qsc_frontend::incremental::Increment) -> Vec<StmtId> {
        let fir_package = self
            .fir_store
            .get_mut(self.package)
            .expect("package should be in store");

        self.lowerer
            .lower_and_update_package(fir_package, &unit_addition.hir)
    }

    fn source_package(&self) -> &CompileUnit {
        self.compiler
            .package_store()
            .get(map_fir_package_to_hir(self.source_package))
            .expect("Could not load package")
    }

    fn next_line_label(&mut self) -> String {
        let label = format!("line_{}", self.lines);
        self.lines += 1;
        label
    }

    #[must_use]
    pub fn get_stack_frames(&self) -> Vec<StackFrame> {
        let globals = Lookup {
            fir_store: &self.fir_store,
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
                    .compiler
                    .package_store()
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

    #[must_use]
    pub fn get_breakpoints(&self, path: &str) -> Vec<BreakpointSpan> {
        let unit = self.source_package();

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

fn get_global(
    fir_store: &IndexMap<PackageId, qsc_fir::fir::Package>,
    id: GlobalId,
) -> Option<Global<'_>> {
    fir_store
        .get(id.package)
        .and_then(|package| match &package.items.get(id.item)?.kind {
            ItemKind::Callable(callable) => Some(Global::Callable(callable)),
            ItemKind::Namespace(..) => None,
            ItemKind::Ty(..) => Some(Global::Udt),
        })
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
    statements: FxHashSet<BreakpointSpan>,
    sources: &'a SourceMap,
    offset: u32,
    package: &'a Package,
}

impl<'a> BreakpointCollector<'a> {
    fn new(sources: &'a SourceMap, offset: u32, package: &'a Package) -> Self {
        Self {
            statements: FxHashSet::default(),
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

fn eval_error(
    package_store: &PackageStore,
    fir_store: &IndexMap<PackageId, qsc_fir::fir::Package>,
    call_stack: Vec<Frame>,
    error: qsc_eval::Error,
) -> Vec<Error> {
    let stack_trace = if call_stack.is_empty() {
        None
    } else {
        Some(format_call_stack(
            package_store,
            &Lookup { fir_store },
            call_stack,
            &error,
        ))
    };

    vec![error::from_eval(error, package_store, stack_trace).into()]
}

fn into_errors(errors: Vec<crate::compile::Error>) -> Vec<Error> {
    errors
        .into_iter()
        .map(|error| Error::Compile(error.into_with_source()))
        .collect::<Vec<_>>()
}
