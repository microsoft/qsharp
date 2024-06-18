// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::serializable_type;
use async_trait::async_trait;
use js_sys::JsString;
use qsc::linter::LintConfig;
use qsc_packages::BuildableProgram;
use qsc_project::{
    EntryType, JSFileEntry, Manifest, ManifestDescriptor, PackageCache, ProjectSystemCallbacks,
};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, iter::FromIterator, path::PathBuf, rc::Rc, str::FromStr, sync::Arc};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "(uri: string) => Promise<IProjectConfig | null>")]
    pub type LoadProjectCallback;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "(uri: string) => Promise<string | null>")]
    pub type ReadFileCallback;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "(uri: string) => Promise<[string, number][]>")]
    pub type ListDirectoryCallback;
}

#[wasm_bindgen]
extern "C" {
    // TODO: communicate failure somehow
    #[wasm_bindgen(typescript_type = "(base: string, path: string) => Promise<string>")]
    pub type ResolvePathCallback;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(
        // TODO: EWWWW
        typescript_type = "(githubRef: [string, string, string, string]) => Promise<string | null>"
    )]
    pub type FetchGithubCallback;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "{ manifestDirectory: string }")]
    pub type ManifestDescriptorObject;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(
        typescript_type = "(uri: string) => Promise<{ manifestDirectory: string }| null>"
    )]
    pub type GetManifestCallback;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "[string, string][]")]
    pub type ProjectSources;
}

impl From<ManifestDescriptorObject> for Option<ManifestDescriptor> {
    fn from(value: ManifestDescriptorObject) -> Self {
        get_manifest_transformer(value.obj, String::default())
    }
}

/// This macro produces a function that calls an async JS function, awaits it, and then applies a function to the resulting value.
/// Ultimately, it returns a function that accepts a String and returns a Rust future that represents a JS Promise. Awaiting that
/// Rust future will await the resolution of the promise.
/// The name of this macro should be read like "convert a JS promise into an async rust function with this mapping function"
macro_rules! into_async_rust_fn_with {
    ($js_async_fn: ident, $map_result: expr) => {{
        use crate::project_system::{map_js_promise, to_js_function};
        use std::future::Future;
        use std::pin::Pin;
        use wasm_bindgen::JsValue;
        use wasm_bindgen_futures::JsFuture;

        let $js_async_fn = to_js_function($js_async_fn, stringify!($js_async_fn));

        let $js_async_fn = move |input: String| {
            let path = JsValue::from_str(&input);
            let res: js_sys::Promise = $js_async_fn
                .call1(&JsValue::NULL, &path)
                .expect("callback should succeed")
                .into();

            let res: JsFuture = res.into();

            Box::pin(map_js_promise(res, move |x| {
                $map_result(x.into(), input.clone())
            })) as Pin<Box<dyn Future<Output = _> + 'static>>
        };
        $js_async_fn
    }};
}

macro_rules! into_async_rust_fn_with_2 {
    ($js_async_fn: ident, $map_result: expr) => {{
        use crate::project_system::{map_js_promise, to_js_function};
        use std::future::Future;
        use std::pin::Pin;
        use wasm_bindgen::JsValue;
        use wasm_bindgen_futures::JsFuture;

        let $js_async_fn = to_js_function($js_async_fn, stringify!($js_async_fn));

        let $js_async_fn = move |(input1, input2): (String, String)| {
            let path1 = JsValue::from_str(&input1);
            let path2 = JsValue::from_str(&input2);
            let res: js_sys::Promise = $js_async_fn
                .call2(&JsValue::NULL, &path1, &path2)
                .expect("callback should succeed")
                .into();

            let res: JsFuture = res.into();

            Box::pin(map_js_promise(res, move |x| {
                $map_result(x.into(), String::new())
            })) as Pin<Box<dyn Future<Output = _> + 'static>>
        };
        $js_async_fn
    }};
}

