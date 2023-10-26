// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use diagnostic::VSDiagnostic;
use katas::check_solution;
use miette::miette;
use num_bigint::BigUint;
use num_complex::Complex64;
use qsc::{
    compile::{self},
    hir::PackageId,
    interpret::{
        output::{self, Receiver},
        stateful,
    },
    PackageStore, PackageType, SourceContents, SourceMap, SourceName, TargetProfile,
};
use qsc_codegen::qir_base::generate_qir;
use qsc_project::{DirEntry, EntryType, FileSystem};
use serde_json::json;
use std::{fmt::Write, path::PathBuf, sync::Arc};
use wasm_bindgen::prelude::*;

mod debug_service;
mod diagnostic;
mod language_service;
mod logging;
mod serializable_type;

#[cfg(test)]
mod tests;

thread_local! {
    static STORE_CORE_STD: (PackageStore, PackageId) = {
        let mut store = PackageStore::new(compile::core());
        let std = store.insert(compile::std(&store, TargetProfile::Full));
        (store, std)
    };
}

#[wasm_bindgen]
pub fn git_hash() -> String {
    let git_hash = env!("QSHARP_GIT_HASH");
    git_hash.into()
}

#[wasm_bindgen]
pub fn get_qir(code: &str) -> Result<String, String> {
    let core = compile::core();
    let mut store = PackageStore::new(core);
    let std = compile::std(&store, TargetProfile::Base);
    let std = store.insert(std);
    let sources = SourceMap::new([("test".into(), code.into())], None);

    let (unit, errors) = qsc::compile::compile(
        &store,
        &[std],
        sources,
        PackageType::Exe,
        TargetProfile::Base,
    );

    // Ensure it compiles before trying to add it to the store.
    if !errors.is_empty() {
        // This should never happen, as the program should be checked for errors before trying to
        // generate code for it. But just in case, simply report the failure.
        return Err("Failed to generate QIR".to_string());
    }

    let package = store.insert(unit);

    generate_qir(&store, package).map_err(|e| e.0.to_string())
}

#[wasm_bindgen]
pub fn get_library_source_content(name: &str) -> Option<String> {
    STORE_CORE_STD.with(|(store, std)| {
        for id in [PackageId::CORE, *std] {
            if let Some(source) = store
                .get(id)
                .expect("package should be in store")
                .sources
                .find_by_name(name)
            {
                return Some(source.contents.to_string());
            }
        }

        None
    })
}

#[wasm_bindgen]
pub fn get_hir(code: &str) -> String {
    let sources = SourceMap::new([("code".into(), code.into())], None);
    let package = STORE_CORE_STD.with(|(store, std)| {
        let (unit, _) = compile::compile(
            store,
            &[*std],
            sources,
            PackageType::Exe,
            TargetProfile::Full,
        );
        unit.package
    });
    package.to_string()
}

struct CallbackReceiver<F>
where
    F: FnMut(&str),
{
    event_cb: F,
}

