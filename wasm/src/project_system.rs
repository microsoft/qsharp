// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::serializable_type;
use async_trait::async_trait;
use qsc::linter::LintConfig;
use qsc_project::{EntryType, FileSystemAsync, JSFileEntry, JSProjectHost};
use serde::{Deserialize, Serialize};
use std::{iter::FromIterator, path::PathBuf, str::FromStr, sync::Arc};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const IPROJECT_HOST: &'static str = r#"
export interface IProjectHost {
    readFile(uri: string): Promise<string | null>;
    listDirectory(uri: string): Promise<[string, number][]>;
    resolvePath(base: string, path: string): Promise<string | null>;
    findManifestDirectory(docUri: string): Promise<string | null>;
}

/**
 * Copy of the ProgramConfig type defined in compiler.ts,
 * but with all the properties required and filled in with defaults where necessary.
 */
export interface IProgramConfig {
    sources: [string, string][];
    languageFeatures: string[];
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

    #[allow(clippy::from_iter_instead_of_collect)]
    pub async fn load_project(&self, directory: String) -> Result<IProjectConfig, JsValue> {
        match self.0.load_project_in_dir(&PathBuf::from(directory)).await {
            Ok(p) => Ok(p.into()),
            Err(e) => Err(JsError::new(&format!("{e}")).into()),
        }
    }
}

impl From<ProjectSources> for Vec<(String, String)> {
    fn from(sources: ProjectSources) -> Self {
        serde_wasm_bindgen::from_value(sources.into())
            .expect("sources object should be an array of string pairs")
    }
}

impl From<qsc_project::Project> for IProjectConfig {
    fn from(value: qsc_project::Project) -> Self {
        let project_config = ProjectConfig {
            project_name: value.name.to_string(),
            project_uri: value.manifest_path.to_string(),
            sources: value
                .sources
                .into_iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect(),
            language_features: value.manifest.language_features,
            lints: value.manifest.lints,
        };
        project_config.into()
    }
}

serializable_type! {
    ProjectConfig,
    {
        pub project_name: String,
        pub project_uri: String,
        pub sources: Vec<(String, String)>,
        pub language_features: Vec<String>,
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
        /**
         * [sourceName, sourceContents] tuples
         *
         * In the future, for projects with dependencies, this will contain the
         * full dependency graph and sources for all the packages referenced by the project.
         */
        sources: [string, string][];
        languageFeatures: string[];
        lints: {
          lint: string;
          level: string;
        }[];
    }"#,
    IProjectConfig
}

/// This returns the common parameters that the compiler/interpreter uses
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn into_qsc_args(
    program: ProgramConfig,
    entry: Option<String>,
) -> (
    qsc::SourceMap,
    qsc::TargetCapabilityFlags,
    qsc::LanguageFeatures,
) {
    let capabilities = qsc::target::Profile::from_str(&program.profile())
        .unwrap_or_else(|()| panic!("Invalid target : {}", program.profile()))
        .into();
    let sources: Vec<(String, String)> = program.sources().into();

    let source_map = qsc::SourceMap::new(
        sources.into_iter().map(|(a, b)| (a.into(), b.into())),
        entry.map(Into::into),
    );

    let language_features = qsc::LanguageFeatures::from_iter(program.languageFeatures());

    (source_map, capabilities, language_features)
}
