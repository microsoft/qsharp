// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::fir::StmtId;
use qsc::interpret::stateful::Interpreter;
use qsc::interpret::{stateful, StepAction, StepResult};
use qsc::{fmt_complex, PackageType, SourceMap, TargetProfile};

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
            interpreter: Interpreter::new(
                false,
                SourceMap::default(),
                PackageType::Lib,
                TargetProfile::Full,
            )
            .expect("Couldn't create interpreter"),
        }
    }

    pub fn load_source(&mut self, path: &str, source: &str) -> bool {
        let source_map = SourceMap::new([(path.into(), source.into())], None);
        match Interpreter::new(true, source_map, qsc::PackageType::Exe, TargetProfile::Full) {
            Ok(interpreter) => {
                self.interpreter = interpreter;
                self.interpreter.set_entry().is_ok()
            }
            Err(_) => false,
        }
    }

    pub fn capture_quantum_state(&mut self) -> JsValue {
        let state = self.interpreter.capture_quantum_state();
        let entries = state
            .0
            .iter()
            .map(|(id, value)| QuantumState {
                name: qsc::format_state_id(id, state.1),
                value: fmt_complex(value),
            })
            .collect::<Vec<_>>();

        let list = QuantumStateList { entries };
        serde_wasm_bindgen::to_value(&list).expect("failed to serialize quantum state list")
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

    pub fn eval_next(
        &mut self,
        event_cb: &js_sys::Function,
        ids: &[u32],
    ) -> Result<JsValue, JsValue> {
        self.eval(event_cb, ids, StepAction::Next)
    }

    pub fn eval_continue(
        &mut self,
        event_cb: &js_sys::Function,
        ids: &[u32],
    ) -> Result<JsValue, JsValue> {
        self.eval(event_cb, ids, StepAction::Continue)
    }

    pub fn eval_step_in(
        &mut self,
        event_cb: &js_sys::Function,
        ids: &[u32],
    ) -> Result<JsValue, JsValue> {
        self.eval(event_cb, ids, StepAction::In)
    }

    pub fn eval_step_out(
        &mut self,
        event_cb: &js_sys::Function,
        ids: &[u32],
    ) -> Result<JsValue, JsValue> {
        self.eval(event_cb, ids, StepAction::Out)
    }

    fn eval(
        &mut self,
        event_cb: &js_sys::Function,
        ids: &[u32],
        step: StepAction,
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
            step,
        ) {
            Ok(value) => Ok(JsValue::from(std::convert::Into::<StructStepResult>::into(
                value,
            ))),
            Err(e) => Err(JsError::from(&e[0]).into()),
        }
    }

    fn run_internal<F>(
        &mut self,
        event_cb: F,
        bps: &[StmtId],
        step: StepAction,
    ) -> Result<StepResult, Vec<stateful::Error>>
    where
        F: Fn(&str),
    {
        let mut out = CallbackReceiver { event_cb };
        let result = self.interpreter.eval_step(&mut out, bps, step);
        let mut success = true;

        let msg: Option<serde_json::Value> = match &result {
            Ok(value) => match value {
                qsc::interpret::StepResult::Return(value) => {
                    Some(serde_json::Value::String(value.to_string()))
                }
                _ => None,
            },
            Err(errors) => {
                // TODO: handle multiple errors
                // https://github.com/microsoft/qsharp/issues/149
                success = false;
                Some(VSDiagnostic::from(&errors[0]).json())
            }
        };
        if let Some(value) = msg {
            let msg_string =
                json!({"type": "Result", "success": success, "result": value}).to_string();
            (out.event_cb)(&msg_string);
        }

        match result {
            Ok(value) => Ok(value),
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

    pub fn get_locals(&self) -> JsValue {
        let locals = self.interpreter.get_locals();
        let variables: Vec<_> = locals
            .into_iter()
            .map(|local| Variable {
                name: (*local.name).to_string(),
                value: local.value.to_string(),
                var_type: local.type_name,
            })
            .collect();
        let variables = VariableList { variables };
        serde_wasm_bindgen::to_value(&variables).expect("failed to serialize variable list")
    }
}

impl Default for DebugService {
    fn default() -> Self {
        Self::new()
    }
}

impl From<StepResult> for StructStepResult {
    fn from(value: StepResult) -> Self {
        match value {
            StepResult::BreakpointHit(value) => StructStepResult {
                id: StepResultId::BreakpointHit.into(),
                value: Into::<usize>::into(value),
            },
            StepResult::Next => StructStepResult {
                id: StepResultId::Next.into(),
                value: 0,
            },
            StepResult::StepIn => StructStepResult {
                id: StepResultId::StepIn.into(),
                value: 0,
            },
            StepResult::StepOut => StructStepResult {
                id: StepResultId::StepOut.into(),
                value: 0,
            },
            StepResult::Return(_) => StructStepResult {
                id: StepResultId::Return.into(),
                value: 0,
            },
        }
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum StepResultId {
    BreakpointHit = 0,
    Next = 1,
    StepIn = 2,
    StepOut = 3,
    Return = 4,
}

impl From<StepResultId> for usize {
    fn from(val: StepResultId) -> Self {
        val as usize
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
struct StructStepResult {
    pub id: usize,
    pub value: usize,
}

#[wasm_bindgen(typescript_custom_section)]
const IStructStepResult: &'static str = r#"
export interface IStructStepResult {
    id: number;
    value: number;
}
"#;

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

#[wasm_bindgen(typescript_custom_section)]
const IVariableList: &'static str = r#"
export interface IVariable {
    name: string;
    value: string;
    var_type: "Array"
        | "BigInt"
        | "Bool"
        | "Closure"
        | "Double"
        | "Global"
        | "Int"
        | "Pauli"
        | "Qubit"
        | "Range"
        | "Result"
        | "String"
        | "Tuple";
}

export interface IVariableList {
    variables: Array<IVariable>
}
"#;

#[derive(Serialize, Deserialize)]
struct VariableList {
    pub variables: Vec<Variable>,
}

// Public fields implementing Copy have automatically generated getters/setters.
// To generate getters/setters for non-Copy public fields, we must
// use #[wasm_bindgen(getter_with_clone)] for the struct
#[derive(Serialize, Deserialize, Clone, Debug, Eq, Hash, PartialEq)]
struct Variable {
    pub name: String,
    pub value: String,
    pub var_type: String,
}

#[wasm_bindgen(typescript_custom_section)]
const IQuantumState: &'static str = r#"
export interface IQuantumState {
    name: string;
    value: string;
}
export interface IQuantumStateList {
    entries: Array<IQuantumState>
}
"#;

#[derive(Serialize, Deserialize)]
struct QuantumStateList {
    pub entries: Vec<QuantumState>,
}

// Public fields implementing Copy have automatically generated getters/setters.
// To generate getters/setters for non-Copy public fields, we must
// use #[wasm_bindgen(getter_with_clone)] for the struct
#[derive(Serialize, Deserialize, Clone, Debug, Eq, Hash, PartialEq)]
struct QuantumState {
    pub name: String,
    pub value: String,
}
