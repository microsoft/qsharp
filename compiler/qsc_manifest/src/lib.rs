use std::path::PathBuf;

use serde::Deserialize;

pub const MANIFEST_FILE_NAME: &str = "qsharp.json";
pub struct ManifestDescriptor {
    manifest: Manifest,
    manifest_dir: PathBuf,
}

#[derive(Deserialize)]
pub struct Manifest {
    author: Option<String>,
    license: Option<String>,
    #[serde(default)]
    exclude_files: Vec<String>,
}

mod fs {
    use crate::{Manifest, ManifestDescriptor, MANIFEST_FILE_NAME};
    use error::Error;
    use std::path::PathBuf;
    mod error {
        use thiserror::Error;
        #[derive(Error, Debug)]
        pub enum Error {
            #[error(transparent)]
            SerdeJson(#[from] serde_json::Error),
            #[error(transparent)]
            Io(#[from] std::io::Error),
        }
    }

    pub trait TraversableFilesystem
    where
        Self: Sized,
    {
        type Error;
        fn parent_dir(&self) -> Option<Self>;
        fn manifest_file(&self) -> Result<Option<ManifestDescriptor>, Self::Error>;
        fn read_file_to_string(file_entry: PathBuf) -> Result<String, Self::Error>;
    }

    impl TraversableFilesystem for std::path::PathBuf {
        type Error = Error;
        fn parent_dir(&self) -> Option<Self> {
            self.parent().map(Into::into)
        }

        fn manifest_file(&self) -> Result<Option<ManifestDescriptor>, Self::Error> {
            let mut listing = self.read_dir()?;
            let entry = listing.find_map(|entry| match entry {
                Ok(entry) if entry.file_name() == MANIFEST_FILE_NAME => Some(entry),
                _ => None,
            });

            let entry = match entry {
                Some(x) => x,
                None => return Ok(None),
            };

            let manifest_dir = entry
                .path()
                .parent()
                .expect("A file will always have a parent directory")
                .into();

            let contents = Self::read_file_to_string(entry.path())?;

            let manifest = serde_json::from_str(&contents)?;

            Ok(Some(ManifestDescriptor {
                manifest,
                manifest_dir,
            }))
        }

        fn read_file_to_string(file_entry: PathBuf) -> Result<String, Self::Error> {
            std::fs::read_to_string(file_entry).map_err(Into::into)
        }
    }
}
