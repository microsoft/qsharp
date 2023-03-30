// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use num_bigint::BigUint;
use num_complex::Complex64;
use qsc_eval::output::Receiver;
use qsc_eval::val::Value;
use qsc_eval::{output, Error, Evaluator};
use qsc_frontend::compile::{compile, std, PackageStore};
use qsc_passes::globals::extract_callables;

use miette::{Diagnostic, Severity};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
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

fn convert_err_to_diagnostic(err: &impl Diagnostic) -> VSDiagnostic {
    let label = err
        .labels()
        .and_then(|mut ls| ls.next())
        .expect("error should have at least one label");
    let offset = label.offset();
    let len = label.len();
    let message = err.to_string();
    let severity = err.severity().unwrap_or(Severity::Error);

    VSDiagnostic {
        start_pos: offset,
        end_pos: offset + len,
        severity: severity as i32,
        message,
    } 
}

fn check_code_internal(code: &str) -> Vec<VSDiagnostic> {
    let mut store = PackageStore::new();
    let std = store.insert(std());
    let unit = compile(&store, [std], [code], "");

    let mut result: Vec<VSDiagnostic> = vec![];

    for err in unit.context.errors() {
        result.push(convert_err_to_diagnostic(err));
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

fn run_internal<F>(code: &str, expr: &str, event_cb: F) -> Result<Value, Error>
where
    F: Fn(&str),
{
    let mut store = PackageStore::new();
    let std = store.insert(std());
    let unit = compile(&store, [std], [code], expr);

    let user = store.insert(unit);
    let unit = store.get(user).expect("Fail");
    if let Some(expr) = &unit.package.entry {
        let globals = extract_callables(&store);
        let mut out = CallbackReceiver { event_cb };
        let evaluator = Evaluator::from_store(&store, user, &globals, &mut out);

        match evaluator.eval_expr(expr) {
            Ok((value, _)) => Ok(value),
            Err(_e) => Err(_e),
        }
    } else {
        // TODO Correct error type/message here
        Err(Error::UserFail(
            "Runtime failure".to_string(),
            std::default::Default::default(),
        ))
    }
}

#[wasm_bindgen]
pub fn run(code: &str, expr: &str, event_cb: &js_sys::Function) -> Result<JsValue, JsValue> {
    // Passing the callback function for output is optional.
    let result = if event_cb.is_function() {
        run_internal(code, expr, |msg: &str| {
            // See example at https://rustwasm.github.io/wasm-bindgen/reference/receiving-js-closures-in-rust.html
            let js_this = JsValue::null();
            let js_dump = JsValue::from(msg);
            let _ = event_cb.call1(&js_this, &js_dump);
        })
    } else {
        run_internal(code, expr, |_msg: &str| ())
    };

    match result {
        Ok(val) => Ok(serde_wasm_bindgen::to_value(&val.to_string())?),
        Err(e) => Err(serde_wasm_bindgen::to_value(&convert_err_to_diagnostic(&e))?)
    }
}

#[wasm_bindgen]
pub fn it_will_fail(val: i32) -> Result<JsValue, JsValue> {
    if val >= 0 {
        Ok(serde_wasm_bindgen::to_value("When it works")?)
    } else {
        Err(serde_wasm_bindgen::to_value(&VSDiagnostic {
            start_pos: 10,
            end_pos: 20,
            severity: 1,
            message: "Tuple type incorrect".to_string()
        })?)
    }
}

#[test]
fn test_callable() {
    let code = "namespace input { operation Foo(a : Int -> Int) : Unit {} }";
    let diag = check_code_internal(code);
    assert_eq!(diag.len(), 1);
    let err = diag.first().unwrap();

    assert_eq!(err.start_pos, 32);
    assert_eq!(err.end_pos, 46);
    assert!(err.message.starts_with("callables"));
}

#[test]
fn test_run() {
    let code = "
namespace Test {
    function Answer() : Int {
        return 42;
    }
}
";
    let expr = "Test.Answer()";
    let _result = run_internal(code, expr, |_msg| {});
    match _result.unwrap() {
        Value::Int(x) => assert_eq!(x, 42),
        _ => panic!("Incorrect value type returned"),
    }
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
    let result = run_internal(code, expr,|_msg_| {});
    assert!(result.is_err());
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
    let result = run_internal(code, expr,|_msg_| {
        assert!(_msg_.contains("hi"));
    });
    assert!(result.is_ok()); 
}