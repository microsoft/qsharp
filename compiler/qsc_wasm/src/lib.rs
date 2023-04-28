// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use katas::run_kata;
use miette::{Diagnostic, Severity};
use num_bigint::BigUint;
use num_complex::Complex64;
use qsc::stateless::{self, compile_execution_context, eval_in_context, Error};
use qsc_eval::{
    output,
    output::{format_state_id, Receiver},
};
use qsc_frontend::compile::{PackageStore, SourceMap};
use qsc_hir::hir::PackageId;
use serde::{Deserialize, Serialize};
use std::{fmt::Write, iter};
use wasm_bindgen::prelude::*;

// These definitions match the values expected by VS Code and Monaco.
enum CompletionKind {
    Method = 1,
    Keyword = 13,
}

#[derive(Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: i32,
}

#[derive(Serialize, Deserialize)]
pub struct CompletionList {
    pub items: Vec<CompletionItem>,
}

// There is no easy way to serialize the result with serde_wasm_bindgen and get
// good TypeScript typing. Here we manually specify the type that the follow
// method will return. At the call-site in the TypeScript, the response should be
// cast to this type. (e.g., var result = get_completions() as ICompletionList).
// It does mean this type decl must be kept up to date with any structural changes.
#[wasm_bindgen(typescript_custom_section)]
const ICompletionList: &'static str = r#"
export interface ICompletionList {
    items: Array<{
        label: string;
        kind: number;
    }>
}
"#;

