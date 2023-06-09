// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::{Diagnostic, SourceCode};
use qsc_frontend::compile::{Source, SourceMap};
use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

#[derive(Clone, Debug)]
pub(super) struct WithSource<S, E> {
    source: Option<S>,
    error: E,
    stack_trace: Option<String>,
}

impl<S, E> WithSource<S, E> {
    pub(super) fn new(source: S, error: E, stack_trace: Option<String>) -> Self {
        WithSource {
            source: Some(source),
            error,
            stack_trace,
        }
    }

    pub(super) fn error(&self) -> &E {
        &self.error
    }

    pub(super) fn stack_trace(&self) -> &Option<String> {
        &self.stack_trace
    }
}

impl<E: Diagnostic> WithSource<Source, E> {
    pub fn from_map(sources: &SourceMap, error: E, stack_trace: Option<String>) -> Self {
        let source = sources.find_by_diagnostic(&error).cloned();
        Self {
            source,
            error,
            stack_trace,
        }
    }
}

impl<S: Debug, E: Error> Error for WithSource<S, E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.error.source()
    }
}

impl<S: SourceCode + Debug, E: Diagnostic> Diagnostic for WithSource<S, E> {
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
        match &self.source {
            None => self.error.source_code(),
            Some(source) => Some(source),
        }
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

impl<S, E: Display> Display for WithSource<S, E> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.error.fmt(f)
    }
}