macro_rules! into_async_rust_fn_with_4 {
    ($js_async_fn: ident, $map_result: expr) => {{
        use crate::project_system::{map_js_promise, to_js_function};
        use std::future::Future;
        use std::pin::Pin;
        use wasm_bindgen::JsValue;
        use wasm_bindgen_futures::JsFuture;

        let $js_async_fn = to_js_function($js_async_fn, stringify!($js_async_fn));

        let $js_async_fn =
            move |(input1, input2, input3, input4): (String, String, String, String)| {
                let path1 = JsValue::from_str(&input1);
                let path2 = JsValue::from_str(&input2);
                let path3 = JsValue::from_str(&input3);
                let path4 = JsValue::from_str(&input4);
                let tuple = js_sys::Array::of4(&path1, &path2, &path3, &path4);
                let res: js_sys::Promise = $js_async_fn
                    .call1(&JsValue::NULL, &tuple)
                    .expect("callback should succeed")
                    .into();

                let res: JsFuture = res.into();

                Box::pin(map_js_promise(res, move |x| {
                    $map_result(x.into(), String::new())
                })) as Pin<Box<dyn Future<Output = _> + 'static>>
            };
        $js_async_fn
    }};
}

pub(crate) async fn map_js_promise<F, T>(res: JsFuture, func: F) -> T
where
    F: Fn(JsValue) -> T,
{
    let res = res.await.expect("js future shouldn't throw an exception");
    func(res)
}

pub(crate) fn to_js_function(val: JsValue, help_text_panic: &'static str) -> js_sys::Function {
    let js_ty = val.js_typeof();
    assert!(
        val.is_function(),
        "expected a valid JS function ({help_text_panic}), received {js_ty:?}"
    );
    Into::<js_sys::Function>::into(val)
}
pub(crate) use into_async_rust_fn_with;

/// Given a [`JsValue`] representing the result of a call to a `list_directory` function,
/// and an unused `String` parameter for API compatibility, assert that `js_val`
/// matches our expected return type of `[string, number][]` and transform that
/// JS data into a [Vec<JSFileEntry>]
pub(crate) fn list_directory_transformer(js_val: JsValue, _: String) -> Vec<JSFileEntry> {
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

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn resolve_path_transformer(js_val: JsValue, _: String) -> Arc<str> {
    js_val
        .as_string()
        .expect("expected string from resolve_path")
        .into()
}

/// Given a [`JsValue`] representing the result of a call to a read file function,
/// and a `String` representing the path that was originally passed in as an
/// argument to that function, assert that `js_val` matches our expected return type of
/// `string` and transform it into a tuple representing the path and the file contents.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn read_file_transformer(
    js_val: JsValue,
    path_buf_string: String,
) -> (Arc<str>, Arc<str>) {
    match js_val.as_string() {
        Some(res) => (Arc::from(path_buf_string), Arc::from(res)),
        // this can happen if the document is completely empty
        None if js_val.is_null() => (Arc::from(path_buf_string), Arc::from("")),
        None => unreachable!("Expected string from JS callback, received {js_val:?}"),
    }
}
/// Given a [`JsValue`] representing the result of a call to a `get_manifest` function,
/// and an unused `String` parameter for API compatibility, assert that `js_val`
/// matches our expected return object shape  and transform it into a [`ManifestDescriptor`],
/// or `None`
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn get_manifest_transformer(js_val: JsValue, _: String) -> Option<ManifestDescriptor> {
    if js_val.is_null() {
        return None;
    }

    let manifest_dir = match js_sys::Reflect::get(&js_val, &JsValue::from_str("manifestDirectory"))
    {
        Ok(v) => v.as_string().unwrap_or_else(|| {
            panic!(
                "manifest callback returned {v:?}, but we expected a string representing its URI"
            )
        }),
        Err(_) => unreachable!("our typescript bindings should guarantee that an object with a manifestDirectory property is returned here"),
    };
    let language_features = match js_sys::Reflect::get(&js_val, &JsValue::from_str("languageFeatures"))
    {
        Ok(v) => match v.dyn_into::<js_sys::Array>()  {
            Ok(arr) => arr
                .into_iter()
                .map(|x| {
                    x.as_string().unwrap_or_else(|| {
                        panic!(
                            "manifest callback returned {x:?}, but we expected a string representing a language feature"
                        )
                    })
                }).collect::<Vec<_>>(),
                Err(_) => Vec::new(),
        },
        _ => Vec::new(),

    };

    let lints: Vec<LintConfig> = match js_sys::Reflect::get(&js_val, &JsValue::from_str("lints")) {
        Ok(v) => match v.dyn_into::<js_sys::Array>() {
            Ok(arr) => arr
                .into_iter()
                .filter_map(|x| serde_wasm_bindgen::from_value::<LintConfig>(x).ok())
                .collect::<Vec<_>>(),
            Err(_) => Vec::new(),
        },
        _ => Vec::new(),
    };

    log::trace!("found manifest at {manifest_dir:?}");

    let manifest_dir = PathBuf::from(manifest_dir);

    Some(ManifestDescriptor {
        manifest: Manifest {
            language_features,
            lints,
            author: Option::default(),
            license: Option::default(),
            dependencies: FxHashMap::default(),
            files: Vec::default(),
        },
        manifest_dir,
    })
}

