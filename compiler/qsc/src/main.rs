// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use qsc_frontend::compile;
use std::{env, fs, io, result::Result, string::String};

fn main() {
    let args: Vec<_> = env::args().collect();
    let input: String = match args.get(1).map(String::as_str) {
        None | Some("-") => io::stdin().lines().map(Result::unwrap).collect(),
        Some(path) => fs::read_to_string(path).unwrap(),
    };
    let expr = args.get(2).map(String::as_str);

    println!("{:#?}", compile(&[&input], expr));
}
