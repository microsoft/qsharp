// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::fir::StmtId;
use qsc::interpret::stateful::Interpreter;
use qsc::interpret::{stateful, StepAction, StepResult};
use qsc::{fmt_complex, PackageType, SourceMap, TargetProfile};

use crate::{serializable_type, CallbackReceiver};
use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen::prelude::*;

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

    pub fn load_source(
        &mut self,
        path: String,
        source: String,
        target_profile: String,
        entry: Option<String>,
    ) -> String {
        let source_map = SourceMap::new(
            [(path.into(), source.into())],
            entry.as_deref().map(|value| value.into()),
        );
        let target = match target_profile.as_str() {
            "base" => TargetProfile::Base,
            "full" => TargetProfile::Full,
            _ => panic!("Invalid target : {}", target_profile),
        };
        match Interpreter::new(true, source_map, qsc::PackageType::Exe, target) {
            Ok(interpreter) => {
                self.interpreter = interpreter;
                match self.interpreter.set_entry() {
                    Ok(()) => "".to_string(),
                    Err(e) => render_errors(e),
                }
            }
            Err(e) => render_errors(e),
        }
    }

    pub fn capture_quantum_state(&mut self) -> IQuantumStateList {
        let state = self.interpreter.capture_quantum_state();
        let entries = state
            .0
            .iter()
            .map(|(id, value)| QuantumState {
                name: qsc::format_state_id(id, state.1),
                value: fmt_complex(value),
            })
            .collect::<Vec<_>>();

        QuantumStateList { entries }.into()
    }

    pub fn get_stack_frames(&self) -> IStackFrameList {
        let frames = self.interpreter.get_stack_frames();

        StackFrameList {
            frames: frames
                .iter()
                .map(|s| StackFrame {
                    name: format!("{} {}", s.name, s.functor),
                    path: s.path.clone(),
                    lo: s.lo,
                    hi: s.hi,
                })
                .collect(),
        }
        .into()
    }

    pub fn eval_next(
        &mut self,
        event_cb: &js_sys::Function,
        ids: &[u32],
    ) -> Result<IStructStepResult, JsValue> {
        self.eval(event_cb, ids, StepAction::Next)
    }

    pub fn eval_continue(
        &mut self,
        event_cb: &js_sys::Function,
        ids: &[u32],
    ) -> Result<IStructStepResult, JsValue> {
        self.eval(event_cb, ids, StepAction::Continue)
    }

    pub fn eval_step_in(
        &mut self,
        event_cb: &js_sys::Function,
        ids: &[u32],
    ) -> Result<IStructStepResult, JsValue> {
        self.eval(event_cb, ids, StepAction::In)
    }

    pub fn eval_step_out(
        &mut self,
        event_cb: &js_sys::Function,
        ids: &[u32],
    ) -> Result<IStructStepResult, JsValue> {
        self.eval(event_cb, ids, StepAction::Out)
    }

    fn eval(
        &mut self,
        event_cb: &js_sys::Function,
        ids: &[u32],
        step: StepAction,
    ) -> Result<IStructStepResult, JsValue> {
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
            Ok(value) => Ok(StructStepResult::from(value).into()),
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
                errors[0]
                    .stack_trace()
                    .clone()
                    .map(serde_json::Value::String)
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

    pub fn get_breakpoints(&self, path: &str) -> IBreakpointSpanList {
        let bps = self.interpreter.get_breakpoints(path);

        BreakpointSpanList {
            spans: bps
                .iter()
                .map(|s| BreakpointSpan {
                    id: s.id,
                    lo: s.lo,
                    hi: s.hi,
                })
                .collect(),
        }
        .into()
    }

    pub fn get_locals(&self) -> IVariableList {
        let locals = self.interpreter.get_locals();
        let variables: Vec<_> = locals
            .into_iter()
            .map(|local| Variable {
                name: (*local.name).to_string(),
                value: local.value.to_string(),
                var_type: local.type_name,
            })
            .collect();
        VariableList { variables }.into()
    }
}

impl Default for DebugService {
    fn default() -> Self {
        Self::new()
    }
}

fn render_errors(errors: Vec<qsc::interpret::stateful::Error>) -> String {
    let mut msg = String::new();
    for error in errors {
        let error_string = render_error(error);
        msg.push_str(&error_string);
    }
    msg
}

fn render_error(error: qsc::interpret::stateful::Error) -> String {
    format!("{:?}\n", miette::Report::new(error))
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

serializable_type! {
    StructStepResult,
    {
        pub id: usize,
        pub value: usize,
    },
    r#"export interface IStructStepResult {
        id: number;
        value: number;
    }"#,
    IStructStepResult
}

serializable_type! {
    BreakpointSpanList,
    {
        pub spans: Vec<BreakpointSpan>,
    },
    r#"export interface IBreakpointSpanList {
        spans: Array<IBreakpointSpan>
    }
    "#,
    IBreakpointSpanList
}

serializable_type! {
    BreakpointSpan,
    {
        pub id: u32,
        pub lo: u32,
        pub hi: u32,
    },
    r#"export interface IBreakpointSpan {
        id: number;
        lo: number;
        hi: number;
    }"#
}

serializable_type! {
    StackFrameList,
    {
        pub frames: Vec<StackFrame>,
    },
    r#"export interface IStackFrameList {
        frames: Array<IStackFrame>
    }
    "#,
    IStackFrameList
}

serializable_type! {
    StackFrame,
    {
        pub name: String,
        pub path: String,
        pub lo: u32,
        pub hi: u32,
    },
    r#"export interface IStackFrame {
        name: string;
        path: string;
        lo: number;
        hi: number;
    }"#
}

serializable_type! {
    VariableList,
    {
        pub variables: Vec<Variable>,
    },
    r#"export interface IVariableList {
        variables: Array<IVariable>
    }"#,
    IVariableList
}

serializable_type! {
    Variable,
    {
        pub name: String,
        pub value: String,
        pub var_type: String,
    },
    r#"export interface IVariable {
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
    }"#
}

serializable_type! {
    QuantumStateList,
    {
        pub entries: Vec<QuantumState>,
    },
    r#"export interface IQuantumStateList {
        entries: Array<IQuantumState>
    }"#,
    IQuantumStateList
}

serializable_type! {
    QuantumState,
    {
        pub name: String,
        pub value: String,
    },
    r#"export interface IQuantumState {
        name: string;
        value: string;
    }"#
}
