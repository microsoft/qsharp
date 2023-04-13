// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_eval::{output::CursorReceiver, stateless};
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

fn validate_module(path: impl AsRef<Path>) {
    for entry in fs::read_dir(path).expect("directory should be readable") {
        let path = entry.expect("entry should be usable").path();
        if path.is_dir() {
            validate_exercise(path);
        }
    }
}

#[test]
fn verify_single_qubit_gates_module() {
    validate_module(katas_qsharp_dir().join("single_qubit_gates"));
}
