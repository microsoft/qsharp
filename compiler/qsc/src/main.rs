// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use clap::Parser;
use miette::{Diagnostic, NamedSource, Report};
use qsc_frontend::{
    compile::{self, compile, Context, PackageStore},
    diagnostic::OffsetError,
};
use std::{
    fs, io,
    path::{Path, PathBuf},
    result::Result,
    string::String,
    sync::Arc,
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
    let sources: Vec<_> = cli.sources.iter().map(read_source).collect();
    let mut store = PackageStore::new();
    let std = store.insert(compile::std());
    let unit = compile(&store, [std], &sources, &cli.entry);

    let sources: Vec<_> = sources.into_iter().map(Arc::new).collect();
    for error in unit.context.errors() {
        let report = error_report(&cli.sources, &sources, &unit.context, error);
        eprintln!("{report:?}");
    }
}

fn read_source(path: impl AsRef<Path>) -> String {
    if path.as_ref().as_os_str() == "-" {
        io::stdin().lines().map(Result::unwrap).collect()
    } else {
        fs::read_to_string(path).unwrap()
    }
}

fn error_report(
    paths: &[PathBuf],
    sources: &[Arc<String>],
    context: &Context,
    error: &compile::Error,
) -> Report {
    let Some(first_label) = error.labels().and_then(|mut ls| ls.next()) else {
        return Report::new(error.clone());
    };

    let index = context.find_source(first_label.offset()).0;
    let name = paths[index].to_str().unwrap();
    let source = NamedSource::new(name, sources[index].clone());
    let offset = -isize::try_from(context.offsets()[index]).unwrap();
    Report::new(OffsetError::new(error.clone(), offset)).with_source_code(source)
}
