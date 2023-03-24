// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(arg_required_else_help(true))]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// build a q# project
    Build(qsc::BuildCommand),
    /// check a q# project
    Check(qsc::CheckCommand),
}
