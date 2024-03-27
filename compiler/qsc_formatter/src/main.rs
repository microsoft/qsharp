// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    env,
    fs::metadata,
    path::{Path, PathBuf},
};
mod formatter;
use clap::{crate_version, Parser};
use formatter::format_str;

#[derive(Debug, Parser)]
#[command(name = "fmt", version = crate_version!())]
#[command(author, about, next_line_help = true)]
struct Cli {
    /// Path to the file or folder contain files to be formatted.
    #[arg()]
    path: PathBuf,

    /// Search for Q# files recursively when a folder is given as PATH.
    #[arg(short, long, default_value("false"))]
    recursive: bool,

    /// Overwrite the file contents with the formatted contents.
    #[arg(short, long, default_value("false"))]
    write: bool,
}

fn main() {
    let cli = Cli::parse();
    let mut file_count = 0;

    assert!(cli.path.exists(), "location not found");
    if cli.path.is_dir() {
        for item in cli
            .path
            .read_dir()
            .expect("unable to read from directory")
            .flatten()
        {
            let subpath = item.path();
            if subpath.is_dir() && cli.recursive {
                recurse_into_dir(&subpath, &mut file_count);
            } else if is_path_qs(&subpath) {
                println!("Formatting {}", subpath.display());
                format_file(&subpath);
                file_count += 1;
            }
        }
    } else if is_path_qs(&cli.path) {
        println!("Formatting {}", cli.path.display());
        format_file(&cli.path);
        file_count += 1;
    } else if cli.path.is_file() {
        panic!("give file is not a Q# file");
    } else {
        panic!("path points to unknown object");
    }

    println!("Ran against {file_count} files.");

    // format_file(path);
}

fn is_path_qs(path: &Path) -> bool {
    if path.is_file() {
        if let Some(ex) = path.extension() {
            if ex == "qs" {
                return true;
            }
        }
    }
    false
}

fn recurse_into_dir(path: &Path, file_count: &mut i32) {
    assert!(path.exists(), "location not found");
    if path.is_dir() {
        for item in path
            .read_dir()
            .expect("unable to read from directory")
            .flatten()
        {
            let subpath = item.path();
            if subpath.is_dir() {
                recurse_into_dir(&subpath, file_count);
            } else if is_path_qs(&subpath) {
                println!("Formatting {}", subpath.display());
                format_file(&subpath);
                *file_count += 1;
            }
        }
    } else if is_path_qs(path) {
        println!("Formatting {}", path.display());
        format_file(path);
        *file_count += 1;
    }
}

fn format_file(path: &Path) {
    // read file from `path` into buffer
    let file_as_string = std::fs::read_to_string(path).expect("file not found");
    // format the buffer
    let formatted = format_str(&file_as_string);
    // write the formatted buffer back to `path`
    //std::fs::write(file_path, formatted).expect("could not write to file");
}
