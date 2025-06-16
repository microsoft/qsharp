// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::process::Command;

fn main() {
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .map_or_else(
            |_| "unknown".to_string(),
            |o| String::from_utf8(o.stdout).expect("output should be parsable string"),
        );
    println!("cargo:rustc-env=QSHARP_GIT_HASH={git_hash}");
}
