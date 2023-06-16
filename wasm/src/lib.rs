// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use js_sys::Function;
use katas::verify_exercise;
use log::{debug, LevelFilter};
use miette::{Diagnostic, Severity};
use num_bigint::BigUint;
use num_complex::Complex64;
use qsc::{
    compile,
    hir::PackageId,
    interpret::{
        output::{self, Receiver},
        stateless,
    },
    telemetry, PackageStore, SourceMap,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    cell::{OnceCell, RefCell},
    fmt::Write,
    iter,
};
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

// Holds a reference to the JavaScript function to call (which must be thread specific)
thread_local! {
    static TELEM_FN: OnceCell<Function> = OnceCell::new();
}

// The global logger that delegates to the thread local JS function (if present and enabled)
struct WasmTelemetryLogger;
impl telemetry::Log for WasmTelemetryLogger {
    fn log(&self, msg: &str) {
        if telemetry::is_telemetry_enabled() {
            TELEM_FN.with(|f| {
                if let Some(jsfn) = f.get() {
                    let _ = jsfn.call1(&JsValue::NULL, &JsValue::from_str(msg));
                }
            });
        }
    }
}
static WASM_TELEMETRY_LOGGER: WasmTelemetryLogger = WasmTelemetryLogger;

#[wasm_bindgen(js_name=initTelemetry)]
pub fn init_telemetry(callback: JsValue) -> Result<(), JsError> {
    // Ensure a function was passed, and set it in the thread local storage
    if !callback.is_function() {
        return Err(JsError::new("Invalid telemetry callback provided"));
    }

    let thefn: Function = callback.dyn_into().unwrap();
    TELEM_FN.with(|f| f.set(thefn)).map_err(|_| {
        JsError::new("attempted to assign the telemetry handler after it was already assigned")
    })?;

    // Ensure that the global logger is set (at most once).
    telemetry::set_telemetry_logger(&WASM_TELEMETRY_LOGGER).map_err(JsError::new)?;

    Ok(())
}

// ******** LOGGING WebAssembly code *********

#[wasm_bindgen]
extern "C" {
    type Error;
    #[wasm_bindgen(constructor)]
    fn new() -> Error;

    #[wasm_bindgen(structural, method, getter)]
    fn stack(error: &Error) -> String;
}

static MY_LOGGER: MyLogger = MyLogger;

// We're in Wasm, so only one thread anyway, but needed to avoid errors without Sync trait on RefCell
thread_local! {
    // Will hold a reference to the JS logging function that was passed in
    static LOG_JS_FN: RefCell<Option<Function>> = RefCell::new(None);
}

struct MyLogger;

impl log::Log for MyLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        // We only get here if logging is enabled, and thus there is a function to call, so a
        // call to the JavaScript side is definitely going to happen here. Hence the relative
        // perf cost of unwrapping the thread_local RefCell is probably negligible.
        LOG_JS_FN.with(|f| {
            let fnborrow = f.borrow();
            if let Some(js_fn) = fnborrow.as_ref() {
                let msg = format!("{}", record.args());
                let level = record.level() as i32;
                let _ = js_fn.call2(&JsValue::NULL, &JsValue::from(level), &JsValue::from(msg));
            }
        });
    }

    fn flush(&self) {}
}

pub fn hook(info: &std::panic::PanicInfo) {
    // Code similar to https://github.com/rustwasm/console_error_panic_hook/blob/master/src/lib.rs#L97
    // for capturing the JS stack as well as the panic info
    let mut msg = info.to_string();
    msg.push_str("\n\nStack:\n\n");
    let e = Error::new();
    let stack = e.stack();
    msg.push_str(&stack);
    msg.push_str("\n\n");

    // Log message to both the logger and to telemetry
    let err_text = format!("Wasm panic occurred: {}", msg);
    log::error!("{}", &err_text);
    telemetry::log(&err_text);
}

