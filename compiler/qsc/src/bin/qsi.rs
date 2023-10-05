// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use clap::{crate_version, Parser};
use miette::{Context, IntoDiagnostic, Report, Result};
use num_bigint::BigUint;
use num_complex::Complex64;
use qsc::{
    interpret::stateful::{self, InterpretResult, Interpreter},
    TargetProfile,
};
use qsc_eval::{
    output::{self, Receiver},
    val::Value,
};
use qsc_frontend::compile::{SourceContents, SourceMap, SourceName};
use qsc_passes::PackageType;
use std::{
    fs,
    io::{self, prelude::BufRead, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    string::String,
};

#[derive(Debug, Parser)]
#[command(name = "qsi", version = concat!(crate_version!(), " (", env!("QSHARP_GIT_HASH"), ")"))]
#[command(author, about, next_line_help = true)]
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

    if cli.exec {
        let mut interpreter = match Interpreter::new(
            !cli.nostdlib,
            SourceMap::new(sources, cli.entry.map(std::convert::Into::into)),
            PackageType::Exe,
            TargetProfile::Full,
        ) {
            Ok(interpreter) => interpreter,
            Err(errors) => {
                for error in errors {
                    eprintln!("error: {:?}", Report::new(error));
                }
                return Ok(ExitCode::FAILURE);
            }
        };
        return Ok(print_exec_result(
            interpreter.eval_entry(&mut TerminalReceiver),
        ));
    }

    let mut interpreter = match Interpreter::new(
        !cli.nostdlib,
        SourceMap::new(sources, None),
        PackageType::Lib,
        TargetProfile::Full,
    ) {
        Ok(interpreter) => interpreter,
        Err(errors) => {
            for error in errors {
                eprintln!("error: {:?}", Report::new(error));
            }
            return Ok(ExitCode::FAILURE);
        }
    };

    if let Some(entry) = cli.entry {
        print_interpret_result(interpreter.eval_fragments(&mut TerminalReceiver, &entry));
    }

    repl(&mut interpreter, &mut TerminalReceiver).into_diagnostic()?;

    Ok(ExitCode::SUCCESS)
}

fn repl(interpreter: &mut Interpreter, receiver: &mut impl Receiver) -> io::Result<()> {
    print_prompt(false);

    let mut lines = io::BufReader::new(io::stdin()).lines();
    while let Some(line) = lines.next() {
        let mut line = line?;

        while line.ends_with('\\') {
            print_prompt(true);
            if let Some(continuation) = lines.next() {
                line.pop(); // Remove backslash.
                line.push_str(&continuation?);
            } else {
                println!();
                return Ok(());
            }
        }

        if !line.trim().is_empty() {
            print_interpret_result(interpreter.eval_fragments(receiver, &line));
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

fn print_interpret_result(result: InterpretResult) {
    match result {
        Ok(Value::Tuple(items)) if items.is_empty() => {}
        Ok(value) => println!("{value}"),
        Err(errors) => {
            for error in errors {
                if let Some(stack_trace) = error.stack_trace() {
                    eprintln!("{stack_trace}");
                }
                let report = Report::new(error);
                eprintln!("error: {report:?}");
            }
        }
    }
}

fn print_exec_result(result: Result<Value, Vec<stateful::Error>>) -> ExitCode {
    match result {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for error in errors {
                if let Some(stack_trace) = error.stack_trace() {
                    eprintln!("{stack_trace}");
                }
                let report = Report::new(error);
                eprintln!("error: {report:?}");
            }
            ExitCode::FAILURE
        }
    }
}
