use crate::Error;
use serde::Deserialize;
use std::path::PathBuf;
use std::{
    env::current_dir,
    fs::{self, DirEntry, FileType},
};

pub const MANIFEST_FILE_NAME: &str = "qsharp.json";

#[derive(Deserialize, Debug, Default)]
pub struct Manifest {
    pub author: Option<String>,
    pub license: Option<String>,
    #[serde(default)]
    pub exclude_files: Vec<String>,
}

/// Describes the contents and location of a Q# manifest file.
#[derive(Debug)]
pub struct ManifestDescriptor {
    pub manifest: Manifest,
    pub manifest_dir: PathBuf,
}

impl ManifestDescriptor {
    pub(crate) fn new(manifest: Manifest, manifest_dir: PathBuf) -> Self {
        Self {
            manifest,
            manifest_dir,
        }
    }
}

impl Manifest {
    /// Starting from the current directory, traverse ancestors until
    /// a manifest is found.
    /// Returns an error if there are any filesystem errors, or if
    /// a manifest file exists but is the wrong format.
    /// Returns `Ok(None)` if there is no file matching the manifest file
    /// name.
    pub fn load() -> std::result::Result<Option<ManifestDescriptor>, Error> {
        let current_dir = current_dir()?;
        Self::load_from_path(current_dir)
    }

    pub fn load_from_path(path: PathBuf) -> std::result::Result<Option<ManifestDescriptor>, Error> {
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
                    return Ok(Some(ManifestDescriptor::new(manifest, manifest_dir)));
                }
            }
        }
        Ok(None)
    }
}
/// Utility function which filters out any [DirEntry] which is not a valid file or
/// was unable to be read.
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
