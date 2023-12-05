// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{future::Future, path::PathBuf, pin::Pin, sync::Arc};

use crate::{diagnostic::VSDiagnostic, serializable_type};
use js_sys::JsString;
use qsc::{self};
use qsc_project::{EntryType, Manifest, ManifestDescriptor};
use qsls::{protocol::DiagnosticUpdate, JSFileEntry};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
pub struct LanguageService(qsls::LanguageService<'static>);

/// This macro produces a function that calls an async JS function, awaits it, and then applies a function to the resulting value.
/// Ultimately, it returns a function that accepts a String and returns a Rust future that represents a JS Promise. Awaiting that
/// Rust future will await the resolution of the promise.
/// The name of this macro should be read like "convert a JS promise into an async rust function with this mapping function"
macro_rules! into_async_rust_fn_with {
    ($js_async_fn: ident, $map_result: expr) => {{
        let $js_async_fn = to_js_function($js_async_fn.obj, stringify!($js_async_fn));

        let $js_async_fn = move |input: String| {
            let path = JsValue::from_str(&input);
            let res: js_sys::Promise = $js_async_fn
                .call1(&JsValue::NULL, &path)
                .expect("callback should succeed")
                .into();

            let res: JsFuture = res.into();

            Box::pin(map_js_promise(res, move |x| $map_result(x, input.clone())))
                as Pin<Box<dyn Future<Output = _> + 'static>>
        };
        $js_async_fn
    }};
}

async fn map_js_promise<F, T>(res: JsFuture, func: F) -> T
where
    F: Fn(JsValue) -> T,
{
    let res = res.await.expect("js future shouldn't throw an exception");
    log::trace!("asynchronous callback from wasm returned {res:?}");
    func(res)
}

fn to_js_function(val: JsValue, help_text_panic: &'static str) -> js_sys::Function {
    val.dyn_ref::<js_sys::Function>()
        .unwrap_or_else(|| {
            panic!(
                "expected a valid JS function ({help_text_panic}), received {:?}",
                val.js_typeof()
            )
        })
        .clone()
}

#[wasm_bindgen]
impl LanguageService {
    #[wasm_bindgen(constructor)]
    pub fn new(
        diagnostics_callback: DiagnosticsCallback,
        read_file: ReadFileCallback,
        list_directory: ListDirectoryCallback,
        get_manifest: GetManifestCallback,
    ) -> Self {
        let transformer = move |js_val: JsValue, path_buf_string: String| match js_val.as_string() {
            Some(res) => return (Arc::from(path_buf_string.as_str()), Arc::from(res)),
            // this can happen if the document is completely empty
            None if js_val.is_null() => (Arc::from(path_buf_string.as_str()), Arc::from("")),
            None => unreachable!("Expected string from JS callback, received {js_val:?}"),
        };
        let read_file = into_async_rust_fn_with!(read_file, transformer);

        let transformer = move |js_val: JsValue, _: String| {
            match js_val.dyn_into::<js_sys::Array>()
        {
            Ok(arr) => arr
                .into_iter()
                .map(|x| {
                    x.dyn_into::<js_sys::Array>()
                        .expect("expected directory listing callback to return array of arrays")
                })
                .filter_map(|js_arr| {
                    let mut arr = js_arr.into_iter().take(2);
                    match (
                        arr.next().unwrap().as_string(),
                        arr.next().unwrap().as_f64(),
                    ) {
                        (Some(a), Some(b)) => Some((a, b as i32)),
                        _ => None,
                    }
                })
                .map(|(name, ty)| JSFileEntry {
                    name,
                    r#type: match ty {
                        0 => EntryType::Unknown,
                        1 => EntryType::File,
                        2 => EntryType::Folder,
                        64 => EntryType::Symlink,
                        _ => unreachable!("expected one of vscode.FileType. Received {ty:?}"),
                    },
                })
                .collect::<Vec<_>>(),
            Err(e) => unreachable!("controlled callback should have returned an array -- our typescript bindings should guarantee this. {e:?}"),
        }
        };
        let list_directory = into_async_rust_fn_with!(list_directory, transformer);

        let transformer = move |js_val: JsValue, _| {
            if js_val.is_null() {
                return None;
            }

            let manifest_dir = match js_sys::Reflect::get(&js_val, &JsValue::from_str("manifestDirectory")) {
                    Ok(v) => v
                        .as_string()
                        .unwrap_or_else(|| panic!("manifest callback returned {:?}, but we expected a string representing its URI", v)),
                    Err(_) => unreachable!("our typescript bindings should guarantee that an object with a manifestDirectory property is returned here"),
                };
            log::trace!("found manifest at {manifest_dir:?}");

            let manifest_dir = PathBuf::from(manifest_dir);

            let exclude_files =
                match js_sys::Reflect::get(&js_val, &JsValue::from_str("excludeFiles")) {
                    Ok(v) => match v.dyn_into::<js_sys::Array>() {
                        Ok(arr) => arr
                            .into_iter()
                            .filter_map(|x| x.as_string())
                            .collect::<Vec<_>>(),
                        Err(e) => unreachable!("controlled callback should have returned an array -- our typescript bindings should guarantee this. {e:?}"),
                    },
                    Err(_) => unreachable!("our typescript bindings should guarantee that an object with an excludeFiles property is returned here"),
                };
            let exclude_regexes =
                match js_sys::Reflect::get(&js_val, &JsValue::from_str("excludeRegexes")) {
                    Ok(v) => match v.dyn_into::<js_sys::Array>() {
                        Ok(arr) => arr
                            .into_iter()
                            .filter_map(|x| x.as_string())
                            .collect::<Vec<_>>(),
                        Err(e) => unreachable!("controlled callback should have returned an array -- our typescript bindings should guarantee this. {e:?}"),
                    },
                    Err(_) => unreachable!("our typescript bindings should guarantee that an object with an excludeRegexes property is returned here"),
                };

            Some(ManifestDescriptor {
                manifest: Manifest {
                    exclude_regexes,
                    exclude_files,
                    ..Default::default()
                },
                manifest_dir,
            })
        };
        let get_manifest = into_async_rust_fn_with!(get_manifest, transformer);

        let diagnostics_callback = to_js_function(diagnostics_callback.obj, "diagnostics_callback");

        let diagnostics_callback = diagnostics_callback
            .dyn_ref::<js_sys::Function>()
            .expect("expected a valid JS function")
            .clone();

        let diagnostics_callback = move |update: DiagnosticUpdate| {
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
        };

        let inner = qsls::LanguageService::new(
            diagnostics_callback,
            read_file,
            list_directory,
            get_manifest,
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

    pub async fn update_document(&mut self, uri: &str, version: u32, text: &str) {
        self.0.update_document(uri, version, text).await;
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
        typescript_type = "(uri: string, version: number | undefined, diagnostics: VSDiagnostic[]) => Promise<void>"
    )]
    pub type DiagnosticsCallback;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "(uri: string) => Promise<string | null>")]
    pub type ReadFileCallback;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "(uri: string) => Promise<[string, number][]>")]
    pub type ListDirectoryCallback;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(
        typescript_type = "(uri: string) => Promise<{ excludeFiles: string[], excludeRegexes: string[], manifestDirectory: string } | null>"
    )]
    pub type GetManifestCallback;
}
