// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use std::{path::PathBuf, process::ExitCode};

use clap::Parser;

use miette::{Diagnostic, NamedSource, Report};
use qsc_frontend::{
    compile::{Context, SourceIndex},
    diagnostic::OffsetError,
};
use std::{string::String, sync::Arc};

use miette::{IntoDiagnostic, Result};
use qsc_eval::evaluate;
use qsc_eval::Env;
use qsc_passes::globals::GlobalId;

use qsc_eval::output::GenericReceiver;
use qsc_frontend::compile::{self, compile, PackageStore};
use qsc_frontend::incremental::{Compiler, Fragment};
use qsc_passes::globals::extract_callables;
use std::io::Write;
use std::{fs, io};

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
    if !cli.open.is_empty() {
        unimplemented!("specifying open not yet implemented");
    }

    repl(cli)
}

fn repl(cli: Cli) -> Result<ExitCode> {
    let sources: Vec<_> = read_source(cli.sources.as_slice()).into_diagnostic()?;
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

    if !unit.context.errors().is_empty() {
        let reporter = ErrorReporter::new(cli, sources, &unit.context);
        for error in unit.context.errors() {
            eprintln!("{:?}", reporter.report(error.clone()));
        }
        return Ok(ExitCode::FAILURE);
    }

    let mut store = PackageStore::new();
    let std = store.insert(compile::std());
    let sources: [&str; 0] = [];
    let user = store.insert(compile(&store, [], sources, ""));
    let mut compiler = Compiler::new(&store, [std]);
    let mut globals = extract_callables(&store);
    let mut env = Env::empty();

    let mut stdout = io::stdout();
    let mut out = GenericReceiver::new(&mut stdout);

    if cli.entry.is_some() {
        match compiler.compile_fragment(&cli.entry.unwrap_or_default()) {
            Fragment::Stmt(stmt) => {
                let (value, new_env) = evaluate(
                    stmt,
                    &store,
                    &globals,
                    compiler.resolutions(),
                    user,
                    env,
                    &mut out,
                )?;

                env = new_env;
                println!("{value}");
            }
            Fragment::Callable(decl) => {
                globals.insert(
                    GlobalId {
                        package: user,
                        node: decl.name.id,
                    },
                    decl,
                );
            }
        }
    }

    if cli.exec {
        return Ok(ExitCode::SUCCESS);
    }

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();

        if io::stdin().read_line(&mut line).into_diagnostic()? == 0 {
            println!();
            break Ok(ExitCode::SUCCESS);
        }

        match compiler.compile_fragment(&line) {
            Fragment::Stmt(stmt) => {
                let (value, new_env) = evaluate(
                    stmt,
                    &store,
                    &globals,
                    compiler.resolutions(),
                    user,
                    env,
                    &mut out,
                )?;

                env = new_env;
                println!("{value}");
            }
            Fragment::Callable(decl) => {
                globals.insert(
                    GlobalId {
                        package: user,
                        node: decl.name.id,
                    },
                    decl,
                );
            }
        }
    }
}

fn read_source(paths: &[PathBuf]) -> io::Result<Vec<String>> {
    paths.iter().map(fs::read_to_string).collect()
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

fn source_name(paths: &[PathBuf], index: SourceIndex) -> &str {
    paths
        .get(index.0)
        .map_or("<unknown>", |p| match p.to_str() {
            Some(name) => name,
            None => "<unknown>",
        })
}
