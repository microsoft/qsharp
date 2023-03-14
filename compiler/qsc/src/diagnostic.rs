// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::{Diagnostic, LabeledSpan, NamedSource, SourceCode};
use qsc_frontend::compile::Context;
use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
    path::Path,
};

pub(super) struct OffsetDiagnostic<D> {
    diagnostic: D,
    source: Option<OffsetSource>,
}

impl<D> OffsetDiagnostic<D> {
    pub(super) fn new(context: &Context, sources: &[(&Path, impl ToString)], diagnostic: D) -> Self
    where
        D: Diagnostic,
    {
        let source = diagnostic.labels().and_then(|mut i| i.next()).map(|label| {
            let id = context.find_source(label.offset());
            let (path, source) = &sources[id.0];
            let name = path.to_string_lossy();
            let source = NamedSource::new(name, source.to_string());
            let offset = context.offsets()[id.0];
            OffsetSource { source, offset }
        });

        Self { diagnostic, source }
    }
}

impl<D: Diagnostic> Diagnostic for OffsetDiagnostic<D> {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.diagnostic.code()
    }

    fn severity(&self) -> Option<miette::Severity> {
        self.diagnostic.severity()
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.diagnostic.help()
    }

    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.diagnostic.url()
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        match &self.source {
            None => None,
            Some(source) => Some(&source.source),
        }
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        if let Some(ls) = self.diagnostic.labels() {
            Some(Box::new(ls.map(|l| {
                LabeledSpan::new(
                    l.label().map(ToString::to_string),
                    l.offset() - self.source.as_ref().map(|s| s.offset).unwrap_or_default(),
                    l.len(),
                )
            })))
        } else {
            None
        }
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        self.diagnostic.related()
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.diagnostic.diagnostic_source()
    }
}

impl<D: Debug> Debug for OffsetDiagnostic<D> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.diagnostic.fmt(f)
    }
}

impl<D: Display> Display for OffsetDiagnostic<D> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.diagnostic.fmt(f)
    }
}

impl<D: Debug + Display> Error for OffsetDiagnostic<D> {}

struct OffsetSource {
    source: NamedSource,
    offset: usize,
}
