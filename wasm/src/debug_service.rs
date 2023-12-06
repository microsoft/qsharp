// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use log::{error, trace};
use qsc::fir::StmtId;
use qsc::interpret::stateful::Interpreter;
use qsc::interpret::{stateful, StepAction, StepResult};
use qsc::{fmt_complex, PackageType, SourceMap, TargetProfile};
use qsc_project::{FileSystemAsync, ProjectSystemCallbacks};

use crate::debug_service::project_system::DebugServiceProjectLoader;
use crate::project_system::{GetManifestCallback, ListDirectoryCallback, ReadFileCallback};
use crate::{serializable_type, CallbackReceiver};
use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct DebugService {
    interpreter: Interpreter,
}
mod project_system {
    use std::{future::Future, pin::Pin, sync::Arc};

    // Copyright (c) Microsoft Corporation.
    // Licensed under the MIT License.
    use crate::project_system::*;
    use async_trait::async_trait;
    use qsc_project::{JSFileEntry, ManifestDescriptor};
    use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

    /// the desugared return type of an "async fn"
    type PinnedFuture<T> = Pin<Box<dyn Future<Output = T>>>;

    /// represents a unary async fn where `Arg` is the input
    /// parameter and `Return` is the return type. The lifetime
    /// `'a` represents the lifetime of the contained `dyn Fn`.
    type AsyncFunction<'a, Arg, Return> = Box<dyn Fn(Arg) -> PinnedFuture<Return> + 'a>;

    /// There are some differences in the implementation here versus the one in the language service.
    /// Because the debugger itself is bound with `wasm_bindgen`, we can't directly store
    /// non-serializeable types on the struct.
    pub struct DebugServiceProjectLoader<'a> {
        /// Callback which lets the service read a file from the target filesystem
        read_file_callback: AsyncFunction<'a, String, (Arc<str>, Arc<str>)>,
        /// Callback which lets the service list directory contents
        /// on the target file system
        list_directory: AsyncFunction<'a, String, Vec<JSFileEntry>>,
        /// Fetch the manifest file for a specific path
        get_manifest_cb: AsyncFunction<'a, String, Option<qsc_project::ManifestDescriptor>>,
    }

    #[async_trait(?Send)]
    impl qsc_project::FileSystemAsync for DebugServiceProjectLoader<'_> {
        type Entry = JSFileEntry;
        async fn read_file(
            &self,
            path: &std::path::Path,
        ) -> miette::Result<(std::sync::Arc<str>, std::sync::Arc<str>)> {
            Ok((self.read_file_callback)(path.to_string_lossy().to_string()).await)
        }

        async fn list_directory(&self, path: &std::path::Path) -> miette::Result<Vec<Self::Entry>> {
            Ok((self.list_directory)(path.to_string_lossy().to_string()).await)
        }
    }

    impl DebugServiceProjectLoader<'_> {
        pub fn new(
            read_file: ReadFileCallback,
            list_directory: ListDirectoryCallback,
            get_manifest: GetManifestCallback,
        ) -> Self {
            let read_file = read_file.into();
            let read_file = into_async_rust_fn_with!(read_file, read_file_transformer);
            let list_directory = list_directory.into();
            let list_directory =
                into_async_rust_fn_with!(list_directory, list_directory_transformer);
            let get_manifest = get_manifest.into();
            let get_manifest = into_async_rust_fn_with!(get_manifest, get_manifest_transformer);

            Self {
                read_file_callback: Box::new(read_file),
                list_directory: Box::new(list_directory),
                get_manifest_cb: Box::new(get_manifest),
            }
        }
        pub async fn get_manifest(&self, uri: &str) -> Option<ManifestDescriptor> {
            (self.get_manifest_cb)(uri.to_string()).await
        }
    }
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

    pub async fn load_source(
        &mut self,
        path: String,
        source: String,
        target_profile: String,
        entry: Option<String>,
        read_file: ReadFileCallback,
        list_directory: ListDirectoryCallback,
        get_manifest: GetManifestCallback,
    ) -> String {
        let loader = DebugServiceProjectLoader::new(read_file, list_directory, get_manifest);
        let manifest = loader.get_manifest(&path).await;
        let sources = if let Some(ref manifest) = manifest {
            match loader.load_project(manifest).await {
                Ok(o) => o.sources,
                Err(e) => {
                    error!("failed to load manifest: {e:?}, defaulting to single-file mode");
                    vec![(path.into(), source.into())]
                }
            }
        } else {
            trace!("Running in single file mode");
            vec![(path.into(), source.into())]
        };

        let source_map = SourceMap::new(sources, entry.as_deref().map(|value| value.into()));
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
