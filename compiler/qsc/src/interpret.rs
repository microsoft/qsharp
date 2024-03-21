// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod debug;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod debugger_tests;

#[cfg(test)]
mod circuit_tests;

use std::rc::Rc;

pub use qsc_eval::{
    debug::Frame,
    output::{self, GenericReceiver},
    val::Closure,
    val::Range as ValueRange,
    val::Result,
    val::Value,
    StepAction, StepResult,
};

use crate::{
    error::{self, WithStack},
    incremental::Compiler,
    location::Location,
};
use debug::format_call_stack;
use miette::Diagnostic;
use num_bigint::BigUint;
use num_complex::Complex;
use qsc_circuit::{
    operations::entry_expr_for_qubit_operation, Builder as CircuitBuilder, Circuit,
    Config as CircuitConfig,
};
use qsc_codegen::qir_base::BaseProfSim;
use qsc_data_structures::{
    language_features::LanguageFeatures,
    line_column::{Encoding, Range},
    span::Span,
};
use qsc_eval::{
    backend::{Backend, Chain as BackendChain, SparseSim},
    debug::{map_fir_package_to_hir, map_hir_package_to_fir},
    output::Receiver,
    val, Env, State, VariableInfo,
};
use qsc_fir::fir::{self, ExecGraphNode, Global, PackageStoreLookup};
use qsc_fir::{
    fir::{Block, BlockId, Expr, ExprId, Package, PackageId, Pat, PatId, Stmt, StmtId},
    visit::{self, Visitor},
};
use qsc_frontend::{
    compile::{CompileUnit, PackageStore, RuntimeCapabilityFlags, Source, SourceMap},
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
    #[error("unsupported runtime capabilities for code generation")]
    #[diagnostic(code("Qsc.Interpret.UnsupportedRuntimeCapabilities"))]
    UnsupportedRuntimeCapabilities,
    #[error("expression does not evaluate to an operation that takes qubit parameters")]
    #[diagnostic(code("Qsc.Interpret.NoCircuitForOperation"))]
    #[diagnostic(help(
        "provide the name of a callable or a lambda expression that only takes qubits as parameters"
    ))]
    NoCircuitForOperation,
}

/// A Q# interpreter.
pub struct Interpreter {
    /// The incremental Q# compiler.
    compiler: Compiler,
    /// The runtime capabilities used for compilation.
    capabilities: RuntimeCapabilityFlags,
    /// The number of lines that have so far been compiled.
    /// This field is used to generate a unique label
    /// for each line evaluated with `eval_fragments`.
    lines: u32,
    // The FIR store
    fir_store: fir::PackageStore,
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
    sim: BackendChain<SparseSim, CircuitBuilder>,
    /// The quantum seed, if any. This is cached here so that it can be used in calls to
    /// `run_internal` which use a passed instance of the simulator instead of the one above.
    quantum_seed: Option<u64>,
    /// The classical seed, if any. This needs to be passed to the evaluator for use in intrinsic
    /// calls that produce classical random numbers.
    classical_seed: Option<u64>,
    /// The evaluator environment.
    env: Env,
}

pub type InterpretResult = std::result::Result<Value, Vec<Error>>;

