// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    diagnostic::{IQSharpError, QSharpError, VSDiagnostic},
    line_column::{Position, Range},
    serializable_type,
};
use async_trait::async_trait;
use qsc::{linter::LintConfig, packages::BuildableProgram, LanguageFeatures};
use qsc_project::{
    EntryType, FileSystemAsync, JSFileEntry, PackageCache, ProjectHostTrait, ProjectSystemCallbacks,
};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, iter::FromIterator, rc::Rc, str::FromStr, sync::Arc};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const IPROJECT_HOST: &'static str = r#"
interface IProjectHost {
    readFile(uri: string): Promise<string | null>;
    listDirectory(uri: string): Promise<[string, number][]>;
    resolvePath(base: string, path: string): Promise<string | null>;
    fetchGithub(owner: string, repo: string, ref: string, path: string): Promise<string | null>;
    findManifestDirectory(docUri: string): Promise<string | null>;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IProjectHost")]
    pub type ProjectHost;

    #[wasm_bindgen(method, structural)]
    async fn readFile(this: &ProjectHost, uri: &str) -> JsValue;

    #[wasm_bindgen(method, structural)]
    async fn listDirectory(this: &ProjectHost, uri: &str) -> JsValue;

    #[wasm_bindgen(method, structural)]
    async fn resolvePath(this: &ProjectHost, base: &str, path: &str) -> JsValue;

    #[wasm_bindgen(method, structural)]
    async fn fetchGithub(
        this: &ProjectHost,
        owner: &str,
        repo: &str,
        r#ref: &str,
        path: &str,
    ) -> JsValue;

    #[wasm_bindgen(method, structural)]
    async fn findManifestDirectory(this: &ProjectHost, docUri: &str) -> JsValue;
}

pub(crate) fn to_js_function(val: JsValue, help_text_panic: &'static str) -> js_sys::Function {
    let js_ty = val.js_typeof();
    assert!(
        val.is_function(),
        "expected a valid JS function ({help_text_panic}), received {js_ty:?}"
    );
    Into::<js_sys::Function>::into(val)
}

/// a minimal implementation for interacting with async JS filesystem callbacks to
/// load project files
#[wasm_bindgen]
pub struct ProjectLoader(ProjectSystemCallbacks);

thread_local! { static PACKAGE_CACHE: Rc<RefCell<PackageCache>> = Rc::default(); }

pub struct AnotherThing {
    pub host: ProjectHost,
}

// TODO: do this??
#[async_trait(?Send)]
impl ProjectHostTrait for ProjectHost {
    async fn read_file(&self, _uri: &str) -> (Arc<str>, Arc<str>) {
        todo!()
    }

    async fn list_directory(&self, _uri: &str) -> Vec<JSFileEntry> {
        todo!()
    }

    async fn resolve_path(&self, _base: &str, _path: &str) -> Option<Arc<str>> {
        todo!()
    }

    async fn fetch_github(
        &self,
        _owner: &str,
        _repo: &str,
        _ref: &str,
        _path: &str,
    ) -> Option<Arc<str>> {
        todo!()
    }

    async fn find_manifest_directory(&self, _doc_uri: &str) -> Option<Arc<str>> {
        todo!()
    }
}

#[async_trait(?Send)]
impl ProjectHostTrait for AnotherThing {
    async fn read_file(&self, uri: &str) -> (Arc<str>, Arc<str>) {
        let name = Arc::from(uri);

        let val = self.host.readFile(uri).await;
        (name, val.as_string().unwrap_or_default().into())
    }

