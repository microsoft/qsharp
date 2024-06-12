// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{manifest::ManifestDescriptor, Dependency, Manifest};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

/// Describes a Q# project
#[derive(Default, Debug)]
pub struct Project {
    pub sources: Vec<(Arc<str>, Arc<str>)>,
    pub manifest: crate::Manifest,
}

/// This enum represents a filesystem object type. It is analogous to [`std::fs::FileType`].
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum EntryType {
    File,
    Folder,
    Symlink,
    Unknown,
}

/// This trait represents a filesystem object. It is analogous to [`std::fs::DirEntry`].
pub trait DirEntry {
    type Error: Send + Sync;
    fn entry_type(&self) -> Result<EntryType, Self::Error>;
    fn path(&self) -> PathBuf;
    fn entry_extension(&self) -> String {
        self.path()
            .extension()
            .map(|x| x.to_string_lossy().to_string())
            .unwrap_or_default()
    }
    fn entry_name(&self) -> String {
        self.path()
            .file_name()
            .expect("canonicalized symlink cannot end in relative path")
            .to_string_lossy()
            .to_string()
    }
}

/// This trait is used to abstract filesystem logic with regards to Q# projects.
/// A Q# project requires some multi-file structure, but that may not actually be
/// an OS filesystem. It could be a virtual filesystem on vscode.dev, or perhaps a
/// cached implementation. This interface defines the minimal filesystem requirements
/// for the Q# project system to function correctly.
#[cfg(feature = "async")]
use async_trait::async_trait;
use qsc_linter::LintConfig;
use rustc_hash::FxHashMap;
#[cfg(feature = "async")]
#[async_trait(?Send)]
pub trait FileSystemAsync {
    type Entry: DirEntry + Send + Sync;
    /// Given a path, parse its contents and return a tuple representing (FileName, FileContents).
    async fn read_file(&self, path: &Path) -> miette::Result<(Arc<str>, Arc<str>)>;

    /// Given a path, list its directory contents (if any).
    /// This function should only return files that end in *.qs and folders.
    async fn list_directory(&self, path: &Path) -> miette::Result<Vec<Self::Entry>>;

    async fn resolve_path(&self, base: &Path, path: &Path) -> miette::Result<PathBuf>;

    /// Given an initial path, fetch files matching <initial_path>/**/*.qs
    async fn collect_project_sources(
        &self,
        initial_path: &Path,
    ) -> miette::Result<Vec<Self::Entry>> {
        let listing = self.list_directory(initial_path).await?;
        if let Some(src_dir) = listing.into_iter().find(|x| {
            let Ok(entry_type) = x.entry_type() else {
                return false;
            };
            entry_type == EntryType::Folder && x.entry_name() == "src"
        }) {
            self.collect_project_sources_inner(&src_dir.path()).await
        } else {
            Err(miette::ErrReport::msg(
                "No `src` directory found for project.",
            ))
        }
    }

    async fn collect_project_sources_inner(
        &self,
        initial_path: &Path,
    ) -> miette::Result<Vec<Self::Entry>> {
        let listing = self.list_directory(initial_path).await?;
        let mut files = vec![];
        for item in filter_hidden_files(listing.into_iter()) {
            match item.entry_type() {
                Ok(EntryType::File) if item.entry_extension() == "qs" => files.push(item),
                Ok(EntryType::Folder) => {
                    files.append(&mut self.collect_project_sources_inner(&item.path()).await?);
                }
                _ => (),
            }
        }
        Ok(files)
    }
    /// Given a [ManifestDescriptor], load project sources.
    async fn load_project(&self, manifest: &ManifestDescriptor) -> miette::Result<Project> {
        let project_path = manifest.manifest_dir.clone();
        let qs_files = self.collect_project_sources(&project_path).await?;

        let qs_files = qs_files.into_iter().map(|file| file.path());

        let mut sources = Vec::with_capacity(qs_files.len());
        for path in qs_files {
            sources.push(self.read_file(&path).await?);
        }

        Ok(Project {
            manifest: manifest.manifest.clone(),
            sources,
        })
    }
}

/// Filters out any hidden files (files that start with '.')
fn filter_hidden_files<Entry: DirEntry>(
    listing: impl Iterator<Item = Entry>,
) -> impl Iterator<Item = Entry> {
    listing.filter(|x| !x.entry_name().starts_with('.'))
}

/// This trait is used to abstract filesystem logic with regards to Q# projects.
/// A Q# project requires some multi-file structure, but that may not actually be
/// an OS filesystem. It could be a virtual filesystem on vscode.dev, or perhaps a
/// cached implementation. This interface defines the minimal filesystem requirements
/// for the Q# project system to function correctly.
pub trait FileSystem {
    type Entry: DirEntry;
    /// Given a path, parse its contents and return a tuple representing (`FileName`, `FileContents`).
    fn read_file(&self, path: &Path) -> miette::Result<(Arc<str>, Arc<str>)>;

