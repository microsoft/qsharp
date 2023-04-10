// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::output::Receiver;
use crate::{eval_expr, Env};
use std::string::String;

use qsc_frontend::compile::{self, PackageStore};

use miette::Diagnostic;

use qsc_frontend::compile::compile;
use qsc_passes::globals::extract_callables;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Eval(crate::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Compile(qsc_frontend::compile::Error),
}

#[derive(Clone, Debug)]
pub struct InterpreterResult {
    pub value: String,
    pub errors: Vec<Error>,
}

impl InterpreterResult {
    #[must_use]
    pub fn new(value: String, errors: Vec<Error>) -> Self {
        Self { value, errors }
    }
}

pub fn eval(
    nostdlib: bool,
    expr: impl AsRef<str>,
    receiver: &mut dyn Receiver,
    sources: impl IntoIterator<Item = impl AsRef<str>>,
) -> InterpreterResult {
    let mut store = PackageStore::new();

    let mut session_deps: Vec<_> = vec![];

    if !nostdlib {
        session_deps.push(store.insert(compile::std()));
    }

    // create a package with all defined dependencies for the session
    let unit = compile(&store, session_deps.clone(), sources, expr.as_ref());
    if !unit.context.errors().is_empty() {
        return InterpreterResult::new(
            String::new(),
            unit.context
                .errors()
                .iter()
                .map(|e| crate::stateless::Error::Compile(e.clone()))
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
        Ok(v) => InterpreterResult::new(format!("{v}"), vec![]),
        Err(e) => InterpreterResult::new(String::new(), vec![crate::stateless::Error::Eval(e)]),
    }
}
