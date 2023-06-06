// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use katas::{run_kata, KATA_ENTRY};
use language_service_wasm::VSDiagnostic;
use num_bigint::BigUint;
use num_complex::Complex64;
use qsc::{
    interpret::{
        output::{self, Receiver},
        stateless,
    },
    SourceMap,
};
use std::fmt::Write;
use wasm_bindgen::prelude::*;

mod language_service_wasm;

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
    severity: number; // [0, 1, 2] = [error, warning, info]
    code?: {
        value: number;  // Can also be a string, but number would be preferable
        target: string; // URI for more info - could be a custom URI for pretty errors
    }
}
"#;

impl std::fmt::Display for VSDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"{{
    "message": "{}",
    "severity": {},
    "start_pos": {},
    "end_pos": {}
}}"#,
            self.message, self.severity, self.start_pos, self.end_pos
        )
    }
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
        let mut msg_str = String::new();
        write!(msg_str, r#"{{"type": "Message", "message": "{}"}}"#, msg)
            .expect("Writing to a string should succeed");
        (self.event_cb)(&msg_str);
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
        let msg = format!(
            r#"{{"type": "Result", "success": false, "result": {}}}"#,
            diag
        );
        (out.event_cb)(&msg);
        return Err(e);
    }
    let context = context.expect("context should be valid");
    for _ in 0..shots {
        let result = context.eval(&mut out);
        let mut success = true;
        let msg = match result {
            Ok(value) => format!(r#""{value}""#),
            Err(errors) => {
                // TODO: handle multiple errors
                // https://github.com/microsoft/qsharp/issues/149
                success = false;
                VSDiagnostic::from(&errors[0]).to_string()
            }
        };

        let msg_string = format!(r#"{{"type": "Result", "success": {success}, "result": {msg}}}"#);
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
            let mut lang_serv = language_service::QSharpLanguageService::new(
                |diagnostics: &[Error]| {
                    error_callback_called = true;
                    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
                    let err = diagnostics.first().unwrap();
                    let diag = VSDiagnostic::from(err);

                    assert_eq!(diag.start_pos, 32);
                    assert_eq!(diag.end_pos, 33);
                    assert_eq!(diag.message, "type error: missing type in item signature\\\\n\\\\nhelp: types cannot be inferred for global declarations");
                },
                |_| {},
            );
            lang_serv.update_code("<code>", code);
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
                assert!(msg.contains(r#""type": "Result", "success": false"#));
                assert!(msg.contains(r#""message": "entry point not found"#));
                assert!(msg.contains(r#""start_pos": 0"#));
            },
            1,
        );
        assert!(result.is_ok());
    }
}
