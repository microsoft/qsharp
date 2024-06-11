// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::serializable_type;
use async_trait::async_trait;
use js_sys::JsString;
use qsc::{linter::LintConfig, PackageStore};
use qsc_project::{EntryType, JSFileEntry, Manifest, ManifestDescriptor, ProjectSystemCallbacks};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::{iter::FromIterator, path::PathBuf, str::FromStr, sync::Arc};
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
        },
        manifest_dir,
    })
}

/// a minimal implementation for interacting with async JS filesystem callbacks to
/// load project files
#[wasm_bindgen]
pub struct ProjectLoader(ProjectSystemCallbacks<'static>);

#[wasm_bindgen]
impl ProjectLoader {
    #[wasm_bindgen(constructor)]
    pub fn new(
        read_file: ReadFileCallback,
        list_directory: ListDirectoryCallback,
        get_manifest: GetManifestCallback,
    ) -> Self {
        let read_file = read_file.into();
        let read_file = into_async_rust_fn_with!(read_file, read_file_transformer);

        let list_directory = list_directory.into();
        let list_directory = into_async_rust_fn_with!(list_directory, list_directory_transformer);

        let get_manifest: JsValue = get_manifest.into();
        let get_manifest = into_async_rust_fn_with!(get_manifest, get_manifest_transformer);
        ProjectLoader(ProjectSystemCallbacks {
            read_file: Box::new(read_file),
            list_directory: Box::new(list_directory),
            get_manifest: Box::new(get_manifest),
        })
    }

    pub async fn load_project(&self, manifest: ManifestDescriptorObject) -> ProjectSources {
        let manifest: Option<ManifestDescriptor> = manifest.into();
        match manifest {
            #[allow(clippy::from_iter_instead_of_collect)]
            Some(manifest) => {
                let res = qsc_project::FileSystemAsync::load_project(self, &manifest)
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
        // TODO: Might change my mind later and make this a hashmap, depending on if/how we do lookups.
        // Vec isn't necessary if ordering is going to be done by the lower layers.
        pub packages: Vec<PackageInfo>,
    },
    r#"export interface IPackageGraphSources {
        root: IPackageInfo;
        packages: IPackageInfo[];
    }"#,
    IPackageGraphSources
}

serializable_type! {
    PackageInfo,
    {
        pub key: PackageKey,
        pub sources: Vec<(String, String)>,
        pub language_features: Vec<String>,
        pub dependencies: FxHashMap<PackageAlias,PackageKey>,
    },
    r#"export interface IPackageInfo {
        key: string;
        sources: [string, string][];
        languageFeatures: string[];
        dependencies: Record<string,string>; // or Map?
    }"#
}

serializable_type! {
    ProjectConfig,
    {
        pub project_name: String,
        pub project_uri: String,
        pub package_graph_sources: PackageGraphSources,
        pub lints: Vec<LintConfig>, // TODO: I feel like this will barf at the serialization boundary if you have an invalid lint name
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
    }"#,
    IProjectConfig
}

type PackageAlias = String;
type PackageKey = String;

/// This returns the common parameters that the compiler/interpreter uses
/// compiles all dependency crates up until the user code
pub(crate) fn into_qsc_args(
    program: IProgramConfig,
    entry: Option<String>,
) -> (
    qsc::SourceMap,
    qsc::TargetCapabilityFlags,
    qsc::LanguageFeatures,
    qsc::PackageStore,
) {
    let program: ProgramConfig = program.into();
    let capabilities = qsc::target::Profile::from_str(&program.target_profile)
        .unwrap_or_else(|()| panic!("Invalid target : {}", program.target_profile))
        .into();
    let package_graph = program.package_graph_sources;
    let (ordered_packages, user_code) =
        into_package_graph_args(package_graph).expect("TODO handle this err: dependency cycle");

    let mut package_store = qsc::PackageStore::new(qsc::compile::core());
    let mut canonical_package_identifier_to_package_id_mapping = FxHashMap::default();

    for package_to_compile in ordered_packages {
        // if this is the first package in the order, it should have zero dependencies
        if package_store.is_empty() {
            assert!(package_to_compile.dependencies.is_empty())
        }
        let sources: Vec<(Arc<str>, Arc<str>)> = package_to_compile
            .sources
            .into_iter()
            .map(|(path, contents)| (path.into(), contents.into()))
            .collect::<Vec<_>>();
        let source_map = qsc::SourceMap::new(sources, None);
        let dependencies = package_to_compile
            .dependencies
            .iter()
            .map(|(alias, key)| {
                (
                    alias.clone(),
                    canonical_package_identifier_to_package_id_mapping
                        .get(key)
                        .copied()
                        .expect("TODO handle this err: missing package"),
                )
            })
            .collect::<FxHashMap<_, _>>();
        // TODO use aliases to resolve dependencies
        // for now just use the package key
        let dependencies = dependencies.iter().map(|(_, b)| *b).collect::<Vec<_>>();
        log::info!("compiling package: {}", package_to_compile.key);
        let (compile_unit, dependency_errors) = qsc::compile::compile(
            &package_store,
            &dependencies[..],
            source_map,
            qsc::PackageType::Lib,
            capabilities,
            qsc::LanguageFeatures::from_iter(package_to_compile.language_features),
        );
        log::info!("\tcompiled package: {}", package_to_compile.key);
        if !dependency_errors.is_empty() {
            todo!("handle errors in dependencies");
        }

        let package_id = package_store.insert(compile_unit);
        canonical_package_identifier_to_package_id_mapping
            .insert(package_to_compile.key, package_id);
    }

    let source_map = qsc::SourceMap::new(
        user_code
            .sources
            .into_iter()
            .map(|(a, b)| (Arc::from(a), Arc::from(b)))
            .collect::<Vec<_>>(),
        entry.map(std::convert::Into::into),
    );

    (
        source_map,
        capabilities,
        qsc::LanguageFeatures::from_iter(user_code.language_features),
        package_store,
    )
}

/// This returns the common parameters that the language service needs from the manifest
pub(crate) fn into_project_args(project: ProjectConfig) -> qsls::LoadProjectResultInner {
    let (sources, language_features, x, y) = into_package_graph_args(project.package_graph_sources);

    (
        project.project_name.into(),
        sources,
        language_features,
        project.lints,
    )
}

#[derive(Debug)]
pub struct DependencyCycle;

impl PackageGraphSources {
    /// Produces an iterator over the packages in the order they should be compiled
    fn compilation_order(self) -> Result<(Vec<PackageInfo>, PackageInfo), DependencyCycle> {
        // The order is defined by which packages depend on which other packages
        // For example, if A depends on B which depends on C, then we compile C, then B, then A
        // If there are cycles, this is an error, and we will report it as such
        let mut in_degree: FxHashMap<&str, usize> = FxHashMap::default();
        let mut graph: FxHashMap<&str, Vec<&str>> = FxHashMap::default();

        // Initialize the graph and in-degrees
        for package in &self.packages {
            in_degree.entry(&package.key).or_insert(0);
            for dep in package.dependencies.values() {
                graph.entry(dep).or_default().push(&package.key);
                *in_degree.entry(&package.key).or_insert(0) += 1;
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
                    let count = in_degree.get_mut(neighbor).unwrap();
                    *count -= 1;
                    if *count == 0 {
                        queue.push(neighbor);
                    }
                }
            }
        }

        if sorted_keys.len() != self.packages.len() {
            return Err(DependencyCycle);
        }

        let mut sorted_packages = self.packages;
        sorted_packages
            .sort_by_key(|pkg| sorted_keys.iter().position(|key| *key == pkg.key).unwrap());

        log::info!("Determined package ordering: {:?}", sorted_keys);

        Ok((sorted_packages, self.root))
    }
}