/// a minimal implementation for interacting with async JS filesystem callbacks to
/// load project files
#[wasm_bindgen]
pub struct ProjectLoader(ProjectSystemCallbacks<'static>);

thread_local! { static PACKAGE_CACHE: Rc<RefCell<PackageCache>> = Rc::default(); }

#[wasm_bindgen]
impl ProjectLoader {
    #[wasm_bindgen(constructor)]
    pub fn new(
        read_file: ReadFileCallback,
        list_directory: ListDirectoryCallback,
        resolve_path: ResolvePathCallback,
        fetch_github: FetchGithubCallback,
    ) -> Self {
        let read_file = read_file.into();
        let read_file = into_async_rust_fn_with!(read_file, read_file_transformer);

        let list_directory = list_directory.into();
        let list_directory = into_async_rust_fn_with!(list_directory, list_directory_transformer);

        let resolve_path = resolve_path.into();
        let resolve_path = into_async_rust_fn_with_2!(resolve_path, resolve_path_transformer);

        let fetch_github = fetch_github.into();
        let fetch_github = into_async_rust_fn_with_4!(fetch_github, resolve_path_transformer);

        ProjectLoader(ProjectSystemCallbacks {
            read_file: Box::new(read_file),
            list_directory: Box::new(list_directory),
            resolve_path: Box::new(resolve_path),
            fetch_github: Box::new(fetch_github),
        })
    }

    pub async fn load_project_with_deps(&self, directory: String) -> IProjectConfig {
        let package_cache = PACKAGE_CACHE.with(Clone::clone);

        let project_config: ProjectConfig = qsc_project::FileSystemAsync::load_project_with_deps(
            self,
            std::path::Path::new(&directory),
            Some(&package_cache),
        )
        .await
        .map_or_else(
            |e| ProjectConfig {
                project_name: String::new(),
                project_uri: String::new(),
                package_graph_sources: PackageGraphSources {
                    root: PackageInfo {
                        sources: Vec::new(),
                        language_features: Vec::new(),
                        dependencies: FxHashMap::default(),
                    },
                    packages: FxHashMap::default(),
                },
                lints: Vec::new(),
                errors: vec![e.to_string()],
            },
            |proj| {
                ProjectConfig {
                    project_name: String::new(), // TODO: ew
                    project_uri: String::new(),  // TODO: ew
                    package_graph_sources: PackageGraphSources {
                        root: PackageInfo {
                            sources: proj
                                .package_graph_sources
                                .root
                                .sources
                                .into_iter()
                                .map(|(path, contents)| (path.to_string(), contents.to_string()))
                                .collect(),
                            language_features: proj
                                .package_graph_sources
                                .root
                                .language_features
                                .into_iter()
                                .collect(),
                            dependencies: proj
                                .package_graph_sources
                                .root
                                .dependencies
                                .into_iter()
                                .map(|(k, v)| (k.to_string(), v.to_string()))
                                .collect(),
                        },
                        packages: proj
                            .package_graph_sources
                            .packages
                            .into_iter()
                            .map(|(k, v)| {
                                (
                                    k.to_string(),
                                    PackageInfo {
                                        sources: v
                                            .sources
                                            .into_iter()
                                            .map(|(path, contents)| {
                                                (path.to_string(), contents.to_string())
                                            })
                                            .collect(),
                                        language_features: v
                                            .language_features
                                            .into_iter()
                                            .collect(),
                                        dependencies: v
                                            .dependencies
                                            .into_iter()
                                            .map(|(k, v)| (k.to_string(), v.to_string()))
                                            .collect(),
                                    },
                                )
                            })
                            .collect(),
                    },
                    lints: proj.lints.into_iter().map(Into::into).collect(),
                    errors: proj.errors.into_iter().map(|r| r.to_string()).collect(),
                }
            },
        );

        project_config.into()
    }

    pub async fn load_project(&self, manifest: ManifestDescriptorObject) -> ProjectSources {
        let manifest: Option<ManifestDescriptor> = manifest.into();
        match manifest {
            #[allow(clippy::from_iter_instead_of_collect)]
            Some(manifest) => {
                // TODO: we shouldn't need this anymore... later
                let res = qsc_project::FileSystemAsync::load_project_sources(self, &manifest)
                    .await
                    .map_or_else(
                        |_| js_sys::Array::new(),
                        |proj| {
                            proj.sources
                                .into_iter()
                                .map(|(path, contents)| {
                                    js_sys::Array::from_iter::<std::slice::Iter<'_, JsString>>(
                                        [path.to_string().into(), contents.to_string().into()]
                                            .iter(),
                                    )
                                })
                                .collect::<_>()
                        },
                    );
                ProjectSources { obj: res.into() }
            }
            None => ProjectSources {
                obj: js_sys::Array::new().into(),
            },
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
        Ok((self.0.read_file)(path.to_string_lossy().to_string()).await)
    }

    async fn list_directory(&self, path: &std::path::Path) -> miette::Result<Vec<Self::Entry>> {
        Ok((self.0.list_directory)(path.to_string_lossy().to_string()).await)
    }

    async fn resolve_path(
        &self,
        base: &std::path::Path,
        path: &std::path::Path,
    ) -> miette::Result<std::path::PathBuf> {
        Ok((self.0.resolve_path)((
            base.to_string_lossy().to_string(),
            path.to_string_lossy().to_string(),
        ))
        .await
        .to_string()
        .into())
    }

    async fn fetch_github(
        &self,
        owner: &str,
        repo: &str,
        r#ref: &str,
        path: &str,
    ) -> miette::Result<std::sync::Arc<str>> {
        Ok((self.0.fetch_github)((
            owner.to_string(),
            repo.to_string(),
            r#ref.to_string(),
            path.to_string(),
        ))
        .await)
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
        pub errors: Vec<String>,
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

impl From<ProgramConfig> for qsc_project::ProgramConfig {
    fn from(value: ProgramConfig) -> Self {
        Self {
            package_graph_sources: value.package_graph_sources.into(),
            target_profile: value.target_profile,
            // TODO(alex) inherit lints
            lints: Vec::default(),
            // TODO(alex) accumulate errors from dependencies here
            errors: Vec::default(),
        }
    }
}

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
            language_features: value.language_features,
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
    } = qsc_packages::BuildableProgram::new(program.into());
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

/// This returns the common parameters that the language service needs from the manifest
pub(crate) fn into_project_args(project: ProjectConfig) -> qsls::LoadProjectResultInner {
    let (sources, language_features) = into_package_graph_args(project.package_graph_sources);

    (
        project.project_name.into(),
        sources,
        language_features,
        project.lints,
    )
}

/// This is the bit that's common to both the compiler and the language service
#[allow(clippy::type_complexity)]
fn into_package_graph_args(
    package_graph: PackageGraphSources,
) -> (Vec<(Arc<str>, Arc<str>)>, qsc::LanguageFeatures) {
    let language_features = qsc::LanguageFeatures::from_iter(package_graph.root.language_features);
    let mut sources = package_graph.root.sources;

    // Concatenate all the dependencies into the sources
    // TODO: Properly convert these into something that the compiler & language service can use
    for (_, other_package) in package_graph.packages {
        sources.extend(other_package.sources);
    }

    (
        sources
            .into_iter()
            .map(|(name, contents)| (name.into(), contents.into()))
            .collect(),
        language_features,
    )
}
