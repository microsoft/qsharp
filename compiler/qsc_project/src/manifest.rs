use serde::Deserialize;
use std::path::PathBuf;

pub const MANIFEST_FILE_NAME: &str = "qsharp.json";
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

#[derive(Deserialize, Debug)]
pub struct Manifest {
    pub author: Option<String>,
    pub license: Option<String>,
    #[serde(default)]
    pub exclude_files: Vec<String>,
}
