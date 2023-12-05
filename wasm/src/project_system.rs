use qsc_project::{EntryType, JSFileEntry, Manifest, ManifestDescriptor};
use std::{future::Future, path::PathBuf, pin::Pin, sync::Arc};
use wasm_bindgen::prelude::*;

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
    #[wasm_bindgen(
        typescript_type = "(uri: string) => Promise<{ excludeFiles: string[], excludeRegexes: string[], manifestDirectory: string } | null>"
    )]
    pub type GetManifestCallback;
}

/// This macro calls an async JS function, awaits it, and then applies a transformer function to it. Ultimately, it returns a function that accepts a String
/// and returns a JS promise that is represented by a Rust future. (I know. Ouch. My head.)
macro_rules! call_async_js_fn {
    ($js_async_fn: ident, $transformer: expr) => {{
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

            let input = input.clone();
            Box::pin(map_js_promise(res, move |x| $transformer(x, input.clone())))
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
    log::trace!("asynchronous callback from wasm returned {res:?}");
    func(res)
}

pub(crate) fn to_js_function(val: JsValue, help_text_panic: &'static str) -> js_sys::Function {
    val.dyn_ref::<js_sys::Function>()
        .unwrap_or_else(|| {
            panic!(
                "expected a valid JS function ({help_text_panic}), received {:?}",
                val.js_typeof()
            )
        })
        .clone()
}
pub(crate) use call_async_js_fn;
use wasm_bindgen_futures::JsFuture;
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
        Err(e) => todo!("result wasn't an array error: {e:?}"),
    }
}
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
        Err(_) => todo!(),
    };
    log::trace!("found manifest at {manifest_dir:?}");

    let manifest_dir = PathBuf::from(manifest_dir);

    let exclude_files = match js_sys::Reflect::get(&js_val, &JsValue::from_str("excludeFiles")) {
        Ok(v) => match v.dyn_into::<js_sys::Array>() {
            Ok(arr) => arr
                .into_iter()
                .filter_map(|x| x.as_string())
                .collect::<Vec<_>>(),
            Err(e) => todo!("result wasn't an array error: {e:?}"),
        },
        Err(_) => todo!(),
    };
    let exclude_regexes = match js_sys::Reflect::get(&js_val, &JsValue::from_str("excludeRegexes"))
    {
        Ok(v) => match v.dyn_into::<js_sys::Array>() {
            Ok(arr) => arr
                .into_iter()
                .filter_map(|x| x.as_string())
                .collect::<Vec<_>>(),
            Err(e) => todo!("result wasn't an array error: {e:?}"),
        },
        Err(_) => todo!(),
    };

    Some(ManifestDescriptor {
        manifest: Manifest {
            exclude_regexes,
            exclude_files,
            ..Default::default()
        },
        manifest_dir,
    })
}
