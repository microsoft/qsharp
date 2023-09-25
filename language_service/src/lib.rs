// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use crate::qsc_utils::compile_document;
use log::trace;
use protocol::{CompletionList, Definition, Hover, SignatureHelp, WorkspaceConfigurationUpdate};
use qsc::{PackageType, TargetProfile};
use qsc_utils::Compilation;
use std::collections::HashMap;

pub mod completion;
pub mod definition;
mod display;
pub mod hover;
pub mod protocol;
mod qsc_utils;
pub mod signature_help;
#[cfg(test)]
mod test_utils;
#[cfg(test)]
mod tests;

pub struct LanguageService<'a> {
    /// Workspace configuration can include compiler settings
    /// that affect error checking and other language server behavior.
    /// Currently these settings apply to all documents in the
    /// workspace. Per-document configurations are not supported.
    configuration: WorkspaceConfiguration,
    /// Associate each known document with a separate compilation.
    document_map: HashMap<String, DocumentState>,
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

struct DocumentState {
    /// This version is the document version provided by the client.
    /// It increases strictly with each text change, though this knowledge should
    /// not be important. The version is only ever used when publishing
    /// diagnostics to help the client associate the list of diagnostics
    /// with a snapshot of the document.
    pub version: u32,
    pub compilation: Compilation,
}

type DiagnosticsReceiver<'a> = dyn Fn(&str, u32, &[qsc::compile::Error]) + 'a;

impl<'a> LanguageService<'a> {
    pub fn new(diagnostics_receiver: impl Fn(&str, u32, &[qsc::compile::Error]) + 'a) -> Self {
        LanguageService {
            configuration: WorkspaceConfiguration::default(),
            document_map: HashMap::new(),
            diagnostics_receiver: Box::new(diagnostics_receiver),
        }
    }

    /// Updates the workspace configuration. If any compiler settings are updated,
    /// a recompilation may be triggered, which will results in a new set of diagnostics
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
    pub fn update_document(&mut self, uri: &str, version: u32, text: &str) {
        trace!("update_document: {uri:?} {version:?}");
        let compilation = compile_document(
            uri,
            text,
            self.configuration.package_type,
            self.configuration.target_profile,
        );
        let errors = compilation.errors.clone();

        // insert() will update the value if the key already exists
        self.document_map.insert(
            uri.to_string(),
            DocumentState {
                version,
                compilation,
            },
        );

        trace!("publishing diagnostics for {uri:?}: {errors:?}");

        // Publish diagnostics
        (self.diagnostics_receiver)(uri, version, &errors);
    }

    /// Indicates that the client is no longer interested in the document,
    /// typically occurs when the document is closed in the editor.
    /// # Panics
    ///
    /// This function will panic if compiler state is invalid or in out-of-memory conditions.
    pub fn close_document(&mut self, uri: &str) {
        trace!("close_document: {uri:?}");
        let document_state = self.document_map.remove(uri);

        // Clear the diagnostics, as each document represents
        // a separate compilation that disappears when the document is closed.
        (self.diagnostics_receiver)(
            uri,
            document_state
                .expect("close_document received for unknown uri")
                .version,
            &[],
        );
    }

    /// # Panics
    ///
    /// This function will panic if compiler state is invalid or in out-of-memory conditions.
    #[must_use]
    pub fn get_completions(&self, uri: &str, offset: u32) -> CompletionList {
        trace!("get_completions: uri: {uri:?}, offset: {offset:?}");
        let res = completion::get_completions(
            &self
                .document_map.get(uri).as_ref()
                .expect("get_completions should not be called before document has been initialized with update_document").compilation,
            uri,
            offset,
        );
        trace!("get_completions result: {res:?}");
        res
    }

    /// # Panics
    ///
    /// This function will panic if compiler state is invalid or in out-of-memory conditions.
    #[must_use]
    pub fn get_definition(&self, uri: &str, offset: u32) -> Option<Definition> {
        trace!("get_definition: uri: {uri:?}, offset: {offset:?}");
        let res = definition::get_definition(
            &self
            .document_map.get(uri).as_ref()
                .expect("get_definition should not be called before document has been initialized with update_document").compilation,
                uri, offset);
        trace!("get_definition result: {res:?}");
        res
    }

    /// # Panics
    ///
    /// This function will panic if compiler state is invalid or in out-of-memory conditions.
    #[must_use]
    pub fn get_hover(&self, uri: &str, offset: u32) -> Option<Hover> {
        trace!("get_hover: uri: {uri:?}, offset: {offset:?}");
        let res = hover::get_hover(
            &self
            .document_map.get(uri).as_ref()
                .expect("get_hover should not be called before document has been initialized with update_document").compilation,
                uri, offset);
        trace!("get_hover result: {res:?}");
        res
    }

    /// # Panics
    ///
    /// This function will panic if compiler state is invalid or in out-of-memory conditions.
    #[must_use]
    pub fn get_signature_help(&self, uri: &str, offset: u32) -> Option<SignatureHelp> {
        trace!("get_signature_help: uri: {uri:?}, offset: {offset:?}");
        let res = signature_help::get_signature_help(
            &self
            .document_map.get(uri).as_ref()
                .expect("get_signature_help should not be called before document has been initialized with update_document").compilation,
                uri, offset);
        trace!("get_signature_help result: {res:?}");
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
        for (uri, state) in &mut self.document_map {
            let version = state.version;
            let contents = &state
                .compilation
                .unit
                .sources
                .find_by_name(uri)
                .expect("source should be found")
                .contents;

            let compilation = compile_document(
                uri,
                contents,
                self.configuration.package_type,
                self.configuration.target_profile,
            );

            *state = DocumentState {
                version,
                compilation,
            };

            trace!(
                "publishing diagnostics for {uri:?}: {:?}",
                state.compilation.errors
            );

            (self.diagnostics_receiver)(uri, version, &state.compilation.errors);
        }
    }
}
