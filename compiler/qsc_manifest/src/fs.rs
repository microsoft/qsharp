use crate::{ManifestDescriptor, MANIFEST_FILE_NAME};

pub use error::Error;
use std::{
    env::current_dir,
    fs::{self, DirEntry, FileType},
};

mod error;

pub type Result<T> = std::result::Result<T, Error>;

// TODO: inject file loader
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
// pub trait Entry {
//     fn name_of_entry(&self) -> Arc<str>;
//     fn path(&self) -> PathBuf;
// }

// impl Entry for DirEntry {
//     fn name_of_entry(&self) -> Arc<str> {
//         Arc::from(self.file_name().to_str().unwrap_or_default())
//     }
//     fn path(&self) -> PathBuf {
//         self.path()
//     }
// }

// pub trait TraversableFilesystem
// where
//     Self: Sized,
// {
//     type Error: From<serde_json::Error> + std::fmt::Debug;
//     type Entry: Entry;
//     fn parent_dir(&self) -> Option<Self>;
//     fn read_file_to_string(&self, file_entry: PathBuf) -> Result<String, Self::Error>;
//     fn read_directory(&self) -> Result<Vec<Result<Self::Entry, Self::Error>>, Self::Error>;

//     fn manifest_file(&self) -> Result<Option<ManifestDescriptor>, Self::Error> {
//         // read the current directory
//         let listing = self.read_directory()?;
//         for item in &listing {
//             println!("found {:?}", item.as_ref().map(|x| x.name_of_entry()));
//         }

//         // check if the current directory contains the manifest file
//         let entry = listing.into_iter().find_map(|entry| match entry {
//             Ok(entry) if &*entry.name_of_entry() == MANIFEST_FILE_NAME => Some(entry),
//             _ => None,
//         });

//         let entry = match entry {
//             Some(x) => x,
//             // if we did not find it, recurse on the parent directory
//             None => match self.parent_dir() {
//                 // there is a parent dir, so let's look for the manifest there.
//                 Some(parent) => {
//                     dbg!(&parent.path());
//                     return parent.manifest_file();
//                 }
//                 // there is no parent directory, so return. we didn't find a manifest
//                 None => return Ok(None),
//             },
//         };

//         //
//         let manifest_dir = entry
//             .path()
//             .expect("A file will always have a parent directory")
//             .into();
//         dbg!(&manifest_dir);

//         let contents = self.read_file_to_string(entry.path())?;

//         let manifest = serde_json::from_str(&contents)?;

//         Ok(Some(ManifestDescriptor::new(manifest, manifest_dir)))
//     }
// }

// impl TraversableFilesystem for std::path::PathBuf {
//     type Error = Error;
//     type Entry = DirEntry;
//     fn parent_dir(&self) -> Option<Self> {
//         self.parent().map(Into::into)
//     }

//     fn read_file_to_string(&self, file_entry: PathBuf) -> Result<String, Self::Error> {
//         std::fs::read_to_string(file_entry).map_err(Into::into)
//     }

//     fn read_directory(&self) -> Result<Vec<Result<Self::Entry, Self::Error>>, Self::Error> {
//         self.read_dir()
//             .map(|iter| iter.map(|x| x.map_err(Into::into)).collect::<Vec<_>>())
//             .map_err(Into::into)
//     }
// }
