// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This module handles the logic that constitutes the Q# project system.
//! This includes locating a manifest file in the filesystem, loading and parsing
//! the manifest, and determining which files are members of the project.

mod error;
#[cfg(all(feature = "fs"))]
mod fs;
mod manifest;
mod project;

pub use error::Error;
#[cfg(all(feature = "fs"))]
pub use fs::StdFs;
pub use manifest::{Manifest, ManifestDescriptor, MANIFEST_FILE_NAME};
#[cfg(all(feature = "async"))]
pub use project::FileSystemAsync;
pub use project::{DirEntry, EntryType, FileSystem, Project};
