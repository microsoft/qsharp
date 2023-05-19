// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

use std::process::Command;

fn main() {
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map_or("----".to_string(), |o| {
            String::from_utf8(o.stdout).expect("output should be parsable string")
        });
    println!("cargo:rustc-env=QSC_GIT_HASH={git_hash}");
}
