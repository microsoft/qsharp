// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    env,
    fmt::Display,
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

struct FileWalker {
    root: PathBuf,
    is_write: bool,
    is_recursive: bool,
    file_count: i32,
    changed_files: Vec<String>,
    skipped_files: Vec<String>,
}

impl FileWalker {
    pub fn new() -> Self {
        let cli = Cli::parse();
        Self {
            root: cli.path,
            is_write: cli.write,
            is_recursive: cli.recursive,
            file_count: 0,
            changed_files: vec![],
            skipped_files: vec![],
        }
    }

    fn format_from_root(&mut self) {
        self.format_file_or_dir(&self.root.clone());
    }

    fn format_file_or_dir(&mut self, path: &Path) {
        use OutputFormatting::*;

        if path.is_dir() {
            let items = match path.read_dir() {
                Ok(items) => items.flatten(),
                Err(e) => {
                    println!("\t{Skip}Could not read from directory: {e}{Reset}");
                    self.skipped_files.push(path.display().to_string());
                    return;
                }
            };
            for item in items {
                let subpath = &item.path();
                if subpath.is_dir() && self.is_recursive {
                    self.format_file_or_dir(subpath);
                } else if is_path_qs(subpath) {
                    self.format_file(subpath);
                }
            }
        } else if is_path_qs(path) {
            self.format_file(path);
        }
    }

    fn format_file(&mut self, path: &Path) {
        use OutputFormatting::*;

        println!("Formatting {}", path.display());

        let file_as_string = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                println!("\t{Skip}Could not read from file: {e}{Reset}");
                self.skipped_files.push(path.display().to_string());
                return; // don't count file in file_count
            }
        };
        if self.is_write {
            let formatted = format_str(&file_as_string);
            if file_as_string != formatted {
                match std::fs::write(path, formatted) {
                    Ok(_) => {
                        self.changed_files.push(path.display().to_string());
                    }
                    Err(e) => {
                        println!("\t{Skip}Could not write to file: {e}{Reset}");
                        self.skipped_files.push(path.display().to_string());
                        return; // don't count file in file_count
                    }
                }
            }
        } else if !calculate_format_edits(&file_as_string).is_empty() {
            self.changed_files.push(path.display().to_string());
        }
        self.file_count += 1;
    }
}

enum OutputFormatting {
    Error,
    Skip,
    Passing,
    Reset,
}

impl Display for OutputFormatting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            OutputFormatting::Error => "\x1B[1;91m", // Bold, Bright Red
            OutputFormatting::Passing => "\x1B[1;92m", // Bold, Bright Green
            OutputFormatting::Skip => "\x1B[1;93m",  // Bold, Bright Yellow
            OutputFormatting::Reset => "\x1B[0m",
        };
        write!(f, "{output}")
    }
}

fn main() -> Result<(), String> {
    use OutputFormatting::*;

    let mut file_walker = FileWalker::new();
    if !file_walker.root.exists() {
        return Err("Given path can not found.".to_string());
    }
    if !file_walker.root.is_dir() && !is_path_qs(&file_walker.root) {
        return Err("Given path is not a folder or Q# file.".to_string());
    }

    file_walker.format_from_root();

    println!("Ran against {} files.", file_walker.file_count);

    if !file_walker.skipped_files.is_empty() {
        println!("{Skip}Skipped {} files:", file_walker.skipped_files.len());
        for f in file_walker.skipped_files.iter() {
            println!("\t{f}");
        }
        print!("{Reset}");
    }

    if file_walker.is_write {
        println!("Updated {} files:", file_walker.changed_files.len());
        for f in file_walker.changed_files.iter() {
            println!("\t{f}");
        }
        Ok(())
    } else if !file_walker.changed_files.is_empty() {
        println!(
            "{Error}{} files are in need of formatting:",
            file_walker.changed_files.len()
        );
        for f in file_walker.changed_files.iter() {
            println!("\t{f}");
        }
        println!("Run the formatter with the `--write` option to correct formatting for the above files.{Reset}");
        Err("Files are not formatted.".to_string())
    } else if !file_walker.skipped_files.is_empty() {
        println!(
            "{Skip}{} files are correctly formatted. {} files skipped.{Reset}",
            file_walker.file_count,
            file_walker.skipped_files.len()
        );
        Ok(())
    } else {
        println!("{Passing}All files are correctly formatted.{Reset}");
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
