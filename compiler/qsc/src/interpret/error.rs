// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use miette::Diagnostic;
use qsc_eval::debug::map_fir_package_to_hir;
use qsc_frontend::{
    compile::{PackageStore, SourceMap},
    error::WithSource,
};
use qsc_hir::hir::PackageId;

#[derive(Clone, Debug)]
pub struct Eval {
    error: WithSource<qsc_eval::Error>,
    stack_trace: Option<String>,
}

impl Eval {
    pub fn new(
        error: qsc_eval::Error,
        source_map: &SourceMap,
        store: &PackageStore,
        stack_trace: Option<String>,
    ) -> Self {
        // TODO: handle more than one span at some point...
        let span = error.spans().next();

        let error = match span {
            Some(span) => {
                let package_id = map_fir_package_to_hir(span.package);
                // TODO: Workaround: the user package in the interpreter is never updated
                // in the package store with each line. So if the package_id
                // is not std or core, we assume the package_id is associated with
                // the user package, but we return the up-to-date source map
                // instead of the one from the package store.
                let sources =
                    if package_id == PackageId::CORE || package_id == PackageId::CORE.successor() {
                        &store
                            .get(package_id)
                            .expect("expected to find std or core package")
                            .sources
                    } else {
                        source_map
                    };

                WithSource::from_map(sources, error)
            }
            None => WithSource::no_sources(error),
        };

        Eval { error, stack_trace }
    }

    pub fn stack_trace(&self) -> Option<&str> {
        match &self.stack_trace {
            Some(s) => Some(s.as_str()),
            None => None,
        }
    }
}

impl Error for Eval {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.error.source()
    }
}

impl Display for Eval {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.error.fmt(f)
    }
}

impl Diagnostic for Eval {
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
