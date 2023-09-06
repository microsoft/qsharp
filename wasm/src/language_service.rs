// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::serializable_type;
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
                        &uri.into(),
                        &version.into(),
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

    pub fn get_completions(&self, uri: &str, offset: u32) -> ICompletionList {
        let completion_list = self.0.get_completions(uri, offset);
        CompletionList {
            items: completion_list
                .items
                .into_iter()
                .map(|i| CompletionItem {
                    label: i.label,
                    kind: (match i.kind {
                        qsls::protocol::CompletionItemKind::Function => "function",
                        qsls::protocol::CompletionItemKind::Interface => "interface",
                        qsls::protocol::CompletionItemKind::Keyword => "keyword",
                        qsls::protocol::CompletionItemKind::Module => "module",
                    })
                    .to_string(),
                    sortText: i.sort_text,
                    detail: i.detail,
                    additionalTextEdits: i.additional_text_edits.map(|edits| {
                        edits
                            .into_iter()
                            .map(|(span, text)| TextEdit {
                                range: Span {
                                    start: span.start,
                                    end: span.end,
                                },
                                newText: text,
                            })
                            .collect()
                    }),
                })
                .collect(),
        }
        .into()
    }

    pub fn get_definition(&self, uri: &str, offset: u32) -> Option<IDefinition> {
        let definition = self.0.get_definition(uri, offset);
        definition.map(|definition| {
            Definition {
                source: definition.source,
                offset: definition.offset,
            }
            .into()
        })
    }

    pub fn get_hover(&self, uri: &str, offset: u32) -> Option<IHover> {
        let hover = self.0.get_hover(uri, offset);
        hover.map(|hover| {
            Hover {
                contents: hover.contents,
                span: Span {
                    start: hover.span.start,
                    end: hover.span.end,
                },
            }
            .into()
        })
    }
}

serializable_type! {
    struct CompletionList {
        pub items: Vec<CompletionItem>,
    },
    r#"export interface ICompletionList {
        items: ICompletionItem[]
    }"#,
    CompletionList,
    ICompletionList
}

serializable_type! {
    struct CompletionItem {
        pub label: String,
        pub kind: String,
        pub sortText: Option<String>,
        pub detail: Option<String>,
        pub additionalTextEdits: Option<Vec<TextEdit>>,
    },
    r#"export interface ICompletionItem {
        label: string;
        kind: "function" | "interface" | "keyword" | "module";
        sortText?: string;
        detail?: string;
        additionalTextEdits?: ITextEdit[];
    }"#
}

serializable_type! {
    struct TextEdit {
        pub range: Span,
        pub newText: String,
    },
    r#"export interface ITextEdit {
        range: ISpan;
        newText: string;
    }"#
}

serializable_type! {
    struct Hover {
        pub contents: String,
        pub span: Span,
    },
    r#"export interface IHover {
        contents: string;
        span: ISpan
    }"#,
    Hover,
    IHover
}

serializable_type! {
    struct Definition {
        pub source: String,
        pub offset: u32,
    },
    r#"export interface IDefinition {
        source: string;
        offset: number;
    }"#,
    Definition,
    IDefinition
}

serializable_type! {
    struct Span {
        pub start: u32,
        pub end: u32,
    },
    r#"export interface ISpan {
        start: number;
        end: number;
    }"#
}

serializable_type! {
    pub(crate) struct VSDiagnostic {
        pub start_pos: usize,
        pub end_pos: usize,
        pub message: String,
        pub severity: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub code: Option<VSDiagnosticCode>,
    },
    r#"export interface IDiagnostic {
        start_pos: number;
        end_pos: number;
        message: string;
        severity: "error" | "warning" | "info"
        code?: {
            value: string;
            target: string;
        }
    }"#
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct VSDiagnosticCode {
    value: String,
    target: String,
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
