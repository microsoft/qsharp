// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use std::sync::Arc;
use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use qsc_eval::interactive::Interpreter;
use qsc_frontend::compile::{Context, SourceIndex};
use qsc_frontend::diagnostic::OffsetError;

use std::string::String;

use miette::{IntoDiagnostic, NamedSource, Result};
use std::io::prelude::BufRead;
use std::io::Write;
use std::{fs, io};

use miette::{Diagnostic, Report};

#[derive(Debug, Parser)]
#[command(author, version, about, next_line_help = true)]
struct Cli {
    /// Use the given file on startup as initial session input
    #[arg(long = "use")]
    sources: Vec<PathBuf>,
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

    repl(cli)
}

fn repl(cli: Cli) -> Result<ExitCode> {
    let sources: Vec<_> = read_source(cli.sources.as_slice()).into_diagnostic()?;

    let interpreter = Interpreter::new(cli.nostdlib, sources.clone());
    if let Err(ref unit) = interpreter {
        let reporter = ErrorReporter::new(cli, sources, &unit.context);
        for error in unit.context.errors() {
            eprintln!("{:?}", reporter.report(error.clone()));
        }
        return Ok(ExitCode::FAILURE);
    }
    let mut interpreter = interpreter.unwrap();

    if let Some(line) = cli.entry {
        let results = interpreter.line(line.clone());
        print_results(results, &line);
    }

    if cli.exec {
        return Ok(ExitCode::SUCCESS);
    }

    loop {
        print_prompt(false);

        let stdin = io::BufReader::new(io::stdin());
        let mut iter = stdin.lines().map(Result::unwrap);
        while let Some(mut line) = iter.next() {
            while !line.is_empty() && &line[line.len() - 1..] == "\\" {
                print_prompt(true);
                line.pop(); // remove '\' from line
                let next = iter.next().unwrap();
                line.push_str(&next);
            }

            // TODO: if multiple statements are entered, we currently only
            // evaluate the first one. We need to evaluate all of them. This
            // will require updates to parsing to read multiple statements
            // followed by the EOF token.
            if !line.trim().is_empty() {
                let results = interpreter.line(line.clone());
                print_results(results, &line);
            }

            print_prompt(false);
        }
    }
}

fn print_results(results: Vec<qsc_eval::interactive::InterpreterResult>, line: &str) {
    for result in results {
        if !result.value.is_empty() {
            println!("{}", result.value);
        }
        if !result.output.is_empty() {
            println!("{}", result.output);
        }
        if !result.errors.is_empty() {
            let reporter = InteractiveErrorReporter::new(line);
            for error in result.errors {
                eprintln!("{:?}", reporter.report(error.clone()));
            }
        }
    }
}

fn print_prompt(is_multiline: bool) {
    if is_multiline {
        print!("    > ");
    } else {
        print!("qsci$ ");
    }
    io::stdout().flush().unwrap();
}

fn read_source(paths: &[PathBuf]) -> io::Result<Vec<String>> {
    paths.iter().map(fs::read_to_string).collect()
}

struct InteractiveErrorReporter {
    line: String,
}

impl InteractiveErrorReporter {
    fn new(line: impl AsRef<str>) -> Self {
        Self {
            line: line.as_ref().to_owned(),
        }
    }

    fn report(&self, error: impl Diagnostic + Send + Sync + 'static) -> Report {
        Report::new(error).with_source_code(self.line.clone())
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

    fn report(&self, error: impl Diagnostic + Send + Sync + 'static) -> Report {
        let Some(first_label) = error.labels().and_then(|mut ls| ls.next()) else {
            return Report::new(error);
        };

        // Use the offset of the first labeled span to find which source code to include in the report.
        let (index, offset) = self.context.source(first_label.offset());
        let name = source_name(&self.paths, index);
        let source = self.sources.get(index.0).unwrap_or(&self.entry).clone();
        let source = NamedSource::new(name, source);

        // Adjust all spans in the error to be relative to the start of this source.
        let offset = -isize::try_from(offset).unwrap();
        Report::new(OffsetError::new(error, offset)).with_source_code(source)
    }
}

fn source_name(paths: &[PathBuf], index: SourceIndex) -> &str {
    paths
        .get(index.0)
        .map_or("<unknown>", |p| match p.to_str() {
            Some("-") => "<stdin>",
            Some(name) => name,
            None => "<unknown>",
        })
}
