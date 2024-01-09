// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::{compile::Error, target::Profile, PackageType};

/// A change to the workspace configuration
#[derive(Clone, Debug, Default, Copy)]
pub struct WorkspaceConfigurationUpdate {
    pub target_profile: Option<Profile>,
    pub package_type: Option<PackageType>,
}

/// Represents a span of text used by the Language Server API
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug)]
pub struct DiagnosticUpdate {
    pub uri: String,
    pub version: Option<u32>,
    pub errors: Vec<Error>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[allow(clippy::module_name_repetitions)]
pub enum CompletionItemKind {
    // It would have been nice to match the numeric values to the ones used by
    // VS Code and Monaco, but unfortunately those two disagree on the values.
    // So we define our own unique enum here to reduce confusion.
    Function,
    Interface,
    Keyword,
    Module,
    Property,
    Variable,
    TypeParameter,
}

#[derive(Debug, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct CompletionList {
    pub items: Vec<CompletionItem>,
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    pub sort_text: Option<String>,
    pub detail: Option<String>,
    pub additional_text_edits: Option<Vec<(Span, String)>>,
}

impl CompletionItem {
    #[must_use]
    pub fn new(label: String, kind: CompletionItemKind) -> Self {
        CompletionItem {
            label,
            kind,
            sort_text: None,
            detail: None,
            additional_text_edits: None,
        }
    }
}

impl PartialEq for CompletionItem {
    // exclude sort text for comparison
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
            && self.kind == other.kind
            && self.detail == other.detail
            && self.additional_text_edits == other.additional_text_edits
    }
}

impl Eq for CompletionItem {}

use std::hash::{Hash, Hasher};

impl Hash for CompletionItem {
    // exclude sort text for hashing
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.label.hash(state);
        self.kind.hash(state);
        self.detail.hash(state);
        self.additional_text_edits.hash(state);
    }
}

#[derive(Debug, PartialEq)]
pub struct Location {
    pub source: String,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct Hover {
    pub contents: String,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct SignatureHelp {
    pub signatures: Vec<SignatureInformation>,
    pub active_signature: u32,
    pub active_parameter: u32,
}

#[derive(Debug, PartialEq)]
pub struct SignatureInformation {
    pub label: String,
    pub documentation: Option<String>,
    pub parameters: Vec<ParameterInformation>,
}

#[derive(Debug, PartialEq)]
pub struct ParameterInformation {
    pub label: Span,
    pub documentation: Option<String>,
}

#[derive(Default, Clone, Copy)]
pub struct NotebookMetadata {
    pub target_profile: Option<Profile>,
}
