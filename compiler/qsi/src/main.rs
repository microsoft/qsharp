// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

use clap::Parser;
use miette::{
    Diagnostic, IntoDiagnostic, MietteError, MietteSpanContents, Report, Result, SourceCode,
    SourceSpan, SpanContents,
};
use num_bigint::BigUint;
use num_complex::Complex64;
use qsc_eval::{
    output::{format_state_id, Receiver},
    stateful::{Error, Interpreter},
    val::Value,
    AggregateError,
};
use qsc_frontend::compile::{SourceIndex, SourceMap};
use std::{
    fs,
    io::{self, prelude::BufRead, Write},
    path::PathBuf,
    process::ExitCode,
    string::String,
    sync::Arc,
};

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
        let reporter = ErrorReporter::new(cli, sources, &unit.sources);
        for error in unit.errors {
            eprintln!("{:?}", reporter.report(Report::new(error)));
        }
        return Ok(ExitCode::FAILURE);
    }
    let mut interpreter = interpreter.expect("interpreter creation failed");
    let mut receiver = TerminalReceiver {};

    if let Some(line) = cli.entry {
        let results = interpreter.line(&line, &mut receiver);
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
                let next = iter.next().expect("Could nod read next line");
                line.push_str(&next);
            }

            if !line.trim().is_empty() {
                let results = interpreter.line(&line, &mut receiver);
                print_results(results, &line);
            }

            print_prompt(false);
        }
    }
}

fn print_results(result: Result<Value, AggregateError<Error>>, line: &str) {
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
    source_map: &'a SourceMap,
    paths: Vec<PathBuf>,
    sources: Vec<Arc<String>>,
    entry: Arc<String>,
}

impl<'a> ErrorReporter<'a> {
    fn new(cli: Cli, sources: Vec<String>, source_map: &'a SourceMap) -> Self {
        Self {
            source_map,
            paths: cli.sources,
            sources: sources.into_iter().map(Arc::new).collect(),
            entry: Arc::new(cli.entry.unwrap_or_default()),
        }
    }

    fn report(&self, error: Report) -> Report {
        let Some(first_label) = error.labels().and_then(|mut ls| ls.next()) else {
            return error;
        };

        // Use the offset of the first labeled span to find which source code to include in the report.
        let (index, offset) = self.source_map.offset(first_label.offset());
        let name = source_name(&self.paths, index);
        let source = self.sources.get(index.0).unwrap_or(&self.entry).clone();

        // Adjust all spans in the error to be relative to the start of this source.
        error.with_source_code(OffsetSource {
            source,
            name: name.to_string(),
            offset,
        })
    }
}

struct OffsetSource {
    source: Arc<String>,
    name: String,
    offset: usize,
}

impl SourceCode for OffsetSource {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        let span = SourceSpan::new((span.offset() - self.offset).into(), span.len().into());
        let contents = self
            .source
            .read_span(&span, context_lines_before, context_lines_after)?;
        let contents_span = *contents.span();

        let contents_span = SourceSpan::new(
            (contents_span.offset() + self.offset).into(),
            contents_span.len().into(),
        );
        Ok(Box::new(MietteSpanContents::new_named(
            self.name.clone(),
            contents.data(),
            contents_span,
            contents.line(),
            contents.column(),
            contents.line_count(),
        )))
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

struct TerminalReceiver;

impl Receiver for TerminalReceiver {
    fn state(
        &mut self,
        states: Vec<(BigUint, Complex64)>,
        qubit_count: usize,
    ) -> Result<(), qsc_eval::output::Error> {
        println!("DumpMachine:");
        for state in states {
            println!(
                "{}: [{}, {}]",
                format_state_id(&state.0, qubit_count),
                state.1.re,
                state.1.im
            );
        }

        Ok(())
    }

    fn message(&mut self, msg: &str) -> Result<(), qsc_eval::output::Error> {
        println!("{msg}");
        Ok(())
    }
}
