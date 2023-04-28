// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use clap::{error::ErrorKind, Parser, ValueEnum};
use miette::{IntoDiagnostic, Result};
use qsc::compile::compile;
use qsc_frontend::compile::{PackageStore, SourceMap};
use qsc_hir::hir::Package;
use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::ExitCode,
    string::String,
    sync::Arc,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Emit {
    Hir,
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
    #[arg(long = "outdir", value_name = "DIR")]
    out_dir: Option<PathBuf>,
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

fn validate_input(sources: &[PathBuf], entry: &str) -> Result<(), clap::Error> {
    if sources.is_empty() {
        if entry.is_empty() {
            let msg = "No input specified";
            let err = clap::Error::raw(ErrorKind::ValueValidation, msg);
            Err(err)
        } else {
            Ok(())
        }
    } else if sources.len() == 1 {
        Ok(())
    } else if sources
        .iter()
        .any(|path| path.display().to_string() == *"-")
    {
        let msg = "Specifying stdin `-` is not allowed with file inputs";
        let err = clap::Error::raw(ErrorKind::ValueValidation, msg);
        Err(err)
    } else {
        Ok(())
    }
}

fn main() -> Result<ExitCode> {
    let cli = Cli::parse();
    validate_input(&cli.input, &cli.entry.clone().unwrap_or_default()).into_diagnostic()?;
    let sources: Vec<_> = cli
        .input
        .iter()
        .map(read_source)
        .map(Result::unwrap)
        .collect();
    let entry = cli.entry.unwrap_or_default();

    let mut store = PackageStore::new();
    let dependencies = if cli.nostdlib {
        vec![]
    } else {
        vec![store.insert(qsc::compile::std())]
    };

    let (unit, reports) = compile(
        &store,
        dependencies,
        SourceMap::new(sources, Some(entry.into())),
    );

    for (_, emit) in cli.emit.iter().enumerate() {
        match emit {
            Emit::Hir => {
                let out_dir = match &cli.out_dir {
                    Some(value) => value.clone(),
                    None => PathBuf::from("."),
                };
                emit_hir(&unit.package, &out_dir);
            }
        }
    }

    if reports.is_empty() {
        Ok(ExitCode::SUCCESS)
    } else {
        for report in reports {
            eprintln!("{report:?}");
        }
        Ok(ExitCode::FAILURE)
    }
}

fn emit_hir(package: &Package, out_dir: impl AsRef<Path>) {
    let path = out_dir.as_ref().join("hir.txt");
    fs::write(path, format!("{package}")).unwrap();
}

fn read_source(path: impl AsRef<Path>) -> io::Result<(Arc<str>, Arc<str>)> {
    if path.as_ref().as_os_str() == "-" {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        Ok(("<stdin>".into(), input.into()))
    } else {
        let path = path.as_ref();
        let content = fs::read_to_string(path).unwrap();
        Ok((path.to_string_lossy().into(), content.into()))
    }
}
