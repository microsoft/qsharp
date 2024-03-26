// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::env;
mod formatter;
use formatter::format_str;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).expect("expected path to file to format");
    // read file from `path` into buffer
    let file_as_string = std::fs::read_to_string(path).expect("file not found");
    // format the buffer
    let formatted = format_str(&file_as_string);
    // write the formatted buffer back to `path`
    std::fs::write(path, formatted).expect("could not write to file");
}
