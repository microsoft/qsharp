// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::output::Receiver;
use crate::val::Value;
use crate::{eval_expr, Env};
use ouroboros::self_referencing;
use qsc_ast::ast::CallableDecl;
use qsc_frontend::compile::{self, compile, PackageStore};
use qsc_passes::globals::{extract_callables, GlobalId};
use std::collections::HashMap;
use std::string::String;

use miette::Diagnostic;

use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Eval(crate::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Compile(qsc_frontend::compile::Error),
    #[error("nothing to evaluate; entry expression is empty")]
    EmptyExpr,
}

#[derive(Clone, Debug)]
pub struct InterpreterResult {
    pub value: Value,
    pub errors: Vec<Error>,
}

impl InterpreterResult {
    #[must_use]
    pub fn new(value: Value, errors: Vec<Error>) -> Self {
        Self { value, errors }
    }
}

pub fn eval(
    nostdlib: bool,
    expr: impl AsRef<str>,
    receiver: &mut dyn Receiver,
    sources: impl IntoIterator<Item = impl AsRef<str>>,
) -> InterpreterResult {
    crate::init();

    let mut store = PackageStore::new();

    let mut session_deps: Vec<_> = vec![];

    if !nostdlib {
        session_deps.push(store.insert(compile::std()));
    }

    // create a package with all defined dependencies for the session
    let unit = compile(&store, session_deps.clone(), sources, expr.as_ref());
    if !unit.context.errors().is_empty() {
        return InterpreterResult::new(
            Value::UNIT,
            unit.context
                .errors()
                .iter()
                .map(|e| Error::Compile(e.clone()))
                .collect(),
        );
    }

    let basis_package = store.insert(unit);
    session_deps.push(basis_package);

    let globals = extract_callables(&store);

    let expr = store
        .get_entry_expr(basis_package)
        .expect("entry expression should be present");
    let resolutions = store
        .get_resolutions(basis_package)
        .expect("package should be present in store");
    let mut env = Env::empty();
    let result = eval_expr(
        expr,
        &store,
        &globals,
        resolutions,
        basis_package,
        &mut env,
        receiver,
    );
    match result {
        Ok(v) => InterpreterResult::new(v, vec![]),
        Err(e) => InterpreterResult::new(Value::UNIT, vec![Error::Eval(e)]),
    }
}

/// # Errors
/// If the compilation of the standard library fails, an error is returned.
/// If the compilation of the sources fails, an error is returned.
/// If the entry expression compilation fails, an error is returned.
pub fn pre_compile_context(
    nostdlib: bool,
    expr: String,
    sources: impl IntoIterator<Item = String>,
) -> Result<ExecutionContext, InterpreterResult> {
    let sources = sources.into_iter().collect::<Vec<_>>();

    let mut store = PackageStore::new();

    let mut session_deps: Vec<_> = vec![];

    if !nostdlib {
        session_deps.push(store.insert(compile::std()));
    }

    create_execution_context(nostdlib, sources, Some(expr))
}

pub fn cached_eval(context: &ExecutionContext, receiver: &mut dyn Receiver) -> InterpreterResult {
    crate::init();

    let result = context.with(|f| {
        let expr = f
            .store
            .get_entry_expr(*f.package)
            .expect("entry expression should be present");
        let resolutions = f
            .store
            .get_resolutions(*f.package)
            .expect("package should be present in store");
        let mut env = Env::empty();
        eval_expr(
            expr,
            f.store,
            f.globals,
            resolutions,
            *f.package,
            &mut env,
            receiver,
        )
    });
    match result {
        Ok(v) => InterpreterResult::new(v, vec![]),
        Err(e) => InterpreterResult::new(Value::UNIT, vec![Error::Eval(e)]),
    }
}

#[self_referencing]
pub struct ExecutionContext {
    store: PackageStore,
    package: compile::PackageId,
    #[borrows(store)]
    #[not_covariant]
    globals: HashMap<GlobalId, &'this CallableDecl>,
}

fn create_execution_context(
    nostdlib: bool,
    sources: impl IntoIterator<Item = impl AsRef<str>>,
    expr: Option<String>,
) -> Result<ExecutionContext, InterpreterResult> {
    let mut store = PackageStore::new();
    let mut session_deps: Vec<_> = vec![];
    if !nostdlib {
        session_deps.push(store.insert(compile::std()));
    }
    let unit = compile(
        &store,
        session_deps.clone(),
        sources,
        &expr.unwrap_or_default(),
    );
    if !unit.context.errors().is_empty() {
        let errors = unit
            .context
            .errors()
            .iter()
            .map(|e| Error::Compile(e.clone()))
            .collect();
        return Err(InterpreterResult::new(Value::UNIT, errors));
    }
    let basis_package = store.insert(unit);

    let context = ExecutionContextBuilder {
        store,
        package: basis_package,
        globals_builder: extract_callables,
    }
    .build();
    Ok(context)
}
