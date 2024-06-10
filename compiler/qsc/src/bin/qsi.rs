// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::process::ExitCode;

use qsc::cli::qsi::Cli;

allocator::assign_global!();

use clap::Parser;

fn main() -> miette::Result<ExitCode> {
    env_logger::init();
    qsc::cli::qsi::exec(Cli::parse())
}
