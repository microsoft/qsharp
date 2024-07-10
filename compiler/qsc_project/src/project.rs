// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    manifest::{GitHubRef, PackageType},
    Manifest, ManifestDescriptor, PackageRef,
};
use async_trait::async_trait;
use futures::FutureExt;
use miette::Diagnostic;
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_linter::LintConfig;
use rustc_hash::FxHashMap;
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    sync::Arc,
};
use thiserror::Error;

/// Describes a Q# project with all its sources and dependencies resolved.
#[derive(Debug)]
pub struct Project {
    /// Friendly name, typically based on project directory name
    /// Not guaranteed to be unique. Don't use as a key.
    pub name: Arc<str>,
    /// A path that represents the whole project.
    /// Typically the `qsharp.json` path for projects, or the document path for single files.
    pub path: Arc<str>,
    /// The package graph, including all sources and per-package
    /// configuration settings.
    pub package_graph_sources: PackageGraphSources,
    /// Lint configuration for the project, typically comes from the root `qsharp.json`.
    pub lints: Vec<LintConfig>,
    /// Any errors encountered while loading the project.
    pub errors: Vec<Error>,
}

impl Project {
    #[must_use]
    /// Given a source file, creates a project that contains just that file and
    /// default configuration options.
    pub fn from_single_file(name: Arc<str>, contents: Arc<str>) -> Self {
        let display_name = PathBuf::from(name.as_ref())
            .file_name()
            .map_or_else(|| name.clone(), |f| f.to_string_lossy().into());

        Self {
            package_graph_sources: PackageGraphSources {
                root: PackageInfo {
                    sources: vec![(name.clone(), contents)],
                    language_features: LanguageFeatures::default(),
                    dependencies: FxHashMap::default(),
                    package_type: None,
                },
                packages: FxHashMap::default(),
            },
            path: name,
            name: display_name,
            lints: Vec::default(),
            errors: Vec::default(),
        }
    }
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

/// Errors that can occur during project loading and dependency resolution.
#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("No `src` directory found for project")]
    #[diagnostic(code("Qsc.Project.NoSrcDir"))]
    NoSrcDir { path: String },

    #[error("Failed to parse manifest: {error}")]
    #[diagnostic(code("Qsc.Project.ManifestParse"))]
    ManifestParse { path: String, error: String },

    #[error("Failed to parse manifest for GitHub dependency {repo}/{owner} : {error}")]
    #[diagnostic(code("Qsc.Project.GitHubManifestParse"))]
    GitHubManifestParse {
        path: String,
        owner: String,
        repo: String,
        error: String,
    },

    #[error("Circular dependency detected between {0} and {1}")]
    #[diagnostic(code("Qsc.Project.CircularDependency"))]
    Circular(String, String),

    #[error("GitHub dependency {0} contains a local dependency {1}, which is not supported")]
    #[diagnostic(code("Qsc.Project.GitHubToLocal"))]
    GitHubToLocal(String, String),

    #[error("File system error: {about_path}: {error}")]
    #[diagnostic(code("Qsc.Project.FileSystem"))]
    FileSystem { about_path: String, error: String },

    #[error("Error fetching from GitHub: {0}")]
    #[diagnostic(code("Qsc.Project.GitHub"))]
    GitHub(String),
}

impl Error {
    /// Returns the document path that the error should be associated with when reporting.
    #[must_use]
    pub fn path(&self) -> Option<&String> {
        match self {
            Error::GitHubManifestParse { path, .. }
            | Error::NoSrcDir { path }
            | Error::ManifestParse { path, .. } => Some(path),
            // Note we don't return the path for `FileSystem` errors,
            // since for most errors such as "file not found", it's more meaningful
            // to report the error for the manifest that was *referencing* the file,
            // rather than for the file itself that provoked the I/O error.
            Error::FileSystem { .. }
            | Error::GitHubToLocal(_, _)
            | Error::Circular(_, _)
            | Error::GitHub(_) => None,
        }
    }
}

type ProjectResult<T> = Result<T, Error>;

/// This trait is used to abstract filesystem logic with regards to Q# projects.
/// A Q# project requires some multi-file structure, but that may not actually be
/// an OS filesystem. It could be a virtual filesystem on vscode.dev, or perhaps a
/// cached implementation. This interface defines the minimal filesystem requirements
/// for the Q# project system to function correctly.
#[async_trait(?Send)]
pub trait FileSystemAsync {
    type Entry: DirEntry;
    /// Given a path, parse its contents and return a tuple representing (FileName, FileContents).
    async fn read_file(&self, path: &Path) -> miette::Result<(Arc<str>, Arc<str>)>;

