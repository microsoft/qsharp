// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

use clap::{ArgGroup, Parser, ValueEnum};
use miette::{Context, IntoDiagnostic, Report};
use qsc::compile::compile;
use qsc_frontend::compile::{PackageStore, SourceContents, SourceMap, SourceName};
use qsc_hir::hir::Package;
use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::ExitCode,
    string::String,
};

#[derive(Debug, Parser)]
#[command(version, arg_required_else_help(true))]
#[clap(group(ArgGroup::new("input").args(["entry", "sources"]).required(true).multiple(true)))]
struct Cli {
    /// Disable automatic inclusion of the standard library.
    #[arg(long)]
    nostdlib: bool,

    /// Emit the compilation unit in the specified format.
    #[arg(long, value_enum)]
    emit: Vec<Emit>,

    /// Write output to compiler-chosen filename in <dir>.
    #[arg(long = "outdir", value_name = "DIR")]
    out_dir: Option<PathBuf>,

    /// Enable verbose output.
    #[arg(short, long)]
    verbose: bool,

    /// Entry expression to execute as the main operation.
    #[arg(short, long)]
    entry: Option<String>,

    /// Q# source files to compile, or `-` to read from stdin.
    #[arg()]
    sources: Vec<PathBuf>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Emit {
    Hir,
}

fn main() -> miette::Result<ExitCode> {
    let cli = Cli::parse();
    let mut store = PackageStore::new();
    let mut dependencies = Vec::new();
    if !cli.nostdlib {
        dependencies.push(store.insert(qsc::compile::std()));
    }

    let sources = cli
        .sources
        .iter()
        .map(read_source)
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;

    let entry = cli.entry.unwrap_or_default();
    let sources = SourceMap::new(sources, Some(entry.into()));
    let (unit, errors) = compile(&store, dependencies, sources);

    let out_dir = cli.out_dir.as_ref().map_or(".".as_ref(), PathBuf::as_path);
    for emit in &cli.emit {
        match emit {
            Emit::Hir => emit_hir(&unit.package, out_dir)
                .into_diagnostic()
                .context("could not emit HIR")?,
        }
    }

    if errors.is_empty() {
        Ok(ExitCode::SUCCESS)
    } else {
        for error in errors {
            if let Some(source) = unit.sources.find_diagnostic(&error) {
                eprintln!("{:?}", Report::new(error).with_source_code(source.clone()));
            } else {
                eprintln!("{:?}", Report::new(error));
            }
        }

        Ok(ExitCode::FAILURE)
    }
}

fn read_source(path: impl AsRef<Path>) -> io::Result<(SourceName, SourceContents)> {
    let path = path.as_ref();
    if path.as_os_str() == "-" {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        Ok(("<stdin>".into(), input.into()))
    } else {
        let contents = fs::read_to_string(path)?;
        Ok((path.to_string_lossy().into(), contents.into()))
    }
}

fn emit_hir(package: &Package, dir: impl AsRef<Path>) -> io::Result<()> {
    let path = dir.as_ref().join("hir.txt");
    fs::write(path, format!("{package}"))
}