impl<F> Receiver for CallbackReceiver<F>
where
    F: FnMut(&str),
{
    fn state(
        &mut self,
        state: Vec<(BigUint, Complex64)>,
        qubit_count: usize,
    ) -> Result<(), output::Error> {
        let mut dump_json = String::new();
        write!(dump_json, r#"{{"type": "DumpMachine","state": {{"#)
            .expect("writing to string should succeed");
        let (last, most) = state
            .split_last()
            .expect("state should always have at least one entry");
        for state in most {
            write!(
                dump_json,
                r#""{}": [{}, {}],"#,
                output::format_state_id(&state.0, qubit_count),
                state.1.re,
                state.1.im
            )
            .expect("writing to string should succeed");
        }
        write!(
            dump_json,
            r#""{}": [{}, {}]}}}}"#,
            output::format_state_id(&last.0, qubit_count),
            last.1.re,
            last.1.im
        )
        .expect("writing to string should succeed");
        (self.event_cb)(&dump_json);
        Ok(())
    }

    fn message(&mut self, msg: &str) -> Result<(), output::Error> {
        let msg_json = json!({"type": "Message", "message": msg});
        (self.event_cb)(&msg_json.to_string());
        Ok(())
    }
}

fn run_internal<F>(code: &str, expr: &str, event_cb: F, shots: u32) -> Result<(), stateful::Error>
where
    F: FnMut(&str),
{
    let source_name = "code";
    let mut out = CallbackReceiver { event_cb };
    let sources = SourceMap::new([(source_name.into(), code.into())], Some(expr.into()));
    let interpreter =
        stateful::Interpreter::new(true, sources, PackageType::Exe, TargetProfile::Full);
    if let Err(err) = interpreter {
        // TODO: handle multiple errors
        // https://github.com/microsoft/qsharp/issues/149
        let e = err[0].clone();
        let diag = VSDiagnostic::from_interpret_error(source_name, &e);
        let msg = json!(
            {"type": "Result", "success": false, "result": diag});
        (out.event_cb)(&msg.to_string());
        return Err(e);
    }
    let mut interpreter = interpreter.expect("context should be valid");
    for _ in 0..shots {
        let result = interpreter.eval_entry(&mut out);
        let mut success = true;
        let msg: serde_json::Value = match result {
            Ok(value) => serde_json::Value::String(value.to_string()),
            Err(errors) => {
                // TODO: handle multiple errors
                // https://github.com/microsoft/qsharp/issues/149
                success = false;
                VSDiagnostic::from_interpret_error(source_name, &errors[0]).json()
            }
        };

        let msg_string = json!({"type": "Result", "success": success, "result": msg}).to_string();
        (out.event_cb)(&msg_string);
    }
    Ok(())
}

#[wasm_bindgen]
pub fn run(
    code: &str,
    expr: &str,
    event_cb: &js_sys::Function,
    shots: u32,
) -> Result<bool, JsValue> {
    if !event_cb.is_function() {
        return Err(JsError::new("Events callback function must be provided").into());
    }

    match run_internal(
        code,
        expr,
        |msg: &str| {
            // See example at https://rustwasm.github.io/wasm-bindgen/reference/receiving-js-closures-in-rust.html
            let _ = event_cb.call1(&JsValue::null(), &JsValue::from(msg));
        },
        shots,
    ) {
        Ok(()) => Ok(true),
        Err(e) => Err(JsError::from(e).into()),
    }
}

fn check_exercise_solution_internal(
    solution_code: &str,
    exercise_sources: Vec<(SourceName, SourceContents)>,
    event_cb: impl Fn(&str),
) -> bool {
    let source_name = "solution";
    let mut sources = vec![(source_name.into(), solution_code.into())];
    for exercise_source in exercise_sources {
        sources.push(exercise_source);
    }
    let mut out = CallbackReceiver { event_cb };
    let result = check_solution(sources, &mut out);
    let mut runtime_success = true;
    let (exercise_success, msg) = match result {
        Ok(value) => (value, serde_json::Value::String(value.to_string())),
        Err(errors) => {
            // TODO: handle multiple errors
            // https://github.com/microsoft/qsharp/issues/149
            runtime_success = false;
            (
                false,
                VSDiagnostic::from_interpret_error(source_name, &errors[0]).json(),
            )
        }
    };
    let msg_string =
        json!({"type": "Result", "success": runtime_success, "result": msg}).to_string();
    (out.event_cb)(&msg_string);
    exercise_success
}

#[wasm_bindgen]
pub fn check_exercise_solution(
    solution_code: &str,
    exercise_sources_js: JsValue,
    event_cb: &js_sys::Function,
) -> bool {
    let exercise_soruces_strs: Vec<String> = serde_wasm_bindgen::from_value(exercise_sources_js)
        .expect("Deserializing code dependencies should succeed");
    let mut exercise_sources: Vec<(SourceName, SourceContents)> = vec![];
    for (index, code) in exercise_soruces_strs.into_iter().enumerate() {
        exercise_sources.push((index.to_string().into(), code.into()));
    }
    check_exercise_solution_internal(solution_code, exercise_sources, |msg: &str| {
        let _ = event_cb.call1(&JsValue::null(), &JsValue::from_str(msg));
    })
}

#[wasm_bindgen]
pub struct ProjectLoader {
    lookup_fn: js_sys::Function,
    list_dir_fn: js_sys::Function,
}

#[wasm_bindgen]
pub struct ManifestDescriptor {
    exclude_files: js_sys::Array,
    exclude_regexes: js_sys::Array,
    root_directory: js_sys::JsString,
}

impl From<ManifestDescriptor> for qsc_project::ManifestDescriptor {
    fn from(value: ManifestDescriptor) -> Self {
        let exclude_files = value
            .exclude_files
            .into_iter()
            .filter_map(|arr_value| arr_value.as_string())
            .collect();
        let exclude_regexes = value
            .exclude_regexes
            .into_iter()
            .filter_map(|arr_value| arr_value.as_string())
            .collect();
        let root_directory = PathBuf::from(String::from(value.root_directory));
        qsc_project::ManifestDescriptor {
            manifest: qsc_project::Manifest {
                author: None,
                license: None,
                exclude_regexes,
                exclude_files,
            },
            manifest_dir: root_directory,
        }
    }
}

/// When this function is called from JS, we return a list of sources that are included in the project.  
#[wasm_bindgen]
pub fn load_project(
    mut proj: ProjectLoader,
    manifest: ManifestDescriptor,
) -> Result<js_sys::Array, String> {
    let proj = proj
        .load_project(manifest.into())
        .map_err(|e| format!("{e:?}"))?;

    let result: js_sys::Array = proj
        .sources
        .into_iter()
        .map(|(file_name, file_contents)| {
            js_sys::Array::from_iter(
                vec![
                    js_sys::JsString::from(&*file_name),
                    js_sys::JsString::from(&*file_contents),
                ]
                .into_iter(),
            )
        })
        .collect();
    Ok(result)
}

pub struct JsFileEntry {
    ty: EntryType,
    extension: String,
    name: String,
    path: String,
}

impl DirEntry for JsFileEntry {
    type Error = String;

    fn entry_type(&self) -> Result<qsc_project::EntryType, Self::Error> {
        Ok(self.ty)
    }

    fn extension(&self) -> String {
        self.extension.clone()
    }

    fn entry_name(&self) -> String {
        self.name.clone()
    }

    fn path(&self) -> std::path::PathBuf {
        PathBuf::from(&self.path)
    }
}

impl FileSystem for ProjectLoader {
    type Entry = JsFileEntry;

    fn read_file(
        &mut self,
        path: &std::path::Path,
    ) -> miette::Result<(std::sync::Arc<str>, std::sync::Arc<str>)> {
        let path = path.to_string_lossy().to_string();
        let lookup_fn = |path: String| self.lookup_fn.call1(&JsValue::null(), &JsValue::from(path));
        let file_contents = lookup_fn(path.clone());
        match file_contents
            .map_err(|e| miette!("error loading file: {:?}", e))?
            .as_string()
        {
            Some(contents) => Ok((Arc::from(path), Arc::from(contents))),
            None => Err(miette!("file {path} not found")),
        }
    }

    fn list_directory(&self, path: &std::path::Path) -> miette::Result<Vec<Self::Entry>> {
        let path = path.to_string_lossy().to_string();
        let list_dir_fn = |path: String| {
            self.list_dir_fn
                .call1(&JsValue::null(), &JsValue::from(path))
        };

        let result = list_dir_fn(path.clone());
        let result = result.map_err(|e| miette!("error loading file: {:?}", e))?;

        if !result.is_array() {
            return todo!("item is not array");
        }

        todo!()
    }
}
