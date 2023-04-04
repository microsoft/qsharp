// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{error::ErrorKind, Parser, ValueEnum};

use miette::{IntoDiagnostic, Result};

use miette::{Diagnostic, NamedSource, Report};
use qsc_frontend::{
    compile::{self, compile, CompileUnit, Context, PackageStore, SourceIndex},
    diagnostic::OffsetError,
};
use std::{fs, io, string::String, sync::Arc};

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
    let sources: Vec<_> = cli.input.iter().map(read_source).collect();

    let mut store = PackageStore::new();
    let dependencies = if cli.nostdlib {
        vec![]
    } else {
        vec![store.insert(compile::std())]
    };
    let unit = compile(
        &store,
        dependencies,
        &sources,
        &cli.entry.clone().unwrap_or_default(),
    );

    for (_, emit) in cli.emit.iter().enumerate() {
        match emit {
            Emit::Ast => {
                let out_dir = match &cli.out_dir {
                    Some(value) => value.clone(),
                    None => PathBuf::from("."),
                };
                emit_compilation_unit(&unit, &out_dir);
            }
        }
    }

    if unit.context.errors().is_empty() {
        Ok(ExitCode::SUCCESS)
    } else {
        let reporter = ErrorReporter::new(cli, sources, &unit.context);
        for error in unit.context.errors() {
            eprintln!("{:?}", reporter.report(error.clone()));
        }
        Ok(ExitCode::FAILURE)
    }
}

struct ErrorReporter<'a> {
    context: &'a Context,
    paths: Vec<PathBuf>,
    sources: Vec<Arc<String>>,
    entry: Arc<String>,
}

impl<'a> ErrorReporter<'a> {
    fn new(cli: Cli, sources: Vec<String>, context: &'a Context) -> Self {
        Self {
            context,
            paths: cli.input,
            sources: sources.into_iter().map(Arc::new).collect(),
            entry: Arc::new(cli.entry.unwrap_or_default()),
        }
    }

    fn report(&self, error: impl Diagnostic + Send + Sync + 'static) -> Report {
        let Some(first_label) = error.labels().and_then(|mut ls| ls.next()) else {
            return Report::new(error);
        };

        // Use the offset of the first labeled span to find which source code to include in the report.
        let (index, offset) = self.context.source(first_label.offset());
        let name = source_name(&self.paths, index);
        let source = self.sources.get(index.0).unwrap_or(&self.entry).clone();
        let source = NamedSource::new(name, source);

        // Adjust all spans in the error to be relative to the start of this source.
        let offset = -isize::try_from(offset).unwrap();
        Report::new(OffsetError::new(error, offset)).with_source_code(source)
    }
}

fn emit_compilation_unit(unit: &CompileUnit, out_dir: impl AsRef<Path>) {
    let path = out_dir.as_ref().join("ast.txt");
    fs::write(path, format!("{unit:#?}")).unwrap();
}

fn read_source(path: impl AsRef<Path>) -> String {
    if path.as_ref().as_os_str() == "-" {
        io::stdin().lines().map(Result::unwrap).collect()
    } else {
        fs::read_to_string(path).unwrap()
    }
}

fn source_name(paths: &[PathBuf], index: SourceIndex) -> &str {
    paths
        .get(index.0)
        .map_or("<unknown>", |p| match p.to_str() {
            Some("-") => "<stdin>",
            Some(name) => name,
            None => "<unknown>",
        })
}
