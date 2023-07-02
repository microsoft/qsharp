// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::interpret::{output::CursorReceiver, stateless};
use std::{
    env, fs,
    io::Cursor,
    path::{Path, PathBuf},
};

fn test_cases_dir() -> PathBuf {
    env::current_dir()
        .expect("test should have current directory")
        .join("test_cases")
}

fn run_check_solution(solution: &str, verification: &str) -> Result<bool, Vec<stateless::Error>> {
    let mut cursor = Cursor::new(Vec::new());
    let mut receiver = CursorReceiver::new(&mut cursor);
    let result = crate::check_solution(
        vec![
            ("solution".into(), solution.into()),
            ("verification".into(), verification.into()),
        ],
        &mut receiver,
    );
    println!("{}", receiver.dump());
    result
}

fn test_check_solution(
    solution_source: impl AsRef<Path>,
    verification_source: impl AsRef<Path>,
    expected_result: bool,
) {
    let solution = fs::read_to_string(solution_source).expect("solution file should be readable");
    let verification =
        fs::read_to_string(verification_source).expect("verification file should be readable");
    let result =
        run_check_solution(&solution, &verification).expect("exercise should run successfully");
    assert!(
        result == expected_result,
        "exercise result is different than expected"
    );
}

#[test]
fn test_check_solution_is_correct() {
    let solution_source = test_cases_dir().join("apply_x").join("Correct.qs");
    let verification_source = test_cases_dir().join("apply_x").join("Verification.qs");
    test_check_solution(solution_source, verification_source, true);
}

#[test]
fn test_check_solution_is_incorrect() {
    let solution_source = test_cases_dir().join("apply_x").join("Incorrect.qs");
    let verification_source = test_cases_dir().join("apply_x").join("Verification.qs");
    test_check_solution(solution_source, verification_source, false);
}
