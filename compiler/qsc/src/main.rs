// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use clap::Parser;
use miette::{Diagnostic, NamedSource, Report};
use qsc_ast::{mut_visit::MutVisitor, visit::Visitor};
use qsc_eval::Evaluator;
use qsc_frontend::{
    compile::{self, compile, Context, PackageStore, SourceIndex},
    diagnostic::OffsetError,
    id::Assigner,
    parse,
    resolve::GlobalTable,
};
use std::{
    fs,
    io::{self, Write},
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
    #[arg(short, long, default_value_t = false)]
    interactive: bool,
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
    if cli.interactive {
        repl().unwrap();
        return Ok(ExitCode::SUCCESS);
    }

    let sources: Vec<_> = cli.sources.iter().map(read_source).collect();
    let mut store = PackageStore::new();
    let std = store.insert(compile::std());
    let unit = compile(&store, [std], &sources, &cli.entry);

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

fn repl() -> io::Result<()> {
    let mut store = PackageStore::new();
    let std = store.insert(compile::std());
    let mut globals = GlobalTable::new();
    globals.set_package(std);
    globals.visit_package(&store.get(std).unwrap().package);
    let mut resolver = globals.into_resolver();
    resolver.push_scope();
    let mut assigner = Assigner::new();
    let mut evaluator = Evaluator::new(&store, std);
    evaluator.push_scope();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 {
            println!();
            break Ok(());
        }

        let (mut stmt, errors) = parse::stmt(&line);
        assert!(errors.is_empty(), "parsing failed");
        assigner.visit_stmt(&mut stmt);
        let stmt = Box::leak(Box::new(stmt));
        resolver.visit_stmt(stmt);
        assert!(resolver.errors().is_empty(), "resolution failed");
        let value = evaluator.repl(resolver.resolutions(), stmt).unwrap();
        println!("{value}");
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