#[wasm_bindgen(js_name=initLogging)]
pub fn init_logging(callback: JsValue, level: i32) -> Result<(), JsError> {
    if !callback.is_function() {
        return Err(JsError::new("Invalid callback"));
    }

    if !(0..=5).contains(&level) {
        return Err(JsError::new("Invalid logging level"));
    }

    let thefn: Function = callback.dyn_into().unwrap(); // Already checked it was a function
    LOG_JS_FN.with(|f| {
        *f.borrow_mut() = Option::Some(thefn);
    });

    // The below will return an error if it was already set
    log::set_logger(&MY_LOGGER).map_err(|e| JsError::new(&e.to_string()))?;
    std::panic::set_hook(Box::new(hook));

    set_log_level(level);
    Ok(())
}

#[wasm_bindgen(js_name=setLogLevel)]
pub fn set_log_level(level: i32) {
    // TODO: Maybe accept a string here too for user-friendliness
    log::set_max_level(match level {
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5 => LevelFilter::Trace,
        _ => LevelFilter::Off,
    });
    log::info!("Log level set to {}", level);
}

#[wasm_bindgen]
pub fn git_hash() -> JsValue {
    let git_hash = env!("QSHARP_GIT_HASH");
    telemetry::log(format!("git_hash: \"{}\"", git_hash).as_str());
    JsValue::from_str(git_hash)
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
    severity: "error" | "warning" | "info"
    code?: {
        value: string;
        target: string;
    }
}
"#;

#[derive(Debug, Serialize, Deserialize)]
pub struct VSDiagnosticCode {
    value: String,
    target: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VSDiagnostic {
    pub start_pos: usize,
    pub end_pos: usize,
    pub message: String,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<VSDiagnosticCode>,
}

impl VSDiagnostic {
    pub fn json(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("serializing VSDiagnostic should succeed")
    }
}

impl<T> From<&T> for VSDiagnostic
where
    T: Diagnostic,
{
    fn from(err: &T) -> Self {
        let label = err.labels().and_then(|mut ls| ls.next());
        let offset = label.as_ref().map_or(0, |lbl| lbl.offset());
        let len = label.as_ref().map_or(1, |lbl| lbl.len().max(1));
        let severity = (match err.severity().unwrap_or(Severity::Error) {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Advice => "info",
        })
        .to_string();

        let mut message = err.to_string();
        for source in iter::successors(err.source(), |e| e.source()) {
            write!(message, ": {source}").expect("message should be writable");
        }
        if let Some(help) = err.help() {
            write!(message, "\n\nhelp: {help}").expect("message should be writable");
        }

        let code = err.code().map(|code| VSDiagnosticCode {
            value: code.to_string(),
            target: "".to_string(),
        });

        VSDiagnostic {
            start_pos: offset,
            end_pos: offset + len,
            severity,
            message,
            code,
        }
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
        let (unit, errors) = compile::compile(store, &[*std], sources);
        (
            unit.package,
            errors.into_iter().map(|error| (&error).into()).collect(),
        )
    })
}

#[wasm_bindgen]
pub fn check_code(code: &str) -> Result<JsValue, JsValue> {
    debug!("In check_code");
    let (_, diags) = compile(code);
    Ok(serde_wasm_bindgen::to_value(&diags)?)
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
    exercise_implementation: &str,
    event_cb: impl Fn(&str),
) -> Result<bool, Vec<stateless::Error>> {
    verify_exercise(
        vec![
            ("exercise".into(), exercise_implementation.into()),
            ("verifier".into(), verification_source.into()),
        ],
        &mut CallbackReceiver { event_cb },
    )
}

#[wasm_bindgen]
pub fn run_kata_exercise(
    verification_source: &str,
    exercise_implementation: &str,
    event_cb: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    match run_kata_exercise_internal(verification_source, exercise_implementation, |msg: &str| {
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
        let (_, diag) = crate::compile(code);
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
        assert!(result.is_ok());
    }
}
