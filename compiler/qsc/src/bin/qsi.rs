// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

use clap::Parser;
use miette::{Diagnostic, IntoDiagnostic, Report, Result};
use num_bigint::BigUint;
use num_complex::Complex64;
use qsc::stateful::{Interpreter, LineError};
use qsc_eval::{
    output::{format_state_id, Receiver},
    val::Value,
};
use qsc_frontend::compile::SourceMap;
use std::sync::Arc;
use std::{
    fs,
    io::{self, prelude::BufRead, Write},
    path::PathBuf,
    process::ExitCode,
    string::String,
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

    let interpreter = Interpreter::new(!cli.nostdlib, SourceMap::new(sources, None));
    if let Err(errors) = interpreter {
        for error in errors {
            eprintln!("{:?}", Report::new(error));
        }
        return Ok(ExitCode::FAILURE);
    }
    let mut interpreter = interpreter.expect("interpreter creation failed");
    let mut receiver = TerminalReceiver {};

    if let Some(line) = cli.entry {
        let results = interpreter.interpret_line(&mut receiver, &line);
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
                let results = interpreter.interpret_line(&mut receiver, &line);
                print_results(results, &line);
            }

            print_prompt(false);
        }
    }
}

fn print_results(result: Result<Value, Vec<LineError>>, line: &str) {
    match result {
        Ok(value) => println!("{value}"),
        Err(errors) => {
            let reporter = InteractiveErrorReporter::new(line);
            for error in errors {
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

fn read_source(paths: &[PathBuf]) -> io::Result<Vec<(Arc<str>, Arc<str>)>> {
    paths
        .iter()
        .map(|p| Ok((p.to_string_lossy().into(), fs::read_to_string(p)?.into())))
        .collect()
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
