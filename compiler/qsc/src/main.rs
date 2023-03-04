// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use clap::Parser;
use miette::{Diagnostic, NamedSource, Report, SourceSpan};
use qsc_frontend::{compile, symbol, ErrorKind};
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

#[derive(Debug, Diagnostic, Error)]
#[error("symbol `{name}` not found in this scope")]
struct SymbolNotFound {
    name: String,
    #[label("not found in this scope")]
    span: SourceSpan,
}

fn main() {
    let cli = Cli::parse();
    let sources: Vec<_> = cli.sources.iter().map(|s| read_source(s)).collect();
    let context = compile(&sources, &cli.entry);

    for error in context.errors() {
        let (id, span) = context.source_span(error.span);
        let source_name = cli.sources[id.0].to_string_lossy();
        let source_code = &sources[id.0];

        match &error.kind {
            ErrorKind::Symbol(symbol::ErrorKind::Unresolved(candidates))
                if candidates.is_empty() =>
            {
                let report = Report::new(SymbolNotFound {
                    name: source_code[span].to_string(),
                    span: (span.lo..span.hi).into(),
                })
                .with_source_code(NamedSource::new(source_name, source_code.to_string()));
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
