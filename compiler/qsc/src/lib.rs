// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use clap::Parser;
use clap::Subcommand;
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

#[derive(Debug, Parser)]
#[command(arg_required_else_help(true))]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// build a q# project
    Build(BuildCommand),
    /// check a q# project
    Check(CheckCommand),
}

#[derive(Debug, Parser)]
#[command(arg_required_else_help(true))]
pub struct BuildCommand {
    pub sources: Vec<PathBuf>,
    #[arg(short, long, default_value = "")]
    pub entry: String,
}

#[derive(Debug, Parser)]
#[command(arg_required_else_help(true))]
pub struct CheckCommand {
    pub sources: Vec<PathBuf>,
    #[arg(short, long, default_value = "")]
    pub entry: String,
}

pub struct ErrorReporter<'a> {
    pub context: &'a Context,
    pub paths: Vec<PathBuf>,
    pub sources: Vec<Arc<String>>,
    pub entry: Arc<String>,
}

impl<'a> ErrorReporter<'a> {
    pub fn new(
        paths: Vec<PathBuf>,
        sources: Vec<String>,
        entry: String,
        context: &'a Context,
    ) -> Self {
        Self {
            context,
            paths: paths,
            sources: sources.into_iter().map(Arc::new).collect(),
            entry: Arc::new(entry),
        }
    }

    pub fn report(&self, diagnostic: impl Diagnostic + Send + Sync + 'static) -> Report {
        let Some(first_label) = diagnostic.labels().and_then(|mut ls| ls.next()) else {
            return Report::new(diagnostic);
        };

        // Use the offset of the first labeled span to find which source code to include in the report.
        let (index, offset) = self.context.source(first_label.offset());
        let name = source_name(&self.paths, index);
        let source = self.sources.get(index.0).unwrap_or(&self.entry).clone();
        let source = NamedSource::new(name, source);

        // Adjust all spans in the error to be relative to the start of this source.
        let offset = -isize::try_from(offset).unwrap();
        Report::new(OffsetError::new(diagnostic, offset)).with_source_code(source)
    }
}

pub fn exec(cli: Commands) -> miette::Result<ExitCode> {
    match cli {
        Commands::Build(cli) => _exec(cli.sources, cli.entry),
        Commands::Check(cli) => _exec(cli.sources, cli.entry),
    }
}

fn _exec(srcs: Vec<PathBuf>, entry: String) -> miette::Result<ExitCode> {
    let sources: Vec<_> = srcs.iter().map(read_source).collect();
    let mut store = PackageStore::new();
    let std = store.insert(compile::std());
    let unit = compile(&store, [std], &sources, &entry);

    if unit.context.errors().is_empty() {
        let user = store.insert(unit);
        match Evaluator::new(&store, user).run() {
            Ok(value) => {
                println!("{value}");
                Ok(ExitCode::SUCCESS)
            }
            Err(error) => {
                let unit = store.get(user).expect("store should have compiled package");
                Err(ErrorReporter::new(srcs, sources, entry, &unit.context).report(error))
            }
        }
    } else {
        let reporter = ErrorReporter::new(srcs, sources, entry, &unit.context);
        for error in unit.context.errors() {
            eprintln!("{:?}", reporter.report(error.clone()));
        }
        Ok(ExitCode::FAILURE)
    }
}

pub fn read_source(path: impl AsRef<Path>) -> String {
    if path.as_ref().as_os_str() == "-" {
        io::stdin().lines().map(Result::unwrap).collect()
    } else {
        fs::read_to_string(path).unwrap()
    }
}

pub fn source_name(paths: &[PathBuf], index: SourceIndex) -> &str {
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
