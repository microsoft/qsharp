// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::Diagnostic;
use qsc_eval::{
    eval_expr,
    output::Receiver,
    val::{GlobalId, Value},
    AggregateError, Env,
};
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap};
use qsc_hir::hir::{CallableDecl, Expr, ItemKind, PackageId};
use qsc_passes::{entry_point::extract_entry, run_default_passes};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
pub enum Error {
    #[error("program encountered an error while running")]
    Eval(#[from] qsc_eval::Error),
    #[error("could not compile source code")]
    Compile(#[from] compile::Error),
    #[error("could not compile source code")]
    Pass(#[from] qsc_passes::Error),
}

/// # Errors
/// If the compilation of the standard library fails, an error is returned.
/// If the compilation of the sources fails, an error is returned.
/// If the entry expression compilation fails, an error is returned.
/// If the evaluation of the entry expression causes an error
pub fn eval(
    stdlib: bool,
    receiver: &mut dyn Receiver,
    sources: SourceMap,
) -> Result<Value, AggregateError<Error>> {
    qsc_eval::init();
    let mut store = PackageStore::new();
    let mut session_deps = Vec::new();

    if stdlib {
        let mut unit = compile::std();
        let pass_errs = run_default_passes(&mut unit);
        if unit.errors.is_empty() && pass_errs.is_empty() {
            session_deps.push(store.insert(unit));
        } else {
            return Err(AggregateError(
                unit.errors
                    .into_iter()
                    .map(Error::Compile)
                    .chain(pass_errs.into_iter().map(Error::Pass))
                    .collect(),
            ));
        }
    }

    // create a package with all defined dependencies for the session
    let mut unit = compile(&store, session_deps.clone(), sources);
    let pass_errs = run_default_passes(&mut unit);
    if !unit.errors.is_empty() || !pass_errs.is_empty() {
        return Err(AggregateError(
            unit.errors
                .into_iter()
                .map(Error::Compile)
                .chain(pass_errs.into_iter().map(Error::Pass))
                .collect(),
        ));
    }

    let basis_package = store.insert(unit);
    session_deps.push(basis_package);

    let expr = get_entry_expr(&store, basis_package)?;
    eval_expr(
        &expr,
        &|id| get_callable(&store, id),
        basis_package,
        &mut Env::with_empty_scope(),
        receiver,
    )
    .map_err(|e| AggregateError(vec![Error::Eval(e)]))
}

/// # Errors
/// If the compilation of the standard library fails, an error is returned.
/// If the compilation of the sources fails, an error is returned.
/// If the entry expression compilation fails, an error is returned.
pub fn compile_execution_context(
    stdlib: bool,
    sources: SourceMap,
) -> Result<ExecutionContext, AggregateError<Error>> {
    create_execution_context(stdlib, sources)
}

/// # Errors
/// If the evaluation of the entry expression causes an error
pub fn eval_in_context(
    context: &ExecutionContext,
    receiver: &mut dyn Receiver,
) -> Result<Value, AggregateError<Error>> {
    qsc_eval::init();
    let expr = get_entry_expr(&context.store, context.package)?;
    eval_expr(
        &expr,
        &|id| get_callable(&context.store, id),
        context.package,
        &mut Env::with_empty_scope(),
        receiver,
    )
    .map_err(|e| AggregateError(vec![Error::Eval(e)]))
}

pub struct ExecutionContext {
    store: PackageStore,
    package: PackageId,
}

fn create_execution_context(
    stdlib: bool,
    sources: SourceMap,
) -> Result<ExecutionContext, AggregateError<Error>> {
    let mut store = PackageStore::new();
    let mut session_deps = Vec::new();

    if stdlib {
        let mut unit = compile::std();
        let pass_errs = run_default_passes(&mut unit);
        if unit.errors.is_empty() && pass_errs.is_empty() {
            session_deps.push(store.insert(unit));
        } else {
            return Err(AggregateError(
                unit.errors
                    .into_iter()
                    .map(Error::Compile)
                    .chain(pass_errs.into_iter().map(Error::Pass))
                    .collect(),
            ));
        }
    }

    let mut unit = compile(&store, session_deps.clone(), sources);
    let pass_errs = run_default_passes(&mut unit);
    if !unit.errors.is_empty() || !pass_errs.is_empty() {
        return Err(AggregateError(
            unit.errors
                .into_iter()
                .map(Error::Compile)
                .chain(pass_errs.into_iter().map(Error::Pass))
                .collect(),
        ));
    }

    let package = store.insert(unit);
    Ok(ExecutionContext { store, package })
}

fn get_entry_expr(
    store: &PackageStore,
    basis_package: PackageId,
) -> Result<Expr, AggregateError<Error>> {
    if let Some(expr) = store.get_entry_expr(basis_package) {
        Ok(expr.clone())
    } else {
        extract_entry(
            &store
                .get(basis_package)
                .expect("package should be in store after insert")
                .package,
        )
        .map_err(|e| AggregateError(e.into_iter().map(Error::Pass).collect()))
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
