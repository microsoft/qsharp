// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod algorithms;
#[rustfmt::skip]
mod algorithms_generated;
#[rustfmt::skip]
mod estimation_generated;
mod language;
#[rustfmt::skip]
mod language_generated;
#[rustfmt::skip]
mod project_generated;

use qsc::{
    compile,
    interpret::{GenericReceiver, Interpreter},
    packages::BuildableProgram,
    LanguageFeatures, PackageType, SourceMap, TargetCapabilityFlags,
};
use qsc_project::{FileSystem, StdFs};

fn compile_and_run(sources: SourceMap) -> String {
    compile_and_run_internal(sources, false)
}

fn compile_and_run_debug(sources: SourceMap) -> String {
    compile_and_run_internal(sources, true)
}

fn compile_and_run_internal(sources: SourceMap, debug: bool) -> String {
    // when we load the project, need to set these
    let (std_id, store) = compile::package_store_with_stdlib(TargetCapabilityFlags::all());

    let mut interpreter = match (if debug {
        Interpreter::new_with_debug
    } else {
        Interpreter::new
    })(
        sources,
        PackageType::Exe,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
        store,
        &[(std_id, None)],
    ) {
        Ok(interpreter) => interpreter,
        Err(errors) => {
            for error in &errors {
                eprintln!("error: {error}");
            }
            panic!("compilation failed (first error: {})", errors[0]);
        }
    };

    check_lints(&interpreter);

    interpreter.set_classical_seed(Some(1));
    interpreter.set_quantum_seed(Some(1));

    let mut output = Vec::new();
    let mut out = GenericReceiver::new(&mut output);
    let val = match interpreter.eval_entry(&mut out) {
        Ok(val) => val,
        Err(errors) => {
            for error in &errors {
                eprintln!("error: {error}");
            }
            panic!("execution failed (first error: {})", errors[0]);
        }
    };
    String::from_utf8(output).expect("output should be valid UTF-8") + &val.to_string()
}

fn compile(sources: SourceMap) {
    let (std_id, store) = compile::package_store_with_stdlib(TargetCapabilityFlags::all());

    match Interpreter::new(
        sources,
        PackageType::Lib,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
        store,
        &[(std_id, None)],
    ) {
        Ok(interpreter) => {
            check_lints(&interpreter);
        }
        Err(errors) => {
            for error in &errors {
                eprintln!("error: {error}");
            }
            panic!("compilation failed (first error: {})", errors[0]);
        }
    };
}

fn compile_project(project_folder: &str) {
    let fs = StdFs;
    let project = fs
        .load_project(project_folder.as_ref(), None)
        .expect("project file should load");

    if !project.errors.is_empty() {
        for e in project.errors {
            eprintln!("{e}");
        }
        panic!("project should load without errors");
    }

    // This builds all the dependencies
    let buildable_program =
        BuildableProgram::new(TargetCapabilityFlags::all(), project.package_graph_sources);

    if !buildable_program.dependency_errors.is_empty() {
        for e in buildable_program.dependency_errors {
            eprintln!("{e}");
        }
        panic!("dependencies should compile without errors");
    }

    let BuildableProgram {
        store,
        user_code,
        user_code_dependencies,
        ..
    } = buildable_program;

    let source_map = qsc::SourceMap::new(user_code.sources, None);

    match Interpreter::new(
        source_map,
        PackageType::Lib,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
        store,
        &user_code_dependencies,
    ) {
        Ok(interpreter) => {
            check_lints(&interpreter);
        }
        Err(errors) => {
            for error in &errors {
                eprintln!("error: {error}");
            }
            panic!("compilation failed (first error: {})", errors[0]);
        }
    };
}

fn check_lints(interpreter: &Interpreter) {
    let lints: Vec<_> = interpreter
        .check_source_lints()
        .into_iter()
        .filter(|lint| lint.level != qsc::linter::LintLevel::Allow)
        .collect();
    if !lints.is_empty() {
        for lint in &lints {
            eprintln!("lint: {lint}");
        }
        panic!("linting failed (first lint: {})", lints[0]);
    }
}
