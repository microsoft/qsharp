// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use clap::Parser;
use miette::{Diagnostic, LabeledSpan, NamedSource, Report, SourceCode};
use qsc_ast::ast::Span;
use qsc_frontend::{compile, symbol, Context, ErrorKind};
use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};
use std::{
    fs, io,
    path::{Path, PathBuf},
    result::Result,
    string::String,
};
use thiserror::Error;

#[derive(Parser)]
struct Cli {
    #[arg(required(true), num_args(1..))]
    sources: Vec<PathBuf>,
    #[arg(short, long, default_value = "")]
    entry: String,
}

struct OffsetDiagnostic<D> {
    diagnostic: D,
    source: Option<OffsetSource>,
}

impl<D> OffsetDiagnostic<D> {
    fn new(context: &Context, sources: &[(&Path, impl ToString)], diagnostic: D) -> Self
    where
        D: Diagnostic,
    {
        if let Some(label) = diagnostic.labels().and_then(|mut l| l.next()) {
            let id = context.find_source(label.offset());
            let (path, source) = &sources[id.0];
            let name = path.to_string_lossy();
            let source = NamedSource::new(name, source.to_string());
            let offset = context.offsets()[id.0];
            Self {
                diagnostic,
                source: Some(OffsetSource { source, offset }),
            }
        } else {
            Self {
                diagnostic,
                source: None,
            }
        }
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

#[derive(Debug, Diagnostic, Error)]
enum SymbolError {
    #[error("`{0}` not found in this scope")]
    NotFound(String, #[label("not found")] Span),
    #[error("`{0}` is ambiguous")]
    Ambiguous(
        String,
        #[label("ambiguous name")] Span,
        #[label("could refer to the item in this namespace")] Span,
        #[label("could also refer to the item in this namespace")] Span,
    ),
}

fn main() {
    let cli = Cli::parse();
    let sources: Vec<_> = cli
        .sources
        .iter()
        .map(|p| (p.as_path(), read_source(p)))
        .collect();
    let context = compile(sources.iter().map(|s| &s.1), &cli.entry);

    for error in context.errors() {
        match &error.kind {
            ErrorKind::Symbol(symbol::ErrorKind::NotFound(name)) => {
                let error = SymbolError::NotFound(name.to_string(), error.span);
                let report = Report::new(OffsetDiagnostic::new(&context, &sources, error));
                eprint!("{report:?}");
            }
            ErrorKind::Symbol(symbol::ErrorKind::Ambiguous(name, first, second)) => {
                let error = SymbolError::Ambiguous(name.to_string(), error.span, *first, *second);
                let report = Report::new(OffsetDiagnostic::new(&context, &sources, error));
                eprint!("{report:?}");
            }
            _ => eprintln!("{error:#?}"),
        }
    }
}

fn read_source(path: &Path) -> String {
    if path.as_os_str() == "-" {
        io::stdin().lines().map(Result::unwrap).collect()
    } else {
        fs::read_to_string(path).unwrap()
    }
}
