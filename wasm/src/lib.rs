// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(unknown_lints, clippy::empty_docs)]
#![allow(non_snake_case)]

use diagnostic::{interpret_errors_into_qsharp_errors, VSDiagnostic};
use katas::check_solution;
use language_service::IOperationInfo;
use num_bigint::BigUint;
use num_complex::Complex64;
use project_system::{into_qsc_args, ProgramConfig};
use qsc::{
    compile::{self, Dependencies},
    format_state_id, get_matrix_latex, get_state_latex,
    hir::PackageId,
    interpret::{
        self,
        output::{self, Receiver},
        CircuitEntryPoint,
    },
    target::Profile,
    LanguageFeatures, PackageStore, PackageType, PauliNoise, SourceContents, SourceMap, SourceName,
    SparseSim, TargetCapabilityFlags,
};
use resource_estimator::{self as re, estimate_entry};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fmt::Write, str::FromStr};
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
#[must_use]
pub fn git_hash() -> String {
    let git_hash = env!("QSHARP_GIT_HASH");
    git_hash.into()
}

#[wasm_bindgen]
pub fn get_qir(program: ProgramConfig) -> Result<String, String> {
    let (source_map, capabilities, language_features, store, deps) =
        into_qsc_args(program, None).map_err(compile_errors_into_qsharp_errors_json)?;

    _get_qir(
        source_map,
        language_features,
        capabilities,
        store,
        &deps[..],
    )
}

pub(crate) fn _get_qir(
    sources: SourceMap,
    language_features: LanguageFeatures,
    capabilities: TargetCapabilityFlags,
    store: PackageStore,
    deps: &qsc::compile::Dependencies,
) -> Result<String, String> {
    qsc::codegen::qir::get_qir(sources, language_features, capabilities, store, deps)
        .map_err(interpret_errors_into_qsharp_errors_json)
}

#[wasm_bindgen]
pub fn get_estimates(program: ProgramConfig, params: &str) -> Result<String, String> {
    let (source_map, capabilities, language_features, store, deps) = into_qsc_args(program, None)
        .map_err(|mut e| {
        // Wrap in `interpret::Error` to match the error type from `Interpreter::new` below
        qsc::interpret::Error::from(e.pop().expect("expected at least one error")).to_string()
    })?;

    let mut interpreter = interpret::Interpreter::new(
        source_map,
        PackageType::Exe,
        capabilities,
        language_features,
        store,
        &deps[..],
    )
    .map_err(|e| e[0].to_string())?;

    estimate_entry(&mut interpreter, params).map_err(|e| match &e[0] {
        re::Error::Interpreter(interpret::Error::Eval(e)) => e.to_string(),
        re::Error::Interpreter(_) => unreachable!("interpreter errors should be eval errors"),
        re::Error::Estimation(e) => e.to_string(),
    })
}

#[wasm_bindgen]
pub fn get_circuit(
    program: ProgramConfig,
    simulate: bool,
    operation: Option<IOperationInfo>,
) -> Result<JsValue, String> {
    let (source_map, capabilities, language_features, store, deps) =
        into_qsc_args(program, None).map_err(compile_errors_into_qsharp_errors_json)?;

    let (package_type, entry_point) = match operation {
        Some(p) => {
            let o: language_service::OperationInfo = p.into();
            // lib package - no need to enforce an entry point since the operation is provided.
            (PackageType::Lib, CircuitEntryPoint::Operation(o.operation))
        }
        None => {
            // exe package - the @EntryPoint attribute will be used.
            (PackageType::Exe, CircuitEntryPoint::EntryPoint)
        }
    };

    let mut interpreter = interpret::Interpreter::new(
        source_map,
        package_type,
        capabilities,
        LanguageFeatures::from_iter(language_features),
        store,
        &deps[..],
    )
    .map_err(interpret_errors_into_qsharp_errors_json)?;

    let circuit = interpreter
        .circuit(entry_point, simulate)
        .map_err(interpret_errors_into_qsharp_errors_json)?;

    serde_wasm_bindgen::to_value(&circuit).map_err(|e| e.to_string())
}

