use crate::{ManifestDescriptor, MANIFEST_FILE_NAME};

pub use error::Error;
use std::{
    env::current_dir,
    fs::{self, DirEntry, FileType},
};

mod error;

pub type Result<T> = std::result::Result<T, Error>;

pub fn find_manifest() -> Result<Option<ManifestDescriptor>> {
    let current_dir = current_dir()?;
    let ancestors = current_dir.ancestors();
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
