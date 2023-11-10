// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{diagnostic::VSDiagnostic, serializable_type};
use js_sys::JsString;
use qsc::{self};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct LanguageService(qsls::LanguageService<'static>);

#[wasm_bindgen]
impl LanguageService {
    #[wasm_bindgen(constructor)]
    pub fn new(diagnostics_callback: DiagnosticsCallback) -> Self {
        let diagnostics_callback = diagnostics_callback
            .dyn_ref::<js_sys::Function>()
            .expect("expected a valid JS function")
            .clone();
        let inner = qsls::LanguageService::new(move |update| {
            let diags = update
                .errors
                .iter()
                .map(|err| VSDiagnostic::from_compile_error(&update.uri, err))
                .collect::<Vec<_>>();
            let _ = diagnostics_callback
                .call3(
                    &JsValue::NULL,
                    &update.uri.into(),
                    &update.version.into(),
                    &serde_wasm_bindgen::to_value(&diags)
                        .expect("conversion to VSDiagnostic should succeed"),
                )
                .expect("callback should succeed");
        });
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

    pub fn update_notebook_document(&mut self, notebook_uri: &str, cells: Vec<ICell>) {
        let cells: Vec<Cell> = cells.into_iter().map(|c| c.into()).collect();
        self.0.update_notebook_document(
            notebook_uri,
            cells
                .iter()
                .map(|s| (s.uri.as_ref(), s.version, s.code.as_ref())),
        );
    }

    pub fn close_notebook_document(&mut self, notebook_uri: &str, cell_uris: Vec<JsString>) {
        let cell_uris = cell_uris
            .iter()
            .map(|s| s.as_string().expect("expected string"))
            .collect::<Vec<_>>();
        self.0
            .close_notebook_document(notebook_uri, cell_uris.iter().map(|s| s.as_str()));
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
                        qsls::protocol::CompletionItemKind::Property => "property",
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

    pub fn get_definition(&self, uri: &str, offset: u32) -> Option<ILocation> {
        let definition = self.0.get_definition(uri, offset);
        definition.map(|definition| {
            Location {
                source: definition.source,
                span: Span {
                    start: definition.span.start,
                    end: definition.span.end,
                },
            }
            .into()
        })
    }

    pub fn get_references(
        &self,
        uri: &str,
        offset: u32,
        include_declaration: bool,
    ) -> Vec<ILocation> {
        let locations = self.0.get_references(uri, offset, include_declaration);
        locations
            .into_iter()
            .map(|loc| {
                Location {
                    source: loc.source,
                    span: Span {
                        start: loc.span.start,
                        end: loc.span.end,
                    },
                }
                .into()
            })
            .collect()
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
                activeSignature: sig_help.active_signature,
                activeParameter: sig_help.active_parameter,
            }
            .into()
        })
    }

    pub fn get_rename(&self, uri: &str, offset: u32, new_name: &str) -> IWorkspaceEdit {
        let locations = self.0.get_rename(uri, offset);

        let mut renames: FxHashMap<String, Vec<TextEdit>> = FxHashMap::default();
        locations.into_iter().for_each(|l| {
            renames.entry(l.source).or_default().push(TextEdit {
                range: Span {
                    start: l.span.start,
                    end: l.span.end,
                },
                newText: new_name.to_string(),
            })
        });

        let workspace_edit = WorkspaceEdit {
            changes: renames.into_iter().collect(),
        };

        workspace_edit.into()
    }

    pub fn prepare_rename(&self, uri: &str, offset: u32) -> Option<ITextEdit> {
        let result = self.0.prepare_rename(uri, offset);
        result.map(|r| {
            TextEdit {
                range: Span {
                    start: r.0.start,
                    end: r.0.end,
                },
                newText: r.1,
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
        kind: "function" | "interface" | "keyword" | "module" | "property";
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
    }"#,
    ITextEdit
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
    Location,
    {
        pub source: String,
        pub span: Span,
    },
    r#"export interface ILocation {
        source: string;
        span: ISpan;
    }"#,
    ILocation
}

serializable_type! {
    SignatureHelp,
    {
        signatures: Vec<SignatureInformation>,
        activeSignature: u32,
        activeParameter: u32,
    },
    r#"export interface ISignatureHelp {
        signatures: ISignatureInformation[];
        activeSignature: number;
        activeParameter: number;
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
        documentation?: string;
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
        label: ISpan;
        documentation?: string;
    }"#
}

serializable_type! {
    WorkspaceEdit,
    {
        changes: Vec<(String, Vec<TextEdit>)>,
    },
    r#"export interface IWorkspaceEdit {
        changes: [string, ITextEdit[]][];
    }"#,
    IWorkspaceEdit
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
    Cell,
    {
        pub uri: String,
        pub version: u32,
        pub code: String
    },
    r#"export interface ICell {
        uri: string;
        version: number;
        code: string;
    }"#,
    ICell
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(
        typescript_type = "(uri: string, version: number | undefined, diagnostics: VSDiagnostic[]) => void"
    )]
    pub type DiagnosticsCallback;
}