    /// Given a path, list its directory contents (if any).
    fn list_directory(&self, path: &Path) -> miette::Result<Vec<Self::Entry>>;

    fn resolve_path(&self, base: &Path, path: &Path) -> miette::Result<PathBuf>;

    /// Given an initial path, fetch files matching <`initial_path`>/**/*.qs
    fn collect_project_sources(&self, initial_path: &Path) -> miette::Result<Vec<Self::Entry>> {
        let listing = self.list_directory(initial_path)?;
        if let Some(src_dir) = listing.into_iter().find(|x| {
            let Ok(entry_type) = x.entry_type() else {
                return false;
            };
            entry_type == EntryType::Folder && x.entry_name() == "src"
        }) {
            self.collect_project_sources_inner(&src_dir.path())
        } else {
            Err(miette::ErrReport::msg(
                "No `src` directory found for project.",
            ))
        }
    }

    fn collect_project_sources_inner(
        &self,
        initial_path: &Path,
    ) -> miette::Result<Vec<Self::Entry>> {
        let listing = self.list_directory(initial_path)?;
        let mut files = vec![];
        for item in filter_hidden_files(listing.into_iter()) {
            match item.entry_type() {
                Ok(EntryType::File) if item.entry_extension() == "qs" => files.push(item),
                Ok(EntryType::Folder) => {
                    files.append(&mut self.collect_project_sources_inner(&item.path())?);
                }
                _ => (),
            }
        }
        Ok(files)
    }

    /// Given a [`ManifestDescriptor`], load project sources.
    fn load_project_sources(&self, manifest: &ManifestDescriptor) -> miette::Result<Project> {
        let project_path = manifest.manifest_dir.clone();
        let qs_files = self.collect_project_sources(&project_path)?;

        let qs_files = qs_files.into_iter().map(|file| file.path());

        let qs_sources = qs_files.map(|path| self.read_file(&path));

        let sources = qs_sources.collect::<miette::Result<_>>()?;
        Ok(Project {
            manifest: manifest.manifest.clone(),
            sources,
        })
    }

    fn parse_manifest_in_dir(&self, directory: &Path) -> Result<Manifest, miette::Error> {
        let listing = self.list_directory(directory)?;
        let qsharp_json = listing
            .into_iter()
            .find(|e| e.entry_name() == "qsharp.json")
            .ok_or(miette::ErrReport::msg(
                "No `qsharp.json` file found in project directory.",
            ))?;
        let (_, manifest_content) = self.read_file(&qsharp_json.path())?;
        let manifest = serde_json::from_str::<Manifest>(&manifest_content).map_err(|e| {
            miette::ErrReport::msg(format!("Failed to parse `qsharp.json` file: {e}"))
        })?;
        Ok(manifest)
    }

    fn read_local_manifest_and_sources(&self, directory: &Path) -> miette::Result<Project> {
        let manifest = self.parse_manifest_in_dir(directory)?;

        self.load_project_sources(&ManifestDescriptor {
            manifest_dir: directory.to_path_buf(),
            manifest,
        })
    }

    fn read_github_manifest_and_sources(&self) -> miette::Result<Project> {
        // TODO: support them obviously
        Err(miette::ErrReport::msg(
            "GitHub dependencies are not supported yet.",
        ))
    }

    fn read_manifest_and_sources(
        &self,
        // global_cache: &mut FxHashMap<PackageKey, miette::Result<(Manifest, PackageInfo)>>,
        // key: PackageKey,
        this_pkg: &Dependency,
    ) -> miette::Result<(Manifest, PackageInfo)> {
        let mut project = match this_pkg {
            Dependency::GitHub { .. } => self.read_github_manifest_and_sources()?,
            Dependency::Path { path } => {
                self.read_local_manifest_and_sources(PathBuf::from(path.clone()).as_path())?
            }
        };

        let mut dependencies = FxHashMap::default();
        for (alias, dep) in &mut project.manifest.dependencies {
            if let Dependency::Path { path: dep_path } = dep {
                if let Dependency::Path { path: this_path } = this_pkg {
                    *dep_path = self
                        .resolve_path(
                            PathBuf::from(this_path).as_path(),
                            PathBuf::from(dep_path.clone()).as_path(),
                        )?
                        .to_string_lossy()
                        .into();
                }
            }
            dependencies.insert(alias.clone().into(), key_for_dependency_definition(dep));
        }

        let language_features = project.manifest.language_features.clone();

        Ok((
            project.manifest,
            PackageInfo {
                sources: project.sources,
                language_features,
                dependencies,
            },
        ))
    }
    fn read_manifest_and_sources_cached(
        &self,
        global_cache: &mut FxHashMap<PackageKey, Result<(Manifest, PackageInfo), String>>,
        key: PackageKey,
        this_pkg: &Dependency,
    ) -> miette::Result<(Manifest, PackageInfo)> {
        if let Some(cached) = global_cache.get(&key) {
            return cached.clone().map_err(miette::ErrReport::msg);
        }

        let result = self.read_manifest_and_sources(this_pkg);
        global_cache.insert(
            key,
            match &result {
                Ok(result) => Ok(result.clone()),
                Err(e) => Err(e.to_string()),
            },
        );
        result
    }

