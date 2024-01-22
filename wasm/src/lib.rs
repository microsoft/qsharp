// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(non_snake_case)]

use diagnostic::VSDiagnostic;
use katas::check_solution;
use num_bigint::BigUint;
use num_complex::Complex64;
use project_system::*;
use qsc::{
    compile,
    hir::PackageId,
    interpret::{
        self,
        output::{self, Receiver},
    },
    target::Profile,
    PackageStore, PackageType, SourceContents, SourceMap, SourceName, SparseSim,
};
use qsc_codegen::qir_base::generate_qir;
use resource_estimator::{self as re, estimate_entry};
use serde_json::json;
use std::{fmt::Write, sync::Arc};
use wasm_bindgen::prelude::*;

mod debug_service;
mod diagnostic;
mod language_service;
mod line_column;
mod logging;
mod project_system;
mod serializable_type;

#[cfg(test)]
mod tests;

thread_local! {
    static STORE_CORE_STD: (PackageStore, PackageId) = {
        let mut store = PackageStore::new(compile::core());
        let std = store.insert(compile::std(&store, Profile::Unrestricted.into()));
        (store, std)
    };
}

#[wasm_bindgen]
pub fn git_hash() -> String {
    let git_hash = env!("QSHARP_GIT_HASH");
    git_hash.into()
}

// can't wasm_bindgen [string; 2] or (string, string)
// so we have to manually assert length of the interior
// array and the content type in the function body
// `sources` should be Vec<[String; 2]> though
pub fn get_source_map(sources: Vec<js_sys::Array>, entry: Option<String>) -> SourceMap {
    let sources = sources.into_iter().map(|js_arr| {
        // map the inner arr elements into (String, String)
        let elem_0 = js_arr.get(0).as_string();
        let elem_1 = js_arr.get(1).as_string();
        (
            Arc::from(elem_0.unwrap_or_default()),
            Arc::from(elem_1.unwrap_or_default()),
        )
    });
    SourceMap::new(sources, entry.as_deref().map(|value| value.into()))
}

#[wasm_bindgen]
pub fn get_qir(sources: Vec<js_sys::Array>) -> Result<String, String> {
    let sources = get_source_map(sources, None);
    _get_qir(sources)
}

// allows testing without wasm bindings.
fn _get_qir(sources: SourceMap) -> Result<String, String> {
    let core = compile::core();
    let mut store = PackageStore::new(core);
    let std = compile::std(&store, Profile::Base.into());
    let std = store.insert(std);

    let (unit, errors) = qsc::compile::compile(
        &store,
        &[std],
        sources,
        PackageType::Exe,
        Profile::Base.into(),
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
pub fn get_estimates(sources: Vec<js_sys::Array>, params: &str) -> Result<String, String> {
    let sources = get_source_map(sources, None);

    let mut interpreter = interpret::Interpreter::new(
        true,
        sources,
        PackageType::Exe,
        Profile::Unrestricted.into(),
    )
    .map_err(|e| e[0].to_string())?;

    estimate_entry(&mut interpreter, params).map_err(|e| match &e[0] {
        re::Error::Interpreter(interpret::Error::Eval(e)) => e.to_string(),
        re::Error::Interpreter(_) => unreachable!("interpreter errors should be eval errors"),
        re::Error::Estimation(e) => e.to_string(),
    })
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
            Profile::Unrestricted.into(),
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

fn run_internal<F>(sources: SourceMap, event_cb: F, shots: u32) -> Result<(), Box<interpret::Error>>
where
    F: FnMut(&str),
{
    let source_name = sources
        .iter()
        .map(|x| x.name.clone())
        .next()
        .expect("There must be a source to process")
        .to_string();
    let mut out = CallbackReceiver { event_cb };
    let mut interpreter = match interpret::Interpreter::new(
        true,
        sources,
        PackageType::Exe,
        Profile::Unrestricted.into(),
    ) {
        Ok(interpreter) => interpreter,
        Err(err) => {
            // TODO: handle multiple errors
            // https://github.com/microsoft/qsharp/issues/149
            let e = err[0].clone();
            let diag = VSDiagnostic::from_interpret_error(&source_name, &e);
            let msg = json!(
                {"type": "Result", "success": false, "result": diag});
            (out.event_cb)(&msg.to_string());
            return Err(Box::new(e));
        }
    };

    for _ in 0..shots {
        let result = interpreter.eval_entry_with_sim(&mut SparseSim::new(), &mut out);
        let mut success = true;
        let msg: serde_json::Value = match result {
            Ok(value) => serde_json::Value::String(value.to_string()),
            Err(errors) => {
                // TODO: handle multiple errors
                // https://github.com/microsoft/qsharp/issues/149
                success = false;
                VSDiagnostic::from_interpret_error(&source_name, &errors[0]).json()
            }
        };

        let msg_string = json!({"type": "Result", "success": success, "result": msg}).to_string();
        (out.event_cb)(&msg_string);
    }
    Ok(())
}

#[wasm_bindgen]
pub fn run(
    sources: Vec<js_sys::Array>,
    expr: &str,
    event_cb: &js_sys::Function,
    shots: u32,
) -> Result<bool, JsValue> {
    if !event_cb.is_function() {
        return Err(JsError::new("Events callback function must be provided").into());
    }
    let sources = get_source_map(sources, Some(expr.into()));
    match run_internal(
        sources,
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

#[wasm_bindgen(typescript_custom_section)]
const TARGET_PROFILE: &'static str = r#"
export type TargetProfile = "base" | "unrestricted";
"#;
