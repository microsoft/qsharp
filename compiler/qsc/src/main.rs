// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use clap::Parser;
use qsc::exec;
use qsc::Cli;

use std::process::ExitCode;

fn main() -> miette::Result<ExitCode> {
    let cli = Cli::parse();
    exec(cli.command)
}