impl Interpreter {
    /// Creates a new incremental compiler, compiling the passed in sources.
    /// # Errors
    /// If compiling the sources fails, compiler errors are returned.
    pub fn new(
        std: bool,
        sources: SourceMap,
        package_type: PackageType,
        capabilities: RuntimeCapabilityFlags,
        language_features: LanguageFeatures,
    ) -> std::result::Result<Self, Vec<Error>> {
        let mut lowerer = qsc_eval::lower::Lowerer::new();
        let mut fir_store = fir::PackageStore::new();

        let compiler = Compiler::new(std, sources, package_type, capabilities, language_features)
            .map_err(into_errors)?;

        for (id, unit) in compiler.package_store() {
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
            capabilities,
            fir_store,
            lowerer,
            env: Env::default(),
            sim: BackendChain::new(
                SparseSim::new(),
                CircuitBuilder::new(CircuitConfig {
                    // When using in conjunction with the simulator,
                    // the circuit builder should *not* perform base profile
                    // decompositions, in order to match the simulator's behavior.
                    //
                    // Note that conditional compilation (e.g. @Config(Base) attributes)
                    // will still respect the selected profile. This also
                    // matches the behavior of the simulator.
                    base_profile: false,
                }),
            ),
            quantum_seed: None,
            classical_seed: None,
            package: map_hir_package_to_fir(package_id),
            source_package: map_hir_package_to_fir(source_package_id),
        })
    }

    pub fn set_quantum_seed(&mut self, seed: Option<u64>) {
        self.quantum_seed = seed;
        self.sim.set_seed(seed);
    }

    pub fn set_classical_seed(&mut self, seed: Option<u64>) {
        self.classical_seed = seed;
    }
    /// Executes the entry expression until the end of execution.
    /// # Errors
    /// Returns a vector of errors if evaluating the entry point fails.
    pub fn eval_entry(
        &mut self,
        receiver: &mut impl Receiver,
    ) -> std::result::Result<Value, Vec<Error>> {
        let graph = self.get_entry_exec_graph()?;
        eval(
            self.source_package,
            self.classical_seed,
            graph,
            self.compiler.package_store(),
            &self.fir_store,
            &mut Env::default(),
            &mut self.sim,
            receiver,
        )
    }

    /// Executes the entry expression until the end of execution, using the given simulator backend
    /// and a new instance of the environment.
    pub fn eval_entry_with_sim(
        &mut self,
        sim: &mut impl Backend<ResultType = impl Into<val::Result>>,
        receiver: &mut impl Receiver,
    ) -> std::result::Result<Value, Vec<Error>> {
        let graph = self.get_entry_exec_graph()?;
        if self.quantum_seed.is_some() {
            sim.set_seed(self.quantum_seed);
        }
        eval(
            self.source_package,
            self.classical_seed,
            graph,
            self.compiler.package_store(),
            &self.fir_store,
            &mut Env::default(),
            sim,
            receiver,
        )
    }

    fn get_entry_exec_graph(&self) -> std::result::Result<Rc<[ExecGraphNode]>, Vec<Error>> {
        let unit = self.fir_store.get(self.source_package);
        if unit.entry.is_some() {
            return Ok(unit.entry_exec_graph.clone());
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

        let (_, graph) = self.lower(&increment);

        // Updating the compiler state with the new AST/HIR nodes
        // is not necessary for the interpreter to function, as all
        // the state required for evaluation already exists in the
        // FIR store. It could potentially save some memory
        // *not* to do hold on to the AST/HIR, but it is done
        // here to keep the package stores consistent.
        self.compiler.update(increment);

        eval(
            self.package,
            self.classical_seed,
            graph.into(),
            self.compiler.package_store(),
            &self.fir_store,
            &mut self.env,
            &mut self.sim,
            receiver,
        )
    }

    /// Runs the given entry expression on a new instance of the environment and simulator,
    /// but using the current compilation.
    pub fn run(
        &mut self,
        receiver: &mut impl Receiver,
        expr: &str,
    ) -> std::result::Result<InterpretResult, Vec<Error>> {
        self.run_with_sim(&mut SparseSim::new(), receiver, expr)
    }

    /// Gets the current quantum state of the simulator.
    pub fn get_quantum_state(&mut self) -> (Vec<(BigUint, Complex<f64>)>, usize) {
        self.sim.capture_quantum_state()
    }

    /// Get the current circuit representation of the program.
    pub fn get_circuit(&self) -> Circuit {
        self.sim.chained.snapshot()
    }

    /// Performs QIR codegen using the given entry expression on a new instance of the environment
    /// and simulator but using the current compilation.
    pub fn qirgen(&mut self, expr: &str) -> std::result::Result<String, Vec<Error>> {
        if self.capabilities != RuntimeCapabilityFlags::empty() {
            return Err(vec![Error::UnsupportedRuntimeCapabilities]);
        }

        let mut sim = BaseProfSim::new();
        let mut stdout = std::io::sink();
        let mut out = GenericReceiver::new(&mut stdout);

        let val = self.run_with_sim(&mut sim, &mut out, expr)??;

        Ok(sim.finish(&val))
    }

    /// Generates a circuit representation for the program.
    ///
    /// `entry` can be the current entrypoint, an entry expression, or any operation
    /// that takes qubits.
    ///
    /// An operation can be specified by its name or a lambda expression that only takes qubits.
    /// e.g. `Sample.Main` , `qs => H(qs[0])`
    pub fn circuit(
        &mut self,
        entry: CircuitEntryPoint,
    ) -> std::result::Result<Circuit, Vec<Error>> {
        let mut sink = std::io::sink();
        let mut out = GenericReceiver::new(&mut sink);
        let mut sim = CircuitBuilder::new(CircuitConfig {
            base_profile: self.capabilities.is_empty(),
        });

        let entry_expr = match entry {
            CircuitEntryPoint::Operation(operation_expr) => {
                // To determine whether the passed in expression is a valid callable name
                // or lambda, we evaluate it and inspect the runtime value.
                let maybe_operation = match self.eval_fragments(&mut out, &operation_expr)? {
                    Value::Closure(b) => Some((b.id, b.functor)),
                    Value::Global(item_id, functor_app) => Some((item_id, functor_app)),
                    _ => None,
                };

                let maybe_invoke_expr = if let Some((item_id, functor_app)) = maybe_operation {
                    // Controlled operations are not supported at the moment.
                    if functor_app.controlled > 0 {
                        return Err(vec![Error::NoCircuitForOperation]);
                    }

                    // Find the item in the HIR
                    let package = map_fir_package_to_hir(item_id.package);
                    let local_item_id = crate::hir::LocalItemId::from(usize::from(item_id.item));
                    let package_store = self.compiler.package_store();

                    let item = package_store
                        .get(package)
                        .and_then(|unit| unit.package.items.get(local_item_id));

                    // Generate the entry expression to invoke the operation.
                    // Will return `None` if item is not a valid callable that takes qubits.
                    item.and_then(|item| entry_expr_for_qubit_operation(item, &operation_expr))
                } else {
                    return Err(vec![Error::NoCircuitForOperation]);
                };

                if maybe_invoke_expr.is_none() {
                    return Err(vec![Error::NoCircuitForOperation]);
                }
                maybe_invoke_expr
            }
            CircuitEntryPoint::EntryExpr(expr) => Some(expr),
            CircuitEntryPoint::EntryPoint => None,
        };

        let val = if let Some(entry_expr) = entry_expr {
            self.run_with_sim(&mut sim, &mut out, &entry_expr)?
        } else {
            self.eval_entry_with_sim(&mut sim, &mut out)
        }?;

        Ok(sim.finish(&val))
    }

    /// Runs the given entry expression on the given simulator with a new instance of the environment
    /// but using the current compilation.
    pub fn run_with_sim(
        &mut self,
        sim: &mut impl Backend<ResultType = impl Into<val::Result>>,
        receiver: &mut impl Receiver,
        expr: &str,
    ) -> std::result::Result<InterpretResult, Vec<Error>> {
        let graph = self.compile_entry_expr(expr)?;

        if self.quantum_seed.is_some() {
            sim.set_seed(self.quantum_seed);
        }

        Ok(eval(
            self.package,
            self.classical_seed,
            graph.into(),
            self.compiler.package_store(),
            &self.fir_store,
            &mut Env::default(),
            sim,
            receiver,
        ))
    }

    fn compile_entry_expr(
        &mut self,
        expr: &str,
    ) -> std::result::Result<Vec<ExecGraphNode>, Vec<Error>> {
        let increment = self
            .compiler
            .compile_entry_expr(expr)
            .map_err(into_errors)?;

        // `lower` will update the entry expression in the FIR store,
        // and it will always return an empty list of statements.
        let (_, graph) = self.lower(&increment);

        // The AST and HIR packages in `increment` only contain an entry
        // expression and no statements. The HIR *can* contain items if the entry
        // expression defined any items.
        assert!(increment.hir.stmts.is_empty());
        assert!(increment.ast.package.nodes.is_empty());

        // Updating the compiler state with the new AST/HIR nodes
        // is not necessary for the interpreter to function, as all
        // the state required for evaluation already exists in the
        // FIR store. It could potentially save some memory
        // *not* to do hold on to the AST/HIR, but it is done
        // here to keep the package stores consistent.
        self.compiler.update(increment);

        Ok(graph)
    }

    fn lower(
        &mut self,
        unit_addition: &qsc_frontend::incremental::Increment,
    ) -> (Vec<StmtId>, Vec<ExecGraphNode>) {
        let fir_package = self.fir_store.get_mut(self.package);
        (
            self.lowerer
                .lower_and_update_package(fir_package, &unit_addition.hir),
            self.lowerer.take_exec_graph(),
        )
    }

    fn next_line_label(&mut self) -> String {
        let label = format!("line_{}", self.lines);
        self.lines += 1;
        label
    }
}

