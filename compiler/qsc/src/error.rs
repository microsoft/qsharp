// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::Diagnostic;
use qsc_frontend::compile::{Source, SourceMap};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug)]
pub struct WithSource<E> {
    error: E,
    source: Option<Source>,
}

impl<E: Diagnostic> WithSource<E> {
    pub fn new(sources: &SourceMap, error: E) -> Self {
        let source = sources.find_diagnostic(&error).cloned();
        Self { error, source }
    }
}

impl<E: Error> Error for WithSource<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.error.source()
    }
}

impl<E: Diagnostic> Diagnostic for WithSource<E> {
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

impl<E: Display> Display for WithSource<E> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.error.fmt(f)
    }
}
