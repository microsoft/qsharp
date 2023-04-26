// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod tests;

use miette::Diagnostic;
use qsc_data_structures::index_map::IndexMap;
use qsc_eval::{
    eval_stmt,
    output::Receiver,
    val::{GlobalId, Value},
    AggregateError, Env,
};
use qsc_frontend::{
    compile::{self, compile, CompileUnit, PackageStore},
    incremental::{self, Compiler, Fragment},
};
use qsc_hir::hir::{CallableDecl, ItemKind, LocalItemId, PackageId, Stmt};
use qsc_passes::run_default_passes;
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
    #[error("could not compile line")]
    Incremental(#[from] incremental::Error),
}

pub struct Interpreter {
    store: PackageStore,
    compiler: Compiler,
    package: PackageId,
    next_item_id: LocalItemId,
    callables: IndexMap<LocalItemId, CallableDecl>,
    env: Env,
}

impl Interpreter {
    /// # Errors
    /// If the compilation of the standard library fails, an error is returned.
    /// If the compilation of the sources fails, an error is returned.
    pub fn new(
        stdlib: bool,
        sources: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Self, (AggregateError<Error>, CompileUnit)> {
        let mut store = PackageStore::new();
        let mut session_deps = Vec::new();

        if stdlib {
            let mut unit = compile::std();
            let pass_errs = run_default_passes(&mut unit);
            if unit.errors.is_empty() && pass_errs.is_empty() {
                session_deps.push(store.insert(unit));
            } else {
                let errors = unit
                    .errors
                    .iter()
                    .map(|e| Error::Compile(e.clone()))
                    .chain(pass_errs.into_iter().map(Error::Pass))
                    .collect();
                return Err((AggregateError(errors), unit));
            }
        }

        let mut unit = compile(&store, session_deps.iter().copied(), sources, "");
        let pass_errs = run_default_passes(&mut unit);
        if !unit.errors.is_empty() || !pass_errs.is_empty() {
            let errors = unit
                .errors
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
        let compiler = Compiler::new(&store, session_deps);
        Ok(Self {
            store,
            compiler,
            package: session_package,
            next_item_id: LocalItemId::default(),
            callables: IndexMap::new(),
            env: Env::with_empty_scope(),
        })
    }

    /// # Errors
    /// If the parsing of the line fails, an error is returned.
    /// If the compilation of the line fails, an error is returned.
    /// If there is a runtime error when interpreting the line, an error is returned.
    pub fn line(
        &mut self,
        line: &str,
        receiver: &mut dyn Receiver,
    ) -> Result<Value, AggregateError<Error>> {
        let mut result = Value::UNIT;
        for fragment in self.compiler.compile_fragment(line) {
            match fragment {
                Fragment::Stmt(stmt) => match self.stmt(receiver, &stmt) {
                    Ok(value) => result = value,
                    Err(err) => return Err(AggregateError(vec![Error::Eval(err)])),
                },
                Fragment::Callable(decl) => {
                    self.callables.insert(self.next_item_id, decl);
                    self.next_item_id = self.next_item_id.successor();
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

    fn stmt(&mut self, receiver: &mut dyn Receiver, stmt: &Stmt) -> Result<Value, qsc_eval::Error> {
        eval_stmt(
            stmt,
            &|id| get_callable(&self.store, &self.callables, self.package, id),
            self.package,
            &mut self.env,
            receiver,
        )
    }
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
