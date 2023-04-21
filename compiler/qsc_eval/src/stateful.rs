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
use qsc_passes::{
    globals::{extract_callables, GlobalId},
    run_default_passes,
};
use std::{collections::HashMap, string::String};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
#[error(transparent)]
#[diagnostic(transparent)]
pub enum Error {
    Eval(crate::Error),
    Compile(qsc_frontend::compile::Error),
    Pass(qsc_passes::Error),
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

    /// # Errors
    /// If the parsing of the line fails, an error is returned.
    /// If the compilation of the line fails, an error is returned.
    /// If there is a runtime error when interpreting the line, an error is returned.
    pub fn line(
        &mut self,
        receiver: &mut dyn Receiver,
        line: impl AsRef<str>,
    ) -> Result<Value, AggregateError<Error>> {
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
            return Err((AggregateError(errors), unit));
        }
    }
    let mut unit = compile(
        &store,
        session_deps.clone(),
        sources,
        &expr.unwrap_or_default(),
    );
    let pass_errs = run_default_passes(&mut unit);
    if !unit.context.errors().is_empty() || !pass_errs.is_empty() {
        let mut errors: Vec<Error> = unit
            .context
            .errors()
            .iter()
            .map(|e| Error::Compile(e.clone()))
            .collect();
        errors.extend(pass_errs.into_iter().map(Error::Pass));

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
) -> Result<Value, AggregateError<Error>> {
    let mut final_result = Value::UNIT;
    let fragments = fields.compiler.compile_fragment(line);
    for fragment in fragments {
        match fragment {
            Fragment::Stmt(stmt) => {
                let mut env = fields.env.take().unwrap_or(Env::with_empty_scope());
                let result = eval_stmt(&stmt, fields.globals, *fields.package, &mut env, receiver);
                let _ = fields.env.insert(env);

                match result {
                    Ok(v) => {
                        final_result = v;
                    }
                    Err(e) => {
                        return Err(AggregateError(vec![Error::Eval(e)]));
                    }
                }
            }
            Fragment::Callable(decl) => {
                let id = GlobalId {
                    package: *fields.package,
                    node: decl.name.id,
                };
                fields.globals.insert(id, Box::leak(Box::new(decl)));
                final_result = Value::UNIT;
            }
            Fragment::Error(errors) => {
                let e = errors
                    .iter()
                    .map(|e| Error::Incremental(e.clone()))
                    .collect();
                return Err(AggregateError(e));
            }
        }
    }
    Ok(final_result)
}
