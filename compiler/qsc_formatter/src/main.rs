// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    env,
    path::{Path, PathBuf},
};
mod formatter;
use clap::{crate_version, Parser};
use formatter::{calculate_format_edits, format_str};

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

fn main() -> Result<(), i32> {
    let cli = Cli::parse();
    let mut file_count = 0;
    let mut changed_files: Vec<String> = vec![];

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
                recurse_into_dir(&subpath, &mut file_count, cli.write, &mut changed_files);
            } else if is_path_qs(&subpath) {
                println!("Formatting {}", subpath.display());
                format_file(&subpath, cli.write, &mut changed_files);
                file_count += 1;
            }
        }
    } else if is_path_qs(&cli.path) {
        println!("Formatting {}", cli.path.display());
        format_file(&cli.path, cli.write, &mut changed_files);
        file_count += 1;
    } else if cli.path.is_file() {
        panic!("give file is not a Q# file");
    } else {
        panic!("path points to unknown object");
    }

    println!("Ran against {file_count} files.");
    if cli.write {
        println!("Updated {} files:", changed_files.len());
        for f in changed_files.iter() {
            println!("\t{f}");
        }
        Ok(())
    } else if !changed_files.is_empty() {
        println!("{} files are in need of formatting:", changed_files.len());
        for f in changed_files.iter() {
            println!("\t{f}");
        }
        println!("Run the formatter to correct formatting for the above files.");
        Err(1)
    } else {
        println!("All files are correctly formatted.");
        Ok(())
    }
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

fn recurse_into_dir(
    path: &Path,
    file_count: &mut i32,
    write: bool,
    changed_files: &mut Vec<String>,
) {
    assert!(path.exists(), "location not found");
    if path.is_dir() {
        for item in path
            .read_dir()
            .expect("unable to read from directory")
            .flatten()
        {
            let subpath = item.path();
            if subpath.is_dir() {
                recurse_into_dir(&subpath, file_count, write, changed_files);
            } else if is_path_qs(&subpath) {
                println!("Formatting {}", subpath.display());
                format_file(&subpath, write, changed_files);
                *file_count += 1;
            }
        }
    } else if is_path_qs(path) {
        println!("Formatting {}", path.display());
        format_file(path, write, changed_files);
        *file_count += 1;
    }
}

fn format_file(path: &Path, write: bool, changed_files: &mut Vec<String>) {
    // read file from `path` into buffer
    let file_as_string = std::fs::read_to_string(path).expect("file not found");
    if write {
        // format the buffer
        let formatted = format_str(&file_as_string);
        // write the formatted buffer back to `path`
        if file_as_string != formatted {
            std::fs::write(path, formatted).expect("could not write to file");
            changed_files.push(path.display().to_string());
        }
    } else {
        let fmt_errors = calculate_format_edits(&file_as_string);
        if !fmt_errors.is_empty() {
            changed_files.push(path.display().to_string());
        }
    }
}
