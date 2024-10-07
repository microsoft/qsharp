// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

allocator::assign_global!();

use clap::{crate_version, Parser};
use miette::{Context, IntoDiagnostic, Report, Result};
use num_bigint::BigUint;
use num_complex::Complex64;
use qsc::{
    hir::PackageId,
    interpret::{self, InterpretResult, Interpreter},
    packages::BuildableProgram,
    PackageStore,
};
use qsc_data_structures::{language_features::LanguageFeatures, target::TargetCapabilityFlags};
use qsc_eval::{
    output::{self, Receiver},
    state::format_state_id,
    val::Value,
};
use qsc_frontend::compile::{SourceContents, SourceMap, SourceName};
use qsc_passes::PackageType;
use qsc_project::{FileSystem, StdFs};
use std::{
    fs,
    io::{self, prelude::BufRead, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    string::String,
    sync::Arc,
};

#[derive(Debug, Parser)]
#[command(name = "qsi", version = concat!(crate_version!(), " (", env!("QSHARP_GIT_HASH"), ")"))]
#[command(author, about, next_line_help = true)]
struct Cli {
    /// Use the given file on startup as initial session input.
    #[arg(long = "use")]
    sources: Vec<PathBuf>,

    /// Execute the given Q# expression on startup.
    #[arg(long)]
    entry: Option<String>,

    /// Disable automatic inclusion of the standard library.
    #[arg(long)]
    nostdlib: bool,

    /// Exit after loading the files or running the given file(s)/entry on the command line.
    #[arg(long)]
    exec: bool,

    /// Path to a Q# manifest for a project
    #[arg(short, long)]
    qsharp_json: Option<PathBuf>,

    /// Language features to compile with
    #[arg(short, long)]
    features: Vec<String>,

    /// Compile the given files and interactive snippets in debug mode.
    #[arg(long)]
    debug: bool,
}

struct TerminalReceiver;

impl Receiver for TerminalReceiver {
    fn state(
        &mut self,
        states: Vec<(BigUint, Complex64)>,
        qubit_count: usize,
    ) -> Result<(), output::Error> {
        println!("DumpMachine:");
        for (qubit, amplitude) in states {
            let id = format_state_id(&qubit, qubit_count);
            println!("{id}: [{}, {}]", amplitude.re, amplitude.im);
        }

        Ok(())
    }

    fn matrix(&mut self, matrix: Vec<Vec<Complex64>>) -> std::result::Result<(), output::Error> {
        println!("Matrix:");
        for row in matrix {
            let row = row.iter().map(|elem| format!("[{}, {}]", elem.re, elem.im));
            println!("{}", row.collect::<Vec<_>>().join(", "));
        }

        Ok(())
    }

    fn message(&mut self, msg: &str) -> Result<(), output::Error> {
        println!("{msg}");
        Ok(())
    }
}

#[allow(clippy::too_many_lines)]
fn main() -> miette::Result<ExitCode> {
    let cli = Cli::parse();
    let mut features = LanguageFeatures::from_iter(cli.features);

    let (store, dependencies, source_map) = if let Some(qsharp_json) = cli.qsharp_json {
        if let Some(dir) = qsharp_json.parent() {
            match load_project(dir, &mut features) {
                Ok(items) => items,
                Err(code) => return Ok(code),
            }
        } else {
            eprintln!("{} must have a parent directory", qsharp_json.display());
            return Ok(ExitCode::FAILURE);
        }
    } else {
        let sources = cli
            .sources
            .iter()
            .map(read_source)
            .collect::<miette::Result<Vec<_>>>()?;

        let mut store = PackageStore::new(qsc::compile::core());
        let dependencies = if cli.nostdlib {
            vec![]
        } else {
            let std_id = store.insert(qsc::compile::std(&store, TargetCapabilityFlags::all()));
            vec![(std_id, None)]
        };
        (
            store,
            dependencies,
            SourceMap::new(sources, cli.entry.clone().map(std::convert::Into::into)),
        )
    };

    if cli.exec {
        let mut interpreter = match (if cli.debug {
            Interpreter::new_with_debug
        } else {
            Interpreter::new
        })(
            source_map,
            PackageType::Exe,
            TargetCapabilityFlags::all(),
            features,
            store,
            &dependencies,
        ) {
            Ok(interpreter) => interpreter,
            Err(errors) => {
                for error in errors {
                    eprintln!("error: {:?}", Report::new(error));
                }
                return Ok(ExitCode::FAILURE);
            }
        };
        return Ok(print_exec_result(
            interpreter.eval_entry(&mut TerminalReceiver),
        ));
    }

    let mut interpreter = match (if cli.debug {
        Interpreter::new_with_debug
    } else {
        Interpreter::new
    })(
        source_map,
        PackageType::Lib,
        TargetCapabilityFlags::all(),
        features,
        store,
        &dependencies,
    ) {
        Ok(interpreter) => interpreter,
        Err(errors) => {
            for error in errors {
                eprintln!("error: {:?}", Report::new(error));
            }
            return Ok(ExitCode::FAILURE);
        }
    };

    if let Some(entry) = cli.entry {
        print_interpret_result(interpreter.eval_fragments(&mut TerminalReceiver, &entry));
    }

    repl(&mut interpreter, &mut TerminalReceiver).into_diagnostic()?;

    Ok(ExitCode::SUCCESS)
}

fn repl(interpreter: &mut Interpreter, receiver: &mut impl Receiver) -> io::Result<()> {
    print_prompt(false);

    let mut lines = io::BufReader::new(io::stdin()).lines();
    while let Some(line) = lines.next() {
        let mut line = line?;

        while line.ends_with('\\') {
            print_prompt(true);
            if let Some(continuation) = lines.next() {
                line.pop(); // Remove backslash.
                line.push_str(&continuation?);
            } else {
                println!();
                return Ok(());
            }
        }

        if !line.trim().is_empty() {
            print_interpret_result(interpreter.eval_fragments(receiver, &line));
        }

        print_prompt(false);
    }

    println!();
    Ok(())
}

fn read_source(path: impl AsRef<Path>) -> miette::Result<(SourceName, SourceContents)> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .into_diagnostic()
        .with_context(|| format!("could not read source file `{}`", path.display()))?;

    Ok((path.to_string_lossy().into(), contents.into()))
}

