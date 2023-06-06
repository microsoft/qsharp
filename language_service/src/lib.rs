// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

mod completion;
mod definition;
mod hover;
mod ls_utils;

use std::collections::HashMap;

use crate::{completion::CompletionList, definition::Definition, hover::Hover};
use ls_utils::CompilationState;
use qsc::{
    compile::{self, Error},
    PackageStore, SourceMap,
};

pub struct LanguageService<'a> {
    compilation_state: HashMap<String, CompilationState>,
    diagnostics_receiver: Box<DiagnosticsReceiver<'a>>,
    logger: Box<Logger<'a>>,
}

type DiagnosticsReceiver<'a> = dyn FnMut(&str, u32, &[Error]) + 'a;
type Logger<'a> = dyn Fn(&str) + 'a;

impl<'a> LanguageService<'a> {
    pub fn new(
        event_callback: impl FnMut(&str, u32, &[Error]) + 'a,
        logger: impl Fn(&str) + 'a,
    ) -> Self {
        LanguageService {
            compilation_state: HashMap::new(),
            diagnostics_receiver: Box::new(event_callback),
            logger: Box::new(logger),
        }
    }

    fn log(&self, msg: &str) {
        (self.logger)(msg);
    }

    /// Updates the version and compilation for the document identified by `uri`.
    /// This should be called before any language service requests have been made
    /// for the document, typically when the document is first opened in the editor.
    /// It should also be called whenever the source code is updated.
    pub fn update_document(&mut self, uri: &str, version: u32, text: &str) {
        self.log(&format!("update_document enter: uri: {uri:?}"));
        let mut package_store = PackageStore::new(compile::core());
        let std_package_id = package_store.insert(compile::std(&package_store));

        // Source map only contains the current document.
        let source_map = SourceMap::new([(uri.into(), text.into())], None);
        let (compile_unit, errors) =
            compile::compile(&package_store, &[std_package_id], source_map);

        self.log(&format!("publishing diagnostics for {uri:?}: {errors:?}"));
        (self.diagnostics_receiver)(uri, version, &errors);

        // insert() will update the value if the key already exists
        self.compilation_state.insert(
            uri.to_string(),
            CompilationState {
                version,
                package_store,
                std_package_id,
                compile_unit,
            },
        );
        self.log(&format!("update_document exit: uri: {uri:?}"));
    }

    /// Indicates that the client is no longer interested in the document,
    /// typically occurs when the document is closed in the editor.
    pub fn close_document(&mut self, uri: &str) {
        self.log(&format!("close_document enter: uri: {uri:?}"));
        let item = self.compilation_state.remove(uri);

        // Clear the diagnostics, as each document represents
        // a separate compilation that disappears when the document is closed.
        self.log(&format!("clearing diagnostics for {uri:?}"));
        (self.diagnostics_receiver)(
            uri,
            item.expect("close_document received for unknown uri")
                .version,
            &[],
        );
        self.log(&format!("close_document exit: uri: {uri:?}"));
    }

    #[must_use]
    pub fn get_completions(&self, uri: &str, offset: u32) -> CompletionList {
        self.log(&format!(
            "get_completions: uri: {uri:?}, offset: {offset:?}"
        ));
        let res = completion::get_completions(
            self
                .compilation_state.get(uri).as_ref()
                .expect("get_completions should not be called before document has been initialized with update_document"),
            uri,
            offset,
        );
        self.log(&format!("get_completions result: {res:?}"));
        res
    }

    #[must_use]
    pub fn get_definition(&self, uri: &str, offset: u32) -> Definition {
        self.log(&format!("get_definition: uri: {uri:?}, offset: {offset:?}"));
        let res = definition::get_definition(
            self
            .compilation_state.get(uri).as_ref()
                .expect("get_definition should not be called before document has been initialized with update_document"),
                uri, offset);
        self.log(&format!("get_definition result: {res:?}"));
        res
    }

    #[must_use]
    pub fn get_hover(&self, uri: &str, offset: u32) -> Option<Hover> {
        self.log(&format!("get_hover: uri: {uri:?}, offset: {offset:?}"));
        let res = hover::get_hover(
            self
            .compilation_state.get(uri).as_ref()
                .expect("get_hover should not be called before document has been initialized with update_document"),
                uri, offset);
        self.log(&format!("get_hover result: {res:?}"));
        res
    }
}
