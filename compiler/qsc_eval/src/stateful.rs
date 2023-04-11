// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod tests;

use crate::{eval_stmt, Env};
use qsc_ast::ast::CallableDecl;
use qsc_passes::globals::GlobalId;
use std::collections::HashMap;
use std::string::String;

use qsc_frontend::compile::{self, CompileUnit, PackageStore};
use qsc_frontend::incremental::{Compiler, Fragment};
use std::io::Cursor;

use crate::output::CursorReceiver;
use crate::stateful::ouroboros_impl_execution_context::BorrowedMutFields;
use miette::Diagnostic;
use ouroboros::self_referencing;
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
    #[error(transparent)]
    #[diagnostic(transparent)]
    Incremental(qsc_frontend::incremental::Error),
}

#[self_referencing]
pub struct ExecutionContext {
    store: PackageStore,
    package: compile::PackageId,
    #[borrows(store)]
    #[covariant]
    compiler: Compiler<'this>,
    #[borrows(store)]
    #[not_covariant]
    globals: HashMap<GlobalId, &'this CallableDecl>,
    env: Option<Env>,
    cursor: Cursor<Vec<u8>>,
    #[borrows(mut cursor)]
    #[not_covariant]
    out: CursorReceiver<'this>,
}

#[derive(Clone, Debug)]
pub struct InterpreterResult {
    pub output: String,
    pub value: String,
    pub errors: Vec<Error>,
}

impl InterpreterResult {
    #[must_use]
    pub fn new(value: String, output: String, errors: Vec<Error>) -> Self {
        Self {
            output,
            value,
            errors,
        }
    }
}

pub struct Interpreter {
    context: ExecutionContext,
}

impl Interpreter {
    /// # Errors
    /// If the compilation of the standard library fails, an error is returned.
    /// If the compilation of the sources fails, an error is returned.
    pub fn new(
        nostdlib: bool,
        sources: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Self, (InterpreterResult, CompileUnit)> {
        let context = match create_execution_context(nostdlib, sources, None) {
            Ok(value) => value,
            Err(value) => return Err(value),
        };
        Ok(Self { context })
    }

    pub fn line(&mut self, line: impl AsRef<str>) -> impl Iterator<Item = InterpreterResult> {
        self.context
            .with_mut(|fields| eval_line_in_context(line, fields))
    }
}

fn create_execution_context(
    nostdlib: bool,
    sources: impl IntoIterator<Item = impl AsRef<str>>,
    expr: Option<String>,
) -> Result<ExecutionContext, (InterpreterResult, CompileUnit)> {
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
        let result = InterpreterResult::new(
            String::new(),
            String::new(),
            unit.context
                .errors()
                .iter()
                .map(|e| Error::Compile(e.clone()))
                .collect(),
        );
        return Err((result, unit));
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
        cursor: Cursor::new(Vec::<u8>::new()),
        out_builder: |cursor| CursorReceiver::new(cursor),
    }
    .build();
    Ok(context)
}

#[allow(clippy::needless_pass_by_value)]
fn eval_line_in_context(
    line: impl AsRef<str>,
    fields: BorrowedMutFields,
) -> impl Iterator<Item = InterpreterResult> {
    let mut results = vec![];
    let fragments = fields.compiler.compile_fragment(line);
    for fragment in fragments {
        match fragment {
            Fragment::Stmt(stmt) => {
                let mut env = fields.env.take().unwrap_or(Env::empty());
                let result = eval_stmt(
                    stmt,
                    fields.store,
                    fields.globals,
                    fields.compiler.resolutions(),
                    *fields.package,
                    &mut env,
                    fields.out,
                );
                let _ = fields.env.insert(env);
                let output = fields.out.dump();

                match result {
                    Ok(v) => {
                        results.push(InterpreterResult::new(format!("{v}"), output, vec![]));
                    }
                    Err(e) => {
                        results.push(InterpreterResult::new(
                            String::new(),
                            output,
                            vec![Error::Eval(e)],
                        ));
                        return results.into_iter();
                    }
                }
            }
            Fragment::Callable(decl) => {
                let id = GlobalId {
                    package: *fields.package,
                    node: decl.name.id,
                };
                fields.globals.insert(id, decl);
                results.push(InterpreterResult::new(String::new(), String::new(), vec![]));
            }
            Fragment::Error(errors) => {
                let e = errors
                    .iter()
                    .map(|e| Error::Incremental(e.clone()))
                    .collect();
                results.push(InterpreterResult::new(String::new(), String::new(), e));
            }
        }
    }
    results.into_iter()
}