#[wasm_bindgen]
pub fn get_completions() -> Result<JsValue, JsValue> {
    let res = CompletionList {
        items: vec![
            CompletionItem {
                label: "CCNOT".to_string(),
                kind: CompletionKind::Method as i32,
            },
            CompletionItem {
                label: "CNOT".to_string(),
                kind: CompletionKind::Method as i32,
            },
            CompletionItem {
                label: "CZ".to_string(),
                kind: CompletionKind::Method as i32,
            },
            CompletionItem {
                label: "X".to_string(),
                kind: CompletionKind::Method as i32,
            },
            CompletionItem {
                label: "Y".to_string(),
                kind: CompletionKind::Method as i32,
            },
            CompletionItem {
                label: "Z".to_string(),
                kind: CompletionKind::Method as i32,
            },
            CompletionItem {
                label: "H".to_string(),
                kind: CompletionKind::Method as i32,
            },
            CompletionItem {
                label: "S".to_string(),
                kind: CompletionKind::Method as i32,
            },
            CompletionItem {
                label: "T".to_string(),
                kind: CompletionKind::Method as i32,
            },
            CompletionItem {
                label: "M".to_string(),
                kind: CompletionKind::Method as i32,
            },
            CompletionItem {
                label: "CheckZero".to_string(),
                kind: CompletionKind::Method as i32,
            },
            CompletionItem {
                label: "DumpMachine".to_string(),
                kind: CompletionKind::Method as i32,
            },
            CompletionItem {
                label: "Equal".to_string(),
                kind: CompletionKind::Method as i32,
            },
            CompletionItem {
                label: "Qubit".to_string(),
                kind: CompletionKind::Method as i32,
            },
            CompletionItem {
                label: "Reset".to_string(),
                kind: CompletionKind::Method as i32,
            },
            CompletionItem {
                label: "@EntryPoint".to_string(),
                kind: CompletionKind::Keyword as i32,
            },
            CompletionItem {
                label: "Adjoint".to_string(),
                kind: CompletionKind::Keyword as i32,
            },
            CompletionItem {
                label: "Controlled".to_string(),
                kind: CompletionKind::Keyword as i32,
            },
            CompletionItem {
                label: "Int".to_string(),
                kind: CompletionKind::Keyword as i32,
            },
            CompletionItem {
                label: "if".to_string(),
                kind: CompletionKind::Keyword as i32,
            },
            CompletionItem {
                label: "else".to_string(),
                kind: CompletionKind::Keyword as i32,
            },
            CompletionItem {
                label: "namespace".to_string(),
                kind: CompletionKind::Keyword as i32,
            },
            CompletionItem {
                label: "open".to_string(),
                kind: CompletionKind::Keyword as i32,
            },
            CompletionItem {
                label: "operation".to_string(),
                kind: CompletionKind::Keyword as i32,
            },
            CompletionItem {
                label: "return".to_string(),
                kind: CompletionKind::Keyword as i32,
            },
            CompletionItem {
                label: "use".to_string(),
                kind: CompletionKind::Keyword as i32,
            },
            CompletionItem {
                label: "Unit".to_string(),
                kind: CompletionKind::Keyword as i32,
            },
        ],
    };
    Ok(serde_wasm_bindgen::to_value(&res)?)
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

#[derive(Debug, Serialize, Deserialize)]
pub struct VSDiagnostic {
    pub start_pos: usize,
    pub end_pos: usize,
    pub message: String,
    pub severity: i32,
}

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

impl<T> From<&T> for VSDiagnostic
where
    T: Diagnostic + ?Sized,
{
    fn from(err: &T) -> Self {
        let label = err.labels().and_then(|mut ls| ls.next());
        let offset = label.as_ref().map_or(0, |lbl| lbl.offset());
        let len = label.as_ref().map_or(1, |lbl| lbl.len().max(1));
        let severity = err.severity().unwrap_or(Severity::Error);

        let mut message = err.to_string();
        for source in iter::successors(err.source(), |e| e.source()) {
            write!(message, ": {source}").expect("message should be writable");
        }
        if let Some(help) = err.help() {
            write!(message, "\n\nhelp: {help}").expect("message should be writable");
        }

        VSDiagnostic {
            start_pos: offset,
            end_pos: offset + len,
            severity: severity as i32,
            message,
        }
    }
}

fn check_code_internal(code: &str) -> Vec<VSDiagnostic> {
    thread_local! {
        static STORE_STD: (PackageStore, PackageId) = {
            let mut store = PackageStore::new();
            let std = store.insert(qsc::compile::std());
            (store, std)
        };
    }

    STORE_STD.with(|(store, std)| {
        let sources = SourceMap::new([("code".into(), code.into())], "".into());
        let (_, reports) = qsc::compile::compile(store, [*std], sources);
        reports
            .into_iter()
            .map(|r| VSDiagnostic::from(AsRef::<dyn Diagnostic + 'static>::as_ref(&r)))
            .collect()
    })
}

#[wasm_bindgen]
pub fn check_code(code: &str) -> Result<JsValue, JsValue> {
    let result = check_code_internal(code);
    Ok(serde_wasm_bindgen::to_value(&result)?)
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
                format_state_id(&state.0, qubit_count),
                state.1.re,
                state.1.im
            )
            .expect("writing to string should succeed");
        }
        write!(
            dump_json,
            r#""{}": [{}, {}]}}}}"#,
            format_state_id(&last.0, qubit_count),
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

fn run_internal<F>(code: &str, expr: &str, event_cb: F, shots: u32) -> Result<(), Error>
where
    F: Fn(&str),
{
    let mut out = CallbackReceiver { event_cb };
    let sources = SourceMap::new([("code".into(), code.into())], expr.into());
    let context = compile_execution_context(true, sources);
    if let Err(err) = context {
        // TODO: handle multiple errors
        // https://github.com/microsoft/qsharp/issues/149
        let e = err.0[0].clone();
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
        let result = eval_in_context(&context, &mut out);
        let mut success = true;
        let msg = match result {
            Ok(value) => format!(r#""{value}""#),
            Err(err) => {
                // TODO: handle multiple errors
                // https://github.com/microsoft/qsharp/issues/149
                let e = err.0[0].clone();
                success = false;
                let diag: VSDiagnostic = (&e).into();
                diag.to_string()
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

fn run_kata_exercise_internal<F>(
    verification_source: &str,
    kata_implementation: &str,
    event_cb: F,
) -> Result<bool, Vec<stateless::Error>>
where
    F: Fn(&str),
{
    let mut out = CallbackReceiver { event_cb };
    run_kata(kata_implementation, verification_source, &mut out)
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
    #[test]
    fn test_missing_type() {
        let code = "namespace input { operation Foo(a) : Unit {} }";
        let diag = crate::check_code_internal(code);
        assert_eq!(diag.len(), 1, "{diag:#?}");
        let err = diag.first().unwrap();

        assert_eq!(err.start_pos, 32);
        assert_eq!(err.end_pos, 33);
        assert_eq!(err.message, "type error: missing type in item signature\n\nhelp: types cannot be inferred for global declarations");
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

        let errors = crate::check_code_internal(code);
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
                assert!(msg.contains(
                    r#""message": "could not compile source code: entry point not found"#
                ));
                assert!(msg.contains(r#""start_pos": 0"#));
            },
            1,
        );
        assert!(result.is_ok());
    }
}
