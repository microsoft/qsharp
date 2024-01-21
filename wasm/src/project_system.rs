// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use async_trait::async_trait;
use js_sys::JsString;
use qsc_project::{EntryType, JSFileEntry, Manifest, ManifestDescriptor, ProjectSystemCallbacks};
use std::iter::FromIterator;
use std::{path::PathBuf, sync::Arc};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

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
        get_manifest_transformer(value.obj, Default::default())
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

            Box::pin(map_js_promise(res, move |x| $map_result(x, input.clone())))
                as Pin<Box<dyn Future<Output = _> + 'static>>
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
        "expected a valid JS function ({help_text_panic}), received {:?}",
        js_ty
    );
    Into::<js_sys::Function>::into(val)
}
pub(crate) use into_async_rust_fn_with;

/// Given a [JsValue] representing the result of a call to a list_directory function,
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
                match (
                    arr.next().unwrap().as_string(),
                    arr.next().unwrap().as_f64(),
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

/// Given a [JsValue] representing the result of a call to a read file function,
/// and a `String` representing the path that was originally passed in as an
/// argument to that function, assert that `js_val` matches our expected return type of
/// `string` and transform it into a tuple representing the path and the file contents.
pub(crate) fn read_file_transformer(
    js_val: JsValue,
    path_buf_string: String,
) -> (Arc<str>, Arc<str>) {
    match js_val.as_string() {
        Some(res) => return (Arc::from(path_buf_string.as_str()), Arc::from(res)),
        // this can happen if the document is completely empty
        None if js_val.is_null() => (Arc::from(path_buf_string.as_str()), Arc::from("")),
        None => unreachable!("Expected string from JS callback, received {js_val:?}"),
    }
}
/// Given a [JsValue] representing the result of a call to a get_manifest function,
/// and an unused `String` parameter for API compatibility, assert that `js_val`
/// matches our expected return object shape  and transform it into a [ManifestDescriptor],
/// or `None`
pub(crate) fn get_manifest_transformer(js_val: JsValue, _: String) -> Option<ManifestDescriptor> {
    if js_val.is_null() {
        return None;
    }

    let manifest_dir = match js_sys::Reflect::get(&js_val, &JsValue::from_str("manifestDirectory"))
    {
        Ok(v) => v.as_string().unwrap_or_else(|| {
            panic!(
                "manifest callback returned {:?}, but we expected a string representing its URI",
                v
            )
        }),
                    Err(_) => unreachable!("our typescript bindings should guarantee that an object with a manifestDirectory property is returned here"),
    };
    log::trace!("found manifest at {manifest_dir:?}");

    let manifest_dir = PathBuf::from(manifest_dir);

    Some(ManifestDescriptor {
        manifest: Manifest {
            ..Default::default()
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
            Some(manifest) => {
                let res = qsc_project::FileSystemAsync::load_project(self, &manifest)
                    .await
                    .map(|proj| {
                        proj.sources
                            .into_iter()
                            .map(|(path, contents)| {
                                js_sys::Array::from_iter::<std::slice::Iter<'_, JsString>>(
                                    [path.to_string().into(), contents.to_string().into()].iter(),
                                )
                            })
                            .collect::<js_sys::Array>()
                    })
                    .unwrap_or_else(|_| js_sys::Array::new());
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
