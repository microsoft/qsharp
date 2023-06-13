// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use log::{error, LevelFilter, Log};
use miette::{Diagnostic, Severity};
use qsc::compile;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fmt::Write, iter, panic, sync::OnceLock};
use wasm_bindgen::prelude::*;

struct Logger(js_sys::Function);
// We'll always be on the same JavaScript thread where the logger function was passed in,
// so let's pretend js_sys::Function is thread-safe to make set_hook() happy.
unsafe impl Send for Logger {}
unsafe impl Sync for Logger {}
impl Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        self.0
            .call1(&JsValue::NULL, &JsValue::from(format!("{}", record.args())))
            .expect("logging callback should not fail");
    }

    fn flush(&self) {}
}

#[wasm_bindgen]
pub struct LanguageService(qsls::LanguageService<'static>);

static LOGGER_SET: OnceLock<bool> = OnceLock::new();

#[wasm_bindgen]
impl LanguageService {
    #[wasm_bindgen(constructor)]
    pub fn new(diagnostics_callback: &js_sys::Function, logger: &js_sys::Function) -> Self {
        if LOGGER_SET.set(true).is_ok() {
            log::set_boxed_logger(Box::new(Logger(logger.clone())))
                .expect("setting logger should succeed");
            log::set_max_level(LevelFilter::Trace);
        }

        panic::set_hook(Box::new(|info: &panic::PanicInfo| {
            error!("{}", info);
        }));

        let diagnostics_callback = diagnostics_callback.clone();
        let inner = qsls::LanguageService::new(
            move |uri: &str, version: u32, errors: &[compile::Error]| {
                let diags = errors.iter().map(VSDiagnostic::from).collect::<Vec<_>>();
                let _ = diagnostics_callback
                    .call3(
                        &JsValue::NULL,
                        &wasm_bindgen::JsValue::from(uri),
                        &wasm_bindgen::JsValue::from(version),
                        &serde_wasm_bindgen::to_value(&diags)
                            .expect("conversion to VSDiagnostic should succeed"),
                    )
                    .expect("callback should succeed");
            },
        );
        LanguageService(inner)
    }

    pub fn update_document(&mut self, uri: &str, version: u32, text: &str) {
        self.0.update_document(uri, version, text);
    }

    pub fn close_document(&mut self, uri: &str) {
        self.0.close_document(uri);
    }

    pub fn get_completions(&self, uri: &str, offset: u32) -> Result<JsValue, JsValue> {
        let completion_list = self.0.get_completions(uri, offset);
        Ok(serde_wasm_bindgen::to_value(&CompletionList {
            items: completion_list
                .items
                .into_iter()
                .map(|i| CompletionItem {
                    label: i.label,
                    kind: (match i.kind {
                        qsls::completion::CompletionItemKind::Function => "function",
                        qsls::completion::CompletionItemKind::Module => "module",
                        qsls::completion::CompletionItemKind::Keyword => "keyword",
                        qsls::completion::CompletionItemKind::Issue => "issue",
                    })
                    .to_string(),
                })
                .collect(),
        })?)
    }

    pub fn get_definition(&self, uri: &str, offset: u32) -> Result<JsValue, JsValue> {
        let definition = self.0.get_definition(uri, offset);
        Ok(match definition {
            Some(definition) => serde_wasm_bindgen::to_value(&Definition {
                source: definition.source,
                offset: definition.offset,
            })?,
            None => JsValue::NULL,
        })
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
        kind: "function" | "module" | "keyword" | "issue";
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
    pub kind: String,
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
    pub severity: String,
}

impl VSDiagnostic {
    pub fn json(&self) -> serde_json::Value {
        json!({
            "message": self.message,
            "severity": self.severity,
            "start_pos": self.start_pos,
            "end_pos": self.end_pos
        })
    }
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
        let severity = (match err.severity().unwrap_or(Severity::Error) {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Advice => "info",
        })
        .to_string();

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
