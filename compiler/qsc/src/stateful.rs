// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod tests;

use crate::compile::{self, compile};
use miette::Diagnostic;
use qsc_data_structures::index_map::IndexMap;
use qsc_eval::{
    eval_stmt,
    output::Receiver,
    val::{GlobalId, Value},
    Env,
};
use qsc_frontend::{
    compile::{CompileUnit, PackageStore, Source, SourceMap},
    incremental::{self, Compiler, Fragment},
};
use qsc_hir::hir::{CallableDecl, ItemKind, LocalItemId, PackageId, Stmt};
use std::{error, fmt::Display};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct SourceError {
    source: Source,
    error: Error,
}

impl error::Error for SourceError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.error.source()
    }
}

impl Diagnostic for SourceError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.error.code()
    }

    fn severity(&self) -> Option<miette::Severity> {
        self.error.severity()
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.error.help()
    }

    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.error.url()
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.source)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.error.labels()
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        self.error.related()
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.error.diagnostic_source()
    }
}

impl Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.error, f)
    }
}

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
pub enum Error {
    #[error("could not compile initial source code")]
    Compile(#[from] compile::Error),
    #[error(transparent)]
    Incremental(#[from] incremental::Error),
    #[error("program encountered an error while running")]
    Eval(#[from] qsc_eval::Error),
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
    pub fn new(std: bool, sources: SourceMap) -> Result<Self, Vec<SourceError>> {
        let mut store = PackageStore::new();
        let mut dependencies = Vec::new();
        if std {
            dependencies.push(store.insert(compile::std()));
        }

        let (unit, errors) = compile(&store, dependencies.iter().copied(), sources);
        if !errors.is_empty() {
            return Err(errors
                .into_iter()
                .map(|error| {
                    let source = error
                        .labels()
                        .and_then(|mut labels| labels.next())
                        .map_or_else(unknown_source, |label| {
                            unit.sources.find_by_offset(label.offset()).clone()
                        });

                    SourceError {
                        source,
                        error: Error::Compile(error),
                    }
                })
                .collect());
        }

        let basis_package = store.insert(unit);
        dependencies.push(basis_package);
        let session_package = store.insert(CompileUnit::default());
        let compiler = Compiler::new(&store, dependencies);

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
    pub fn line(&mut self, line: &str, receiver: &mut dyn Receiver) -> Result<Value, Vec<Error>> {
        let mut result = Value::unit();
        for fragment in self.compiler.compile_fragment(line) {
            match fragment {
                Fragment::Stmt(stmt) => match self.stmt(receiver, &stmt) {
                    Ok(value) => result = value,
                    Err(err) => return Err(vec![Error::Eval(err)]),
                },
                Fragment::Callable(decl) => {
                    self.callables.insert(self.next_item_id, decl);
                    self.next_item_id = self.next_item_id.successor();
                    result = Value::unit();
                }
                Fragment::Error(errors) => {
                    return Err(errors.into_iter().map(Error::Incremental).collect());
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

fn unknown_source() -> Source {
    Source {
        name: "<unknown>".into(),
        contents: "".into(),
        offset: 0,
    }
}
