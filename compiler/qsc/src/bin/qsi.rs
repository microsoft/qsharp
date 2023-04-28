// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

use clap::Parser;
use miette::{Context, IntoDiagnostic, Report, Result};
use num_bigint::BigUint;
use num_complex::Complex64;
use qsc::stateful::{Interpreter, LineError};
use qsc_eval::{
    output::{self, Receiver},
    val::Value,
};
use qsc_frontend::compile::{SourceContents, SourceMap, SourceName};
use std::{
    fs,
    io::{self, prelude::BufRead, Write},
    path::PathBuf,
    process::ExitCode,
    string::String,
};
use std::{path::Path, sync::Arc};

#[derive(Debug, Parser)]
#[command(author, version, about, next_line_help = true)]
struct Cli {
    /// Use the given file on startup as initial session input.
    #[arg(long = "use")]
    sources: Vec<PathBuf>,

    /// Execute the given Q# expression on startup.
    #[arg(long)]
    entry: Option<String>,

    /// Disable automatic inclusion of the standard library.
    #[arg(long)]
    nostdlib: bool,

    /// Exit after loading the files or running the given file(s)/entry on the command line.
    #[arg(long)]
    exec: bool,
}

struct TerminalReceiver;

impl Receiver for TerminalReceiver {
    fn state(
        &mut self,
        states: Vec<(BigUint, Complex64)>,
        qubit_count: usize,
    ) -> Result<(), output::Error> {
        println!("DumpMachine:");
        for (qubit, amplitude) in states {
            let id = output::format_state_id(&qubit, qubit_count);
            println!("{id}: [{}, {}]", amplitude.re, amplitude.im);
        }

        Ok(())
    }

    fn message(&mut self, msg: &str) -> Result<(), output::Error> {
        println!("{msg}");
        Ok(())
    }
}

fn main() -> miette::Result<ExitCode> {
    let cli = Cli::parse();
    let sources = cli
        .sources
        .iter()
        .map(read_source)
        .collect::<miette::Result<Vec<_>>>()?;

    let mut interpreter = match Interpreter::new(!cli.nostdlib, SourceMap::new(sources, None)) {
        Ok(interpreter) => interpreter,
        Err(errors) => {
            for error in errors {
                eprintln!("{:?}", Report::new(error));
            }
            return Ok(ExitCode::FAILURE);
        }
    };

    if let Some(entry) = cli.entry {
        print_result(
            &entry,
            interpreter.interpret_line(&mut TerminalReceiver, &entry),
        );
    }

    if !cli.exec {
        repl(&mut interpreter, &mut TerminalReceiver).into_diagnostic()?;
    }

    Ok(ExitCode::SUCCESS)
}

fn repl(interpreter: &mut Interpreter, receiver: &mut dyn Receiver) -> io::Result<()> {
    print_prompt(false);

    let mut lines = io::BufReader::new(io::stdin()).lines();
    while let Some(line) = lines.next() {
        let mut line = line?;

        while line.ends_with('\\') {
            print_prompt(true);
            let continuation = lines.next().expect("should have continuation line")?;
            line.pop(); // Remove backslash.
            line.push_str(&continuation);
        }

        if !line.trim().is_empty() {
            print_result(&line, interpreter.interpret_line(receiver, &line));
        }

        print_prompt(false);
    }

    println!();
    Ok(())
}

fn read_source(path: impl AsRef<Path>) -> miette::Result<(SourceName, SourceContents)> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .into_diagnostic()
        .with_context(|| format!("could not read source file `{}`", path.display()))?;

    Ok((path.to_string_lossy().into(), contents.into()))
}

fn print_prompt(continuation: bool) {
    if continuation {
        print!("    > ");
    } else {
        print!("qsi$ ");
    }

    io::stdout().flush().expect("standard out should flush");
}

fn print_result(line: &str, result: Result<Value, Vec<LineError>>) {
    match result {
        Ok(value) => println!("{value}"),
        Err(errors) => {
            let source: Arc<str> = line.into();
            for error in errors {
                let report = Report::new(error).with_source_code(Arc::clone(&source));
                eprintln!("{report:?}");
            }
        }
    }
}