/// Describes the entry point for circuit generation.
pub enum CircuitEntryPoint {
    /// An operation. This must be a callable name or a lambda
    /// expression that only takes qubits as arguments.
    /// The callable name must be visible in the current package.
    Operation(String),
    /// An explicitly provided entry expression.
    EntryExpr(String),
    /// The entry point for the current package.
    EntryPoint,
}

/// A debugger that enables step-by-step evaluation of code
/// and inspecting state in the interpreter.
pub struct Debugger {
    interpreter: Interpreter,
    /// The encoding (utf-8 or utf-16) used for character offsets
    /// in line/character positions returned by the Interpreter.
    position_encoding: Encoding,
    /// The current state of the evaluator.
    state: State,
}

impl Debugger {
    pub fn new(
        sources: SourceMap,
        capabilities: RuntimeCapabilityFlags,
        position_encoding: Encoding,
        language_features: LanguageFeatures,
    ) -> std::result::Result<Self, Vec<Error>> {
        let interpreter = Interpreter::new(
            true,
            sources,
            PackageType::Exe,
            capabilities,
            language_features,
        )?;
        let source_package_id = interpreter.source_package;
        let unit = interpreter.fir_store.get(source_package_id);
        let entry_exec_graph = unit.entry_exec_graph.clone();
        Ok(Self {
            interpreter,
            position_encoding,
            state: State::new(source_package_id, entry_exec_graph, None),
        })
    }

