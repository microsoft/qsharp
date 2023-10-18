// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

mod compilation;
pub mod completion;
pub mod definition;
mod display;
pub mod hover;
pub mod protocol;
mod qsc_utils;
pub mod rename;
pub mod signature_help;
#[cfg(test)]
mod test_utils;
#[cfg(test)]
mod tests;

use compilation::Compilation;
use log::trace;
use miette::Diagnostic;
use protocol::{CompletionList, Definition, Hover, SignatureHelp, WorkspaceConfigurationUpdate};
use qsc::{PackageType, TargetProfile};
use std::collections::HashMap;

type CompilationId = String;
type DocumentUri = String;

pub struct LanguageService<'a> {
    /// Workspace configuration can include compiler settings
    /// that affect error checking and other language server behavior.
    /// Currently these settings apply to all documents in the
    /// workspace. Per-document configurations are not supported.
    configuration: WorkspaceConfiguration,
    /// Currently each Q# file is its own unique compilation.
    /// For notebooks, each notebook is a compilation, each cell is a document
    /// within that compilation.
    /// CompilationId is the document uri for single-file compilations,
    /// notebook uri for notebooks
    compilations: HashMap<CompilationId, CompilationState>,
    /// All known documents
    /// (cell uri -> notebook uri, or identity in the case of single-file compilation)
    documents: HashMap<DocumentUri, CompilationId>,
    /// Callback which will receive diagnostics (compilation errors)
    /// whenever a (re-)compilation occurs.
    diagnostics_receiver: Box<DiagnosticsReceiver<'a>>,
}

#[derive(Debug)]
struct WorkspaceConfiguration {
    pub target_profile: TargetProfile,
    pub package_type: PackageType,
}

impl Default for WorkspaceConfiguration {
    fn default() -> Self {
        Self {
            target_profile: TargetProfile::Full,
            package_type: PackageType::Exe,
        }
    }
}

struct CompilationState {
    // TODO: versions are supposed to be associated with documents not compilations (??)
    /// This version is the document version provided by the client.
    /// It increases strictly with each text change, though this knowledge should
    /// not be important. The version is only ever used when publishing
    /// diagnostics to help the client associate the list of diagnostics
    /// with a snapshot of the document.
    pub version: u32,
    pub project: Compilation,
}

type DiagnosticsReceiver<'a> = dyn Fn(&str, u32, &[qsc::compile::Error]) + 'a;

impl<'a> LanguageService<'a> {
    pub fn new(diagnostics_receiver: impl Fn(&str, u32, &[qsc::compile::Error]) + 'a) -> Self {
        LanguageService {
            configuration: WorkspaceConfiguration::default(),
            compilations: HashMap::new(),
            documents: HashMap::new(),
            diagnostics_receiver: Box::new(diagnostics_receiver),
        }
    }

    /// Updates the workspace configuration. If any compiler settings are updated,
    /// a recompilation may be triggered, which will result in a new set of diagnostics
    /// being published.
    pub fn update_configuration(&mut self, configuration: &WorkspaceConfigurationUpdate) {
        trace!("update_configuration: {configuration:?}");

        let need_recompile = self.apply_configuration(configuration);

        // Some configuration options require a recompilation as they impact error checking
        if need_recompile {
            self.recompile_all();
        }
    }

    /// Indicates that the document has been opened or the source has been updated.
    /// This should be called before any language service requests have been made
    /// for the document, typically when the document is first opened in the editor.
    /// It should also be called whenever the source code is updated.
    ///
    /// LSP: textDocument/didOpen, textDocument/didChange
    pub fn update_document(&mut self, uri: &str, version: u32, text: &str) {
        trace!("update_document: {uri:?} {version:?}");
        let project = Compilation::new_open_document(
            uri,
            text,
            self.configuration.package_type,
            self.configuration.target_profile,
        );

        self.publish_diagnostics(&project, version, uri);

        // Associate each known document with a separate compilation.
        self.compilations
            .insert(uri.to_string(), CompilationState { version, project });
        self.documents.insert(uri.to_string(), uri.to_string());
    }

    /// The uri refers to the notebook itself, not any of the individual cells.
    ///
    /// This function expects all Q# content in the notebook every time
    /// it is called, not just the changed cells.
    ///
    /// At this layer we expect the editor to have stripped
    /// off all non-Q# content, including Python cells and lines
    /// containing the "%%qsharp" cell magic.
    ///
    /// LSP: notebookDocument/didOpen, notebookDocument/didChange
    pub fn update_notebook_document(
        &mut self,
        notebook_uri: &str,
        version: u32,
        cells: &[(&str, &str)],
    ) {
        trace!("update_notebook_document: {notebook_uri:?} {version:?}");
        let project = Compilation::new_notebook(cells);

        for (document_uri, _) in cells {
            // TODO: version is likely all wrong here
            self.publish_diagnostics(&project, version, document_uri);
        }

        // TODO: Do we need to clear diagnostics for removed cells?

        self.compilations.insert(
            notebook_uri.to_string(),
            CompilationState { version, project },
        );

        for (cell_uri, _) in cells {
            self.documents
                .insert((*cell_uri).to_string(), notebook_uri.to_string());
        }
    }

