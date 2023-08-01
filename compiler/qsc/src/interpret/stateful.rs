// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::{
    compile::{self, compile},
    error::WithSource,
};
use miette::Diagnostic;
use qsc_data_structures::index_map::IndexMap;
use qsc_eval::{
    backend::SparseSim,
    debug::CallStack,
    eval_stmt,
    output::Receiver,
    val::{GlobalId, Value},
    Env, Global, GlobalLookup,
};
use qsc_frontend::{
    compile::{CompileUnit, PackageStore, Source, SourceMap},
    incremental::{self, Compiler, Fragment},
};
use qsc_hir::hir::{CallableDecl, ItemKind, LocalItemId, PackageId, Stmt};
use qsc_passes::{PackageType, PassContext};
use std::{collections::HashSet, sync::Arc};
use thiserror::Error;

use super::{debug::format_call_stack, stateless};

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub struct CompileError(WithSource<Source, compile::Error>);

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub struct LineError(WithSource<Arc<str>, LineErrorKind>);

impl LineError {
    #[must_use]
    pub fn kind(&self) -> &LineErrorKind {
        self.0.error()
    }

    #[must_use]
    pub fn stack_trace(&self) -> &Option<String> {
        self.0.stack_trace()
    }
}

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
pub enum LineErrorKind {
    #[error(transparent)]
    Compile(#[from] incremental::Error),
    #[error(transparent)]
    Pass(#[from] qsc_passes::Error),
    #[error("runtime error")]
    Eval(#[from] qsc_eval::Error),
}

struct Lookup<'a> {
    store: &'a PackageStore,
    package: PackageId,
    udts: &'a HashSet<LocalItemId>,
    callables: &'a IndexMap<LocalItemId, CallableDecl>,
}

impl<'a> GlobalLookup<'a> for Lookup<'a> {
    fn get(&self, id: GlobalId) -> Option<Global<'a>> {
        get_global(self.store, self.udts, self.callables, self.package, id)
    }
}

pub struct Interpreter {
    store: PackageStore,
    package: PackageId,
    compiler: Compiler,
    udts: HashSet<LocalItemId>,
    callables: IndexMap<LocalItemId, CallableDecl>,
    passes: PassContext,
    env: Env,
    sim: SparseSim,
}

impl Interpreter {
    /// # Errors
    /// If the compilation of the standard library fails, an error is returned.
    /// If the compilation of the sources fails, an error is returned.
    pub fn new(std: bool, sources: SourceMap) -> Result<Self, Vec<CompileError>> {
        let mut store = PackageStore::new(compile::core());
        let mut dependencies = Vec::new();
        if std {
            dependencies.push(store.insert(compile::std(&store)));
        }

        let (unit, errors) = compile(&store, &dependencies, sources, PackageType::Lib);
        if !errors.is_empty() {
            return Err(errors
                .into_iter()
                .map(|error| CompileError(WithSource::from_map(&unit.sources, error, None)))
                .collect());
        }

        dependencies.push(store.insert(unit));
        let package = store.insert(CompileUnit::default());
        let compiler = Compiler::new(&store, dependencies);
        Ok(Self {
            store,
            package,
            compiler,
            udts: HashSet::new(),
            callables: IndexMap::new(),
            passes: PassContext::default(),
            env: Env::with_empty_scope(),
            sim: SparseSim::new(),
        })
    }

    /// # Errors
    /// If the parsing of the line fails, an error is returned.
    /// If the compilation of the line fails, an error is returned.
    /// If there is a runtime error when interpreting the line, an error is returned.
    pub fn interpret_line(
        &mut self,
        receiver: &mut impl Receiver,
        line: &str,
    ) -> Result<Value, Vec<LineError>> {
        let mut result = Value::unit();

        let mut fragments = self.compiler.compile_fragments(line).map_err(|errors| {
            let source = line.into();
            errors
                .into_iter()
                .map(|error| LineError(WithSource::new(Arc::clone(&source), error.into(), None)))
                .collect::<Vec<_>>()
        })?;

        let pass_errors = fragments
            .iter_mut()
            .flat_map(|fragment| {
                self.passes
                    .run(self.store.core(), self.compiler.assigner_mut(), fragment)
            })
            .collect::<Vec<_>>();
        if !pass_errors.is_empty() {
            let source = line.into();
            return Err(pass_errors
                .into_iter()
                .map(|error| LineError(WithSource::new(Arc::clone(&source), error.into(), None)))
                .collect());
        }

        for fragment in fragments {
            match fragment {
                Fragment::Item(item) => match item.kind {
                    ItemKind::Callable(callable) => self.callables.insert(item.id, callable),
                    ItemKind::Namespace(..) => {}
                    ItemKind::Ty(..) => {
                        self.udts.insert(item.id);
                    }
                },
                Fragment::Stmt(stmt) => match self.eval_stmt(receiver, &stmt) {
                    Ok(value) => result = value,
                    Err((error, call_stack)) => {
                        let stack_trace = if call_stack.is_empty() {
                            None
                        } else {
                            Some(self.render_call_stack(&call_stack, &error))
                        };

                        return Err(vec![LineError(WithSource::new(
                            line.into(),
                            error.into(),
                            stack_trace,
                        ))]);
                    }
                },
            }
        }

        Ok(result)
    }

    fn eval_stmt(
        &mut self,
        receiver: &mut impl Receiver,
        stmt: &Stmt,
    ) -> Result<Value, (qsc_eval::Error, CallStack)> {
        let globals = Lookup {
            store: &self.store,
            package: self.package,
            udts: &self.udts,
            callables: &self.callables,
        };

        eval_stmt(
            stmt,
            &globals,
            &mut self.env,
            &mut self.sim,
            self.package,
            receiver,
        )
    }

    fn render_call_stack(&self, call_stack: &CallStack, error: &dyn std::error::Error) -> String {
        let globals = Lookup {
            store: &self.store,
            package: self.package,
            udts: &self.udts,
            callables: &self.callables,
        };
        format_call_stack(&self.store, &globals, call_stack, error)
    }
}

fn get_global<'a>(
    store: &'a PackageStore,
    udts: &'a HashSet<LocalItemId>,
    callables: &'a IndexMap<LocalItemId, CallableDecl>,
    package: PackageId,
    id: GlobalId,
) -> Option<Global<'a>> {
    if id.package == package {
        udts.contains(&id.item)
            .then_some(Global::Udt)
            .or_else(|| callables.get(id.item).map(Global::Callable))
    } else {
        stateless::get_global(store, id)
    }
}
