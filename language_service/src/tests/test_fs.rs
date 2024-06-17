// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! An in-memory file system implementation for the unit tests.
//! Method signatures and behaviors are somewhat specialized to the way the
//! language service expects the fs to behave; if we want to reuse this in other
//! tests, it could use some work to make methods a little more general.

use qsc::LanguageFeatures;
use qsc_project::{EntryType, FileSystem, JSFileEntry, Manifest, ManifestDescriptor};
use rustc_hash::FxHashMap;
use std::sync::Arc;

use crate::state::LoadProjectResult;

pub(crate) enum FsNode {
    Dir(FxHashMap<Arc<str>, FsNode>),
    File(Arc<str>),
}

/// A file system operation error.
#[derive(Debug)]
pub(crate) enum FsError {
    NotFound,
}

impl FsNode {
    pub fn read_file(&self, file: String) -> (Arc<str>, Arc<str>) {
        let mut curr = Some(self);

        for part in file.split('/') {
            curr = curr.and_then(|node| match node {
                FsNode::Dir(dir) => dir.get(part),
                FsNode::File(_) => None,
            });
        }

        match curr {
            Some(FsNode::File(contents)) => (file.into(), contents.clone()),
            Some(FsNode::Dir(_)) | None => (file.into(), "".into()),
        }
    }

    pub fn write_file(&mut self, file: &str, contents: &str) -> Result<(), FsError> {
        let mut curr = Some(self);

        for part in file.split('/') {
            curr = curr.and_then(|node| match node {
                FsNode::Dir(dir) => dir.get_mut(part),
                FsNode::File(_) => None,
            });
        }

        if let Some(FsNode::File(curr_contents)) = curr {
            *curr_contents = contents.into();
            Ok(())
        } else {
            Err(FsError::NotFound)
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn list_directory(&self, dir_name: String) -> Vec<JSFileEntry> {
        let mut curr = Some(self);

        for part in dir_name.split('/') {
            curr = curr.and_then(|node| match node {
                FsNode::Dir(dir) => dir.get(part),
                FsNode::File(_) => None,
            });
        }

        match curr {
            Some(FsNode::Dir(dir)) => dir
                .iter()
                .map(|(name, node)| JSFileEntry {
                    name: format!("{dir_name}/{name}"),
                    r#type: match node {
                        FsNode::Dir(_) => EntryType::Folder,
                        FsNode::File(_) => EntryType::File,
                    },
                })
                .collect(),
            Some(FsNode::File(_)) | None => Vec::default(),
        }
    }

    pub fn resolve_path(base: &str, path: &str) -> String {
        let mut parts = base.split('/').collect::<Vec<_>>();

        for part in path.split('/') {
            if part == ".." {
                match parts.pop() {
                    Some(_) => continue,
                    None => panic!("path traversal outside of root"),
                }
            }
            parts.push(part);
        }

        parts.join("/")

        // TODO: shouldn't we need a little more validation than this?
        // like checking that the resolved path is actually within the root?
        // or like checking that the resolved path is actually a file or directory?
        // or like checking that the resolved path is not a symlink?
        // thanks copilot!
    }

    pub fn get_manifest(&self, file: &str) -> Option<ManifestDescriptor> {
        let mut curr = Some(self);
        let mut curr_path = String::new();
        let mut last_manifest_dir = None;
        let mut last_manifest = None;

        for part in file.split('/') {
            curr = curr.and_then(|node| match node {
                FsNode::Dir(dir) => {
                    if let Some(FsNode::File(manifest)) = dir.get("qsharp.json") {
                        // The semantics of get_manifest is that we only return the manifest
                        // if we've succeeded in parsing it
                        if let Ok(manifest) = serde_json::from_str::<Manifest>(manifest) {
                            last_manifest_dir = Some(curr_path.trim_end_matches('/').to_string());
                            last_manifest = Some(manifest);
                        }
                    }
                    curr_path = format!("{curr_path}{part}/");
                    dir.get(part)
                }
                FsNode::File(_) => None,
            });
        }

        match curr {
            Some(FsNode::Dir(_)) | None => None,
            Some(FsNode::File(_)) => last_manifest_dir.map(|dir| ManifestDescriptor {
                manifest: last_manifest.unwrap_or_default(),
                manifest_dir: dir.into(),
            }),
        }
    }

    pub fn remove(&mut self, path: &str) {
        let mut curr_parent = Some(self);
        let mut curr_name = None;

        for part in path.split('/') {
            if let Some(name) = curr_name {
                if let Some(FsNode::Dir(dir)) = curr_parent {
                    curr_parent = dir.get_mut(name);
                }
            }

            curr_name = Some(part);
        }

        let name = curr_name.expect("file name should have been set");

        match curr_parent {
            Some(FsNode::Dir(dir)) => dir.remove(name),
            Some(FsNode::File(_)) | None => panic!("path {path} does not exist"),
        };
    }

    pub fn load_project_with_deps(&self, file: &str) -> LoadProjectResult {
        let manifest = self.get_manifest(file);

        if let Some(manifest) = manifest {
            // TODO: I guess this should actually consume the deps?
            let project = FileSystem::load_project_with_deps(self, &manifest.manifest_dir, None);
            if let Ok(project) = project {
                let project = project.package_graph_sources.root;
                Some((
                    manifest.compilation_uri(),
                    project.sources,
                    LanguageFeatures::from_iter(project.language_features),
                    manifest.manifest.lints,
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl FileSystem for FsNode {
    type Entry = JSFileEntry;

    fn read_file(&self, path: &std::path::Path) -> miette::Result<(Arc<str>, Arc<str>)> {
        Ok(self.read_file(path.to_string_lossy().into()))
    }

    fn list_directory(&self, path: &std::path::Path) -> miette::Result<Vec<Self::Entry>> {
        Ok(self.list_directory(path.to_string_lossy().into()))
    }

    fn resolve_path(
        &self,
        base: &std::path::Path,
        path: &std::path::Path,
    ) -> miette::Result<std::path::PathBuf> {
        Ok(Self::resolve_path(&base.to_string_lossy(), &path.to_string_lossy()).into())
    }

    fn fetch_github(
        &self,
        _owner: &str,
        _repo: &str,
        _ref: &str,
        _path: &str,
    ) -> miette::Result<Arc<str>> {
        Err(miette::Error::msg(
            "github references not supported for this file system",
        ))
    }
}

pub(crate) fn dir<const COUNT: usize>(
    name: &str,
    contents: [(Arc<str>, FsNode); COUNT],
) -> (Arc<str>, FsNode) {
    (name.into(), FsNode::Dir(contents.into_iter().collect()))
}

pub(crate) fn file(name: &str, contents: &str) -> (Arc<str>, FsNode) {
    (name.into(), FsNode::File(Arc::from(contents)))
}
