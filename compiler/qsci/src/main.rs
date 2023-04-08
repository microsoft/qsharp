// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use qsc_eval::interactive::Interpreter;

use std::string::String;

use miette::{IntoDiagnostic, Result};
use std::io::prelude::BufRead;
use std::io::Write;
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
    let mut interpreter = Interpreter::new(cli.nostdlib, sources);

    if let Some(line) = cli.entry {
        let results = interpreter.line(line);
        for result in results {
            if !result.value.is_empty() {
                println!("{}", result.value);
            }
            if !result.output.is_empty() {
                println!("{}", result.output);
            }
            if !result.errors.is_empty() {
                eprintln!("{}", result.errors.join("\n"));
            }
        }
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
                let results = interpreter.line(line);
                for result in results {
                    if !result.value.is_empty() {
                        println!("{}", result.value);
                    }
                    if !result.output.is_empty() {
                        println!("{}", result.output);
                    }
                    if !result.errors.is_empty() {
                        eprintln!("{}", result.errors.join("\n"));
                    }
                }
            }

            print_prompt(false);
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
