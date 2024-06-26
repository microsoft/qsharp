// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{Manifest, ManifestDescriptor};
use async_trait::async_trait;
use futures::FutureExt;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

/// Describes a Q# project
#[derive(Debug)]
pub struct Project {
    /// Friendly name, typically based on project directory name
    pub name: Arc<str>,
    /// Full path to the project's `qsharp.json` file
    pub manifest_path: Arc<str>,
    pub sources: Vec<(Arc<str>, Arc<str>)>,
    pub manifest: crate::Manifest,
}

/// This enum represents a filesystem object type. It is analogous to [`std::fs::FileType`].
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum EntryType {
    File,
    Folder,
    Symlink,
    Unknown,
}

/// This trait represents a filesystem object. It is analogous to [`std::fs::DirEntry`].
pub trait DirEntry {
    type Error: Send + Sync;
    fn entry_type(&self) -> Result<EntryType, Self::Error>;
    fn path(&self) -> PathBuf;
    fn entry_extension(&self) -> String {
        self.path()
            .extension()
            .map(|x| x.to_string_lossy().to_string())
            .unwrap_or_default()
    }
    fn entry_name(&self) -> String {
        self.path()
            .file_name()
            .expect("canonicalized symlink cannot end in relative path")
            .to_string_lossy()
            .to_string()
    }
}

/// This trait is used to abstract filesystem logic with regards to Q# projects.
/// A Q# project requires some multi-file structure, but that may not actually be
/// an OS filesystem. It could be a virtual filesystem on vscode.dev, or perhaps a
/// cached implementation. This interface defines the minimal filesystem requirements
/// for the Q# project system to function correctly.
#[async_trait(?Send)]
pub trait FileSystemAsync {
    type Entry: DirEntry;
    /// Given a path, parse its contents and return a tuple representing (FileName, FileContents).
    async fn read_file(&self, path: &Path) -> miette::Result<(Arc<str>, Arc<str>)>;

    /// Given a path, list its directory contents (if any).
    /// This function should only return files that end in *.qs and folders.
    async fn list_directory(&self, path: &Path) -> miette::Result<Vec<Self::Entry>>;

    /// Given a base path and a relative path, join the segments and normalize
    /// the path, i.e. replace '..', '.', and redundant separators.
    async fn resolve_path(&self, base: &Path, path: &Path) -> miette::Result<PathBuf>;

    /// Given an initial path, fetch files matching <initial_path>/**/*.qs
    async fn collect_project_sources(
        &self,
        initial_path: &Path,
    ) -> miette::Result<Vec<Self::Entry>> {
        let listing = self.list_directory(initial_path).await?;
        if let Some(src_dir) = listing.into_iter().find(|x| {
            let Ok(entry_type) = x.entry_type() else {
                return false;
            };
            entry_type == EntryType::Folder && x.entry_name() == "src"
        }) {
            self.collect_project_sources_inner(&src_dir.path()).await
        } else {
            Err(miette::ErrReport::msg(
                "No `src` directory found for project.",
            ))
        }
    }

    async fn collect_project_sources_inner(
        &self,
        initial_path: &Path,
    ) -> miette::Result<Vec<Self::Entry>> {
        let listing = self.list_directory(initial_path).await?;
        let mut files = vec![];
        for item in filter_hidden_files(listing.into_iter()) {
            match item.entry_type() {
                Ok(EntryType::File) if item.entry_extension() == "qs" => files.push(item),
                Ok(EntryType::Folder) => {
                    files.append(&mut self.collect_project_sources_inner(&item.path()).await?);
                }
                _ => (),
            }
        }
        Ok(files)
    }