    async fn list_directory(&self, uri: &str) -> Vec<JSFileEntry> {
        let js_val = self.host.listDirectory(uri).await;
        match js_val.dyn_into::<js_sys::Array>() {
            Ok(arr) => arr
                .into_iter()
                .map(|x| {
                    x.dyn_into::<js_sys::Array>()
                        .expect("expected directory listing callback to return array of arrays")
                })
                .filter_map(|js_arr| {
                    let mut arr = js_arr.into_iter().take(2);
                    #[allow(clippy::cast_possible_truncation)]
                    match (
                        arr.next().expect("should be string").as_string(),
                        arr.next().expect("should be float").as_f64(),
                    ) {
                        (Some(a), Some(b)) => Some((a, b as i32)),
                        _ => None,
                    }
                })
                .map(|(name, ty)| JSFileEntry {
                    name,
                    r#type: match ty {
                        0 => EntryType::Unknown,
                        1 => EntryType::File,
                        2 => EntryType::Folder,
                        64 => EntryType::Symlink,
                        _ => unreachable!("expected one of vscode.FileType. Received {ty:?}"),
                    },
                })
                .collect::<Vec<_>>(),
                Err(e) => unreachable!("controlled callback should have returned an array -- our typescript bindings should guarantee this. {e:?}"),
        }
    }

    async fn resolve_path(&self, base: &str, path: &str) -> Option<Arc<str>> {
        let js_val = self.host.resolvePath(base, path).await;
        js_val.as_string().map(Into::into)
    }

    async fn fetch_github(
        &self,
        owner: &str,
        repo: &str,
        r#ref: &str,
        path: &str,
    ) -> Option<Arc<str>> {
        let js_val = self.host.fetchGithub(owner, repo, r#ref, path).await;
        js_val.as_string().map(Into::into)
    }

    async fn find_manifest_directory(&self, doc_uri: &str) -> Option<Arc<str>> {
        let js_val = self.host.findManifestDirectory(doc_uri).await;
        js_val.as_string().map(Into::into)
    }
}

#[wasm_bindgen]
impl ProjectLoader {
    #[wasm_bindgen(constructor)]
    pub fn new(project_host: ProjectHost) -> Self {
        ProjectLoader(ProjectSystemCallbacks {
            project_host: Box::new(AnotherThing { host: project_host }),
        })
    }

    pub async fn load_project_with_deps(
        &self,
        directory: String,
    ) -> Result<IProjectConfig, IQSharpError> {
        let package_cache = PACKAGE_CACHE.with(Clone::clone);

        let dir_path = std::path::Path::new(&directory);
        let project_config: ProjectConfig =
            match FileSystemAsync::load_project_with_deps(self, dir_path, Some(&package_cache))
                .await
            {
                Ok(project_config) => ProjectConfig {
                    project_name: dir_path
                        .file_name()
                        .map_or("Q# project".into(), |f| f.to_string_lossy().into()),
                    project_uri: project_config.compilation_uri.to_string(),
                    package_graph_sources: project_config.package_graph_sources.into(),
                    lints: project_config.lints.into_iter().map(Into::into).collect(),
                    errors: project_config
                        .errors
                        .into_iter()
                        .map(|r| r.to_string())
                        .collect(),
                },
                Err(e) => {
                    return Err(QSharpError {
                        document: directory,
                        diagnostic: VSDiagnostic {
                            range: Range {
                                start: Position {
                                    line: 0,
                                    character: 0,
                                },
                                end: Position {
                                    line: 0,
                                    character: 1,
                                },
                            },
                            message: e.to_string(),
                            severity: "error".into(),
                            code: Some("project.error".into()),
                            uri: None,
                            related: Vec::default(),
                        },
                        stack: None,
                    }
                    .into());
                }
            };

        Ok(project_config.into())
    }
}

impl From<qsc_project::PackageInfo> for PackageInfo {
    fn from(value: qsc_project::PackageInfo) -> Self {
        Self {
            sources: value
                .sources
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            language_features: value.language_features.into(),
            dependencies: value
                .dependencies
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}

impl From<qsc_project::PackageGraphSources> for PackageGraphSources {
    fn from(value: qsc_project::PackageGraphSources) -> Self {
        Self {
            root: value.root.into(),
            packages: value
                .packages
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.into()))
                .collect(),
        }
    }
}

#[async_trait(?Send)]
impl qsc_project::FileSystemAsync for ProjectLoader {
    type Entry = JSFileEntry;
    async fn read_file(
        &self,
        path: &std::path::Path,
    ) -> miette::Result<(std::sync::Arc<str>, std::sync::Arc<str>)> {
        FileSystemAsync::read_file(&self.0, path).await
    }

    async fn list_directory(&self, path: &std::path::Path) -> miette::Result<Vec<Self::Entry>> {
        FileSystemAsync::list_directory(&self.0, path).await
    }

    async fn resolve_path(
        &self,
        base: &std::path::Path,
        path: &std::path::Path,
    ) -> miette::Result<std::path::PathBuf> {
        FileSystemAsync::resolve_path(&self.0, base, path).await
    }