#[allow(clippy::needless_pass_by_value)]
fn interpret_errors_into_qsharp_errors_json(errs: Vec<qsc::interpret::Error>) -> String {
    serde_json::to_string(&interpret_errors_into_qsharp_errors(&errs))
        .expect("serializing errors to json should succeed")
}

fn compile_errors_into_qsharp_errors_json(errs: Vec<qsc::compile::Error>) -> String {
    interpret_errors_into_qsharp_errors_json(errs.into_iter().map(Into::into).collect())
}

#[wasm_bindgen]
#[must_use]
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
pub fn get_ast(
    code: &str,
    language_features: Vec<String>,
    profile: &str,
) -> Result<String, String> {
    let language_features = LanguageFeatures::from_iter(language_features);
    let sources = SourceMap::new([("code".into(), code.into())], None);
    let profile =
        Profile::from_str(profile).map_err(|()| format!("Invalid target profile {profile}"))?;
    let package = STORE_CORE_STD.with(|(store, std)| {
        let (unit, _) = compile::compile(
            store,
            &[(*std, None)],
            sources,
            PackageType::Exe,
            profile.into(),
            language_features,
        );
        unit.ast.package
    });
    Ok(format!("{package}"))
}

#[wasm_bindgen]
pub fn get_hir(
    code: &str,
    language_features: Vec<String>,
    profile: &str,
) -> Result<String, String> {
    let language_features = LanguageFeatures::from_iter(language_features);
    let sources = SourceMap::new([("code".into(), code.into())], None);
    let profile =
        Profile::from_str(profile).map_err(|()| format!("Invalid target profile {profile}"))?;
    let package = STORE_CORE_STD.with(|(store, std)| {
        let (unit, _) = compile::compile(
            store,
            &[(*std, None)],
            sources,
            PackageType::Exe,
            profile.into(),
            language_features,
        );
        unit.package
    });
    Ok(package.to_string())
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
                format_state_id(&state.0, qubit_count),
                state.1.re,
                state.1.im
            )
            .expect("writing to string should succeed");
        }
        write!(
            dump_json,
            r#""{}": [{}, {}]}}, "#,
            format_state_id(&last.0, qubit_count),
            last.1.re,
            last.1.im
        )
        .expect("writing to string should succeed");

        let json_latex = serde_json::to_string(&get_state_latex(&state, qubit_count))
            .expect("serialization should succeed");
        write!(
            dump_json,
            r#" "stateLatex": {json_latex}, "qubitCount": {qubit_count} }} "#
        )
        .expect("writing to string should succeed");
        (self.event_cb)(&dump_json);
        Ok(())
    }

    fn matrix(&mut self, matrix: Vec<Vec<Complex64>>) -> Result<(), output::Error> {
        let mut dump_json = String::new();

        // Write the type and open the array or rows.
        write!(dump_json, r#"{{"type": "Matrix","matrix": ["#)
            .expect("writing to string should succeed");

        // Map each row to a string representation of the row, and join them with commas.
        // The row is an array, and each element is a tuple formatted as "[re, im]".
        // e.g. {"type": "Matrix", "matrix": [
        //   [[1, 2], [3, 4], [5, 6]],
        //   [[7, 8], [9, 10], [11, 12]]
        // ]}
        let row_strings = matrix
            .iter()
            .map(|row| {
                let row_str = row
                    .iter()
                    .map(|elem| format!("[{}, {}]", elem.re, elem.im))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{row_str}]")
            })
            .collect::<Vec<_>>()
            .join(", ");

        // Close the array of rows and the JSON object.
        let latex_string = serde_json::to_string(&get_matrix_latex(&matrix))
            .expect("serialization should succeed");
        write!(
            dump_json,
            r#"{row_strings}], "matrixLatex": {latex_string} }}"#
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

#[allow(clippy::too_many_arguments)]
fn run_internal_with_features<F>(
    sources: SourceMap,
    event_cb: F,
    shots: u32,
    language_features: LanguageFeatures,
    capabilities: TargetCapabilityFlags,
    store: PackageStore,
    dependencies: &Dependencies,
    pauliNoise: &PauliNoise,
) -> Result<(), Box<interpret::Error>>
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
        sources,
        PackageType::Exe,
        capabilities,
        language_features,
        store,
        dependencies,
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
        let result =
            interpreter.eval_entry_with_sim(&mut SparseSim::new_with_noise(pauliNoise), &mut out);
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
    program: ProgramConfig,
    expr: &str,
    event_cb: &js_sys::Function,
    shots: u32,
) -> Result<bool, JsValue> {
    runWithPauliNoise(program, expr, event_cb, shots, &JsValue::null())
}

#[wasm_bindgen]
pub fn runWithPauliNoise(
    program: ProgramConfig,
    expr: &str,
    event_cb: &js_sys::Function,
    shots: u32,
    pauliNoise: &JsValue,
) -> Result<bool, JsValue> {
    let (source_map, capabilities, language_features, store, deps) =
        into_qsc_args(program, Some(expr.into())).map_err(|mut e| {
            // Wrap in `interpret::Error` and `JsError` to match the error type
            // `run_internal_with_features` below
            JsError::from(qsc::interpret::Error::from(
                e.pop().expect("expected at least one error"),
            ))
        })?;

    if !event_cb.is_function() {
        return Err(JsError::new("Events callback function must be provided").into());
    }

    let event_cb = |msg: &str| {
        // See example at https://rustwasm.github.io/wasm-bindgen/reference/receiving-js-closures-in-rust.html
        let _ = event_cb.call1(&JsValue::null(), &JsValue::from(msg));
    };

    // See if the pauliNoise JsValue is an array
    let noise = if pauliNoise.is_array() {
        let pauliArray = js_sys::Array::from(pauliNoise);
        if pauliArray.length() != 3 {
            return Err(JsError::new("Pauli noise must have 3 probabilities").into());
        }
        PauliNoise::from_probabilities(
            pauliArray
                .get(0)
                .as_f64()
                .expect("Probabilities should be floats"),
            pauliArray
                .get(1)
                .as_f64()
                .expect("Probabilities should be floats"),
            pauliArray
                .get(2)
                .as_f64()
                .expect("Probabilities should be floats"),
        )
        .expect("Unable to create Pauli noise from the array provided")
    } else {
        PauliNoise::default()
    };

    match run_internal_with_features(
        source_map,
        event_cb,
        shots,
        language_features,
        capabilities,
        store,
        &deps[..],
        &noise,
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
#[must_use]
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

serializable_type! {
    DocFile,
    {
        filename: String,
        metadata: String,
        contents: String,
    },
    r#"export interface IDocFile {
        filename: string;
        metadata: string;
        contents: string;
    }"#,
    IDocFile
}

#[wasm_bindgen]
#[must_use]
pub fn generate_docs(additional_program: Option<ProgramConfig>) -> Vec<IDocFile> {
    let docs = if let Some(additional_program) = additional_program {
        let Ok((source_map, capabilities, language_features, package_store, dependencies)) =
            into_qsc_args(additional_program, None)
        else {
            // Can't generate docs if building dependencies failed
            return Vec::new();
        };

        qsc_doc_gen::generate_docs::generate_docs(
            Some((package_store, &dependencies, source_map)),
            Some(capabilities),
            Some(language_features),
        )
    } else {
        qsc_doc_gen::generate_docs::generate_docs(None, None, None)
    };

    let mut result: Vec<IDocFile> = vec![];

    for (name, metadata, contents) in docs {
        result.push(
            DocFile {
                filename: name.to_string(),
                metadata: metadata.to_string(),
                contents: contents.to_string(),
            }
            .into(),
        );
    }

    result
}

#[wasm_bindgen(typescript_custom_section)]
const TARGET_PROFILE: &'static str = r#"
export type TargetProfile = "base" | "adaptive_ri" | "unrestricted";
"#;

#[wasm_bindgen(typescript_custom_section)]
const LANGUAGE_FEATURES: &'static str = r#"
export type LanguageFeatures = "v2-preview-syntax";
"#;
