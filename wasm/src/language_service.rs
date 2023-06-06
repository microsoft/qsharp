// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::{Diagnostic, Severity};
use qsc::compile;
use serde::{Deserialize, Serialize};
use std::{fmt::Write, iter, panic};
use wasm_bindgen::prelude::*;

// Wrapper for a JavaScript function that implements Send + Sync
// so that it can be used as a panic hook
struct JsPanicHook(js_sys::Function);

impl JsPanicHook {
    pub fn call1(&self, context: &JsValue, arg1: &JsValue) -> Result<JsValue, JsValue> {
        self.0.call1(context, arg1)
    }
}

// We'll always be on the same JavaScript thread where the logger function was passed in,
// so let's pretend js_sys::Function is thread-safe to make set_hook() happy.
unsafe impl Send for JsPanicHook {}
unsafe impl Sync for JsPanicHook {}

#[wasm_bindgen]
pub struct LanguageService(language_service::LanguageService<'static>);

#[wasm_bindgen]
impl LanguageService {
    #[wasm_bindgen(constructor)]
    pub fn new(diagnostics_callback: &js_sys::Function, logger: &js_sys::Function) -> Self {
        let diagnostics_callback = diagnostics_callback.clone();
        let logger = logger.clone();
        let panic_logger = JsPanicHook(logger.clone());
        let inner = language_service::LanguageService::new(
            move |errors: &[compile::Error]| {
                let diags = errors.iter().map(VSDiagnostic::from).collect::<Vec<_>>();
                let value = serde_wasm_bindgen::to_value(&diags)
                    .expect("conversion to VSDiagnostic should succeed");
                diagnostics_callback
                    .call1(&JsValue::null(), &value)
                    .expect("callback should succeed");
            },
            move |msg: &str| {
                logger
                    .call1(
                        &JsValue::null(),
                        &serde_wasm_bindgen::to_value(msg)
                            .expect("string conversion should succeed"),
                    )
                    .expect("callback should succeed");
            },
        );

        panic::set_hook(Box::new(move |info: &panic::PanicInfo| {
            panic_logger
                .call1(
                    &JsValue::null(),
                    &serde_wasm_bindgen::to_value(&info.to_string())
                        .expect("expected to be able to convert string to JsValue"),
                )
                .expect("panic logger failed, nothing else we can do");
        }));

        LanguageService(inner)
    }

    pub fn update_code(&mut self, uri: &str, code: &str) {
        self.0.update_code(uri, code);
    }

    pub fn get_completions(&self, uri: &str, offset: u32) -> Result<JsValue, JsValue> {
        let completion_list = self.0.get_completions(uri, offset);
        Ok(serde_wasm_bindgen::to_value(&CompletionList {
            items: completion_list
                .items
                .into_iter()
                .map(|i| CompletionItem {
                    label: i.label,
                    kind: i.kind,
                })
                .collect(),
        })?)
    }

    pub fn get_definition(&self, uri: &str, offset: u32) -> Result<JsValue, JsValue> {
        let definition = self.0.get_definition(uri, offset);
        Ok(serde_wasm_bindgen::to_value(&Definition {
            source: definition.source,
            offset: definition.offset,
        })?)
    }

    pub fn get_hover(&self, uri: &str, offset: u32) -> Result<JsValue, JsValue> {
        let hover = self.0.get_hover(uri, offset);
        Ok(match hover {
            Some(hover) => serde_wasm_bindgen::to_value(&Hover {
                contents: hover.contents,
                span: Span {
                    start: hover.span.start,
                    end: hover.span.end,
                },
            })?,
            None => JsValue::NULL,
        })
    }
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

#[derive(Serialize, Deserialize)]
pub struct CompletionList {
    pub items: Vec<CompletionItem>,
}

#[derive(Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: i32,
}

#[wasm_bindgen(typescript_custom_section)]
const IHover: &'static str = r#"
export interface IHover {
    contents: string;
    span: { start: number; end: number }
}
"#;

#[derive(Serialize, Deserialize)]
pub struct Hover {
    pub contents: String,
    pub span: Span,
}

#[wasm_bindgen(typescript_custom_section)]
const IDefinition: &'static str = r#"
export interface IDefinition {
    source: string;
    offset: number;
}
"#;

#[derive(Serialize, Deserialize)]
pub struct Definition {
    pub source: String,
    pub offset: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VSDiagnostic {
    pub start_pos: usize,
    pub end_pos: usize,
    pub message: String,
    pub severity: i32,
}

impl<T> From<&T> for VSDiagnostic
where
    T: Diagnostic,
{
    fn from(err: &T) -> Self {
        let label = err.labels().and_then(|mut ls| ls.next());
        let offset = label.as_ref().map_or(0, |lbl| lbl.offset());
        // Monaco handles 0-length diagnostics just fine...?
        let len = label.as_ref().map_or(1, |lbl| lbl.len());
        let severity = match err.severity().unwrap_or(Severity::Error) {
            Severity::Error => 0,
            Severity::Warning => 1,
            Severity::Advice => 2,
        };

        let mut pre_message = err.to_string();
        for source in iter::successors(err.source(), |e| e.source()) {
            write!(pre_message, ": {source}").expect("message should be writable");
        }
        if let Some(help) = err.help() {
            write!(pre_message, "\n\nhelp: {help}").expect("message should be writable");
        }

        // Newlines in JSON need to be double escaped
        // TODO: Maybe some other chars too: https://stackoverflow.com/a/5191059
        let message = pre_message.replace('\n', "\\\\n");

        VSDiagnostic {
            start_pos: offset,
            end_pos: offset + len,
            severity,
            message,
        }
    }
}
