// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::language_service::VSDiagnostic;
use katas::{run_kata, KATA_ENTRY};
use num_bigint::BigUint;
use num_complex::Complex64;
use qsc::{
    compile,
    interpret::{
        output::{self, Receiver},
        stateless,
    },
    PackageStore, SourceMap,
};
use qsc_hir::hir::PackageId;
use serde_json::json;
use std::fmt::Write;
use wasm_bindgen::prelude::*;

mod language_service;

#[wasm_bindgen]
pub fn git_hash() -> JsValue {
    JsValue::from_str(env!("QSHARP_GIT_HASH"))
}

#[wasm_bindgen(typescript_custom_section)]
const IDiagnostic: &'static str = r#"
export interface IDiagnostic {
    start_pos: number;
    end_pos: number;
    message: string;
    severity: "error" | "warning" | "info"
    code?: {
        value: number;  // Can also be a string, but number would be preferable
        target: string; // URI for more info - could be a custom URI for pretty errors
    }
}
"#;

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
    let context = stateless::Context::new(true, sources);
    if let Err(err) = context {
        // TODO: handle multiple errors
        // https://github.com/microsoft/qsharp/issues/149
        let e = err[0].clone();
        let diag: VSDiagnostic = (&e).into();
        let msg = json!(
            {"type": "Result", "success": false, "result": diag});
        (out.event_cb)(&msg.to_string());
        return Err(e);
    }
    let context = context.expect("context should be valid");
    for _ in 0..shots {
        let result = context.eval(&mut out);
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
pub fn get_hir(code: &str) -> Result<JsValue, JsValue> {
    let (package, _) = compile(code);
    let hir = package.to_string();
    Ok(serde_wasm_bindgen::to_value(&hir)?)
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
        let (unit, errors) = compile::compile(store, &[*std], sources);
        (
            unit.package,
            errors.into_iter().map(|error| (&error).into()).collect(),
        )
    })
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

fn run_kata_exercise_internal(
    verification_source: &str,
    kata_implementation: &str,
    event_cb: impl Fn(&str),
) -> Result<bool, Vec<stateless::Error>> {
    let sources = SourceMap::new(
        [
            ("kata".into(), kata_implementation.into()),
            ("verifier".into(), verification_source.into()),
        ],
        Some(KATA_ENTRY.into()),
    );

    run_kata(sources, &mut CallbackReceiver { event_cb })
}

#[wasm_bindgen]
pub fn run_kata_exercise(
    verification_source: &str,
    kata_implementation: &str,
    event_cb: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    match run_kata_exercise_internal(verification_source, kata_implementation, |msg: &str| {
        let _ = event_cb.call1(&JsValue::null(), &JsValue::from_str(msg));
    }) {
        Ok(v) => Ok(JsValue::from_bool(v)),
        // TODO: Unify with the 'run' code. Failure of user code is not 'exceptional', and
        // should be reported with a Result event (also for success) and not an exception.
        Err(e) => {
            // TODO: Handle multiple errors.
            let first_error = e
                .first()
                .expect("Running kata failed but no errors were reported");
            Err(JsError::from(first_error).into())
        }
    }
}

#[cfg(test)]
mod test {
    use qsc::compile::Error;

    use crate::VSDiagnostic;

    #[test]
    fn test_missing_type() {
        let code = "namespace input { operation Foo(a) : Unit {} }";
        let mut error_callback_called = false;
        {
            let mut lang_serv = qsls::LanguageService::new(
                |_: &str, _: u32, diagnostics: &[Error]| {
                    error_callback_called = true;
                    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
                    let err = diagnostics.first().unwrap();
                    let diag = VSDiagnostic::from(err);

                    assert_eq!(diag.start_pos, 32);
                    assert_eq!(diag.end_pos, 33);
                    assert_eq!(diag.message, "type error: missing type in item signature\\\\n\\\\nhelp: types cannot be inferred for global declarations");
                },
            );
            lang_serv.update_document("<code>", 1, code);
        }
        assert!(error_callback_called)
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

    // #[test]
    // fn fail_ry() {
    //     let code = "namespace Sample {
    //         operation main() : Result[] {
    //             use q1 = Qubit();
    //             Ry(q1);
    //             let m1 = M(q1);
    //             return [m1];
    //         }
    //     }";

    //     let errors = crate::check_code_internal(code);
    //     assert_eq!(errors.len(), 1, "{errors:#?}");

    //     let error = errors.first().unwrap();
    //     assert_eq!(error.start_pos, 111);
    //     assert_eq!(error.end_pos, 117);
    //     assert_eq!(
    //         error.message,
    //         "type error: expected (Double, Qubit), found Qubit"
    //     );
    // }

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
        assert!(result.is_ok());
    }
}
