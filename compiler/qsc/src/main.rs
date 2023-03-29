// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::error::ErrorKind;
use clap::Parser;
use clap::ValueEnum;
use miette::{IntoDiagnostic, Result};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Emit {
    /// Abstract syntax tree
    Ast,
}

#[derive(Debug, Parser)]
#[command(version, arg_required_else_help(true))]
struct Cli {
    /// Disable automatic inclusion of the standard library
    #[arg(long)]
    nostdlib: bool,
    /// Emit the compilation unit in the specified format
    #[arg(long, value_enum)]
    emit: Vec<Emit>,
    /// Write output to compiler-chosen filename in <dir>
    #[arg(long, value_name = "DIR")]
    outdir: Option<PathBuf>,
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
    /// Entry expression to execute as the main operation.
    #[arg(short, long)]
    entry: Option<String>,
    /// Q# source files to compile, or `-` to read from stdin
    #[arg()]
    input: Vec<PathBuf>,
}

fn validate_input(v: &[PathBuf]) -> Result<(), clap::Error> {
    if v.is_empty() {
        let msg = "No input files specified";
        let err = clap::Error::raw(ErrorKind::ValueValidation, msg);
        Err(err)
    } else if v.len() == 1 {
        Ok(())
    } else {
        if v.iter()
            .any(|path| path.display().to_string() == *"-")
        {
            let msg = "Specifying stdin `-` is not allowed with file inputs";
            let err = clap::Error::raw(ErrorKind::ValueValidation, msg);
            Err(err)
        } else {
            Ok(())
        }
    }
}

fn main() -> Result<ExitCode> {
    let cli = Cli::parse();
    validate_input(&cli.input).into_diagnostic()?;
    println!("{cli:?}");
    Ok(ExitCode::SUCCESS)
}
