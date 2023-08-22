// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::compile::SourceMap;
use miette::Diagnostic;
use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

#[derive(Clone, Debug)]
pub struct WithSource<E> {
    sources: SourceMap,
    error: E,
    //    stack_trace: Option<String>,
}

impl<E> WithSource<E> {
    pub fn error(&self) -> &E {
        &self.error
    }
}

//     pub fn stack_trace(&self) -> &Option<String> {
//         &self.stack_trace
//     }
// }

impl<E: Diagnostic> WithSource<E> {
    pub fn from_map(sources: &SourceMap, error: E) -> Self {
        // Filter the source map to avoid cloning all sources
        // and only clone the relevant ones
        let offsets = error
            .labels()
            .map(|labels| {
                labels
                    .map(|label| u32::try_from(label.offset()).expect("offset should fit into u32"))
            })
            .into_iter()
            .flatten();

        Self {
            sources: sources.filter(offsets),
            error,
            //stack_trace,
        }
    }

    pub fn no_sources(error: E) -> Self {
        Self {
            sources: SourceMap::default(),
            error,
        }
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
        Some(&self.sources)
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