    fn collect_package(
        &self,
        global_cache: &mut FxHashMap<PackageKey, Result<(Manifest, PackageInfo), String>>,
        stack: &mut Vec<PackageKey>,
        packages: &mut FxHashMap<PackageKey, PackageInfo>,
        errors: &mut Vec<miette::Report>,
        this_pkg: &Dependency,
    ) -> Option<PackageInfo> {
        let key = key_for_dependency_definition(this_pkg);
        let result = self.read_manifest_and_sources_cached(global_cache, key.clone(), this_pkg);
        let pkg = match result {
            Ok(pkg) => pkg.1,
            Err(e) => {
                errors.push(e);
                return None;
            }
        };

        stack.push(key.clone());

        for (alias, dep_key) in &pkg.dependencies {
            if stack.contains(dep_key) {
                // TODO: ok to disallow circular dependencies?
                // Technically we could support them but it's a pain
                errors.push(miette::ErrReport::msg(format!(
                    "Circular dependency detected: {alias} -> {dep_key}"
                )));
                continue;
            }

            let dependency = decode_dependency_defintion_from_key(dep_key);
            if matches!(dependency, Dependency::Path { .. })
                && matches!(this_pkg, Dependency::GitHub { .. })
            {
                errors.push(miette::ErrReport::msg(
                    "Local dependencies are not allowed in GitHub dependencies.",
                ));
                continue;
            }

            if let Some(dep_pkg) =
                self.collect_package(global_cache, stack, packages, errors, &dependency)
            {
                // TODO: do we ever end up processing the same package twice, and is that a big deal?
                packages.insert(dep_key.clone(), dep_pkg);
            }

            // TODO: absolute paths in manifests
            // TODO: os-specific slashes in manifests
        }

        stack.pop();
        Some(pkg)
    }

    fn load_project_with_deps(
        &self,
        directory: &Path,
        global_cache: Option<&mut PackageCache>,
    ) -> miette::Result<ProgramConfig> {
        let manifest = self.parse_manifest_in_dir(directory)?;

        let mut errors = vec![];
        let mut packages = FxHashMap::default();
        let mut stack = vec![];

        let root = self.collect_package(
            global_cache.unwrap_or(&mut FxHashMap::default()),
            &mut stack,
            &mut packages,
            &mut errors,
            &Dependency::Path {
                path: directory.to_string_lossy().into(),
            },
        );

        match root {
            None => Err(miette::ErrReport::msg(format!(
                "Failed to load root package : {}",
                errors
                    .into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            ))),
            Some(this_pkg) => Ok(ProgramConfig {
                package_graph_sources: PackageGraphSources {
                    root: this_pkg,
                    packages,
                },
                lints: manifest.lints,
                errors,
            }),
        }
    }
}

fn key_for_dependency_definition(dep: &Dependency) -> PackageKey {
    serde_json::to_string(dep)
        .expect("dependency should be serializable")
        .into()
}

fn decode_dependency_defintion_from_key(key: &PackageKey) -> Dependency {
    serde_json::from_str(key).expect("dependency should be deserializable")
}

type PackageKey = Arc<str>;
type PackageAlias = Arc<str>;
type PackageCache = FxHashMap<PackageKey, Result<(Manifest, PackageInfo), String>>;

#[derive(Clone)]
pub struct PackageInfo {
    pub sources: Vec<(Arc<str>, Arc<str>)>,
    pub language_features: Vec<String>,
    pub dependencies: FxHashMap<PackageAlias, PackageKey>,
}

pub struct PackageGraphSources {
    pub root: PackageInfo,
    pub packages: FxHashMap<PackageKey, PackageInfo>,
}

pub struct ProgramConfig {
    pub package_graph_sources: PackageGraphSources,
    pub lints: Vec<LintConfig>,
    pub errors: Vec<miette::Report>,
}
