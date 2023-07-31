// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::language_service::VSDiagnostic;
use katas::check_solution;
use num_bigint::BigUint;
use num_complex::Complex64;
use qsc::{
    compile::{self},
    hir::PackageId,
    interpret::{
        output::{self, Receiver},
        stateless,
    },
    PackageStore, PackageType, SourceContents, SourceMap, SourceName,
};
use serde_json::json;
use std::fmt::Write;
use wasm_bindgen::prelude::*;

mod language_service;
mod logging;

#[wasm_bindgen]
pub fn git_hash() -> JsValue {
    let git_hash = env!("QSHARP_GIT_HASH");
    JsValue::from_str(git_hash)
}

impl VSDiagnostic {
    pub fn json(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("serializing VSDiagnostic should succeed")
    }
}

fn compile(code: &str) -> (qsc::hir::Package, Vec<VSDiagnostic>) {
    thread_local! {
        static STORE_STD: (PackageStore, PackageId) = {
            let mut store = PackageStore::new(compile::core());
            let std = store.insert(compile::std(&store));
            (store, std)
        };
    }

    STORE_STD.with(|(store, std)| {
        let sources = SourceMap::new([("code".into(), code.into())], None);
        let (unit, errors) = compile::compile(store, &[*std], sources, PackageType::Exe);
        (
            unit.package,
            errors.into_iter().map(|error| (&error).into()).collect(),
        )
    })
}

#[wasm_bindgen]
pub fn get_hir(code: &str) -> Result<JsValue, JsValue> {
    let (package, _) = compile(code);
    let hir = package.to_string();
    Ok(serde_wasm_bindgen::to_value(&hir)?)
}

struct CallbackReceiver<F>
where
    F: Fn(&str),
{
    event_cb: F,
}

impl<F> Receiver for CallbackReceiver<F>
where
    F: Fn(&str),
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