    /// Resumes execution with specified `StepAction`.
    /// # Errors
    /// Returns a vector of errors if evaluating the entry point fails.
    pub fn eval_step(
        &mut self,
        receiver: &mut impl Receiver,
        breakpoints: &[StmtId],
        step: StepAction,
    ) -> std::result::Result<StepResult, Vec<Error>> {
        self.state
            .eval(
                &self.interpreter.fir_store,
                &mut self.interpreter.env,
                &mut self.interpreter.sim,
                receiver,
                breakpoints,
                step,
            )
            .map_err(|(error, call_stack)| {
                eval_error(
                    self.interpreter.compiler.package_store(),
                    &self.interpreter.fir_store,
                    call_stack,
                    error,
                )
            })
    }

    #[must_use]
    pub fn get_stack_frames(&self) -> Vec<StackFrame> {
        let frames = self.state.get_stack_frames();
        let stack_frames = frames
            .iter()
            .map(|frame| {
                let callable = self
                    .interpreter
                    .fir_store
                    .get_global(frame.id)
                    .expect("frame should exist");
                let functor = format!("{}", frame.functor);
                let name = match callable {
                    Global::Callable(decl) => decl.name.name.to_string(),
                    Global::Udt => "udt".into(),
                };

                StackFrame {
                    name,
                    functor,
                    location: Location::from(
                        frame.span,
                        map_fir_package_to_hir(frame.id.package),
                        self.interpreter.compiler.package_store(),
                        map_fir_package_to_hir(self.interpreter.source_package),
                        self.position_encoding,
                    ),
                }
            })
            .collect();
        stack_frames
    }

    pub fn capture_quantum_state(&mut self) -> (Vec<(BigUint, Complex<f64>)>, usize) {
        self.interpreter.sim.capture_quantum_state()
    }

    pub fn circuit(&self) -> Circuit {
        self.interpreter.get_circuit()
    }

    #[must_use]
    pub fn get_breakpoints(&self, path: &str) -> Vec<BreakpointSpan> {
        let unit = self.source_package();

        if let Some(source) = unit.sources.find_by_name(path) {
            let package = self
                .interpreter
                .fir_store
                .get(self.interpreter.source_package);
            let mut collector = BreakpointCollector::new(
                &unit.sources,
                source.offset,
                package,
                self.position_encoding,
            );
            collector.visit_package(package);
            let mut spans: Vec<_> = collector.statements.into_iter().collect();

            // Sort by start position (line first, column next)
            spans.sort_by_key(|s| (s.range.start.line, s.range.start.column));
            spans
        } else {
            Vec::new()
        }
    }

