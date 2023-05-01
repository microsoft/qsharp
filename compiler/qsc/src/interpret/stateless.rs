// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    compile::{self, compile},
    error::WithSource,
};
use miette::Diagnostic;
use qsc_eval::{
    debug::CallStack,
    eval_expr,
    output::Receiver,
    val::{GlobalId, Value},
    Env,
};
use qsc_frontend::compile::{PackageStore, Source, SourceMap};
use qsc_hir::hir::{CallableDecl, Expr, ItemKind, PackageId};
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

pub struct Context {
    store: PackageStore,
    package: PackageId,
}

impl Context {
    /// # Errors
    ///
    /// Returns a vector of errors if compiling the given sources fails.
    pub fn new(std: bool, sources: SourceMap) -> Result<Self, Vec<Error>> {
        let mut store = PackageStore::new();
        let mut dependencies = Vec::new();
        if std {
            dependencies.push(store.insert(compile::std()));
        }

        let (unit, errors) = compile(&store, dependencies, sources);
        if errors.is_empty() {
            let package = store.insert(unit);
            Ok(Self { store, package })
        } else {
            Err(errors
                .into_iter()
                .map(|error| Error(WithSource::from_map(&unit.sources, error.into(), None)))
                .collect())
        }
    }

    /// # Errors
    ///
    /// Returns a vector of errors if evaluating the entry point fails.
    pub fn eval(&self, receiver: &mut dyn Receiver) -> Result<Value, Vec<Error>> {
        qsc_eval::init();

        eval_expr(
            &get_entry_expr(&self.store, self.package)?,
            &|id| get_callable(&self.store, id),
            self.package,
            &mut Env::with_empty_scope(),
            receiver,
        )
        .map_err(|(error, call_stack)| {
            let package = self
                .store
                .get(self.package)
                .expect("package should be in store");

            let stack_trace = if call_stack.is_empty() {
                None
            } else {
                Some(render_call_stack(
                    &self.store,
                    self.package,
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

fn render_call_stack(
    store: &PackageStore,
    package: PackageId,
    call_stack: &CallStack,
    error: &dyn std::error::Error,
) -> String {
    format_call_stack(
        store,
        package,
        &|id| get_callable(store, id),
        call_stack,
        error,
    )
}

fn get_entry_expr(store: &PackageStore, package: PackageId) -> Result<Expr, Vec<Error>> {
    let unit = store.get(package).expect("store should have package");
    match &unit.package.entry {
        Some(entry) => Ok(entry.clone()),
        None => extract_entry(&unit.package).map_err(|errors| {
            errors
                .into_iter()
                .map(|error| Error(WithSource::from_map(&unit.sources, error.into(), None)))
                .collect()
        }),
    }
}

pub(super) fn get_callable(store: &PackageStore, id: GlobalId) -> Option<&CallableDecl> {
    store.get(id.package).and_then(|unit| {
        let item = unit.package.items.get(id.item)?;
        if let ItemKind::Callable(callable) = &item.kind {
            Some(callable)
        } else {
            None
        }
    })
}