fn run_internal<F>(code: &str, expr: &str, event_cb: F, shots: u32) -> Result<(), stateless::Error>
where
    F: Fn(&str),
{
    let mut out = CallbackReceiver { event_cb };
    let sources = SourceMap::new([("code".into(), code.into())], Some(expr.into()));
    let interpreter = stateless::Interpreter::new(true, sources);
    if let Err(err) = interpreter {
        // TODO: handle multiple errors
        // https://github.com/microsoft/qsharp/issues/149
        let e = err[0].clone();
        let diag: VSDiagnostic = (&e).into();
        let msg = json!(
            {"type": "Result", "success": false, "result": diag});
        (out.event_cb)(&msg.to_string());
        return Err(e);
    }
    let interpreter = interpreter.expect("context should be valid");
    for _ in 0..shots {
        let mut eval_ctx = interpreter.new_eval_context();
        let result = eval_ctx.eval_entry(&mut out);
        let mut success = true;
        let msg: serde_json::Value = match result {
            Ok(value) => serde_json::Value::String(value.to_string()),
            Err(errors) => {
                // TODO: handle multiple errors
                // https://github.com/microsoft/qsharp/issues/149
                success = false;
                VSDiagnostic::from(&errors[0]).json()
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
) -> Result<JsValue, JsValue> {
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
        Ok(()) => Ok(JsValue::TRUE),
        Err(e) => Err(JsError::from(e).into()),
    }
}

fn check_exercise_solution_internal(
    solution_code: &str,
    exercise_sources: Vec<(SourceName, SourceContents)>,
    event_cb: impl Fn(&str),
) -> bool {
    let mut sources = vec![("solution".into(), solution_code.into())];
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
            (false, VSDiagnostic::from(&errors[0]).json())
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
) -> Result<JsValue, JsValue> {
    let exercise_soruces_strs: Vec<String> = serde_wasm_bindgen::from_value(exercise_sources_js)
        .expect("Deserializing code dependencies should succeed");
    let mut exercise_sources: Vec<(SourceName, SourceContents)> = vec![];
    for (index, code) in exercise_soruces_strs.into_iter().enumerate() {
        exercise_sources.push((index.to_string().into(), code.into()));
    }
    let success = check_exercise_solution_internal(solution_code, exercise_sources, |msg: &str| {
        let _ = event_cb.call1(&JsValue::null(), &JsValue::from_str(msg));
    });

    Ok(JsValue::from_bool(success))
}

#[cfg(test)]
mod test {
    #[test]
    fn test_missing_type() {
        let code = "namespace input { operation Foo(a) : Unit {} }";
        let (_, mut diag) = crate::compile(code);
        assert_eq!(diag.len(), 2, "{diag:#?}");
        let err_1 = diag.pop().unwrap();
        let err_2 = diag.pop().unwrap();

        assert_eq!(err_1.start_pos, 32);
        assert_eq!(err_1.end_pos, 33);
        assert_eq!(err_1.message, "type error: insufficient type information to infer type\n\nhelp: provide a type annotation");
        assert_eq!(err_2.start_pos, 32);
        assert_eq!(err_2.end_pos, 33);
        assert_eq!(err_2.message, "type error: missing type in item signature\n\nhelp: types cannot be inferred for global declarations");
    }

    #[test]
    fn test_run_two_shots() {
        let code = "
            namespace Test {
                function Answer() : Int {
                    return 42;
                }
            }
        ";
        let expr = "Test.Answer()";
        let count = std::cell::Cell::new(0);

        let _result = crate::run_internal(
            code,
            expr,
            |_msg| {
                assert!(_msg.contains("42"));
                count.set(count.get() + 1);
            },
            2,
        );
        assert_eq!(count.get(), 2);
    }

    #[test]
    fn fail_ry() {
        let code = "namespace Sample {
            operation main() : Result[] {
                use q1 = Qubit();
                Ry(q1);
                let m1 = M(q1);
                return [m1];
            }
        }";

        let (_, errors) = crate::compile(code);
        assert_eq!(errors.len(), 1, "{errors:#?}");

        let error = errors.first().unwrap();
        assert_eq!(error.start_pos, 111);
        assert_eq!(error.end_pos, 117);
        assert_eq!(
            error.message,
            "type error: expected (Double, Qubit), found Qubit"
        );
    }

    #[test]
    fn test_message() {
        let code = r#"namespace Sample {
            open Microsoft.Quantum.Diagnostics;

            operation main() : Unit {
                Message("hi");
                return ();
            }
        }"#;
        let expr = "Sample.main()";
        let result = crate::run_internal(
            code,
            expr,
            |_msg_| {
                assert!(_msg_.contains("hi") || _msg_.contains("result"));
            },
            1,
        );
        assert!(result.is_ok());
    }
    #[test]
    fn message_with_escape_sequences() {
        let code = r#"namespace Sample {
            open Microsoft.Quantum.Diagnostics;

            operation main() : Unit {
                Message("\ta\n\t");

                return ();
            }
        }"#;
        let expr = "Sample.main()";
        let result = crate::run_internal(
            code,
            expr,
            |_msg_| {
                assert!(_msg_.contains(r#"\ta\n\t"#) || _msg_.contains("result"));
            },
            1,
        );
        assert!(result.is_ok());
    }
    #[test]
    fn message_with_backslashes() {
        let code = r#"namespace Sample {
            open Microsoft.Quantum.Diagnostics;

            operation main() : Unit {
                Message("hi \\World");
                Message("hello { \\World [");

                return ();
            }
        }"#;
        let expr = "Sample.main()";
        let result = crate::run_internal(
            code,
            expr,
            |_msg_| {
                assert!(
                    _msg_.contains("hello { \\\\World [")
                        || _msg_.contains("hi \\\\World")
                        || _msg_.contains("result")
                );
            },
            1,
        );
        assert!(result.is_ok());
    }
    #[test]
    fn test_entrypoint() {
        let code = r#"namespace Sample {
            @EntryPoint()
            operation main() : Unit {
                Message("hi");
                return ();
            }
        }"#;
        let expr = "";
        let result = crate::run_internal(
            code,
            expr,
            |_msg_| {
                assert!(_msg_.contains("hi") || _msg_.contains("result"));
            },
            1,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_entrypoint() {
        let code = "namespace Sample {
            operation main() : Result[] {
                use q1 = Qubit();
                let m1 = M(q1);
                return [m1];
            }
        }";
        let expr = "";
        let result = crate::run_internal(
            code,
            expr,
            |msg| {
                assert!(msg.contains(r#""success":false"#));
                assert!(msg.contains(r#""message":"entry point not found"#));
                assert!(msg.contains(r#""start_pos":0"#));
            },
            1,
        );
        assert!(result.is_err());
    }
}
