// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::env::current_dir;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::path::PathBuf;

use crate::verify_kata;

fn katas_qsharp_source_dir() -> PathBuf {
    current_dir().unwrap().join("qs")
}

fn validate_exercise(exercise_dir: PathBuf) {
    let mut verification_source_file = exercise_dir.clone();
    verification_source_file.push("verify.qs");
    let verification_source =
        read_to_string(verification_source_file).expect("Unable to read verification file.");

    // Validate that the reference implementation yields success.
    let mut reference_file = exercise_dir.clone();
    reference_file.push("reference.qs");
    let reference = read_to_string(reference_file).expect("Unable to read reference file.");
    let is_exercise_valid = verify_kata(verification_source.as_str(), reference.as_str());
    assert!(is_exercise_valid, "Exercise is invalid.");
}

fn validate_module(module_dir: PathBuf) {
    for entry in read_dir(module_dir).expect("Unable to read module dir") {
        let path = entry
            .expect("No path for entry in module directory.")
            .path();
        if path.is_dir() {
            validate_exercise(path);
        }
    }
}

#[test]
fn verify_single_qubit_gates_module() {
    let mut module_dir = katas_qsharp_source_dir();
    module_dir.push("single_qubit_gates");
    validate_module(module_dir.clone());
}