    async fn fetch_github(
        &self,
        owner: &str,
        repo: &str,
        r#ref: &str,
        path: &str,
    ) -> miette::Result<std::sync::Arc<str>> {
        FileSystemAsync::fetch_github(&self.0, owner, repo, r#ref, path).await
    }
}

serializable_type! {
    ProgramConfig,
    {
        pub package_graph_sources: PackageGraphSources,
        pub target_profile: String,
    },
    r#"export interface IProgramConfig {
        packageGraphSources: IPackageGraphSources;
        targetProfile: TargetProfile;
    }"#,
    IProgramConfig
}

serializable_type! {
    PackageGraphSources,
    {
        pub root: PackageInfo,
        pub packages: FxHashMap<PackageKey,PackageInfo>,
    },
    r#"
    export type PackageKey = string;

    export interface IPackageGraphSources {
        root: IPackageInfo;
        packages: Record<PackageKey,IPackageInfo>;
    }"#,
    IPackageGraphSources
}

serializable_type! {
    PackageInfo,
    {
        pub sources: Vec<(String, String)>,
        pub language_features: Vec<String>,
        pub dependencies: FxHashMap<PackageAlias,PackageKey>,
    },
    r#"export interface IPackageInfo {
        sources: [string, string][];
        languageFeatures: string[];
        dependencies: Record<string,string>;
    }"#
}

serializable_type! {
    ProjectConfig,
    {
        pub project_name: String,
        pub project_uri: String,
        pub package_graph_sources: PackageGraphSources,
        pub lints: Vec<LintConfig>, // TODO: I feel like this will barf at the serialization boundary if you have an invalid lint name
        pub errors: Vec<String>, // TODO: QSharpError
    },
    r#"export interface IProjectConfig {
        /**
         * Friendly name for the project, based on the name of the Q# document or project directory
         */
        projectName: string;
        /**
         * Uri for the qsharp.json or the root source file (single file projects)
         */
        projectUri: string;
        packageGraphSources: IPackageGraphSources;
        lints: {
          lint: string;
          level: string;
        }[];
        errors: string[];
    }"#,
    IProjectConfig
}

type PackageAlias = String;
type PackageKey = String;

impl From<PackageGraphSources> for qsc_project::PackageGraphSources {
    fn from(value: PackageGraphSources) -> Self {
        Self {
            root: value.root.into(),
            packages: value
                .packages
                .into_iter()
                .map(|(k, v)| (Arc::from(k), v.into()))
                .collect(),
        }
    }
}

impl From<PackageInfo> for qsc_project::PackageInfo {
    fn from(value: PackageInfo) -> Self {
        Self {
            sources: value
                .sources
                .into_iter()
                .map(|(k, v)| (Arc::from(k), Arc::from(v)))
                .collect(),
            language_features: LanguageFeatures::from_iter(value.language_features),
            dependencies: value
                .dependencies
                .into_iter()
                .map(|(k, v)| (Arc::from(k), Arc::from(v)))
                .collect(),
        }
    }
}

/// This returns the common parameters that the compiler/interpreter uses
#[allow(clippy::type_complexity)]
pub(crate) fn into_qsc_args(
    program: IProgramConfig,
    entry: Option<String>,
) -> (
    qsc::SourceMap,
    qsc::TargetCapabilityFlags,
    qsc::LanguageFeatures,
    qsc::PackageStore,
    Vec<(qsc::hir::PackageId, Option<Arc<str>>)>,
) {
    let program: ProgramConfig = program.into();
    let capabilities = qsc::target::Profile::from_str(&program.target_profile)
        .unwrap_or_else(|()| panic!("Invalid target : {}", program.target_profile))
        .into();
    let BuildableProgram {
        store,
        user_code,
        user_code_dependencies,
    } = BuildableProgram::new(
        &program.target_profile,
        program.package_graph_sources.into(),
    );
    // let package_graph = program.package_graph_sources;
    // let (sources, language_features) = into_package_graph_args(package_graph);
    let sources = user_code.sources;

    let source_map = qsc::SourceMap::new(sources, entry.map(std::convert::Into::into));
    let language_features = qsc::LanguageFeatures::from_iter(user_code.language_features);

    (
        source_map,
        capabilities,
        language_features,
        store,
        user_code_dependencies,
    )
}
