// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    compile::{self, compile},
    error::WithSource,
};
use miette::Diagnostic;
use qsc_data_structures::index_map::IndexMap;
use qsc_eval::{
    backend::{Backend, SparseSim},
    debug::{map_fir_package_to_hir, map_hir_package_to_fir, Frame},
    eval_expr,
    output::Receiver,
    val::{self, GlobalId, Value},
    Env, Global, NodeLookup, State,
};
use qsc_fir::fir::{BlockId, ExprId, PatId, StmtId};
use qsc_fir::fir::{ItemKind, PackageId};
use qsc_frontend::compile::{PackageStore, Source, SourceMap, TargetProfile};
use qsc_passes::PackageType;
use thiserror::Error;

use super::debug::format_call_stack;

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

pub struct Interpreter {
    store: PackageStore,
    fir_store: IndexMap<PackageId, qsc_fir::fir::Package>,
    package: PackageId,
}

pub struct EvalContext<'a, Sim> {
    interpreter: &'a Interpreter,
    env: Env,
    sim: Sim,
    lookup: Lookup<'a>,
    state: State,
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

impl Interpreter {
    /// # Errors
    ///
    /// Returns a vector of errors if compiling the given sources fails.
    pub fn new(std: bool, sources: SourceMap) -> Result<Self, Vec<Error>> {
        let mut fir_store = IndexMap::new();
        let mut fir_lowerer = qsc_eval::lower::Lowerer::new();
        let core = compile::core();

        let core_fir = fir_lowerer.lower_package(&core.package);
        fir_store.insert(
            map_hir_package_to_fir(qsc_hir::hir::PackageId::CORE),
            core_fir,
        );
        let mut store = PackageStore::new(core);
        let mut dependencies = Vec::new();

        if std {
            let std = compile::std(&store, TargetProfile::Full);
            let std_fir = fir_lowerer.lower_package(&std.package);
            let id = store.insert(std);
            fir_store.insert(map_hir_package_to_fir(id), std_fir);
            dependencies.push(id);
        }

        let (unit, errors) = compile(
            &store,
            &dependencies,
            sources,
            PackageType::Exe,
            TargetProfile::Full,
        );
        if errors.is_empty() {
            let user_fir = fir_lowerer.lower_package(&unit.package);
            let package = store.insert(unit);
            fir_store.insert(map_hir_package_to_fir(package), user_fir);
            Ok(Self {
                store,
                fir_store,
                package: map_hir_package_to_fir(package),
            })
        } else {
            Err(errors
                .into_iter()
                .map(|error| Error(WithSource::from_map(&unit.sources, error.into(), None)))
                .collect())
        }
    }

    #[must_use]
    pub fn new_eval_context(&self) -> EvalContext<SparseSim> {
        EvalContext {
            interpreter: self,
            env: Env::with_empty_scope(),
            sim: SparseSim::new(),
            lookup: Lookup {
                fir_store: &self.fir_store,
            },
            state: State::new(self.package),
        }
    }
}

impl<'a, Sim, ResultType> EvalContext<'a, Sim>
where
    Sim: Backend<ResultType = ResultType>,
    ResultType: PartialEq + Into<val::Result>,
{
    /// # Errors
    ///
    /// Returns a vector of errors if evaluating the entry point fails.
    pub fn eval_entry(&mut self, receiver: &mut impl Receiver) -> Result<Value, Vec<Error>> {
        let expr = self.get_entry_expr()?;
        eval_expr(
            &mut self.state,
            expr,
            &self.lookup,
            &mut self.env,
            &mut self.sim,
            receiver,
        )
        .map_err(|(error, call_stack)| {
            let package = self
                .interpreter
                .store
                .get(map_fir_package_to_hir(self.interpreter.package))
                .expect("package should be in store");

            let stack_trace = if call_stack.is_empty() {
                None
            } else {
                Some(render_call_stack(
                    &self.interpreter.store,
                    &Lookup {
                        fir_store: &self.interpreter.fir_store,
                    },
                    call_stack,
                    &error,
                ))
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
            .interpreter
            .fir_store
            .get(self.interpreter.package)
            .expect("store should have package");
        if let Some(entry) = unit.entry {
            return Ok(entry);
        };
        let unit = self
            .interpreter
            .store
            .get(map_fir_package_to_hir(self.interpreter.package))
            .expect("store should have package");
        Err(vec![Error(WithSource::from_map(
            &unit.sources,
            ErrorKind::NoEntryPoint,
            None,
        ))])
    }
}

fn render_call_stack(
    store: &PackageStore,
    globals: &impl NodeLookup,
    call_stack: Vec<Frame>,
    error: &dyn std::error::Error,
) -> String {
    format_call_stack(store, globals, call_stack, error)
}

pub(super) fn get_global(
    fir_store: &IndexMap<PackageId, qsc_fir::fir::Package>,
    id: GlobalId,
) -> Option<Global> {
    fir_store
        .get(id.package)
        .and_then(|package| match &package.items.get(id.item)?.kind {
            ItemKind::Callable(callable) => Some(Global::Callable(callable)),
            ItemKind::Namespace(..) => None,
            ItemKind::Ty(..) => Some(Global::Udt),
        })
}
