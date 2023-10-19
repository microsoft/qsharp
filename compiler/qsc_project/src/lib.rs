//! This module handles the logic that constitutes the Q# project system.
//! This includes locating a manifest file in the filesystem, loading and parsing
//! the manifest, and determining which files are members of the project.

mod error;
mod manifest;
mod project;

pub use error::Error;
pub use manifest::{Manifest, MANIFEST_FILE_NAME};
pub use project::{FileSystem, Project, FS};
