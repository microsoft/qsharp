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

fn verify_module() {
    let mut exercise_dir = katas_qsharp_source_dir();
    exercise_dir.push("single_qubit_gates");
    exercise_dir.push("task_01");

    let mut reference_file = exercise_dir.clone();
    reference_file.push("reference.qs");
    let mut reference = read_to_string(reference_file).expect("Unable to read file");
    // TODO (cesarzc): this should probably be done in the verify_kata function.
    reference.insert_str(0, "namespace Kata {\n");
    reference.push_str("\n}");
    println!("{}", reference);

    let mut verification_source_file = exercise_dir.clone();
    verification_source_file.push("verify.qs");
    let mut verification_source = read_to_string(verification_source_file).expect("Unable to read file");
    // TODO (cesarzc): this should probably be done in the verify_kata function.
    verification_source.insert_str(0, "namespace Kata {\n");
    verification_source.push_str("\n}");
    println!("{}", verification_source);
    verify_kata(verification_source.as_str(), reference.as_str());
}

#[test]
fn verify_single_qubit_gates_kata() {
    verify_module();
}