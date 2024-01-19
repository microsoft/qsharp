// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(feature = "fs")]
use crate::Error;
#[cfg(feature = "fs")]
use std::{
    env::current_dir,
    fs::{self, DirEntry, FileType},
};

use serde::Deserialize;
use std::{path::PathBuf, sync::Arc};

pub const MANIFEST_FILE_NAME: &str = "qsharp.json";

/// A Q# manifest, used to describe project metadata.
#[derive(Deserialize, Debug, Default, Clone)]
pub struct Manifest {
    pub author: Option<String>,
    pub license: Option<String>,
}

/// Describes the contents and location of a Q# manifest file.
#[derive(Debug)]
pub struct ManifestDescriptor {
    pub manifest: Manifest,
    pub manifest_dir: PathBuf,
}

impl ManifestDescriptor {
    /// Generate a canonical compilation URI for the project associated with this manifest
    pub fn compilation_uri(&self) -> Arc<str> {
        Arc::from(format!(
            "{}/qsharp.json",
            self.manifest_dir.to_string_lossy()
        ))
    }
}

#[cfg(feature = "fs")]
impl Manifest {
    /// Starting from the current directory, traverse ancestors until
    /// a manifest is found.
    /// Returns an error if there are any filesystem errors, or if
    /// a manifest file exists but is the wrong format.
    /// Returns `Ok(None)` if there is no file matching the manifest file
    /// name.
    pub fn load(
        manifest_path: Option<PathBuf>,
    ) -> std::result::Result<Option<ManifestDescriptor>, Error> {
        let dir = match manifest_path {
            Some(path) => path,
            None => current_dir()?,
        };
        Self::load_from_path(dir)
    }

    /// Given a [PathBuf], traverse [PathBuf::ancestors] until a Manifest is found.
    /// Returns [None] if no manifest named [MANIFEST_FILE_NAME] is found.
    /// Returns an error if a manifest is found, but is not parsable into the
    /// expected format.
    pub fn load_from_path(path: PathBuf) -> std::result::Result<Option<ManifestDescriptor>, Error> {
        // if the given path points to a file, change it to point to the parent folder.
        // This lets consumers pass in either the path directly to the manifest file, or the path
        // to the folder containing the manifest file.
        let path = if path.is_file() {
            let mut path = path;
            path.pop();
            path
        } else {
            path
        };
        let ancestors = path.ancestors();
        for ancestor in ancestors {
            let listing = ancestor.read_dir()?;
            for item in listing.into_iter().filter_map(only_valid_files) {
                if item.file_name().to_str() == Some(MANIFEST_FILE_NAME) {
                    let mut manifest_dir = item.path();
                    // pop off the file name itself
                    manifest_dir.pop();

                    let manifest = fs::read_to_string(item.path())?;
                    let manifest = serde_json::from_str(&manifest)?;
                    return Ok(Some(ManifestDescriptor {
                        manifest,
                        manifest_dir,
                    }));
                }
            }
        }
        Ok(None)
    }
}

/// Utility function which filters out any [DirEntry] which is not a valid file or
/// was unable to be read.
#[cfg(feature = "fs")]
fn only_valid_files(item: std::result::Result<DirEntry, std::io::Error>) -> Option<DirEntry> {
    match item {
        Ok(item)
            if (item
                .file_type()
                .as_ref()
                .map(FileType::is_file)
                .unwrap_or_default()) =>
        {
            Some(item)
        }
        _ => None,
    }
}
