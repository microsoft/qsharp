// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::Diagnostic;
use qsc_eval::debug::map_fir_package_to_hir;
use qsc_frontend::{compile::PackageStore, error::WithSource};
use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub struct WithStack<E>
where
    E: Diagnostic + Error,
{
    error: E,
    stack_trace: Option<String>,
}

impl<E: Diagnostic> WithStack<E> {
    pub(super) fn new(error: E, stack_trace: Option<String>) -> Self {
        WithStack { error, stack_trace }
    }

    pub(super) fn stack_trace(&self) -> &Option<String> {
        &self.stack_trace
    }
}

impl<E: Display + Diagnostic> Display for WithStack<E> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        std::fmt::Display::fmt(&self.error, f)
    }
}

// #[diagnostic(transparent)] does not seem to work with generics
impl<E: Diagnostic> Diagnostic for WithStack<E> {
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
        self.error.source_code()
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

pub type Eval = WithStack<WithSource<qsc_eval::Error>>;

pub fn eval(error: qsc_eval::Error, store: &PackageStore, stack_trace: Option<String>) -> Eval {
    // TODO: handle more than one span at some point...
    let span = error.spans().next();

    let error = match span {
        Some(span) => {
            let sources = &store
                .get(map_fir_package_to_hir(span.package))
                .expect("expected to find package id in store")
                .sources;

            WithSource::from_map(sources, error)
        }
        None => WithSource::no_sources(error),
    };

    WithStack::new(error, stack_trace)
}
