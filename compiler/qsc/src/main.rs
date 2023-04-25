// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use clap::{error::ErrorKind, Parser, ValueEnum};
use miette::{
    IntoDiagnostic, MietteError, MietteSpanContents, Report, Result, SourceCode, SourceSpan,
    SpanContents,
};
use qsc_frontend::compile::{self, compile, PackageStore, SourceIndex, SourceMap};
use qsc_hir::hir::Package;
use qsc_passes::{entry_point::extract_entry, run_default_passes};
use std::{
    fs, io,
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
    let sources: Vec<_> = cli.input.iter().map(read_source).collect();

    let mut store = PackageStore::new();
    let dependencies = if cli.nostdlib {
        vec![]
    } else {
        let mut std = compile::std();
        if !std.errors.is_empty() {
            let reporter = ErrorReporter::new(cli, sources, &std.sources);
            for error in std.errors.drain(..) {
                eprintln!("{:?}", reporter.report(Report::new(error)));
            }
            return Ok(ExitCode::FAILURE);
        }
        let pass_errs = run_default_passes(&mut std);
        if !pass_errs.is_empty() {
            let reporter = ErrorReporter::new(cli, sources, &std.sources);
            for error in pass_errs {
                eprintln!("{:?}", reporter.report(Report::new(error)));
            }
            return Ok(ExitCode::FAILURE);
        }
        vec![store.insert(std)]
    };
    let mut unit = compile(
        &store,
        dependencies,
        &sources,
        &cli.entry.clone().unwrap_or_default(),
    );
    let pass_errs = run_default_passes(&mut unit);

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

    if unit.errors.is_empty() && pass_errs.is_empty() {
        if cli.entry.is_none() {
            match extract_entry(&unit.package) {
                Ok(..) => Ok(ExitCode::SUCCESS),
                Err(errors) => {
                    let reporter = ErrorReporter::new(cli, sources, &unit.sources);
                    for error in errors {
                        eprintln!("{:?}", reporter.report(Report::new(error)));
                    }
                    Ok(ExitCode::FAILURE)
                }
            }
        } else {
            Ok(ExitCode::SUCCESS)
        }
    } else {
        let reporter = ErrorReporter::new(cli, sources, &unit.sources);
        for error in unit.errors.drain(..) {
            eprintln!("{:?}", reporter.report(Report::new(error)));
        }
        for error in pass_errs {
            eprintln!("{:?}", reporter.report(Report::new(error)));
        }
        Ok(ExitCode::FAILURE)
    }
}

struct ErrorReporter<'a> {
    source_map: &'a SourceMap,
    paths: Vec<PathBuf>,
    sources: Vec<Arc<String>>,
    entry: Arc<String>,
}

impl<'a> ErrorReporter<'a> {
    fn new(cli: Cli, sources: Vec<String>, source_map: &'a SourceMap) -> Self {
        Self {
            source_map,
            paths: cli.input,
            sources: sources.into_iter().map(Arc::new).collect(),
            entry: Arc::new(cli.entry.unwrap_or_default()),
        }
    }

    fn report(&self, error: Report) -> Report {
        let Some(first_label) = error.labels().and_then(|mut ls| ls.next()) else {
            return error;
        };

        // Use the offset of the first labeled span to find which source code to include in the report.
        let (index, offset) = self.source_map.offset(first_label.offset());
        let name = source_name(&self.paths, index);
        let source = self.sources.get(index.0).unwrap_or(&self.entry).clone();

        // Adjust all spans in the error to be relative to the start of this source.
        error.with_source_code(OffsetSource {
            source,
            name: name.to_string(),
            offset,
        })
    }
}

struct OffsetSource {
    source: Arc<String>,
    name: String,
    offset: usize,
}

impl SourceCode for OffsetSource {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        let span = SourceSpan::new((span.offset() - self.offset).into(), span.len().into());
        let contents = self
            .source
            .read_span(&span, context_lines_before, context_lines_after)?;
        let contents_span = *contents.span();

        let contents_span = SourceSpan::new(
            (contents_span.offset() + self.offset).into(),
            contents_span.len().into(),
        );
        Ok(Box::new(MietteSpanContents::new_named(
            self.name.clone(),
            contents.data(),
            contents_span,
            contents.line(),
            contents.column(),
            contents.line_count(),
        )))
    }
}

fn emit_hir(package: &Package, out_dir: impl AsRef<Path>) {
    let path = out_dir.as_ref().join("hir.txt");
    fs::write(path, format!("{package}")).unwrap();
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
