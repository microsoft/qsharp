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

    assert!(cli.path.exists(), "Given path can not found.");
    if !cli.path.is_dir() && is_path_qs(&cli.path) {
        panic!("Given path is not a folder or Q# file.");
    }

    format_file_or_dir(
        &cli.path,
        cli.write,
        cli.recursive,
        &mut file_count,
        &mut changed_files,
    );

    println!("Ran against {file_count} files.");
    if cli.write {
        println!("Updated {} files:", changed_files.len());
        for f in changed_files.iter() {
            println!("\t{f}");
        }
        Ok(())
    } else if !changed_files.is_empty() {
        println!(
            "\x1B[1;91m{} files are in need of formatting:",
            changed_files.len()
        );
        for f in changed_files.iter() {
            println!("\t{f}");
        }
        println!("Run the formatter to correct formatting for the above files.\x1B[0m");
        Err(1)
    } else {
        println!("\x1B[1;92mAll files are correctly formatted.\x1B[0m");
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

fn format_file_or_dir(
    path: &Path,
    is_write: bool,
    is_recursive: bool,
    file_count: &mut i32,
    changed_files: &mut Vec<String>,
) {
    if path.is_dir() {
        for item in path
            .read_dir()
            .expect("unable to read from directory")
            .flatten()
        {
            let subpath = item.path();
            if subpath.is_dir() && is_recursive {
                format_file_or_dir(&subpath, is_write, is_recursive, file_count, changed_files);
            } else if is_path_qs(&subpath) {
                println!("Formatting {}", subpath.display());
                format_file(&subpath, is_write, changed_files);
                *file_count += 1;
            }
        }
    } else if is_path_qs(path) {
        println!("Formatting {}", path.display());
        format_file(path, is_write, changed_files);
        *file_count += 1;
    }
}

fn format_file(path: &Path, write: bool, changed_files: &mut Vec<String>) {
    let file_as_string = std::fs::read_to_string(path).expect("file not found");
    if write {
        let formatted = format_str(&file_as_string);
        if file_as_string != formatted {
            std::fs::write(path, formatted).expect("could not write to file");
            changed_files.push(path.display().to_string());
        }
    } else if !calculate_format_edits(&file_as_string).is_empty() {
        changed_files.push(path.display().to_string());
    }
}
