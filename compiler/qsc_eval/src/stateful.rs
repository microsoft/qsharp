// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod tests;

use crate::{
    eval_stmt,
    output::Receiver,
    stateful::ouroboros_impl_execution_context::BorrowedMutFields,
    val::{GlobalId, Value},
    AggregateError, Env,
};
use miette::Diagnostic;
use ouroboros::self_referencing;
use qsc_data_structures::index_map::IndexMap;
use qsc_frontend::{
    compile::{self, compile, CompileUnit, PackageStore},
    incremental::{self, Compiler, Fragment},
};
use qsc_hir::hir::{CallableDecl, ItemKind, LocalItemId, PackageId};
use qsc_passes::run_default_passes;
use std::string::String;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
#[error(transparent)]
#[diagnostic(transparent)]
pub enum Error {
    Eval(crate::Error),
    Compile(compile::Error),
    Pass(qsc_passes::Error),
    Incremental(incremental::Error),
}

#[self_referencing]
pub struct ExecutionContext {
    store: PackageStore,
    #[borrows(store)]
    #[covariant]
    compiler: Compiler<'this>,
    package: PackageId,
    next_item_id: LocalItemId,
    callables: IndexMap<LocalItemId, CallableDecl>,
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
        Ok(Self {
            context: create_execution_context(stdlib, sources, None)?,
        })
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
    let mut session_deps = Vec::new();

    if stdlib {
        let mut unit = compile::std();
        let pass_errs = run_default_passes(&mut unit);
        if unit.context.errors().is_empty() && pass_errs.is_empty() {
            session_deps.push(store.insert(unit));
        } else {
            let errors = unit
                .context
                .errors()
                .iter()
                .map(|e| Error::Compile(e.clone()))
                .chain(pass_errs.into_iter().map(Error::Pass))
                .collect();
            return Err((AggregateError(errors), unit));
        }
    }

    let mut unit = compile(
        &store,
        session_deps.iter().copied(),
        sources,
        &expr.unwrap_or_default(),
    );
    let pass_errs = run_default_passes(&mut unit);
    if !unit.context.errors().is_empty() || !pass_errs.is_empty() {
        let errors = unit
            .context
            .errors()
            .iter()
            .map(|e| Error::Compile(e.clone()))
            .chain(pass_errs.into_iter().map(Error::Pass))
            .collect();
        return Err((AggregateError(errors), unit));
    }

    let basis_package = store.insert(unit);
    session_deps.push(basis_package);

    let sources: [&str; 0] = [];
    let session_package = store.insert(compile(&store, [], sources, ""));
    Ok(ExecutionContextBuilder {
        store,
        compiler_builder: |store| Compiler::new(store, session_deps),
        package: session_package,
        next_item_id: LocalItemId::default(),
        callables: IndexMap::new(),
        env: None,
    }
    .build())
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
    let mut result = Value::UNIT;
    for fragment in fields.compiler.compile_fragment(line) {
        match fragment {
            Fragment::Stmt(stmt) => {
                let mut env = fields.env.take().unwrap_or(Env::with_empty_scope());
                let eval_result = eval_stmt(
                    &stmt,
                    &|id| get_callable(fields.store, fields.callables, *fields.package, id),
                    *fields.package,
                    &mut env,
                    receiver,
                );
                *fields.env = Some(env);
                match eval_result {
                    Ok(value) => result = value,
                    Err(err) => return Err(AggregateError(vec![Error::Eval(err)])),
                }
            }
            Fragment::Callable(decl) => {
                fields.callables.insert(*fields.next_item_id, decl);
                *fields.next_item_id = fields.next_item_id.successor();
                result = Value::UNIT;
            }
            Fragment::Error(errors) => {
                return Err(AggregateError(
                    errors.into_iter().map(Error::Incremental).collect(),
                ));
            }
        }
    }

    Ok(result)
}

fn get_callable<'a>(
    store: &'a PackageStore,
    callables: &'a IndexMap<LocalItemId, CallableDecl>,
    package: PackageId,
    id: GlobalId,
) -> Option<&'a CallableDecl> {
    if id.package == package {
        callables.get(id.item)
    } else {
        store.get(id.package).and_then(|unit| {
            let item = unit.package.items.get(id.item)?;
            if let ItemKind::Callable(callable) = &item.kind {
                Some(callable)
            } else {
                None
            }
        })
    }
}
