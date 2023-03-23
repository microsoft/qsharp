// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::env::current_dir;
use std::path::PathBuf;
use std::fs::read_to_string;

use crate::{verify_kata};

fn katas_qsharp_source_dir() -> PathBuf {
    let katas_qsharp_source_dir = current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .join("qs");

    katas_qsharp_source_dir.to_path_buf()
}

fn validate_exercise(exercise_dir: PathBuf) {
    let mut reference_file = exercise_dir.clone();
    reference_file.push("reference.qs");
    let reference = read_to_string(reference_file).expect("Unable to read reference file.");

    let mut verification_source_file = exercise_dir.clone();
    verification_source_file.push("verify.qs");
    let verification_source = read_to_string(verification_source_file).expect("Unable to read verification file.");
    let is_exercise_valid = verify_kata(verification_source.as_str(), reference.as_str());
    assert!(is_exercise_valid, "Exercise is invalid.");
}

// TODO (cesarzc): implement.
//fn validate_module() {
//}

#[test]
fn verify_single_qubit_gates_kata() {
    let mut exercise_dir = katas_qsharp_source_dir();
    exercise_dir.push("single_qubit_gates");
    exercise_dir.push("task_01");
    validate_exercise(exercise_dir);
}