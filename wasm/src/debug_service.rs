// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::fir::StmtId;
use qsc::interpret::stateful;
use qsc::interpret::{stateful::Interpreter, Value};
use qsc::{PackageType, SourceMap};
use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen::prelude::*;

use crate::{language_service::VSDiagnostic, CallbackReceiver};

#[wasm_bindgen]
pub struct DebugService {
    interpreter: Interpreter,
}

#[wasm_bindgen]
impl DebugService {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(false, SourceMap::default(), PackageType::Lib)
                .expect("Couldn't create interpreter"),
        }
    }

    pub fn load_source(&mut self, path: &str, source: &str) -> bool {
        let source_map = SourceMap::new([(path.into(), source.into())], None);
        match Interpreter::new(true, source_map, qsc::PackageType::Exe) {
            Ok(interpreter) => {
                self.interpreter = interpreter;
                self.interpreter.set_entry().is_ok()
            }
            Err(_) => false,
        }
    }

    pub fn get_stack_frames(&self) -> JsValue {
        let frames = self.interpreter.get_stack_frames();

        let list = StackFrameList {
            frames: frames
                .iter()
                .map(|s| StackFrame {
                    name: format!("{} {}", s.name, s.functor),
                    path: s.path.clone(),
                    lo: s.lo,
                    hi: s.hi,
                })
                .collect(),
        };
        serde_wasm_bindgen::to_value(&list).expect("failed to serialize stack frame list")
    }

    pub fn eval_continue(
        &mut self,
        event_cb: &js_sys::Function,
        ids: &[u32],
    ) -> Result<JsValue, JsValue> {
        if !event_cb.is_function() {
            return Err(JsError::new("Events callback function must be provided").into());
        }
        let bps: Vec<_> = ids.iter().map(|f| StmtId::from(*f)).collect();

        match self.run_internal(
            |msg: &str| {
                // See example at https://rustwasm.github.io/wasm-bindgen/reference/receiving-js-closures-in-rust.html
                let _ = event_cb.call1(&JsValue::null(), &JsValue::from(msg));
            },
            &bps,
        ) {
            Ok(None) => Ok(JsValue::UNDEFINED),
            Ok(Some(v)) => {
                // we've hit a breakpoint
                // Convert the stmt id to a number
                Ok(JsValue::from(std::convert::Into::<usize>::into(v)))
            }
            Err(e) => Err(JsError::from(&e[0]).into()),
        }
    }

    fn run_internal<F>(
        &mut self,
        event_cb: F,
        bps: &[StmtId],
    ) -> Result<Option<StmtId>, Vec<stateful::Error>>
    where
        F: Fn(&str),
    {
        let mut out = CallbackReceiver { event_cb };
        let result = self.interpreter.eval_continue(&mut out, bps);
        let mut success = true;
        let mut return_value = None;
        let msg: serde_json::Value = match &result {
            Ok(None) => {
                let value = self.interpreter.get_result();
                serde_json::Value::String(value.to_string())
            }
            Ok(value) => {
                return_value = *value;
                serde_json::Value::String(Value::unit().to_string())
            }
            Err(errors) => {
                // TODO: handle multiple errors
                // https://github.com/microsoft/qsharp/issues/149
                success = false;
                VSDiagnostic::from(&errors[0]).json()
            }
        };

        let msg_string = json!({"type": "Result", "success": success, "result": msg}).to_string();
        (out.event_cb)(&msg_string);
        match &result {
            Ok(_) => Ok(return_value),
            Err(errors) => Err(Vec::from_iter(errors.iter().cloned())),
        }
    }

    pub fn get_breakpoints(&self, path: &str) -> JsValue {
        let bps = self.interpreter.get_breakpoints(path);

        let spans = BreakpointSpanList {
            spans: bps
                .iter()
                .map(|s| BreakpointSpan {
                    id: s.id,
                    lo: s.lo,
                    hi: s.hi,
                })
                .collect(),
        };
        serde_wasm_bindgen::to_value(&spans).expect("failed to serialize breakpoint location list")
    }
}

impl Default for DebugService {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen(typescript_custom_section)]
const IBreakpointSpanList: &'static str = r#"
export interface IBreakpointSpan {
    id: number;
    lo: number;
    hi: number;
}

export interface IBreakpointSpanList {
    spans: Array<IBreakpointSpan>
}
"#;

#[derive(Serialize, Deserialize)]
struct BreakpointSpanList {
    pub spans: Vec<BreakpointSpan>,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, Hash, PartialEq)]
struct BreakpointSpan {
    pub id: u32,
    pub lo: u32,
    pub hi: u32,
}

#[wasm_bindgen(typescript_custom_section)]
const IStackFrameList: &'static str = r#"
export interface IStackFrame {
    name: string;
    path: string;
    lo: number;
    hi: number;
}

export interface IStackFrameList {
    frames: Array<IStackFrame>
}
"#;

#[derive(Serialize, Deserialize)]
struct StackFrameList {
    pub frames: Vec<StackFrame>,
}

// Public fields implementing Copy have automatically generated getters/setters.
// To generate getters/setters for non-Copy public fields, we must
// use #[wasm_bindgen(getter_with_clone)] for the struct
#[derive(Serialize, Deserialize, Clone, Debug, Eq, Hash, PartialEq)]
struct StackFrame {
    pub name: String,
    pub path: String,
    pub lo: u32,
    pub hi: u32,
}