    /// Given a path, list its directory contents (if any).
    /// This function should only return files that end in *.qs and folders.
    async fn list_directory(&self, path: &Path) -> miette::Result<Vec<Self::Entry>>;

    /// Given a base path and a relative path, join the segments and normalize
    /// the path, i.e. replace '..', '.', and redundant separators.
    async fn resolve_path(&self, base: &Path, path: &Path) -> miette::Result<PathBuf>;

    /// Given a path to a file hosted on GitHub, fetches it from GitHub and returns the contents.
    async fn fetch_github(
        &self,
        owner: &str,
        repo: &str,
        r#ref: &str,
        path: &str,
    ) -> miette::Result<Arc<str>>;

    /// Given an initial path, fetch files matching <initial_path>/**/*.qs
    async fn collect_project_sources(
        &self,
        initial_path: &Path,
    ) -> ProjectResult<Vec<Self::Entry>> {
        let listing = self
            .list_directory(initial_path)
            .await
            .map_err(|e| Error::FileSystem {
                about_path: initial_path.to_string_lossy().to_string(),
                error: e.to_string(),
            })?;
        if let Some(src_dir) = listing.into_iter().find(|x| {
            let Ok(entry_type) = x.entry_type() else {
                return false;
            };
            entry_type == EntryType::Folder && x.entry_name() == "src"
        }) {
            self.collect_project_sources_inner(&src_dir.path()).await
        } else {
            Err(Error::NoSrcDir {
                path: initial_path.to_string_lossy().to_string(),
            })
        }
    }

