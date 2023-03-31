// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Parser;
use miette::Result;

use miette::{Diagnostic, NamedSource, Report};
use qsc_eval::{output::GenericReceiver, Evaluator};
use qsc_frontend::{
    compile::{self, compile, Context, PackageStore, SourceIndex},
    diagnostic::OffsetError,
};
use qsc_passes::globals::extract_callables;
use std::{fs, io, string::String, sync::Arc};

#[derive(Debug, Parser)]
#[command(author, version, about, next_line_help = true)]
struct Cli {
    /// Use the given file on startup as initial session input
    #[arg(long = "use")]
    sources: Vec<PathBuf>,
    /// Open the given namespace(s) on startup before executing the entry expression or starting the REPL
    #[arg(long)]
    open: Vec<String>,
    /// Execute the given Q# expression on startup
    #[arg(long)]
    entry: Option<String>,
    /// Disable automatic inclusion of the standard library
    #[arg(long)]
    nostdlib: bool,
    /// Exit after loading the files or running the given file(s)/entry on the command line
    #[arg(long)]
    exec: bool,
}

fn main() -> Result<ExitCode> {
    let cli = Cli::parse();
    if cli.exec {
        unimplemented!("exec mode not yet implemented");
    }
    if !cli.open.is_empty() {
        unimplemented!("specifying open not yet implemented");
    }

    let sources: Vec<_> = cli.sources.iter().map(read_source).collect();
    let mut store = PackageStore::new();
    let deps = if cli.nostdlib {
        vec![]
    } else {
        vec![store.insert(compile::std())]
    };

    let unit = compile(
        &store,
        deps,
        &sources,
        &cli.entry.clone().unwrap_or_default(),
    );

    if unit.context.errors().is_empty() {
        let user = store.insert(unit);
        let unit = store
            .get(user)
            .expect("compile unit should be in package store");
        if let Some(expr) = &unit.package.entry {
            let globals = extract_callables(&store);
            let mut stdout = io::stdout();
            let mut out = GenericReceiver::new(&mut stdout);
            let evaluator = Evaluator::from_store(&store, user, &globals, &mut out);
            match evaluator.eval_expr(expr) {
                Ok((value, _)) => {
                    println!("{value}");
                    Ok(ExitCode::SUCCESS)
                }
                Err(error) => Err(ErrorReporter::new(cli, sources, &unit.context).report(error)),
            }
        } else {
            Ok(ExitCode::SUCCESS)
        }
    } else {
        let reporter = ErrorReporter::new(cli, sources, &unit.context);
        for error in unit.context.errors() {
            eprintln!("{:?}", reporter.report(error.clone()));
        }
        Ok(ExitCode::FAILURE)
    }
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
            entry: Arc::new(cli.entry.unwrap_or_default()),
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

fn read_source(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).unwrap()
}

fn source_name(paths: &[PathBuf], index: SourceIndex) -> &str {
    paths
        .get(index.0)
        .map_or("<unknown>", |p| match p.to_str() {
            Some(name) => name,
            None => "<unknown>",
        })
}
