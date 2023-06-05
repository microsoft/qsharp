// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::language_service;
use miette::{Diagnostic, Severity};
use qsc::compile::Error;
use serde::{Deserialize, Serialize};
use std::{fmt::Write, iter};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct QSharpLanguageService(language_service::QSharpLanguageService<'static>);

#[wasm_bindgen]
impl QSharpLanguageService {
    #[wasm_bindgen(constructor)]
    pub fn new(diagnostics_callback: &js_sys::Function) -> Self {
        let diagnostics_callback = diagnostics_callback.clone();
        let inner = language_service::QSharpLanguageService::new(move |errors: &[Error]| {
            let diags = errors.iter().map(VSDiagnostic::from).collect::<Vec<_>>();
            let value = serde_wasm_bindgen::to_value(&diags)
                .expect("conversion to VSDiagnostic should succeed");
            diagnostics_callback
                .call1(&JsValue::null(), &value)
                .expect("callback should succeed");
        });
        QSharpLanguageService(inner)
    }

    pub fn update_code(&mut self, uri: &str, code: &str) {
        self.0.update_code(uri, code);
    }

    pub fn get_completions(&self, uri: &str, offset: u32) -> Result<JsValue, JsValue> {
        let res = self.0.get_completions(uri, offset);
        Ok(serde_wasm_bindgen::to_value(&CompletionList {
            items: res
                .items
                .into_iter()
                .map(|i| CompletionItem {
                    label: i.label,
                    kind: i.kind,
                })
                .collect(),
        })?)
    }
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

#[derive(Serialize, Deserialize)]
pub struct Hover {
    pub contents: String,
    pub span: Span,
}

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
