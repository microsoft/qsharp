// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::pedantic)]

use qsc_frontend::parse;
use std::{env, fs, io, result::Result, string::String};

fn main() {
    let args: Vec<_> = env::args().collect();
    let input: String = match args.get(1).map(String::as_str) {
        None | Some("-") => io::stdin().lines().map(Result::unwrap).collect(),
        Some(path) => fs::read_to_string(path).unwrap(),
    };

    let (package, errors) = parse::package(&input);
    println!("Errors: {errors:#?}");
    println!("AST: {package:#?}");
}
