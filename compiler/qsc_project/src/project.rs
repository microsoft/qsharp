// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use regex_lite::Regex;

use crate::manifest::ManifestDescriptor;

/// Describes a Q# project
#[derive(Default, Debug)]
pub struct Project {
    pub sources: Vec<(Arc<str>, Arc<str>)>,
    pub manifest: crate::Manifest,
}

/// This enum represents a filesystem object type. It is analogous to [std::fs::FileType].
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum EntryType {
    File,
    Folder,
    Symlink,
}

/// This trait represents a filesystem object. It is analogous to [std::fs::DirEntry].
pub trait DirEntry {
    type Error;
    fn entry_type(&self) -> Result<EntryType, Self::Error>;
    fn extension(&self) -> String;
    fn entry_name(&self) -> String;
    fn path(&self) -> PathBuf;
}

/// This trait is used to abstract filesystem logic with regards to Q# projects.
/// A Q# project requires some multi-file structure, but that may not actually be
/// an OS filesystem. It could be a virtual filesystem on vscode.dev, or perhaps a
/// cached implementation. This interface defines the minimal filesystem requirements
/// for the Q# project system to function correctly.
pub trait FileSystem {
    type Entry: DirEntry;
    /// Given a path, parse its contents and return a tuple representing (FileName, FileContents).
    fn read_file(&self, path: &Path) -> miette::Result<(Arc<str>, Arc<str>)>;

    /// Given a path, list its directory contents (if any).
    fn list_directory(&self, path: &Path) -> miette::Result<Vec<Self::Entry>>;

    /// Given an initial path and some regexes to exclude, fetch files that don't match
    /// those regexes.
    fn fetch_files_with_exclude_pattern(
        &self,
        exclude_patterns: &[Regex],
        exclude_files: &[String],
        initial_path: &Path,
    ) -> miette::Result<Vec<Self::Entry>> {
        let listing = self.list_directory(initial_path)?;
        let mut files = vec![];
        for item in listing {
            let file_name = item.entry_name();
            let name = item.path().to_string_lossy().to_string();
            if regex_matches(exclude_patterns, &name) || exclude_files.contains(&file_name) {
                continue;
            }
            match item.entry_type() {
                Ok(EntryType::File) if item.extension() == "qs" => files.push(item),
                Ok(EntryType::Folder) => files.append(&mut self.fetch_files_with_exclude_pattern(
                    exclude_patterns,
                    exclude_files,
                    &item.path(),
                )?),
                _ => (),
            }
        }
        Ok(files)
    }

    /// Given a [ManifestDescriptor], load project sources.
    fn load_project(&self, manifest: ManifestDescriptor) -> miette::Result<Project> {
        let qs_files = self.fetch_files_with_exclude_pattern(
            &manifest.exclude_regexes()?,
            manifest.exclude_files(),
            &manifest.manifest_dir,
        )?;

        let qs_files = qs_files.into_iter().map(|file| file.path());

        let qs_sources = qs_files.map(|path| self.read_file(&path));

        let sources = qs_sources.collect::<miette::Result<_>>()?;
        Ok(Project {
            manifest: manifest.manifest,
            sources,
        })
    }
}

fn regex_matches(exclude_patterns: &[Regex], entry_name: &str) -> bool {
    exclude_patterns
        .iter()
        .any(|pattern| matches!(pattern.find(entry_name), Some(item) if item.as_str().len() == entry_name.len()))
}
