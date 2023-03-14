// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

mod diagnostic;

use crate::diagnostic::OffsetDiagnostic;
use clap::Parser;
use miette::Report;
use qsc_frontend::compile::{self, compile, PackageStore};
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
    let sources: Vec<_> = cli
        .sources
        .iter()
        .map(|p| (p.as_path(), read_source(p)))
        .collect();

    let mut store = PackageStore::new();
    let std = store.insert(compile::std());
    let unit = compile(
        &PackageStore::new(),
        [std],
        sources.iter().map(|s| s.1.as_str()),
        &cli.entry,
    );

    for error in unit.context.errors() {
        let error = OffsetDiagnostic::new(&unit.context, &sources, error.clone());
        eprint!("{:?}", Report::new(error));
    }
}

fn read_source(path: &Path) -> String {
    if path.as_os_str() == "-" {
        io::stdin().lines().map(Result::unwrap).collect()
    } else {
        fs::read_to_string(path).unwrap()
    }
}
