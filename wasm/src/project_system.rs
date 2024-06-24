// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    diagnostic::{IQSharpError, QSharpError, VSDiagnostic},
    line_column::{Position, Range},
    serializable_type,
};
use async_trait::async_trait;
use qsc::{linter::LintConfig, packages::BuildableProgram, LanguageFeatures};
use qsc_project::{EntryType, FileSystemAsync, JSFileEntry, JSProjectHost, PackageCache};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, iter::FromIterator, rc::Rc, str::FromStr, sync::Arc};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const IPROJECT_HOST: &'static str = r#"
export interface IProjectHost {
    readFile(uri: string): Promise<string | null>;
    listDirectory(uri: string): Promise<[string, number][]>;
    resolvePath(base: string, path: string): Promise<string | null>;
    fetchGithub(owner: string, repo: string, ref: string, path: string): Promise<string | null>;
    findManifestDirectory(docUri: string): Promise<string | null>;
}

/**
 * Copy of the ProgramConfig type defined in compiler.ts,
 * but with all the properties required and filled in with defaults where necessary.
 */
export interface IProgramConfig {
    packageGraphSources: IPackageGraphSources;
    profile: TargetProfile;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IProjectHost")]
    pub type ProjectHost;

    // Methods of `IProjectHost``, expected to be implemented JS-side
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

    /// Alias for an array of [sourceName, sourceContents] tuples
    #[wasm_bindgen(typescript_type = "[string, string][]")]
    pub type ProjectSources;

    #[wasm_bindgen(typescript_type = "IProgramConfig")]
    pub type ProgramConfig;

    // Getters for IProgramConfig
    #[wasm_bindgen(method, getter, structural)]
    fn sources(this: &ProgramConfig) -> ProjectSources;

    #[wasm_bindgen(method, getter, structural)]
    fn languageFeatures(this: &ProgramConfig) -> Vec<String>;

    #[wasm_bindgen(method, getter, structural)]
    fn profile(this: &ProgramConfig) -> String;

    #[wasm_bindgen(method, getter, structural)]
    fn packageGraphSources(this: &ProgramConfig) -> IPackageGraphSources;
}

pub(crate) fn to_js_function(val: JsValue, help_text_panic: &'static str) -> js_sys::Function {
    let js_ty = val.js_typeof();
    assert!(
        val.is_function(),
        "expected a valid JS function ({help_text_panic}), received {js_ty:?}"
    );
    Into::<js_sys::Function>::into(val)
}

thread_local! { static PACKAGE_CACHE: Rc<RefCell<PackageCache>> = Rc::default(); }

/// a minimal implementation for interacting with async JS filesystem callbacks to
/// load project files
#[wasm_bindgen]
pub struct ProjectLoader(ProjectHost);

#[async_trait(?Send)]
impl JSProjectHost for ProjectHost {
    async fn read_file(&self, uri: &str) -> (Arc<str>, Arc<str>) {
        let name = Arc::from(uri);

        let val = self.readFile(uri).await;
        (name, val.as_string().unwrap_or_default().into())
    }

    async fn list_directory(&self, uri: &str) -> Vec<JSFileEntry> {
        let js_val = self.listDirectory(uri).await;
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
        let js_val = self.resolvePath(base, path).await;
        js_val.as_string().map(Into::into)
    }

    async fn fetch_github(
        &self,
        owner: &str,
        repo: &str,
        r#ref: &str,
        path: &str,
    ) -> Option<Arc<str>> {
        let js_val = self.fetchGithub(owner, repo, r#ref, path).await;
        js_val.as_string().map(Into::into)
    }

    async fn find_manifest_directory(&self, doc_uri: &str) -> Option<Arc<str>> {
        let js_val = self.findManifestDirectory(doc_uri).await;
        js_val.as_string().map(Into::into)
    }
}

#[wasm_bindgen]
impl ProjectLoader {
    #[wasm_bindgen(constructor)]
    pub fn new(project_host: ProjectHost) -> Self {
        ProjectLoader(project_host)
    }

    pub async fn load_project_with_deps(
        &self,
        directory: String,
    ) -> Result<IProjectConfig, IQSharpError> {
        let package_cache = PACKAGE_CACHE.with(Clone::clone);

        let dir_path = std::path::Path::new(&directory);
        let project_config: IProjectConfig = match self
            .0
            .load_project_with_deps(dir_path, Some(&package_cache))
            .await
        {
            Ok(loaded_project) => loaded_project.into(),
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

        Ok(project_config)
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

    // #[allow(clippy::from_iter_instead_of_collect)]
    // pub async fn load_project(&self, directory: String) -> Result<IProjectConfig, JsValue> {
    //     match self.0.load_project_in_dir(&PathBuf::from(directory)).await {
    //         Ok(p) => Ok(p.into()),
    //         Err(e) => Err(JsError::new(&format!("{e}")).into()),
    //     }
    // }
}

impl From<qsc_project::PackageGraphSources> for PackageGraphSources {
    fn from(value: qsc_project::PackageGraphSources) -> Self {
        Self {
            root: value.root.into(),
            packages: value
                .packages
                .into_iter()
                .map(|(pkg_key, pkg_info)| (pkg_key.to_string(), pkg_info.into()))
                .collect(),
        }
    }
}

// serializable_type! {
//     ProgramConfig,
//     {
//         pub package_graph_sources: PackageGraphSources,
//         pub target_profile: String,
//     },
//     r#"export interface IProgramConfig {
//         packageGraphSources: IPackageGraphSources;
//         targetProfile: TargetProfile;
//     }"#,
//     IProgramConfig
// }

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

impl From<ProjectSources> for Vec<(String, String)> {
    fn from(sources: ProjectSources) -> Self {
        serde_wasm_bindgen::from_value(sources.into())
            .expect("sources object should be an array of string pairs")
    }
}

impl From<qsc_project::LoadedProject> for IProjectConfig {
    fn from(value: qsc_project::LoadedProject) -> Self {
        let project_config = ProjectConfig {
            project_name: value.name.to_string(),
            project_uri: value.manifest_path.to_string(),
            lints: value.lints,
            package_graph_sources: value.package_graph_sources.into(),
            errors: value.errors.into_iter().map(|e| e.to_string()).collect(), // TODO: proper diagnostics
        };
        project_config.into()
    }
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
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn into_qsc_args(
    program: ProgramConfig,
    entry: Option<String>,
) -> (
    qsc::SourceMap,
    qsc::TargetCapabilityFlags,
    qsc::LanguageFeatures,
    qsc::PackageStore,
    Vec<(qsc::hir::PackageId, Option<Arc<str>>)>,
) {
    let capabilities = qsc::target::Profile::from_str(&program.profile())
        .unwrap_or_else(|()| panic!("Invalid target : {}", program.profile()))
        .into();
    let package_graph_sources: PackageGraphSources = program.packageGraphSources().into();

    let BuildableProgram {
        store,
        user_code,
        user_code_dependencies,
    } = BuildableProgram::new(&program.profile(), package_graph_sources.into());
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
