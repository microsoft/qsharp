// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    compile::{self, compile},
    error::WithSource,
};
use miette::Diagnostic;
use qsc_eval::{
    backend::SparseSim,
    debug::CallStack,
    output::Receiver,
    val::{GlobalId, Value},
    Env, Global, GlobalLookup, State,
};
use qsc_frontend::compile::{PackageStore, Source, SourceMap};
use qsc_hir::hir::{Block, Expr, ItemKind, PackageId};
use qsc_passes::entry_point::extract_entry;
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
#[diagnostic(transparent)]
enum ErrorKind {
    #[error(transparent)]
    Compile(#[from] compile::Error),
    #[error(transparent)]
    Pass(#[from] qsc_passes::Error),
    #[error("runtime error")]
    Eval(#[from] qsc_eval::Error),
}

pub struct Interpreter {
    context: Context,
}

pub struct Context {
    store: PackageStore,
    package: PackageId,
}

pub struct EvalContext<'a> {
    context: &'a Context,
    env: Env,
    sim: SparseSim,
    lookup: Lookup<'a>,
    state: State<'a>,
}

struct Lookup<'this> {
    store: &'this PackageStore,
}

impl<'a> GlobalLookup<'a> for Lookup<'a> {
    fn get(&self, id: GlobalId) -> Option<Global<'a>> {
        get_global(self.store, id)
    }
}

impl Interpreter {
    /// # Errors
    ///
    /// Returns a vector of errors if compiling the given sources fails.
    pub fn new(std: bool, sources: SourceMap) -> Result<Self, Vec<Error>> {
        let mut store = PackageStore::new(compile::core());
        let mut dependencies = Vec::new();
        if std {
            dependencies.push(store.insert(compile::std(&store)));
        }

        let (unit, errors) = compile(&store, &dependencies, sources);
        if errors.is_empty() {
            let package = store.insert(unit);
            let context = Context { store, package };
            Ok(Self { context })
        } else {
            Err(errors
                .into_iter()
                .map(|error| Error(WithSource::from_map(&unit.sources, error.into(), None)))
                .collect())
        }
    }

    #[must_use]
    pub fn eval_context(&self) -> EvalContext {
        EvalContext {
            context: &self.context,
            env: Env::with_empty_scope(),
            sim: SparseSim::new(),
            lookup: Lookup {
                store: &self.context.store,
            },
            state: State::new(self.context.package),
        }
    }
}

impl<'a> EvalContext<'a> {
    /// # Errors
    ///
    /// Returns a vector of errors if evaluating the entry point fails.
    pub fn eval(&mut self, receiver: &mut dyn Receiver) -> Result<Value, Vec<Error>> {
        if let Some(expr) = get_entry_expr(&self.context.store, self.context.package) {
            self.state.push_expr(expr);
        } else {
            let block = find_entry_block(&self.context.store, self.context.package)?;
            self.state.push_block(&mut self.env, block);
        }

        self.state
            .eval(&self.lookup, &mut self.env, &mut self.sim, receiver)
            .map_err(|(error, call_stack)| {
                let package = self
                    .context
                    .store
                    .get(self.context.package)
                    .expect("package should be in store");

                let stack_trace = if call_stack.is_empty() {
                    None
                } else {
                    Some(render_call_stack(
                        &self.context.store,
                        &Lookup {
                            store: &self.context.store,
                        },
                        &call_stack,
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
}

fn render_call_stack<'a>(
    store: &PackageStore,
    globals: &impl GlobalLookup<'a>,
    call_stack: &CallStack,
    error: &dyn std::error::Error,
) -> String {
    format_call_stack(store, globals, call_stack, error)
}

fn get_entry_expr(store: &PackageStore, package: PackageId) -> Option<&Expr> {
    let unit = store.get(package).expect("store should have package");
    unit.package.entry.as_ref()
}

fn find_entry_block(store: &PackageStore, package: PackageId) -> Result<&Block, Vec<Error>> {
    let unit = store.get(package).expect("store should have package");
    extract_entry(&unit.package).map_err(|errors| {
        errors
            .into_iter()
            .map(|error| Error(WithSource::from_map(&unit.sources, error.into(), None)))
            .collect()
    })
}

pub(super) fn get_global(store: &PackageStore, id: GlobalId) -> Option<Global> {
    store
        .get(id.package)
        .and_then(|unit| match &unit.package.items.get(id.item)?.kind {
            ItemKind::Callable(callable) => Some(Global::Callable(callable)),
            ItemKind::Namespace(..) => None,
            ItemKind::Ty(..) => Some(Global::Udt),
        })
}
