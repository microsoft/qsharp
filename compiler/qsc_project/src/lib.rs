// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This module handles the logic that constitutes the Q# project system.
//! This includes locating a manifest file in the filesystem, loading and parsing
//! the manifest, and determining which files are members of the project.

mod error;
#[cfg(feature = "fs")]
mod fs;
mod js;
mod manifest;
mod project;

pub use error::StdFsError;
#[cfg(feature = "fs")]
pub use fs::StdFs;
pub use js::{JSFileEntry, JSProjectHost};
pub use manifest::{Manifest, ManifestDescriptor, PackageRef, MANIFEST_FILE_NAME};
pub use project::FileSystemAsync;
pub use project::{
    dependency_definition_from_key, key_for_dependency_definition, DirEntry, EntryType, Error,
    FileSystem, PackageCache, PackageGraphSources, PackageInfo, Project,
};
