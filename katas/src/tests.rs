// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_eval::{output::CursorReceiver, stateless, stateless::eval, stateless::Error, val::Value, AggregateError};
use std::{
    env, fs,
    io::Cursor,
    path::{Path, PathBuf},
};

fn katas_qsharp_dir() -> PathBuf {
    env::current_dir()
        .expect("test should have current directory")
        .join("content")
}

fn run_kata(
    sources: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<bool, Vec<stateless::Error>> {
    let mut cursor = Cursor::new(Vec::new());
    let mut receiver = CursorReceiver::new(&mut cursor);
    let result = crate::run_kata(sources, &mut receiver);
    println!("{}", receiver.dump());
    result
}

fn validate_exercise(path: impl AsRef<Path>) {
    let path = path.as_ref();
    let verify = fs::read_to_string(path.join("verify.qs")).expect("file should be readable");
    let reference = fs::read_to_string(path.join("reference.qs")).expect("file should be readable");
    let result = run_kata([&reference, &verify]).expect("reference should succeed");
    assert!(result, "reference should return true");

    let placeholder =
        fs::read_to_string(path.join("placeholder.qs")).expect("file should be readable");
    // TODO: Assert that running returns false. This isn't reliable until the controlled functor is supported.
    run_kata([&placeholder, &verify]).expect("placeholder should succeed");
}

fn validate_example(path: impl AsRef<Path>) {
    let mut cursor = Cursor::new(Vec::new());
    let mut receiver = CursorReceiver::new(&mut cursor);
    let path = path.as_ref();
    let source = fs::read_to_string(path.join("example.qs")).expect("file should be readable");
    let result = eval(true, "Kata.Main()", &mut receiver, [&source]);
    println!("{}", receiver.dump());
    result.expect("Running an example should succeed");
}

fn validate_item(path: impl AsRef<Path>) {
    
}

fn validate_kata(path: impl AsRef<Path>) {
    for entry in fs::read_dir(path).expect("directory should be readable") {
        let path = entry.expect("entry should be usable").path();
        if path.is_dir() {
            validate_exercise(path);
        }
    }
}

#[test]
fn validate_single_qubit_gates_kata() {
    validate_kata(katas_qsharp_dir().join("single_qubit_gates"));
}

#[test]
fn validate_multi_qubit_gates_kata() {
    validate_kata(katas_qsharp_dir().join("multi_qubit_gates"));
}
