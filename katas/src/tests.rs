// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{EXAMPLE_ENTRY, KATA_ENTRY};
use qsc::{
    interpret::{output::CursorReceiver, stateless, Value},
    SourceMap,
};
use std::{
    collections, env, fs,
    io::Cursor,
    path::{Path, PathBuf},
};

fn katas_qsharp_dir() -> PathBuf {
    env::current_dir()
        .expect("test should have current directory")
        .join("content")
}

fn get_example_source_path(sources_map: &collections::HashMap<String, std::option::Option<PathBuf>>) -> Option<PathBuf> {
    match sources_map.get("example.qs") {
        Some(p) => p.clone(),
        _ => None
    }
}

fn get_exercise_source_paths(sources_map: &collections::HashMap<String, std::option::Option<PathBuf>>) -> Option<(PathBuf, PathBuf, PathBuf)> {
    // TODO (cesarzc): maybe use lambda.
    let placeholder_path = match sources_map.get("placeholder.qs") {
        Some(p) => p.clone(),
        _ => None
    };

    let reference_path = match sources_map.get("reference.qs") {
        Some(p) => p.clone(),
        _ => None
    };

    let verify_path = match sources_map.get("verify.qs") {
        Some(p) => p.clone(),
        _ => None
    };

    if placeholder_path.is_none() || reference_path.is_none() || verify_path.is_none() {
        return None;
    }

    Some((placeholder_path.expect("path should be some"), reference_path.expect("path should be some"), verify_path.expect("path should be some")))
}

fn run_kata(kata: &str, verifier: &str) -> Result<bool, Vec<stateless::Error>> {
    let sources = SourceMap::new(
        [
            ("kata".into(), kata.into()),
            ("verifier".into(), verifier.into()),
        ],
        Some(KATA_ENTRY.into()),
    );

    let mut cursor = Cursor::new(Vec::new());
    let mut receiver = CursorReceiver::new(&mut cursor);
    let result = crate::run_kata(sources, &mut receiver);
    println!("{}", receiver.dump());
    result
}

fn validate_exercise(placeholder_source: impl AsRef<Path>, reference_source: impl AsRef<Path>, verify_source: impl AsRef<Path>) {
    println!("validate_exercise");
    let verify = fs::read_to_string(verify_source).expect("file should be readable");
    let reference = fs::read_to_string(reference_source).expect("file should be readable");
    let result = run_kata(&reference, &verify).expect("reference should succeed");
    assert!(result, "reference should return true");

    let placeholder =
        fs::read_to_string(placeholder_source).expect("file should be readable");
    let result = run_kata(&placeholder, &verify).expect("placeholder should succeed");
    assert!(!result, "placeholder should return false");
}

//fn validate_exercise(path: impl AsRef<Path>) {
//    let path = path.as_ref();
//    let verify = fs::read_to_string(path.join("verify.qs")).expect("file should be readable");
//    let reference = fs::read_to_string(path.join("reference.qs")).expect("file should be readable");
//    let result = run_kata(&reference, &verify).expect("reference should succeed");
//    assert!(result, "reference should return true");
//
//    let placeholder =
//        fs::read_to_string(path.join("placeholder.qs")).expect("file should be readable");
//    let result = run_kata(&placeholder, &verify).expect("placeholder should succeed");
//    assert!(!result, "placeholder should return false");
//}

fn validate_example(example_source: impl AsRef<Path>) {
    let mut cursor = Cursor::new(Vec::new());
    let mut receiver = CursorReceiver::new(&mut cursor);
    let example = fs::read_to_string(example_source).expect("file should be readable");
    let sources = SourceMap::new(
        [
            ("example".into(), example.into())
        ],
        Some(EXAMPLE_ENTRY.into()),
    );
    let context = stateless::Context::new(true, sources).expect("context new instance expected to be usable");
    println!("{}", receiver.dump());
    let succeeded = matches!(context.eval(&mut receiver), Ok(_));
    assert!(succeeded, "running an example shoud succeed");
    println!("{}", receiver.dump());
}

fn validate_item(path: impl AsRef<Path>) {
    let mut example_sources: collections::HashMap<String, std::option::Option<PathBuf>> =
        collections::HashMap::from([("example.qs".to_string(), None)]);

    let mut exercise_sources: collections::HashMap<String, std::option::Option<PathBuf>> =
        collections::HashMap::from([
            ("placeholder.qs".to_string(), None),
            ("reference.qs".to_string(), None),
            ("verify.qs".to_string(), None),
        ]);

    // 
    for entry in fs::read_dir(path).expect("directory should be readable") {
        let path = entry.expect("entry should be usable").path();
        if path.is_file() {
            let filename = path.file_name().expect("file name should be readable");
            let key = filename.to_str().expect("file name str should be valid");
            if exercise_sources.contains_key(key) {
                exercise_sources.insert(key.to_string(), Some(path.clone()));
            }

            if example_sources.contains_key(key) {
                example_sources.insert(key.to_string(), Some(path.clone()));
            }
        }
    }

    //
    let example_source_path = get_example_source_path(&example_sources);
    let exercise_source_paths = get_exercise_source_paths(&exercise_sources);
    assert!(!(example_source_path.is_some() && exercise_source_paths.is_some()), "item cannot be both example and exercise");
    if let Some(example_path) = example_source_path {
        validate_example(example_path);
    }

    if let Some((placeholder_source, reference_source, verify_source)) = exercise_source_paths {
        validate_exercise(placeholder_source, reference_source, verify_source);
    }


}

fn validate_kata(path: impl AsRef<Path>) {
    for entry in fs::read_dir(path).expect("directory should be readable") {
        let path = entry.expect("entry should be usable").path();
        if path.is_dir() {
            validate_item(&path);
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
