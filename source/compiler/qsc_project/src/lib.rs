// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This module handles the logic that constitutes the Q# project system.
//! This includes locating a manifest file in the filesystem, loading and parsing
//! the manifest, and determining which files are members of the project.

#[cfg(test)]
mod tests;

mod error;
#[cfg(feature = "fs")]
mod fs;
mod js;
mod manifest;
pub mod openqasm;
mod project;

#[cfg(feature = "fs")]
pub use error::StdFsError;
#[cfg(feature = "fs")]
pub use fs::StdFs;
pub use js::{JSFileEntry, JSProjectHost};
pub use manifest::{MANIFEST_FILE_NAME, Manifest, ManifestDescriptor, PackageRef, PackageType};
pub use project::FileSystemAsync;
pub use project::{
    DependencyCycle, DirEntry, EntryType, Error, FileSystem, GITHUB_SCHEME, PackageCache,
    PackageGraphSources, PackageInfo, Project, ProjectType, key_for_package_ref,
    package_ref_from_key,
};
