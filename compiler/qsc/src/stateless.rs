// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    compile::{self, compile},
    error::WithSource,
};
use miette::Diagnostic;
use qsc_eval::{
    eval_expr,
    output::Receiver,
    val::{GlobalId, Value},
    Env,
};
use qsc_frontend::compile::{PackageStore, SourceMap};
use qsc_hir::hir::{CallableDecl, Expr, ItemKind, PackageId};
use qsc_passes::entry_point::extract_entry;
use thiserror::Error;

pub struct ExecutionContext {
    store: PackageStore,
    package: PackageId,
}

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
///
/// If the compilation of the sources fails, an error is returned.
/// If the evaluation of the entry expression causes an error
pub fn eval(
    std: bool,
    receiver: &mut dyn Receiver,
    sources: SourceMap,
) -> Result<Value, Vec<WithSource<Error>>> {
    qsc_eval::init();
    let mut store = PackageStore::new();
    let mut dependencies = Vec::new();
    if std {
        dependencies.push(store.insert(compile::std()));
    }

    // create a package with all defined dependencies for the session
    let (unit, errors) = compile(&store, dependencies.clone(), sources);
    if !errors.is_empty() {
        return Err(errors
            .into_iter()
            .map(|error| WithSource::new(&unit.sources, error.into()))
            .collect());
    }

    let basis_package = store.insert(unit);
    dependencies.push(basis_package);

    let expr = get_entry_expr(&store, basis_package)?;
    eval_expr(
        &expr,
        &|id| get_callable(&store, id),
        basis_package,
        &mut Env::with_empty_scope(),
        receiver,
    )
    .map_err(|error| {
        vec![WithSource::new(
            &store.get(basis_package).unwrap().sources,
            error.into(),
        )]
    })
}

/// # Errors
///
/// If the evaluation of the entry expression causes an error
pub fn eval_in_context(
    context: &ExecutionContext,
    receiver: &mut dyn Receiver,
) -> Result<Value, Vec<WithSource<Error>>> {
    qsc_eval::init();
    // let expr = get_entry_expr(&context.store, context.package)?;
    eval_expr(
        &get_entry_expr(&context.store, context.package)?,
        &|id| get_callable(&context.store, id),
        context.package,
        &mut Env::with_empty_scope(),
        receiver,
    )
    .map_err(|error| {
        vec![WithSource::new(
            &context.store.get(context.package).unwrap().sources,
            error.into(),
        )]
    })
}

/// # Errors
///
/// If the compilation of the sources fails, an error is returned.
pub fn compile_execution_context(
    std: bool,
    sources: SourceMap,
) -> Result<ExecutionContext, Vec<Error>> {
    let mut store = PackageStore::new();
    let mut dependencies = Vec::new();
    if std {
        dependencies.push(store.insert(compile::std()));
    }

    let (unit, errors) = compile(&store, dependencies, sources);
    if errors.is_empty() {
        let package = store.insert(unit);
        Ok(ExecutionContext { store, package })
    } else {
        Err(errors.into_iter().map(Into::into).collect())
    }
}

fn get_entry_expr(
    store: &PackageStore,
    basis_package: PackageId,
) -> Result<Expr, Vec<WithSource<Error>>> {
    if let Some(expr) = store.get_entry_expr(basis_package) {
        Ok(expr.clone())
    } else {
        let package = &store
            .get(basis_package)
            .expect("package should be in store")
            .package;

        extract_entry(package).map_err(|errors| {
            let sources = &store.get(basis_package).unwrap().sources;
            errors
                .into_iter()
                .map(|error| WithSource::new(sources, error.into()))
                .collect()
        })
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
