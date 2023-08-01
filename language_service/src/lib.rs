// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

pub mod completion;
pub mod definition;
mod display;
pub mod hover;
mod qsc_utils;
#[cfg(test)]
mod test_utils;

use crate::{
    completion::CompletionList, definition::Definition, hover::Hover, qsc_utils::compile_document,
};
use log::trace;
use qsc::PackageType;
use qsc_utils::Compilation;
use std::collections::HashMap;

pub struct LanguageService<'a> {
    /// Associate each known document with a separate compilation.
    document_map: HashMap<String, DocumentState>,
    /// Callback which will receive diagnostics (compilation errors)
    /// whenever a (re-)compilation occurs.
    diagnostics_receiver: Box<DiagnosticsReceiver<'a>>,
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

type DiagnosticsReceiver<'a> = dyn FnMut(&str, u32, &[qsc::compile::Error]) + 'a;

impl<'a> LanguageService<'a> {
    pub fn new(diagnostics_receiver: impl FnMut(&str, u32, &[qsc::compile::Error]) + 'a) -> Self {
        LanguageService {
            document_map: HashMap::new(),
            diagnostics_receiver: Box::new(diagnostics_receiver),
        }
    }

    /// Indicates that the document has been opened or the source has been updated.
    /// This should be called before any language service requests have been made
    /// for the document, typically when the document is first opened in the editor.
    /// It should also be called whenever the source code is updated.
    pub fn update_document(
        &mut self,
        uri: &str,
        version: u32,
        text: &str,
        package_type: PackageType,
    ) {
        trace!("update_document: {uri:?} {version:?}");
        let compilation = compile_document(uri, text, package_type);
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
}