fn print_prompt(continuation: bool) {
    if continuation {
        print!("    > ");
    } else {
        print!("qsi$ ");
    }

    io::stdout().flush().expect("standard out should flush");
}

fn print_interpret_result(result: InterpretResult) {
    match result {
        Ok(Value::Tuple(items)) if items.is_empty() => {}
        Ok(value) => println!("{value}"),
        Err(errors) => {
            for error in errors {
                if let Some(stack_trace) = error.stack_trace() {
                    eprintln!("{stack_trace}");
                }
                let report = Report::new(error);
                eprintln!("error: {report:?}");
            }
        }
    }
}

fn print_exec_result(result: Result<Value, Vec<interpret::Error>>) -> ExitCode {
    match result {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for error in errors {
                if let Some(stack_trace) = error.stack_trace() {
                    eprintln!("{stack_trace}");
                }
                let report = Report::new(error);
                eprintln!("error: {report:?}");
            }
            ExitCode::FAILURE
        }
    }
}

/// Loads a project from the given directory and returns the package store, the list of
/// dependencies, and the source map.
/// Pre-populates the package store with all of the compiled dependencies.
#[allow(clippy::type_complexity)]
fn load_project(
    dir: impl AsRef<Path>,
    features: &mut LanguageFeatures,
) -> Result<(PackageStore, Vec<(PackageId, Option<Arc<str>>)>, SourceMap), ExitCode> {
    let fs = StdFs;
    let project = match fs.load_project(dir.as_ref(), None) {
        Ok(project) => project,
        Err(errs) => {
            for e in errs {
                eprintln!("{e:?}");
            }
            return Err(ExitCode::FAILURE);
        }
    };

    if !project.errors.is_empty() {
        for e in project.errors {
            eprintln!("{e:?}");
        }
        return Err(ExitCode::FAILURE);
    }

    // This builds all the dependencies
    let buildable_program =
        BuildableProgram::new(TargetCapabilityFlags::all(), project.package_graph_sources);

    if !buildable_program.dependency_errors.is_empty() {
        for e in buildable_program.dependency_errors {
            eprintln!("{e:?}");
        }
        return Err(ExitCode::FAILURE);
    }

    let BuildableProgram {
        store,
        user_code,
        user_code_dependencies,
        ..
    } = buildable_program;

    let source_map = qsc::SourceMap::new(user_code.sources, None);

    features.merge(LanguageFeatures::from_iter(user_code.language_features));

    Ok((store, user_code_dependencies, source_map))
}