    #[must_use]
    pub fn get_locals(&self) -> Vec<VariableInfo> {
        self.interpreter
            .env
            .get_variables_in_top_frame()
            .into_iter()
            .filter(|v| !v.name.starts_with('@'))
            .collect()
    }

    fn source_package(&self) -> &CompileUnit {
        self.interpreter
            .compiler
            .package_store()
            .get(map_fir_package_to_hir(self.interpreter.source_package))
            .expect("Could not load package")
    }
}

/// Wrapper function for `qsc_eval::eval` that handles error conversion.
#[allow(clippy::too_many_arguments)]
fn eval(
    package: PackageId,
    classical_seed: Option<u64>,
    exec_graph: Rc<[ExecGraphNode]>,
    package_store: &PackageStore,
    fir_store: &fir::PackageStore,
    env: &mut Env,
    sim: &mut impl Backend<ResultType = impl Into<val::Result>>,
    receiver: &mut impl Receiver,
) -> InterpretResult {
    qsc_eval::eval(
        package,
        classical_seed,
        exec_graph,
        fir_store,
        env,
        sim,
        receiver,
    )
    .map_err(|(error, call_stack)| eval_error(package_store, fir_store, call_stack, error))
}

/// Represents a stack frame for debugging.
pub struct StackFrame {
    /// The name of the callable.
    pub name: String,
    /// The functor of the callable.
    pub functor: String,
    /// The source location of the call site.
    pub location: Location,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BreakpointSpan {
    /// The id of the statement representing the breakpoint location.
    pub id: u32,
    /// The source range of the call site.
    pub range: Range,
}

struct BreakpointCollector<'a> {
    statements: FxHashSet<BreakpointSpan>,
    sources: &'a SourceMap,
    offset: u32,
    package: &'a Package,
    position_encoding: Encoding,
}

impl<'a> BreakpointCollector<'a> {
    fn new(
        sources: &'a SourceMap,
        offset: u32,
        package: &'a Package,
        position_encoding: Encoding,
    ) -> Self {
        Self {
            statements: FxHashSet::default(),
            sources,
            offset,
            package,
            position_encoding,
        }
    }

    fn get_source(&self, offset: u32) -> &Source {
        self.sources
            .find_by_offset(offset)
            .expect("Couldn't find source file")
    }

    fn add_stmt(&mut self, stmt: &qsc_fir::fir::Stmt) {
        let source: &Source = self.get_source(stmt.span.lo);
        if source.offset == self.offset {
            let span = stmt.span - source.offset;
            if span != Span::default() {
                let bps = BreakpointSpan {
                    id: stmt.id.into(),
                    range: Range::from_span(self.position_encoding, &source.contents, &span),
                };
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
            qsc_fir::fir::StmtKind::Item(_) | qsc_fir::fir::StmtKind::Semi(_) => {
                self.add_stmt(stmt_res);
            }
        };
    }

    fn get_block(&self, id: BlockId) -> &'a Block {
        self.package
            .blocks
            .get(id)
            .expect("couldn't find block in FIR")
    }

    fn get_expr(&self, id: ExprId) -> &'a Expr {
        self.package
            .exprs
            .get(id)
            .expect("couldn't find expr in FIR")
    }

    fn get_pat(&self, id: PatId) -> &'a Pat {
        self.package.pats.get(id).expect("couldn't find pat in FIR")
    }

    fn get_stmt(&self, id: StmtId) -> &'a Stmt {
        self.package
            .stmts
            .get(id)
            .expect("couldn't find stmt in FIR")
    }
}

fn eval_error(
    package_store: &PackageStore,
    fir_store: &fir::PackageStore,
    call_stack: Vec<Frame>,
    error: qsc_eval::Error,
) -> Vec<Error> {
    let stack_trace = if call_stack.is_empty() {
        None
    } else {
        Some(format_call_stack(
            package_store,
            fir_store,
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
