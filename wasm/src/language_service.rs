// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::{Diagnostic, Severity};
use qsc::{self, compile};
use serde::{Deserialize, Serialize};
use std::{fmt::Write, iter};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct LanguageService(qsls::LanguageService<'static>);

#[wasm_bindgen]
impl LanguageService {
    #[wasm_bindgen(constructor)]
    pub fn new(diagnostics_callback: &js_sys::Function) -> Self {
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

    pub fn update_document(&mut self, uri: &str, version: u32, text: &str, is_exe: bool) {
        self.0.update_document(
            uri,
            version,
            text,
            if is_exe {
                qsc::PackageType::Exe
            } else {
                qsc::PackageType::Lib
            },
        );
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
                        qsls::completion::CompletionItemKind::Interface => "interface",
                        qsls::completion::CompletionItemKind::Keyword => "keyword",
                        qsls::completion::CompletionItemKind::Module => "module",
                    })
                    .to_string(),
                    sortText: i.sort_text,
                    detail: i.detail,
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
        kind: "function" | "interface" | "keyword" | "module";
        sortText?: string;
        detail?: string;
    }>
}
"#;

#[derive(Serialize, Deserialize)]
pub struct CompletionList {
    pub items: Vec<CompletionItem>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)] // These types propagate to JS which expects camelCase
pub struct CompletionItem {
    pub label: String,
    pub sortText: Option<String>,
    pub kind: String,
    pub detail: Option<String>,
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
