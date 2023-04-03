// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use std::collections::HashMap;
use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use qsc_ast::ast::CallableDecl;
use qsc_ast::ast::Stmt;

use std::string::String;

use miette::{IntoDiagnostic, Result};
use qsc_eval::evaluate;
use qsc_eval::Env;
use qsc_passes::globals::GlobalId;

use qsc_eval::output::GenericReceiver;
use qsc_frontend::compile::{self, compile, PackageId, PackageStore};
use qsc_frontend::incremental::{Compiler, Fragment};
use qsc_passes::globals::extract_callables;
use std::io::prelude::BufRead;
use std::io::Write;
use std::{fs, io};

#[derive(Debug, Parser)]
#[command(author, version, about, next_line_help = true)]
struct Cli {
    /// Use the given file on startup as initial session input
    #[arg(long = "use")]
    sources: Vec<PathBuf>,
    /// Open the given namespace(s) on startup before executing the entry expression or starting the REPL
    #[arg(long)]
    open: Vec<String>,
    /// Execute the given Q# expression on startup
    #[arg(long)]
    entry: Option<String>,
    /// Disable automatic inclusion of the standard library
    #[arg(long)]
    nostdlib: bool,
    /// Exit after loading the files or running the given file(s)/entry on the command line
    #[arg(long)]
    exec: bool,
}

fn main() -> Result<ExitCode> {
    let cli = Cli::parse();
    if !cli.open.is_empty() {
        unimplemented!("specifying open not yet implemented");
    }

    repl(cli)
}

fn repl(cli: Cli) -> Result<ExitCode> {
    let sources: Vec<_> = read_source(cli.sources.as_slice()).into_diagnostic()?;
    let mut store = PackageStore::new();

    let deps = if cli.nostdlib {
        vec![]
    } else {
        vec![store.insert(compile::std())]
    };

    let user = store.insert(compile(&store, [], sources, ""));
    let mut compiler = Compiler::new(&store, deps);
    let mut globals = extract_callables(&store);
    let mut env = Env::empty();

    let mut stdout = io::stdout();
    let mut out = GenericReceiver::new(&mut stdout);

    let mut execute_line = |line: String, env: Env| match compiler.compile_fragment(&line) {
        Ok(fragment) => match fragment {
            Fragment::Stmt(stmt) => eval(stmt, &store, &globals, &compiler, user, env, &mut out),
            Fragment::Callable(decl) => {
                globals.insert(
                    GlobalId {
                        package: user,
                        node: decl.name.id,
                    },
                    decl,
                );
                env
            }
        },
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            env
        }
    };
    if cli.entry.is_some() {
        let line = cli.entry.unwrap_or_default();
        env = execute_line(line, env);
    }

    if cli.exec {
        return Ok(ExitCode::SUCCESS);
    }

    loop {
        print_prompt(false);

        let stdin = io::BufReader::new(io::stdin());
        let mut iter = stdin.lines().map(Result::unwrap);
        while let Some(mut line) = iter.next() {
            while !line.is_empty() && &line[line.len() - 1..] == "\\" {
                print_prompt(true);
                line.pop(); // remove '\' from line
                let next = iter.next().unwrap();
                line.push_str(&next);
            }

            env = execute_line(line, env);

            print_prompt(false);
        }
    }
}

fn eval(
    stmt: &Stmt,
    store: &PackageStore,
    globals: &HashMap<GlobalId, &CallableDecl>,
    compiler: &Compiler,
    package: PackageId,
    env: Env,
    out: &mut GenericReceiver,
) -> Env {
    let (result, new_env) = evaluate(
        stmt,
        store,
        globals,
        compiler.resolutions(),
        package,
        env,
        out,
    );

    match result {
        Ok(value) => {
            println!("{value}");
        }
        Err(errors) => {
            eprintln!("{errors}");
        }
    }

    new_env
}

fn print_prompt(is_multiline: bool) {
    if is_multiline {
        print!("    > ");
    } else {
        print!("qsci$ ");
    }
    io::stdout().flush().unwrap();
}

fn read_source(paths: &[PathBuf]) -> io::Result<Vec<String>> {
    paths.iter().map(fs::read_to_string).collect()
}
