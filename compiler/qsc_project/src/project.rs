// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{manifest::GitHubRef, Dependency, Manifest, ManifestDescriptor};
use std::{
    cell::RefCell,
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
use async_trait::async_trait;
use futures::FutureExt;
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_linter::LintConfig;
use rustc_hash::FxHashMap;
#[async_trait(?Send)]
pub trait FileSystemAsync {
    type Entry: DirEntry;
    /// Given a path, parse its contents and return a tuple representing (FileName, FileContents).
    async fn read_file(&self, path: &Path) -> miette::Result<(Arc<str>, Arc<str>)>;

    /// Given a path, list its directory contents (if any).
    /// This function should only return files that end in *.qs and folders.
    async fn list_directory(&self, path: &Path) -> miette::Result<Vec<Self::Entry>>;

    async fn resolve_path(&self, base: &Path, path: &Path) -> miette::Result<PathBuf>;

    // TODO: stretching the definition of "file system" here...
    // maybe we can call this struct "HostAsync" or something
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
    async fn load_project_sources(&self, manifest: &ManifestDescriptor) -> miette::Result<Project> {
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

    async fn parse_manifest_in_dir(&self, directory: &Path) -> Result<Manifest, miette::Error> {
        let manifest_path = self
            .resolve_path(directory, Path::new("qsharp.json"))
            .await?;
        let (_, manifest_content) = self.read_file(&manifest_path).await?;
        let manifest = serde_json::from_str::<Manifest>(&manifest_content).map_err(|e| {
            miette::ErrReport::msg(format!("Failed to parse `qsharp.json` file: {e}"))
        })?;
        Ok(manifest)
    }

    async fn read_local_manifest_and_sources(&self, directory: &Path) -> miette::Result<Project> {
        let manifest = self.parse_manifest_in_dir(directory).await?;

        self.load_project_sources(&ManifestDescriptor {
            manifest_dir: directory.to_path_buf(),
            manifest,
        })
        .await
    }

    async fn read_github_manifest_and_sources(&self, dep: &GitHubRef) -> miette::Result<Project> {
        let path = dep
            .path
            .as_ref()
            .map(|p| if p == "/" { "" } else { p })
            .unwrap_or_default();
        let manifest_path = format!("{path}/qsharp.json",);
        let manifest_content = self
            .fetch_github(&dep.owner, &dep.repo, &dep.r#ref, &manifest_path)
            .await?;
        let manifest = serde_json::from_str::<Manifest>(&manifest_content).map_err(|e| {
            miette::ErrReport::msg(format!("Failed to parse `qsharp.json` file: {e}"))
        })?;

        // TODO: file list should be required for github packages (and possibly local packages too)
        let mut sources = vec![];
        for file in &manifest.files {
            let path = format!("{path}/{file}");
            let contents = self
                .fetch_github(&dep.owner, &dep.repo, &dep.r#ref, &path)
                .await?;
            sources.push((
                format!(
                    "qsharp-github-source:{}/{}/{}{path}",
                    dep.owner, dep.repo, dep.r#ref
                )
                .into(),
                contents,
            ));
        }

        Ok(Project { sources, manifest })
    }

    async fn read_manifest_and_sources(
        &self,
        this_pkg: &Dependency,
    ) -> miette::Result<(Manifest, PackageInfo)> {
        let mut project = match this_pkg {
            Dependency::GitHub { github } => self.read_github_manifest_and_sources(github).await?,
            Dependency::Path { path } => {
                self.read_local_manifest_and_sources(PathBuf::from(path.clone()).as_path())
                    .await?
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
                        )
                        .await?
                        .to_string_lossy()
                        .into();
                }
            }
            dependencies.insert(alias.clone().into(), key_for_dependency_definition(dep));
        }

        let language_features = LanguageFeatures::from_iter(&project.manifest.language_features);

        Ok((
            project.manifest,
            PackageInfo {
                sources: project.sources,
                language_features,
                dependencies,
            },
        ))
    }

    async fn read_manifest_and_sources_cached(
        &self,
        global_cache: &RefCell<FxHashMap<PackageKey, Result<(Manifest, PackageInfo), String>>>,
        key: PackageKey,
        this_pkg: &Dependency,
    ) -> miette::Result<(Manifest, PackageInfo)> {
        {
            let cache = global_cache.borrow();
            if let Some(cached) = cache.get(&key) {
                return cached.clone().map_err(miette::ErrReport::msg);
            }
        }

        let result = self.read_manifest_and_sources(this_pkg).await;

        {
            let mut cache = global_cache.borrow_mut();
            cache.insert(
                key,
                match &result {
                    Ok(result) => Ok(result.clone()),
                    Err(e) => Err(e.to_string()),
                },
            );
        }
        result
    }

    #[allow(clippy::too_many_arguments)]
    async fn collect_deps(
        &self,
        key: Arc<str>,
        pkg: &PackageInfo,
        global_cache: &RefCell<FxHashMap<PackageKey, Result<(Manifest, PackageInfo), String>>>,
        stack: &mut Vec<PackageKey>,
        packages: &mut FxHashMap<PackageKey, PackageInfo>,
        errors: &mut Vec<miette::Report>,
        this_pkg: &Dependency,
    ) {
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

            let dep_result = self
                .read_manifest_and_sources_cached(global_cache, dep_key.clone(), &dependency)
                .await;

            match dep_result {
                Ok((_, pkg)) => {
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
                    // TODO: do we ever end up processing the same package twice, and is that a big deal?
                    packages.insert(dep_key.clone(), pkg);
                }
                Err(e) => {
                    errors.push(e);
                }
            };

            // TODO: absolute paths in manifests
            // TODO: os-specific slashes in manifests
        }

        stack.pop();
    }

    async fn load_project_with_deps(
        &self,
        directory: &Path,
        global_cache: Option<&RefCell<PackageCache>>,
    ) -> miette::Result<ProgramConfig> {
        let manifest = self.parse_manifest_in_dir(directory).await?;

        let mut errors = vec![];
        let mut packages = FxHashMap::default();
        let mut stack = vec![];

        let root_dep = Dependency::Path {
            path: directory.to_string_lossy().into(),
        };

        let result = self.read_manifest_and_sources(&root_dep).await;
        let root = match result {
            Ok(pkg) => Some(pkg.1),
            Err(e) => {
                errors.push(e);
                None
            }
        };

        match root {
            None => Err(miette::ErrReport::msg(format!(
                "Failed to load root package : {}",
                errors
                    .into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            ))),
            Some(root) => {
                self.collect_deps(
                    key_for_dependency_definition(&root_dep),
                    &root,
                    global_cache.unwrap_or(&RefCell::new(FxHashMap::default())),
                    &mut stack,
                    &mut packages,
                    &mut errors,
                    &root_dep,
                )
                .await;
                Ok(ProgramConfig {
                    package_graph_sources: PackageGraphSources { root, packages },
                    lints: manifest.lints,
                    errors,
                    target_profile: "unrestricted".into(), // TODO(alex) where should we get the
                                                           // profile from? also maybe
                                                           // TODO(minestarks)
                })
            }
        }
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

    fn fetch_github(
        &self,
        owner: &str,
        repo: &str,
        r#ref: &str,
        path: &str,
    ) -> miette::Result<Arc<str>>;

    fn load_project_with_deps(
        &self,
        directory: &Path,
        global_cache: Option<&RefCell<PackageCache>>,
    ) -> miette::Result<ProgramConfig> {
        // rather than rewriting all the async code in the project loader,
        // we're calling the async implementation here, doing some tricks to make it run
        // synchronously

        let fs = ToFileSystemAsync { fs: self };

        // This is a bit risky. It will fail at runtime if there are *any* await
        // points in the async code. Right now, we know that will never be the case
        // because we just passed in our synchronous FS functions to the project loader.
        // But what if someone unwittingly sneaks anpther await point into the async implementation
        // in the future?
        FutureExt::now_or_never(fs.load_project_with_deps(directory, global_cache))
            .expect("fun should be a function that returns immediately")
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
pub type PackageCache = FxHashMap<PackageKey, Result<(Manifest, PackageInfo), String>>;

#[derive(Clone, Debug)]
pub struct PackageInfo {
    pub sources: Vec<(Arc<str>, Arc<str>)>,
    pub language_features: LanguageFeatures,
    pub dependencies: FxHashMap<PackageAlias, PackageKey>,
}

#[derive(Clone, Debug)]
pub struct PackageGraphSources {
    pub root: PackageInfo,
    pub packages: FxHashMap<PackageKey, PackageInfo>,
}

impl PackageGraphSources {
    #[must_use]
    pub fn with_no_dependencies(
        sources: Vec<(Arc<str>, Arc<str>)>,
        language_features: LanguageFeatures,
    ) -> Self {
        Self {
            root: PackageInfo {
                sources,
                language_features,
                dependencies: FxHashMap::default(),
            },
            packages: FxHashMap::default(),
        }
    }
}

pub struct ProgramConfig {
    pub package_graph_sources: PackageGraphSources,
    pub lints: Vec<LintConfig>,
    pub errors: Vec<miette::Report>,
    pub target_profile: String,
}

impl ProgramConfig {
    /// Given a source map and profile, create a default program config which
    /// has no dependencies.
    /// Useful for testing and single-file scenarios.
    #[must_use]
    pub fn with_no_dependencies(
        sources: Vec<(Arc<str>, Arc<str>)>,
        target_profile: String,
    ) -> Self {
        Self {
            package_graph_sources: PackageGraphSources::with_no_dependencies(
                sources,
                LanguageFeatures::default(),
            ),
            lints: Vec::default(),
            errors: Vec::default(),
            target_profile,
        }
    }
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

        // if sorted_keys.len() != self.packages.len() {
        //     return Err(DependencyCycle);
        // }

        let mut sorted_packages = self.packages.into_iter().collect::<Vec<_>>();
        sorted_packages.sort_by_key(|(a_key, _pkg)| {
            sorted_keys
                .iter()
                .position(|key| key.as_str() == &**a_key)
                .unwrap_or_else(|| panic!("package {a_key} should be in sorted keys list"))
        });

        log::info!("build plan: {:#?}", sorted_keys);

        Ok((sorted_packages, self.root))
    }
}

/// Turns a `FileSystem` into a `FileSystemAsync`
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
