// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use ariadne::{Label, Report, ReportKind, Source};
use clap::Parser;
use qsc_frontend::{compile, symbol, ErrorKind};
use std::{
    fs, io,
    path::{Path, PathBuf},
    result::Result,
    string::String,
};

#[derive(Parser)]
struct Cli {
    #[arg(required(true), num_args(1..))]
    sources: Vec<PathBuf>,

    #[arg(short, long, default_value = "")]
    entry: String,
}

fn main() {
    let cli = Cli::parse();
    let sources: Vec<_> = cli.sources.iter().map(|s| read_source(s)).collect();
    let context = compile(&sources, &cli.entry);

    for error in context.errors() {
        let (source_id, span) = context.source_span(error.span);
        let source_name = cli.sources[source_id.0].to_string_lossy();
        let source_code = &sources[source_id.0];

        match &error.kind {
            ErrorKind::Symbol(symbol::ErrorKind::Unresolved(candidates))
                if candidates.is_empty() =>
            {
                let symbol = &source_code[span];
                Report::build(ReportKind::Error, &source_name, span.lo)
                    .with_message(format!("symbol `{symbol}` not found in this scope"))
                    .with_label(
                        Label::new((&source_name, span.lo..span.hi))
                            .with_message("not found in this scope"),
                    )
                    .finish()
                    .print((&source_name, Source::from(source_code)))
                    .unwrap();
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
