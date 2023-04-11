// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::env::current_dir;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use crate::run_kata;

use qsc_eval::output::GenericReceiver;

fn katas_qsharp_source_dir() -> PathBuf {
    current_dir()
        .expect("Unable to get the katas crate current directory")
        .join("qs")
}

fn validate_exercise(exercise_dir: &Path) {
    let exercise_name = format!(
        "{}",
        exercise_dir
            .file_name()
            .expect("Unable to obtain exercice name.")
            .to_string_lossy()
    );
    let mut verification_source_file = PathBuf::from(exercise_dir);
    verification_source_file.push("verify.qs");
    let verification_source =
        read_to_string(verification_source_file).expect("Unable to read verification file.");

    // Validate that both the placeholder and the reference implementation compile successfully.
    let mut reference_file_path = PathBuf::from(exercise_dir);
    reference_file_path.push("reference.qs");
    let reference_source = read_to_string(reference_file_path)
        .expect("Unable to read reference source implementation file.");

    let mut placeholder_file_path = PathBuf::from(exercise_dir);
    placeholder_file_path.push("placeholder.qs");
    let placeholder_source = read_to_string(placeholder_file_path)
        .expect("Unable to read placeholder source implementation file.");

    let placeholder_sources = vec![placeholder_source, verification_source.clone()];
    let reference_sources = vec![reference_source, verification_source];

    let mut stdout = io::stdout();
    let mut out = GenericReceiver::new(&mut stdout);
    let reference_succeeds = run_kata(reference_sources, &mut out);
    match reference_succeeds {
        Ok(value) => {
            assert!(value, "Reference implementation for exercise '{exercise_name}' expected to succeed but failed.");
        }
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            panic!("Reference implementation for exercise '{exercise_name}' expected to compile but failed.");
        }
    }
    if let Err(errors) = reference_succeeds {
        for error in errors {
            eprintln!("{error}");
        }
        panic!("Reference implementation for exercise '{exercise_name}' expected to succeed but failed.");
    }

    let mut stdout = io::stdout();
    let mut out = GenericReceiver::new(&mut stdout);
    let placeholder_succeeds = run_kata(placeholder_sources, &mut out);
    if let Err(errors) = placeholder_succeeds {
        for error in errors {
            eprintln!("{error}");
            if let qsc_eval::stateless::Error::Compile(_) = error {
                panic!("Placeholder implementation for exercise '{exercise_name}' expected to compile but failed.");
            }
        }
    }
    // N.B. Since verify_kata is doing evaluation, but it is not possible to determine correctness of some katas until
    //      the controlled functor is supported.
    //assert!(
    //    _placeholder_fails,
    //    "Placeholder implementation for exercise '{exercise_name}' expected to fail but succeeded.",
    //);
}

fn validate_module(module_dir: &PathBuf) {
    for entry in read_dir(module_dir).expect("Unable to read module dir") {
        let path = entry
            .expect("No path for entry in module directory.")
            .path();
        if path.is_dir() {
            validate_exercise(&path);
        }
    }
}

#[test]
fn verify_single_qubit_gates_module() {
    let mut module_dir = katas_qsharp_source_dir();
    module_dir.push("single_qubit_gates");
    validate_module(&module_dir);
}
