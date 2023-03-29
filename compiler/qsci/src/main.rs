// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use miette::Result;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    /// Use the given file on startup as initial session input
    #[arg(long)]
    _use: Vec<PathBuf>,
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
    println!("{cli:?}");
    Ok(ExitCode::SUCCESS)
}
