// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::Parser;
use qsx::Cli;
use qsx::Commands;
use std::process::ExitCode;

fn main() -> miette::Result<ExitCode> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build(cmd) => qsc::exec(qsc::Commands::Build(cmd)),
        Commands::Check(cmd) => qsc::exec(qsc::Commands::Check(cmd)),
        Commands::Run(cmd) => qsc::exec(qsc::Commands::Run(cmd)),
    }
}