    /// Given a [ManifestDescriptor], load project sources.
    async fn load_project(&self, manifest: &ManifestDescriptor) -> miette::Result<Project> {
        let project_path = manifest.manifest_dir.clone();
        let qs_files = self.collect_project_sources(&project_path).await?;

        let qs_files = qs_files.into_iter().map(|file| file.path());

        let mut sources = Vec::with_capacity(qs_files.len());
        for path in qs_files {
            sources.push(self.read_file(&path).await?);
        }

        let manifest_path = self
            .resolve_path(&manifest.manifest_dir, Path::new("qsharp.json"))
            .await?;

        Ok(Project {
            name: manifest
                .manifest_dir
                .file_name()
                .map(|f| f.to_string_lossy().into())
                .unwrap_or(format!("Q# project at {}", manifest.manifest_dir.display()).into()),
            manifest_path: manifest_path.to_string_lossy().into(),
            manifest: manifest.manifest.clone(),
            sources,
        })
    }

    /// Given a directory path, parse the manifest and load the project sources.
    async fn load_project_in_dir(&self, directory: &Path) -> miette::Result<Project> {
        let manifest = self.parse_manifest_in_dir(directory).await?;

        self.load_project(&ManifestDescriptor {
            manifest_dir: directory.to_path_buf(),
            manifest,
        })
        .await
    }

    async fn parse_manifest_in_dir(&self, directory: &Path) -> Result<Manifest, miette::Error> {
        let manifest_path = self
            .resolve_path(directory, Path::new("qsharp.json"))
            .await?;
        let (_, manifest_content) = self.read_file(&manifest_path).await?;
        let manifest = serde_json::from_str::<Manifest>(&manifest_content).map_err(|e| {
            miette::ErrReport::msg(format!("Failed to parse `qsharp.json` file: {e}"))
        })?;
        Ok(manifest)
    }
}

/// Filters out any hidden files (files that start with '.')
fn filter_hidden_files<Entry: DirEntry>(
    listing: impl Iterator<Item = Entry>,
) -> impl Iterator<Item = Entry> {
    listing.filter(|x| !x.entry_name().starts_with('.'))
}

/// This trait is used to abstract filesystem logic with regards to Q# projects.
/// A Q# project requires some multi-file structure, but that may not actually be
/// an OS filesystem. It could be a virtual filesystem on vscode.dev, or perhaps a
/// cached implementation. This interface defines the minimal filesystem requirements
/// for the Q# project system to function correctly.
pub trait FileSystem {
    type Entry: DirEntry;
    /// Given a path, parse its contents and return a tuple representing (`FileName`, `FileContents`).
    fn read_file(&self, path: &Path) -> miette::Result<(Arc<str>, Arc<str>)>;

    /// Given a path, list its directory contents (if any).
    fn list_directory(&self, path: &Path) -> miette::Result<Vec<Self::Entry>>;

    fn resolve_path(&self, base: &Path, path: &Path) -> miette::Result<PathBuf>;

    /// Given a [`ManifestDescriptor`], load project sources.
    fn load_project(&self, manifest: &ManifestDescriptor) -> miette::Result<Project> {
        // Rather than rewriting all the async code in the project loader,
        // we call the async implementation here, doing some tricks to make it
        // run synchronously.

        let fs = ToFileSystemAsync { fs: self };

        // WARNING: This will panic if there are *any* await points in the
        // load_project implementation. Right now, we know that will never be the case
        // because we just passed in our synchronous FS functions to the project loader.
        // Proceed with caution if you make the `FileSystemAsync` implementation any
        // more complex.
        FutureExt::now_or_never(fs.load_project(manifest)).expect("load_project should never await")
    }
}

/// Trivial wrapper to turn a `FileSystem` into a `FileSystemAsync`
struct ToFileSystemAsync<'a, FS>
where
    FS: ?Sized,
{
    fs: &'a FS,
}

#[async_trait(?Send)]
impl<FS, E> FileSystemAsync for ToFileSystemAsync<'_, FS>
where
    E: DirEntry,
    FS: FileSystem<Entry = E> + ?Sized,
{
    type Entry = E;

    async fn read_file(&self, path: &Path) -> miette::Result<(Arc<str>, Arc<str>)> {
        self.fs.read_file(path)
    }
    async fn list_directory(&self, path: &Path) -> miette::Result<Vec<Self::Entry>> {
        self.fs.list_directory(path)
    }
    async fn resolve_path(&self, base: &Path, path: &Path) -> miette::Result<PathBuf> {
        self.fs.resolve_path(base, path)
    }
}
