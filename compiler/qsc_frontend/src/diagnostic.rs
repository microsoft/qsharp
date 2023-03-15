// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::{Diagnostic, LabeledSpan, Severity, SourceCode};
use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

#[derive(Clone)]
pub struct OffsetError<T> {
    error: T,
    offset: isize,
}

impl<T> OffsetError<T> {
    pub fn new(error: T, offset: isize) -> Self {
        Self { error, offset }
    }
}

impl<T: Diagnostic> Diagnostic for OffsetError<T> {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.error.code()
    }

    fn severity(&self) -> Option<Severity> {
        self.error.severity()
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.error.help()
    }

    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.error.url()
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        self.error.source_code()
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        self.error
            .labels()
            .map(|labels| -> Box<dyn Iterator<Item = _>> {
                Box::new(labels.map(|label| offset_span(self.offset, &label)))
            })
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        self.error.related()
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.error.diagnostic_source()
    }
}

impl<T: Debug> Debug for OffsetError<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.error.fmt(f)
    }
}

impl<T: Display> Display for OffsetError<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.error.fmt(f)
    }
}

impl<T: Debug + Display> Error for OffsetError<T> {}

fn offset_span(offset: isize, label: &LabeledSpan) -> LabeledSpan {
    let offset = label
        .offset()
        .checked_add_signed(offset)
        .expect("Offset shouldn't overflow.");
    LabeledSpan::new(label.label().map(ToString::to_string), offset, label.len())
}
