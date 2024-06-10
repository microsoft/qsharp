// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod qsc;
pub mod qsi;

use clap::Parser;
use clap::Subcommand;

use miette::IntoDiagnostic;
use std::ffi::OsString;
use std::{path::PathBuf, process::ExitCode, string::String};

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
    Format(FormatCommand),
}

#[derive(Debug, Parser)]
#[command(arg_required_else_help(true))]
pub struct FormatCommand {
    pub sources: Vec<PathBuf>,
    #[arg(short, long, default_value = "")]
    pub entry: String,
}

#[derive(Debug, Parser)]
#[command(arg_required_else_help(true))]
pub struct RunCommand {
    pub sources: Vec<PathBuf>,
    #[arg(short, long, default_value = "")]
    pub entry: String,
}

pub fn exec_subcommand<I, T>(itr: I) -> miette::Result<ExitCode>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = crate::cli::qsc::Cli::try_parse_from(itr).into_diagnostic()?;
    let _ = crate::cli::qsc::exec(cli);
    todo!()
}

pub fn exec_qsc<I, T>(itr: I) -> miette::Result<ExitCode>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = crate::cli::qsc::Cli::try_parse_from(itr).into_diagnostic()?;
    crate::cli::qsc::exec(cli)
}

pub fn exec_qsi<I, T>(itr: I) -> miette::Result<ExitCode>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = crate::cli::qsi::Cli::try_parse_from(itr).into_diagnostic()?;
    crate::cli::qsi::exec(cli)
}
