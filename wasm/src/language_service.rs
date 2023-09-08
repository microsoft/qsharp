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

    pub fn update_configuration(&mut self, config: IWorkspaceConfiguration) {
        let config: WorkspaceConfiguration = config.into();
        self.0
            .update_configuration(&qsls::protocol::WorkspaceConfigurationUpdate {
                target_profile: config.targetProfile.map(|s| match s.as_str() {
                    "base" => qsc::TargetProfile::Base,
                    "full" => qsc::TargetProfile::Full,
                    _ => panic!("invalid target profile"),
                }),
                package_type: config.packageType.map(|s| match s.as_str() {
                    "lib" => qsc::PackageType::Lib,
                    "exe" => qsc::PackageType::Exe,
                    _ => panic!("invalid package type"),
                }),
            })
    }

    pub fn update_document(&mut self, uri: &str, version: u32, text: &str) {
        self.0.update_document(uri, version, text);
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

    pub fn get_signature_help(&self, uri: &str, offset: u32) -> Option<ISignatureHelp> {
        let sig_help = self.0.get_signature_help(uri, offset);
        sig_help.map(|sig_help| {
            SignatureHelp {
                signatures: sig_help
                    .signatures
                    .into_iter()
                    .map(|sig| SignatureInformation {
                        label: sig.label,
                        documentation: sig.documentation,
                        parameters: sig
                            .parameters
                            .into_iter()
                            .map(|param| ParameterInformation {
                                label: Span {
                                    start: param.label.start,
                                    end: param.label.end,
                                },
                                documentation: param.documentation,
                            })
                            .collect(),
                    })
                    .collect(),
                active_signature: sig_help.active_signature,
                active_parameter: sig_help.active_parameter,
            }
            .into()
        })
    }
}

serializable_type! {
    WorkspaceConfiguration,
    {
        pub targetProfile: Option<String>,
        pub packageType: Option<String>,
    },
    r#"export interface IWorkspaceConfiguration {
        targetProfile?: "full" | "base";
        packageType?: "exe" | "lib";
    }"#,
    IWorkspaceConfiguration
}

serializable_type! {
    CompletionList,
    {
        pub items: Vec<CompletionItem>,
    },
    r#"export interface ICompletionList {
        items: ICompletionItem[]
    }"#,
    ICompletionList
}

serializable_type! {
    CompletionItem,
    {
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
    TextEdit,
    {
        pub range: Span,
        pub newText: String,
    },
    r#"export interface ITextEdit {
        range: ISpan;
        newText: string;
    }"#
}

serializable_type! {
    Hover,
    {
        pub contents: String,
        pub span: Span,
    },
    r#"export interface IHover {
        contents: string;
        span: ISpan
    }"#,
    IHover
}

serializable_type! {
    Definition,
    {
        pub source: String,
        pub offset: u32,
    },
    r#"export interface IDefinition {
        source: string;
        offset: number;
    }"#,
    IDefinition
}

serializable_type! {
    SignatureHelp,
    {
        signatures: Vec<SignatureInformation>,
        active_signature: u32,
        active_parameter: u32,
    },
    r#"export interface ISignatureHelp {
        signatures: ISignatureInformation[];
        active_signature: number;
        active_parameter: number;
    }"#,
    ISignatureHelp
}

serializable_type! {
    SignatureInformation,
    {
        label: String,
        documentation: Option<String>,
        parameters: Vec<ParameterInformation>,
    },
    r#"export interface ISignatureInformation {
        label: string;
        documentation: string | undefined;
        parameters: IParameterInformation[];
    }"#
}

serializable_type! {
    ParameterInformation,
    {
        label: Span,
        documentation: Option<String>,
    },
    r#"export interface IParameterInformation {
        label: { start: number; end: number };
        documentation: string | undefined;
    }"#
}

serializable_type! {
    Span,
    {
        pub start: u32,
        pub end: u32,
    },
    r#"export interface ISpan {
        start: number;
        end: number;
    }"#
}

serializable_type! {
    VSDiagnostic,
    {
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
