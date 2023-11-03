use std::path::PathBuf;

use crate::{DirEntry, EntryType};

pub struct JsFileEntry {
    ty: EntryType,
    extension: String,
    name: String,
    path: String,
}

impl From<[String; 2]> for JsFileEntry {
    fn from([path_string, _contents]: [String; 2]) -> Self {
        let path = PathBuf::from(path_string.clone());
        let name = path
            .file_name()
            .map(|x| x.to_string_lossy())
            .unwrap_or_default()
            .to_string();

        let extension = path
            .extension()
            .map(|x| x.to_string_lossy())
            .unwrap_or_default()
            .to_string();

        let ty = if path.is_dir() {
            EntryType::Folder
        } else if path.is_file() {
            EntryType::File
        } else {
            EntryType::Symlink
        };

        Self {
            ty,
            extension,
            name,
            path: path_string,
        }
    }
}

impl DirEntry for JsFileEntry {
    type Error = String;

    fn entry_type(&self) -> Result<EntryType, Self::Error> {
        Ok(self.ty)
    }

    fn extension(&self) -> String {
        self.extension.clone()
    }

    fn entry_name(&self) -> String {
        self.name.clone()
    }

    fn path(&self) -> std::path::PathBuf {
        PathBuf::from(&self.path)
    }
}