    async fn collect_project_sources_inner(
        &self,
        initial_path: &Path,
    ) -> ProjectResult<Vec<Self::Entry>> {
        let listing = self
            .list_directory(initial_path)
            .await
            .map_err(|e| Error::FileSystem {
                about_path: initial_path.to_string_lossy().to_string(),
                error: e.to_string(),
            })?;
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

    /// Given a directory, loads the project sources
    /// and the sources for all its dependencies.
    async fn load_project(
        &self,
        directory: &Path,
        global_cache: Option<&RefCell<PackageCache>>,
    ) -> Result<Project, Vec<Error>> {
        let manifest = self
            .parse_manifest_in_dir(directory)
            .await
            .map_err(|e| vec![e])?;

        let root = self
            .read_local_manifest_and_sources(directory)
            .await
            .map_err(|e| vec![e])?;

        let mut errors = vec![];
        let mut packages = FxHashMap::default();
        let mut stack = vec![];

        let root_path = directory.to_string_lossy().to_string();
        let root_ref = PackageRef::Path { path: root_path };

        self.collect_deps(
            key_for_package_ref(&root_ref),
            &root,
            global_cache.unwrap_or(&RefCell::new(FxHashMap::default())),
            &mut stack,
            &mut packages,
            &mut errors,
            &root_ref,
        )
        .await;

        let name = directory
            .file_name()
            .map(|f| f.to_string_lossy().into())
            .unwrap_or(format!("Q# project at {}", directory.display()))
            .into();

        let manifest_path = self
            .resolve_path(directory, Path::new("qsharp.json"))
            .await
            .map_err(|e| {
                vec![Error::FileSystem {
                    about_path: directory.to_string_lossy().to_string(),
                    error: e.to_string(),
                }]
            })?
            .to_string_lossy()
            .into();

        Ok(Project {
            package_graph_sources: PackageGraphSources { root, packages },
            lints: manifest.lints,
            errors,
            name,
            path: manifest_path,
        })
    }

    /// Given a directory, attemps to parse a `qsharp.json` in that directory
    /// according to the manifest schema.
    async fn parse_manifest_in_dir(&self, directory: &Path) -> ProjectResult<Manifest> {
        let manifest_path = self
            .resolve_path(directory, Path::new("qsharp.json"))
            .await
            .map_err(|e| Error::FileSystem {
                about_path: directory.to_string_lossy().to_string(),
                error: e.to_string(),
            })?;
        let (_, manifest_content) =
            self.read_file(&manifest_path)
                .await
                .map_err(|e| Error::FileSystem {
                    about_path: manifest_path.to_string_lossy().to_string(),
                    error: e.to_string(),
                })?;
        let manifest = serde_json::from_str::<Manifest>(&manifest_content).map_err(|e| {
            Error::ManifestParse {
                path: manifest_path.to_string_lossy().to_string(),
                error: e.to_string(),
            }
        })?;
        Ok(manifest)
    }

    /// Load the sources for a single package at the given directory. Also load its
    /// dependency information but don't recurse into dependencies yet.
    async fn read_local_manifest_and_sources(
        &self,
        directory: &Path,
    ) -> ProjectResult<PackageInfo> {
        let manifest = self.parse_manifest_in_dir(directory).await?;

        let manifest = ManifestDescriptor {
            manifest_dir: directory.to_path_buf(),
            manifest,
        };

        let project_path = manifest.manifest_dir.clone();

        // If the `files` field exists in the manifest, prefer that.
        // Otherwise, collect all files in the project directory.
        let qs_files: Vec<PathBuf> = if manifest.manifest.files.is_empty() {
            let qs_files = self.collect_project_sources(&project_path).await?;
            qs_files.into_iter().map(|file| file.path()).collect()
        } else {
            let mut v = vec![];
            for file in manifest.manifest.files {
                v.push(
                    self.resolve_path(&project_path, Path::new(&file))
                        .await
                        .map_err(|e| Error::FileSystem {
                            about_path: project_path.to_string_lossy().to_string(),
                            error: e.to_string(),
                        })?,
                );
            }
            v
        };

        let mut sources = Vec::with_capacity(qs_files.len());
        for path in qs_files {
            sources.push(self.read_file(&path).await.map_err(|e| Error::FileSystem {
                about_path: path.to_string_lossy().to_string(),
                error: e.to_string(),
            })?);
        }

        let mut dependencies = FxHashMap::default();

        // For any local dependencies, convert relative paths to absolute,
        // so that multiple references to the same package, from different packages,
        // get merged correctly.
        for (alias, mut dep) in manifest.manifest.dependencies {
            if let PackageRef::Path { path: dep_path } = &mut dep {
                *dep_path = self
                    .resolve_path(&project_path, &PathBuf::from(dep_path.clone()))
                    .await
                    .map_err(|e| Error::FileSystem {
                        about_path: project_path.to_string_lossy().to_string(),
                        error: e.to_string(),
                    })?
                    .to_string_lossy()
                    .into();
            }
            dependencies.insert(alias.into(), key_for_package_ref(&dep));
        }

        Ok(PackageInfo {
            sources,
            language_features: LanguageFeatures::from_iter(&manifest.manifest.language_features),
            dependencies,
            package_type: manifest.manifest.package_type,
        })
    }

    /// Load the sources for a single package at the given GitHub ref. Also load its
    /// dependency information but don't recurse into dependencies yet.
    async fn read_github_manifest_and_sources(
        &self,
        dep: &GitHubRef,
    ) -> ProjectResult<PackageInfo> {
        let path_trimmed_seps = dep
            .path
            .as_ref()
            .map(|p| {
                p.strip_suffix('/')
                    .unwrap_or(p)
                    .strip_prefix('/')
                    .unwrap_or(p)
            })
            .unwrap_or_default();

        let manifest_path = format!("{path_trimmed_seps}/qsharp.json",);
        let manifest_content = self
            .fetch_github(&dep.owner, &dep.repo, &dep.r#ref, &manifest_path)
            .await
            .map_err(|e| Error::GitHub(e.to_string()))?;

        let manifest = serde_json::from_str::<Manifest>(&manifest_content).map_err(|e| {
            Error::GitHubManifestParse {
                path: format!(
                    "qsharp-github-source:{}/{}/{}{}/qsharp.json",
                    dep.owner,
                    dep.repo,
                    dep.r#ref,
                    if path_trimmed_seps.is_empty() {
                        String::new()
                    } else {
                        format!("/{path_trimmed_seps}")
                    }
                ),
                owner: dep.owner.clone(),
                repo: dep.repo.clone(),
                error: e.to_string(),
            }
        })?;

        // Look at the files field in the manifest to find the sources.
        let mut sources = vec![];
        for file in &manifest.files {
            let path = if path_trimmed_seps.is_empty() {
                format!("/{file}")
            } else {
                format!("/{path_trimmed_seps}/{file}")
            };
            let contents = self
                .fetch_github(&dep.owner, &dep.repo, &dep.r#ref, &path)
                .await
                .map_err(|e| Error::GitHub(e.to_string()))?;

            // Use the well-known URI scheme for the generated GitHub source paths.
            // This will allow the editor to recognize these URIs as GitHub sources
            // and open them using cached contents.
            sources.push((
                format!(
                    "qsharp-github-source:{}/{}/{}{path}",
                    dep.owner, dep.repo, dep.r#ref
                )
                .into(),
                contents,
            ));
        }

        Ok(PackageInfo {
            sources,
            language_features: LanguageFeatures::from_iter(&manifest.language_features),
            dependencies: manifest
                .dependencies
                .into_iter()
                .map(|(k, v)| (k.into(), key_for_package_ref(&v)))
                .collect(),
            package_type: manifest.package_type,
        })
    }

    /// Load the sources and dependency information for a single package,
    /// using a previously cached version if available.
    async fn read_manifest_and_sources(
        &self,
        global_cache: &RefCell<PackageCache>,
        key: PackageKey,
        this_pkg: &PackageRef,
    ) -> ProjectResult<PackageInfo> {
        match this_pkg {
            PackageRef::GitHub { github } => {
                {
                    let cache = global_cache.borrow();
                    if let Some(cached) = cache.get(&key) {
                        return cached.clone();
                    }
                }

                let result = self.read_github_manifest_and_sources(github).await;

                let mut cache = global_cache.borrow_mut();
                cache.insert(key, result.clone());

                result
            }
            PackageRef::Path { path } => {
                // Local dependencies are not cached at the moment, to make the multi-project
                // editing experience as intuitive as possible. This may change if we start
                // hitting perf issues, but careful consideration is needed into when to
                // invalidate the cache.
                self.read_local_manifest_and_sources(PathBuf::from(path.clone()).as_path())
                    .await
            }
        }
    }

    /// Recursive method to load sources for all dependencies and their
    /// dependencies, etc.
    #[allow(clippy::too_many_arguments)]
    async fn collect_deps(
        &self,
        key: Arc<str>,
        pkg: &PackageInfo,
        global_cache: &RefCell<PackageCache>,
        stack: &mut Vec<PackageKey>,
        packages: &mut FxHashMap<PackageKey, PackageInfo>,
        errors: &mut Vec<Error>,
        this_pkg: &PackageRef,
    ) {
        stack.push(key.clone());

        for (alias, dep_key) in &pkg.dependencies {
            if stack.contains(dep_key) {
                errors.push(Error::Circular(key.to_string(), dep_key.to_string()));
                continue;
            }

            let dependency = package_ref_from_key(dep_key);
            if matches!(dependency, PackageRef::Path { .. })
                && matches!(this_pkg, PackageRef::GitHub { .. })
            {
                errors.push(Error::GitHubToLocal(key.to_string(), alias.to_string()));
                continue;
            }

            let dep_result = self
                .read_manifest_and_sources(global_cache, dep_key.clone(), &dependency)
                .await;

            match dep_result {
                Ok(pkg) => {
                    self.collect_deps(
                        dep_key.clone(),
                        &pkg,
                        global_cache,
                        stack,
                        packages,
                        errors,
                        &dependency,
                    )
                    .await;
                    packages.insert(dep_key.clone(), pkg);
                }
                Err(e) => {
                    errors.push(e);
                }
            };
        }

        stack.pop();
    }
}

/// Filters out any hidden files (files that start with '.')
fn filter_hidden_files<Entry: DirEntry>(
    listing: impl Iterator<Item = Entry>,
) -> impl Iterator<Item = Entry> {
    listing.filter(|x| !x.entry_name().starts_with('.'))
}

/// We're using JSON to generate a key for the packages,
/// but something more readable would also be okay as long as we can
/// guarantee uniqueness.
#[must_use]
pub fn key_for_package_ref(dep: &PackageRef) -> PackageKey {
    serde_json::to_string(dep)
        .expect("dependency should be serializable")
        .into()
}

#[must_use]
pub fn package_ref_from_key(key: &PackageKey) -> PackageRef {
    serde_json::from_str(key).expect("dependency should be deserializable")
}

/// A `PackageKey` is a global unique identifier for a package, and it's
/// simply generated from the dependency information in the manifest
/// (local or GitHub path).
type PackageKey = Arc<str>;

/// A `PackageAlias` is the name that a package uses to refer to one of its
/// dependencies. The same package can live under different aliases in the
/// same package graph.
type PackageAlias = Arc<str>;

/// Long-lived cache that can optionally be used for loading packages.
pub type PackageCache = FxHashMap<PackageKey, ProjectResult<PackageInfo>>;

type Sources = Vec<(Arc<str>, Arc<str>)>;

#[derive(Clone, Debug)]
pub struct PackageInfo {
    pub sources: Sources,
    pub language_features: LanguageFeatures,
    pub dependencies: FxHashMap<PackageAlias, PackageKey>,
    pub package_type: Option<PackageType>,
}

#[derive(Clone, Debug)]
pub struct PackageGraphSources {
    pub root: PackageInfo,
    pub packages: FxHashMap<PackageKey, PackageInfo>,
}

#[derive(Debug)]
pub struct DependencyCycle;

pub type OrderedDependencies = Vec<(Arc<str>, PackageInfo)>;

impl PackageGraphSources {
    /// Produces an ordered vector over the packages in the order they should be compiled
    pub fn compilation_order(self) -> Result<(OrderedDependencies, PackageInfo), DependencyCycle> {
        // The order is defined by which packages depend on which other packages
        // For example, if A depends on B which depends on C, then we compile C, then B, then A
        // If there are cycles, this is an error, and we will report it as such
        let mut in_degree: FxHashMap<&str, usize> = FxHashMap::default();
        let mut graph: FxHashMap<&str, Vec<&str>> = FxHashMap::default();

        // Initialize the graph and in-degrees
        for (key, package_info) in &self.packages {
            in_degree.entry(key).or_insert(0);
            for dep in package_info.dependencies.values() {
                graph.entry(dep).or_default().push(key);
                *in_degree.entry(key).or_insert(0) += 1;
            }
        }

        let mut queue: Vec<&str> = in_degree
            .iter()
            .filter_map(|(key, &deg)| if deg == 0 { Some(*key) } else { None })
            .collect();

        let mut sorted_keys = Vec::new();

        while let Some(node) = queue.pop() {
            sorted_keys.push(node.to_string());
            if let Some(neighbors) = graph.get(node) {
                for &neighbor in neighbors {
                    let count = in_degree
                        .get_mut(neighbor)
                        .expect("graph pre-calculated this");
                    *count -= 1;
                    if *count == 0 {
                        queue.push(neighbor);
                    }
                }
            }
        }

        let mut sorted_packages = self.packages.into_iter().collect::<Vec<_>>();
        sorted_packages.sort_by_key(|(a_key, _pkg)| {
            sorted_keys
                .iter()
                .position(|key| key.as_str() == &**a_key)
                .unwrap_or_else(|| panic!("package {a_key} should be in sorted keys list"))
        });

        log::debug!("build plan: {:#?}", sorted_keys);

        Ok((sorted_packages, self.root))
    }

    #[must_use]
    pub fn with_no_dependencies(
        sources: Vec<(Arc<str>, Arc<str>)>,
        language_features: LanguageFeatures,
        package_type: Option<PackageType>,
    ) -> Self {
        Self {
            root: PackageInfo {
                sources,
                language_features,
                dependencies: FxHashMap::default(),
                package_type,
            },
            packages: FxHashMap::default(),
        }
    }
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

    fn fetch_github(
        &self,
        owner: &str,
        repo: &str,
        r#ref: &str,
        path: &str,
    ) -> miette::Result<Arc<str>>;

    fn load_project(
        &self,
        directory: &Path,
        global_cache: Option<&RefCell<PackageCache>>,
    ) -> Result<Project, Vec<Error>> {
        // Rather than rewriting all the async code in the project loader,
        // we call the async implementation here, doing some tricks to make it
        // run synchronously.

        let fs = ToFileSystemAsync { fs: self };

        // WARNING: This will panic if there are *any* await points in the
        // load_project implementation. Right now, we know that will never be the case
        // because we just passed in our synchronous FS functions to the project loader.
        // Proceed with caution if you make the `FileSystemAsync` implementation any
        // more complex.
        FutureExt::now_or_never(fs.load_project(directory, global_cache))
            .expect("load_project should never await")
    }
}

/// Trivial wrapper to turn a `FileSystem` into a `FileSystemAsync`
struct ToFileSystemAsync<'a, FS>
where
    FS: ?Sized,
{
    fs: &'a FS,
}

#[async_trait(?Send)]
impl<FS, E> FileSystemAsync for ToFileSystemAsync<'_, FS>
where
    E: DirEntry,
    FS: FileSystem<Entry = E> + ?Sized,
{
    type Entry = E;

    async fn read_file(&self, path: &Path) -> miette::Result<(Arc<str>, Arc<str>)> {
        self.fs.read_file(path)
    }

    async fn list_directory(&self, path: &Path) -> miette::Result<Vec<Self::Entry>> {
        self.fs.list_directory(path)
    }

    async fn resolve_path(&self, base: &Path, path: &Path) -> miette::Result<PathBuf> {
        self.fs.resolve_path(base, path)
    }

    async fn fetch_github(
        &self,
        owner: &str,
        repo: &str,
        r#ref: &str,
        path: &str,
    ) -> miette::Result<Arc<str>> {
        self.fs.fetch_github(owner, repo, r#ref, path)
    }
}
