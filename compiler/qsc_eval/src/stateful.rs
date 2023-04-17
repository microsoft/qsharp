// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod tests;

use crate::{
    eval_stmt, output::Receiver, stateful::ouroboros_impl_execution_context::BorrowedMutFields,
    val::Value, AggregateError, Env,
};
use miette::Diagnostic;
use ouroboros::self_referencing;
use qsc_frontend::{
    compile::{self, compile, CompileUnit, PackageStore},
    incremental::{Compiler, Fragment},
};
use qsc_hir::hir::{CallableDecl, PackageId};
use qsc_passes::globals::{extract_callables, GlobalId};
use std::{collections::HashMap, string::String};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Eval(crate::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Compile(qsc_frontend::compile::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Incremental(qsc_frontend::incremental::Error),
}

#[self_referencing]
pub struct ExecutionContext {
    store: PackageStore,
    package: PackageId,
    #[borrows(store)]
    #[covariant]
    compiler: Compiler<'this>,
    #[borrows(store)]
    #[not_covariant]
    globals: HashMap<GlobalId, &'this CallableDecl>,
    env: Option<Env>,
}

pub struct Interpreter {
    context: ExecutionContext,
}

impl Interpreter {
    /// # Errors
    /// If the compilation of the standard library fails, an error is returned.
    /// If the compilation of the sources fails, an error is returned.
    pub fn new(
        stdlib: bool,
        sources: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Self, (AggregateError<Error>, CompileUnit)> {
        let context = match create_execution_context(stdlib, sources, None) {
            Ok(value) => value,
            Err(value) => return Err(value),
        };
        Ok(Self { context })
    }

    pub fn line(
        &mut self,
        receiver: &mut dyn Receiver,
        line: impl AsRef<str>,
    ) -> impl Iterator<Item = Result<Value, AggregateError<Error>>> {
        self.context
            .with_mut(|fields| eval_line_in_context(receiver, line, fields))
    }
}

fn create_execution_context(
    stdlib: bool,
    sources: impl IntoIterator<Item = impl AsRef<str>>,
    expr: Option<String>,
) -> Result<ExecutionContext, (AggregateError<Error>, CompileUnit)> {
    let mut store = PackageStore::new();
    let mut session_deps: Vec<_> = vec![];
    if stdlib {
        let unit = compile::std();
        if unit.context.errors().is_empty() {
            session_deps.push(store.insert(unit));
        } else {
            let errors = unit
                .context
                .errors()
                .iter()
                .map(|e| Error::Compile(e.clone()))
                .collect();
            return Err((AggregateError(errors), unit));
        }
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

        return Err((AggregateError(errors), unit));
    }
    let basis_package = store.insert(unit);
    session_deps.push(basis_package);
    let sources: [&str; 0] = [];
    let session_package = store.insert(compile(&store, [], sources, ""));
    let context = ExecutionContextBuilder {
        store,
        package: session_package,
        compiler_builder: |store| Compiler::new(store, session_deps),
        globals_builder: extract_callables,
        env: None,
    }
    .build();
    Ok(context)
}

// We can't take a mutable reference to the BorrowedMutFields
// because it isn't declared as mutable in the ouroboros macro.
// So we take the owned value and allow the clippy lint.
#[allow(clippy::needless_pass_by_value)]
fn eval_line_in_context(
    receiver: &mut dyn Receiver,
    line: impl AsRef<str>,
    fields: BorrowedMutFields,
) -> impl Iterator<Item = Result<Value, AggregateError<Error>>> {
    let mut results = vec![];
    let fragments = fields.compiler.compile_fragment(line);
    for fragment in fragments {
        match fragment {
            Fragment::Stmt(stmt) => {
                let mut env = fields.env.take().unwrap_or(Env::with_empty_scope());
                let result = eval_stmt(&stmt, fields.globals, *fields.package, &mut env, receiver);
                let _ = fields.env.insert(env);

                match result {
                    Ok(v) => {
                        results.push(Ok(v));
                    }
                    Err(e) => {
                        results.push(Err(AggregateError(vec![Error::Eval(e)])));
                        return results.into_iter();
                    }
                }
            }
            Fragment::Callable(decl) => {
                let id = GlobalId {
                    package: *fields.package,
                    node: decl.name.id,
                };
                fields.globals.insert(id, Box::leak(Box::new(decl)));
                results.push(Ok(Value::UNIT));
            }
            Fragment::Error(errors) => {
                let e = errors
                    .iter()
                    .map(|e| Error::Incremental(e.clone()))
                    .collect();
                results.push(Err(AggregateError(e)));
            }
        }
    }
    results.into_iter()
}