    /// Indicates that the client is no longer interested in the document,
    /// typically occurs when the document is closed in the editor.
    ///
    /// LSP: textDocument/didClose
    /// # Panics
    ///
    /// This function will panic if compiler state is invalid or in out-of-memory conditions.
    pub fn close_document(&mut self, uri: &str) {
        trace!("close_document: {uri:?}");

        // TODO: We're treating "uri" as a project URI here but actually
        // it's a document URI. Works for now, whatever.
        let document_state = self.compilations.remove(uri);

        // Clear the diagnostics, as each document represents
        // a separate project that disappears when the document is closed.
        (self.diagnostics_receiver)(
            uri,
            document_state
                .expect("close_document received for unknown uri")
                .version,
            &[],
        );
    }

    /// LSP: textDocument/completion
    #[must_use]
    pub fn get_completions(&self, uri: &str, offset: u32) -> CompletionList {
        trace!("get_completions: uri: {uri:?}, offset: {offset:?}");
        let res = completion::get_completions(
            &self
                .compilations.get(self.documents.get(uri).expect("get_completions should not be called before document has been initialized with update_document")).as_ref()
                .expect("get_completions should not be called before document has been initialized with update_document").project,
            uri,
            offset,
        );
        trace!("get_completions result: {res:?}");
        res
    }

    /// LSP: textDocument/definition
    #[must_use]
    pub fn get_definition(&self, uri: &str, offset: u32) -> Option<Definition> {
        trace!("get_definition: uri: {uri:?}, offset: {offset:?}");
        let res = definition::get_definition(
            &self
            .compilations.get(self.documents.get(uri).expect("get_completions should not be called before document has been initialized with update_document")).as_ref()
            .expect("get_definition should not be called before document has been initialized with update_document").project,
                uri, offset);
        trace!("get_definition result: {res:?}");
        res
    }

    /// LSP: textDocument/hover
    #[must_use]
    pub fn get_hover(&self, uri: &str, offset: u32) -> Option<Hover> {
        trace!("get_hover: uri: {uri:?}, offset: {offset:?}");
        let res = hover::get_hover(
            &self
            .compilations.get(self.documents.get(uri).expect("get_completions should not be called before document has been initialized with update_document")).as_ref()
            .expect("get_hover should not be called before document has been initialized with update_document").project,
                uri, offset);
        trace!("get_hover result: {res:?}");
        res
    }

    #[must_use]
    pub fn get_signature_help(&self, uri: &str, offset: u32) -> Option<SignatureHelp> {
        trace!("get_signature_help: uri: {uri:?}, offset: {offset:?}");
        let res = signature_help::get_signature_help(
            &self
            .compilations.get(self.documents.get(uri).expect("get_signature_help should not be called before document has been initialized with update_document")).as_ref()
            .expect("get_signature_help should not be called before document has been initialized with update_document").project,
                uri, offset);
        trace!("get_signature_help result: {res:?}");
        res
    }

    fn publish_diagnostics(&self, project: &Compilation, version: u32, document_uri: &str) {
        // TODO: this will definitely cause issues when we have errors that go
        // across cells
        let errors_for_document = project
            .errors
            .clone()
            .into_iter()
            .filter(|error| match error.labels() {
                Some(labels) => labels
                    .map(|label| u32::try_from(label.offset()).expect("offset should fit into u32"))
                    .any(|offset| {
                        document_uri
                            == project
                                .current_unit()
                                .sources
                                .find_by_offset(offset)
                                .expect("should find document for error span")
                                .name
                                .as_ref()
                    }),
                None => true, // No labels for an error means it definitely applies to this (all) documents
            })
            .collect::<Vec<_>>()
            .clone();

        trace!("publishing diagnostics for {document_uri:?}: {errors_for_document:?}");

        // Publish diagnostics
        (self.diagnostics_receiver)(document_uri, version, &errors_for_document);
    }

    #[must_use]
    pub fn get_rename(&self, uri: &str, offset: u32) -> Vec<protocol::Span> {
        trace!("get_rename: uri: {uri:?}, offset: {offset:?}");
        let res = rename::get_rename(
            &self
            .document_map.get(uri).as_ref()
                .expect("get_rename should not be called before document has been initialized with update_document").compilation,
                uri, offset);
        trace!("get_rename result: {res:?}");
        res
    }

    #[must_use]
    pub fn prepare_rename(&self, uri: &str, offset: u32) -> Option<(protocol::Span, String)> {
        trace!("prepare_rename: uri: {uri:?}, offset: {offset:?}");
        let res = rename::prepare_rename(
            &self
            .document_map.get(uri).as_ref()
                .expect("prepare_rename should not be called before document has been initialized with update_document").compilation,
                uri, offset);
        trace!("prepare_rename result: {res:?}");
        res
    }

    fn apply_configuration(&mut self, configuration: &WorkspaceConfigurationUpdate) -> bool {
        let mut need_recompile = false;

        if let Some(package_type) = configuration.package_type {
            need_recompile |= self.configuration.package_type != package_type;
            self.configuration.package_type = package_type;
        }

        if let Some(target_profile) = configuration.target_profile {
            need_recompile |= self.configuration.target_profile != target_profile;
            self.configuration.target_profile = target_profile;
        }

        trace!("need_recompile after configuration update: {need_recompile:?}");
        need_recompile
    }

    /// Recompiles the currently known documents with
    /// the current configuration. Publishes updated
    /// diagnostics for all documents.
    fn recompile_all(&mut self) {
        for state in self.compilations.values_mut() {
            state.project.recompile(
                self.configuration.package_type,
                self.configuration.target_profile,
            );
        }

        for document_uri in self.documents.values() {
            let state = self
                .compilations
                .get(document_uri)
                .expect("project looked up with document uri should exist");
            self.publish_diagnostics(&state.project, state.version, document_uri);
        }
    }
}
