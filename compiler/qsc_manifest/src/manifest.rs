#[cfg(test)]
mod tests;

use serde::Deserialize;
use std::path::PathBuf;

pub const MANIFEST_FILE_NAME: &str = "qsharp.json";
#[derive(Debug)]
pub struct ManifestDescriptor {
    manifest: Manifest,
    manifest_dir: PathBuf,
}
impl ManifestDescriptor {
    pub(crate) fn new(manifest: Manifest, manifest_dir: PathBuf) -> Self {
        Self {
            manifest,
            manifest_dir,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Manifest {
    author: Option<String>,
    license: Option<String>,
    #[serde(default)]
    exclude_files: Vec<String>,
}
