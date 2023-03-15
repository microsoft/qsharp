// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use clap::Parser;
use miette::{Diagnostic, NamedSource, Report};
use qsc_eval::Evaluator;
use qsc_frontend::{
    compile::{self, compile, Context, PackageStore, SourceIndex},
    diagnostic::OffsetError,
};
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
    result::Result,
    string::String,
    sync::Arc,
};

#[derive(Parser)]
#[command(arg_required_else_help(true))]
struct Cli {
    sources: Vec<PathBuf>,
    #[arg(short, long, default_value = "")]
    entry: String,
}

fn main() -> Result<ExitCode, qsc_eval::Error> {
    let cli = Cli::parse();
    let sources: Vec<_> = cli.sources.iter().map(read_source).collect();
    let mut store = PackageStore::new();
    let std = store.insert(compile::std());
    let unit = compile(&store, [std], &sources, &cli.entry);

    if unit.context.errors().is_empty() {
        let value = Evaluator::new(&unit).run()?;
        println!("{value}");
        Ok(ExitCode::SUCCESS)
    } else {
        let sources: Vec<_> = sources.into_iter().map(Arc::new).collect();
        let entry = Arc::new(cli.entry);
        for error in unit.context.errors() {
            let report = error_report(&cli.sources, &sources, &entry, &unit.context, error.clone());
            eprintln!("{report:?}");
        }

        Ok(ExitCode::FAILURE)
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
    entry: &Arc<String>,
    context: &Context,
    error: compile::Error,
) -> Report {
    let Some(first_label) = error.labels().and_then(|mut ls| ls.next()) else {
        return Report::new(error);
    };

    // Use the offset of the first labeled span to find which source code to include in the report.
    let (index, offset) = context.source(first_label.offset());
    let name = source_name(paths, index);
    let source = sources.get(index.0).unwrap_or(entry).clone();
    let source = NamedSource::new(name, source);

    // Adjust all spans in the error to be relative to the start of this source.
    let offset = -isize::try_from(offset).unwrap();
    Report::new(OffsetError::new(error, offset)).with_source_code(source)
}

fn source_name(paths: &[PathBuf], index: SourceIndex) -> &str {
    if index.0 == paths.len() {
        "<entry>"
    } else {
        paths
            .get(index.0)
            .map_or("<unknown>", |p| match p.to_str() {
                Some("-") => "<stdin>",
                Some(name) => name,
                None => "<unknown>",
            })
    }
}
