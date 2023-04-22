// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{eval_expr, output::Receiver, val::Value, AggregateError, Env, GlobalDefId};
use miette::Diagnostic;
use qsc_frontend::compile::{self, compile, PackageStore};
use qsc_hir::hir::{Expr, ItemKind, PackageId};
use qsc_passes::{entry_point::extract_entry, run_default_passes};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
#[error(transparent)]
#[diagnostic(transparent)]
pub enum Error {
    Eval(crate::Error),
    Compile(qsc_frontend::compile::Error),
    Pass(qsc_passes::Error),
}

/// # Errors
/// If the compilation of the standard library fails, an error is returned.
/// If the compilation of the sources fails, an error is returned.
/// If the entry expression compilation fails, an error is returned.
/// If the evaluation of the entry expression causes an error
pub fn eval(
    stdlib: bool,
    expr: impl AsRef<str>,
    receiver: &mut dyn Receiver,
    sources: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<Value, AggregateError<Error>> {
    crate::init();

    let mut store = PackageStore::new();

    let mut session_deps: Vec<_> = vec![];

    if stdlib {
        let mut unit = compile::std();
        let pass_errs = run_default_passes(&mut unit);
        if unit.context.errors().is_empty() && pass_errs.is_empty() {
            session_deps.push(store.insert(unit));
        } else {
            let mut errors: Vec<Error> = unit
                .context
                .errors()
                .iter()
                .map(|e| Error::Compile(e.clone()))
                .collect();
            errors.extend(pass_errs.into_iter().map(Error::Pass));
            return Err(AggregateError(errors));
        }
    }

    // create a package with all defined dependencies for the session
    let mut unit = compile(&store, session_deps.clone(), sources, expr.as_ref());
    let pass_errs = run_default_passes(&mut unit);
    if !unit.context.errors().is_empty() || !pass_errs.is_empty() {
        let mut errors: Vec<Error> = unit
            .context
            .errors()
            .iter()
            .map(|e| Error::Compile(e.clone()))
            .collect();
        errors.extend(pass_errs.into_iter().map(Error::Pass));
        return Err(AggregateError(errors));
    }

    let basis_package = store.insert(unit);
    session_deps.push(basis_package);

    let global = |id: GlobalDefId| {
        store.get(id.package).and_then(|unit| {
            let item = unit.package.items.get(id.def)?;
            if let ItemKind::Callable(callable) = &item.kind {
                Some(callable)
            } else {
                None
            }
        })
    };

    let expr = get_entry_expr(&store, basis_package)?;
    let mut env = Env::with_empty_scope();
    let result = eval_expr(&expr, &global, basis_package, &mut env, receiver);
    match result {
        Ok(v) => Ok(v),
        Err(e) => Err(AggregateError(vec![Error::Eval(e)])),
    }
}

/// # Errors
/// If the compilation of the standard library fails, an error is returned.
/// If the compilation of the sources fails, an error is returned.
/// If the entry expression compilation fails, an error is returned.
pub fn compile_execution_context(
    stdlib: bool,
    expr: impl AsRef<str>,
    sources: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<ExecutionContext, AggregateError<Error>> {
    create_execution_context(stdlib, sources, expr.as_ref())
}

/// # Errors
/// If the evaluation of the entry expression causes an error
pub fn eval_in_context(
    context: &ExecutionContext,
    receiver: &mut dyn Receiver,
) -> Result<Value, AggregateError<Error>> {
    crate::init();

    let expr = get_entry_expr(&context.store, context.package)?;
    let global = |id: GlobalDefId| {
        context.store.get(id.package).and_then(|unit| {
            let item = unit.package.items.get(id.def)?;
            if let ItemKind::Callable(callable) = &item.kind {
                Some(callable)
            } else {
                None
            }
        })
    };

    let mut env = Env::with_empty_scope();
    eval_expr(&expr, &global, context.package, &mut env, receiver)
        .map_err(|e| AggregateError(vec![Error::Eval(e)]))
}

pub struct ExecutionContext {
    store: PackageStore,
    package: PackageId,
}

fn create_execution_context(
    stdlib: bool,
    sources: impl IntoIterator<Item = impl AsRef<str>>,
    expr: &str,
) -> Result<ExecutionContext, AggregateError<Error>> {
    let mut store = PackageStore::new();
    let mut session_deps: Vec<_> = vec![];

    if stdlib {
        let mut unit = compile::std();
        let pass_errs = run_default_passes(&mut unit);
        if unit.context.errors().is_empty() && pass_errs.is_empty() {
            session_deps.push(store.insert(unit));
        } else {
            let mut errors: Vec<Error> = unit
                .context
                .errors()
                .iter()
                .map(|e| Error::Compile(e.clone()))
                .collect();
            errors.extend(pass_errs.into_iter().map(Error::Pass));
            return Err(AggregateError(errors));
        }
    }

    let mut unit = compile(&store, session_deps.clone(), sources, expr);
    let pass_errs = run_default_passes(&mut unit);
    if !unit.context.errors().is_empty() || !pass_errs.is_empty() {
        let mut errors: Vec<Error> = unit
            .context
            .errors()
            .iter()
            .map(|e| Error::Compile(e.clone()))
            .collect();
        errors.extend(pass_errs.into_iter().map(Error::Pass));
        return Err(AggregateError(errors));
    }
    let basis_package = store.insert(unit);

    Ok(ExecutionContext {
        store,
        package: basis_package,
    })
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
