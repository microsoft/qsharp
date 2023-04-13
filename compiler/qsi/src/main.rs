// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

use std::sync::Arc;
use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use qsc_eval::output::CursorReceiver;
use qsc_eval::stateful::{Error, Interpreter};
use qsc_eval::val::Value;
use qsc_eval::AggregateError;
use qsc_frontend::compile::{Context, SourceIndex};
use qsc_frontend::diagnostic::OffsetError;

use std::string::String;

use miette::{Diagnostic, IntoDiagnostic, NamedSource, Report, Result};
use std::io::prelude::BufRead;
use std::io::{Cursor, Write};
use std::{fs, io};

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

    let interpreter = Interpreter::new(!cli.nostdlib, sources.clone());
    if let Err((_, unit)) = interpreter {
        let reporter = ErrorReporter::new(cli, sources, &unit.context);
        for error in unit.context.errors() {
            eprintln!("{:?}", reporter.report(error.clone()));
        }
        return Ok(ExitCode::FAILURE);
    }
    let mut interpreter = interpreter.expect("interpreter creation failed");
    let mut cursor = Cursor::new(Vec::<u8>::new());
    let mut receiver = CursorReceiver::new(&mut cursor);

    if let Some(line) = cli.entry {
        let results = interpreter.line(&mut receiver, line.clone());
        print_results(results, &receiver.dump(), &line);
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
                let next = iter.next().expect("Could nod read next line");
                line.push_str(&next);
            }

            if !line.trim().is_empty() {
                let results = interpreter.line(&mut receiver, line.clone());
                print_results(results, &receiver.dump(), &line);
            }

            print_prompt(false);
        }
    }
}

fn print_results(
    results: impl Iterator<Item = Result<Value, AggregateError<Error>>>,
    output: &str,
    line: &str,
) {
    for result in results {
        if !output.is_empty() {
            println!("{output}");
        }
        match result {
            Ok(value) => println!("{value}"),
            Err(errors) => {
                let reporter = InteractiveErrorReporter::new(line);
                for error in errors.0 {
                    eprintln!("{:?}", reporter.report(error.clone()));
                }
            }
        }
    }
}

fn print_prompt(is_multiline: bool) {
    if is_multiline {
        print!("    > ");
    } else {
        print!("qsi$ ");
    }
    io::stdout().flush().expect("Could not flush stdout");
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
        let offset = -isize::try_from(offset).expect("Could not convert offset to isize");
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
