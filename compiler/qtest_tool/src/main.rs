// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use clap::Parser;
use miette::{Diagnostic, NamedSource, Report};
use qsc_eval::Evaluator;
use qsc_frontend::{
    compile::{self, compile, CompileUnit, Context, PackageStore, SourceIndex},
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
    #[arg(short, long)]
    tree: Option<PathBuf>,
}

struct ErrorReporter<'a> {
    context: &'a Context,
    paths: Vec<PathBuf>,
    sources: Vec<Arc<String>>,
    entry: Arc<String>,
}

impl<'a> ErrorReporter<'a> {
    fn new(cli: Cli, sources: Vec<String>, context: &'a Context) -> Self {
        Self {
            context,
            paths: cli.sources,
            sources: sources.into_iter().map(Arc::new).collect(),
            entry: Arc::new(cli.entry),
        }
    }

    fn report(&self, diagnostic: impl Diagnostic + Send + Sync + 'static) -> Report {
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

fn main() -> miette::Result<ExitCode> {
    let cli = Cli::parse();
    let sources: Vec<_> = cli.sources.iter().map(read_source).collect();
    let mut store = PackageStore::new();
    let std = store.insert(compile::std());
    let unit = compile(&store, [std], &sources, &cli.entry);

    if let Some(tree_path) = &cli.tree {
        print_compilation_unit(&unit, tree_path);
    }

    if unit.context.errors().is_empty() {
        let user = store.insert(unit);
        match Evaluator::new(&store, user).run() {
            Ok(value) => {
                println!("{value}");
                Ok(ExitCode::SUCCESS)
            }
            Err(error) => {
                let unit = store.get(user).expect("store should have compiled package");
                Err(ErrorReporter::new(cli, sources, &unit.context).report(error))
            }
        }
    } else {
        let reporter = ErrorReporter::new(cli, sources, &unit.context);
        for error in unit.context.errors() {
            eprintln!("{:?}", reporter.report(error.clone()));
        }
        Ok(ExitCode::FAILURE)
    }
}

fn print_compilation_unit(unit: &CompileUnit, path: &Path) {
    if path.as_os_str() == "-" {
        println!("{unit:#?}");
    } else {
        fs::write(path, format!("{unit:#?}")).unwrap();
    }
}

fn read_source(path: impl AsRef<Path>) -> String {
    if path.as_ref().as_os_str() == "-" {
        io::stdin().lines().map(Result::unwrap).collect()
    } else {
        fs::read_to_string(path).unwrap()
    }
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
