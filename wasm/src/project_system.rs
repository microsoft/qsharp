// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{diagnostic::project_errors_into_qsharp_errors, serializable_type};
use async_trait::async_trait;
use miette::Report;
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
    fetchGithub(owner: string, repo: string, ref: string, path: string): Promise<string>;
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
    #[wasm_bindgen(method, structural, catch)]
    async fn readFile(this: &ProjectHost, uri: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, structural)]
    async fn listDirectory(this: &ProjectHost, uri: &str) -> JsValue;

    #[wasm_bindgen(method, structural)]
    async fn resolvePath(this: &ProjectHost, base: &str, path: &str) -> JsValue;

    #[wasm_bindgen(method, structural, catch)]
    async fn fetchGithub(
        this: &ProjectHost,
        owner: &str,
        repo: &str,
        r#ref: &str,
        path: &str,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, structural)]
    async fn findManifestDirectory(this: &ProjectHost, docUri: &str) -> JsValue;

    /// Alias for an array of [sourceName, sourceContents] tuples
    #[wasm_bindgen(typescript_type = "[string, string][]")]
    pub type ProjectSources;

    #[wasm_bindgen(typescript_type = "IProgramConfig")]
    pub type ProgramConfig;

    // Getters for IProgramConfig
    #[wasm_bindgen(method, getter, structural)]
    fn packageGraphSources(this: &ProgramConfig) -> IPackageGraphSources;

    #[wasm_bindgen(method, getter, structural)]
    fn profile(this: &ProgramConfig) -> String;
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
    async fn read_file(&self, uri: &str) -> miette::Result<(Arc<str>, Arc<str>)> {
        let name = Arc::from(uri);

        match self.readFile(uri).await {
            Ok(val) => Ok((name, val.as_string().unwrap_or_default().into())),

            Err(js_val) => {
                let err: js_sys::Error = js_val
                    .dyn_into()
                    .expect("exception should be an error type");
                let message = err
                    .message()
                    .as_string()
                    .expect("error message should be a string");
                Err(Report::msg(message))
            }
        }
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
    ) -> miette::Result<Arc<str>> {
        match self.fetchGithub(owner, repo, r#ref, path).await {
            Ok(js_val) => Ok(js_val
                .as_string()
                .expect("fetchGithub should return a string or throw")
                .into()),
            Err(js_val) => {
                let err: js_sys::Error = js_val
                    .dyn_into()
                    .expect("exception should be an error type");
                let message = err
                    .message()
                    .as_string()
                    .expect("error message should be a string");
                Err(Report::msg(message))
            }
        }
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
    ) -> Result<IProjectConfig, String> {
        let package_cache = PACKAGE_CACHE.with(Clone::clone);

        let dir_path = std::path::Path::new(&directory);
        let project_config = match self.0.load_project(dir_path, Some(&package_cache)).await {
            Ok(loaded_project) => loaded_project,
            Err(errs) => return Err(project_errors_into_qsharp_errors_json(&directory, &errs)),
        };

        // Will return error if project has errors
        project_config.try_into()
    }
}

fn project_errors_into_qsharp_errors_json(
    project_dir: &str,
    errs: &[qsc_project::Error],
) -> String {
    serde_json::to_string(&project_errors_into_qsharp_errors(project_dir, errs))
        .expect("serializing errors to json should succeed")
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
            package_type: value.package_type.map(|x| {
                match x {
                    qsc_project::PackageType::Exe => "exe",
                    qsc_project::PackageType::Lib => "lib",
                }
                .into()
            }),
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
                .map(|(pkg_key, pkg_info)| (pkg_key.to_string(), pkg_info.into()))
                .collect(),
        }
    }
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
        pub package_type: Option<String>,
    },
    r#"
    
    export interface IPackageInfo {
        sources: [string, string][];
        languageFeatures: string[];
        dependencies: Record<string,string>;
        packageType?: "exe" | "lib";
    }"#
}

impl TryFrom<qsc_project::Project> for IProjectConfig {
    type Error = String;

    fn try_from(value: qsc_project::Project) -> Result<Self, Self::Error> {
        if !value.errors.is_empty() {
            return Err(project_errors_into_qsharp_errors_json(
                &value.path,
                &value.errors,
            ));
        }
        let project_config = ProjectConfig {
            project_name: value.name.to_string(),
            project_uri: value.path.to_string(),
            lints: value.lints,
            package_graph_sources: value.package_graph_sources.into(),
        };
        Ok(project_config.into())
    }
}

serializable_type! {
    ProjectConfig,
    {
        pub project_name: String,
        pub project_uri: String,
        pub package_graph_sources: PackageGraphSources,
        pub lints: Vec<LintConfig>,
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
            package_type: value.package_type.map(|x| match x.as_str() {
                "exe" => qsc_project::PackageType::Exe,
                "lib" => qsc_project::PackageType::Lib,
                _ => unreachable!(
                    "expected one of 'exe' or 'lib' -- should be guaranteed by TS types"
                ),
            }),
        }
    }
}

/// This returns the common parameters that the compiler/interpreter uses
#[allow(clippy::type_complexity)]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn into_qsc_args(
    program: ProgramConfig,
    entry: Option<String>,
) -> Result<
    (
        qsc::SourceMap,
        qsc::TargetCapabilityFlags,
        qsc::LanguageFeatures,
        qsc::PackageStore,
        Vec<(qsc::hir::PackageId, Option<Arc<str>>)>,
    ),
    Vec<qsc::compile::Error>,
> {
    let capabilities = qsc::target::Profile::from_str(&program.profile())
        .unwrap_or_else(|()| panic!("Invalid target : {}", program.profile()))
        .into();

    let pkg_graph: PackageGraphSources = program.packageGraphSources().into();
    let pkg_graph: qsc_project::PackageGraphSources = pkg_graph.into();

    // this function call builds all dependencies as a part of preparing the package store
    // for building the user code.
    let buildable_program = BuildableProgram::new(capabilities, pkg_graph);

    if !buildable_program.dependency_errors.is_empty() {
        return Err(buildable_program.dependency_errors);
    }

    let BuildableProgram {
        store,
        user_code,
        user_code_dependencies,
        ..
    } = buildable_program;

    let source_map = qsc::SourceMap::new(user_code.sources, entry.map(std::convert::Into::into));
    let language_features = qsc::LanguageFeatures::from_iter(user_code.language_features);

    Ok((
        source_map,
        capabilities,
        language_features,
        store,
        user_code_dependencies,
    ))
}
