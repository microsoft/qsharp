// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use num_bigint::BigUint;
use num_complex::Complex64;
use qsc_eval::output::Receiver;
use qsc_eval::{output, Error, Evaluator};
use qsc_frontend::compile::{compile, std, PackageStore};
use qsc_passes::globals::extract_callables;
use katas::verify_kata;

use miette::{Diagnostic, Severity};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::io;
use wasm_bindgen::prelude::*;

// TODO: Below is an example of how to return typed structures from Rust via Wasm
// to the consuming JavaScript/TypeScript code. To be replaced with the implementation.

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

#[derive(Serialize, Deserialize)]
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
    T: Diagnostic,
{
    fn from(err: &T) -> Self {
        let label = err
            .labels()
            .and_then(|mut ls| ls.next())
            .expect("error should have at least one label");
        let offset = label.offset();
        let len = label.len().max(1);
        let message = err.to_string();
        let severity = err.severity().unwrap_or(Severity::Error);

        VSDiagnostic {
            start_pos: offset,
            end_pos: offset + len,
            severity: severity as i32,
            message,
        }
    }
}

fn check_code_internal(code: &str) -> Vec<VSDiagnostic> {
    let mut store = PackageStore::new();
    let std = store.insert(std());
    let unit = compile(&store, [std], [code], "");

    let mut result: Vec<VSDiagnostic> = vec![];

    for err in unit.context.errors() {
        result.push(err.into());
    }
    result
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
    fn state(&mut self, state: Vec<(BigUint, Complex64)>) -> Result<(), output::Error> {
        let mut dump_json = String::new();
        write!(dump_json, r#"{{"type": "DumpMachine","state": {{"#)
            .expect("writing to string should succeed");
        let (last, most) = state
            .split_last()
            .expect("state should always have at least one entry");
        for state in most {
            write!(
                dump_json,
                r#""|{}⟩": [{}, {}],"#,
                state.0.to_str_radix(2),
                state.1.re,
                state.1.im
            )
            .expect("writing to string should succeed");
        }
        write!(
            dump_json,
            r#""|{}⟩": [{}, {}]}}}}"#,
            last.0.to_str_radix(2),
            last.1.re,
            last.1.im
        )
        .expect("writing to string should succeed");
        (self.event_cb)(&dump_json);
        Ok(())
    }

    fn message(&mut self, msg: String) -> Result<(), output::Error> {
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
    let mut store = PackageStore::new();
    let std = store.insert(std());
    let unit = compile(&store, [std], [code], expr);
    // TODO: Fail here with diagnostics if compile failed

    let user = store.insert(unit);
    let unit = store.get(user).expect("Fail");
    if let Some(expr) = &unit.package.entry {
        let globals = extract_callables(&store);
        let mut out = CallbackReceiver { event_cb };
        for _ in 0..shots {
            let evaluator = Evaluator::from_store(&store, user, &globals, &mut out);
            Evaluator::init();
            let mut success = true;
            let result: String;
            match &evaluator.eval_expr(expr) {
                Ok((val, _)) => {
                    result = format!(r#""{}""#, val);
                }
                Err(e) => {
                    success = false;
                    let diag: VSDiagnostic = e.into();
                    result = diag.to_string();
                }
            }
            let msg_string =
                format!(r#"{{"type": "Result", "success": {success}, "result": {result}}}"#);
            (out.event_cb)(&msg_string);
        }
        Ok(())
    } else {
        Err(Error::EmptyExpr)
    }
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

#[wasm_bindgen]
pub fn verify_kata_implementation(
    verification_source: &str,
    kata_implementation: &str
) -> Result<JsValue, JsValue> {
    let mut stdout = io::stdout();
    let mut out = output::GenericReceiver::new(&mut stdout);
    let succeeds = verify_kata(
        verification_source,
        kata_implementation,
        &mut out,
    );

    Ok(JsValue::from_bool(succeeds))
}

#[cfg(test)]
mod test {
    #[test]
    fn test_callable() {
        let code = "namespace input { operation Foo(a : Int -> Int) : Unit {} }";
        let diag = crate::check_code_internal(code);
        assert_eq!(diag.len(), 1);
        let err = diag.first().unwrap();

        assert_eq!(err.start_pos, 32);
        assert_eq!(err.end_pos, 46);
        assert!(err.message.starts_with("callables"));
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
        operation main() : Result {
            use q1 = Qubit();
            Ry(q1);
            let m1 = M(q1);
            return [m1];
        }
    }";
        let expr = "Sample.main()";
        let result = crate::run_internal(
            code,
            expr,
            |_msg_| {
                assert!(_msg_.contains(r#""type": "Result", "success": false"#));
                assert!(_msg_.contains(r#""message": "mismatched types""#));
                assert!(_msg_.contains(r#""start_pos": 99"#));
            },
            1,
        );
        assert!(result.is_ok());
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
}
