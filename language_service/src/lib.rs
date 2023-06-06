// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

mod completion;
mod definition;
mod hover;
mod ls_utils;

use crate::{completion::CompletionList, definition::Definition, hover::Hover};
use ls_utils::CompilationState;
use qsc::{
    compile::{self, Error},
    PackageStore, SourceMap,
};

pub struct LanguageService<'a> {
    compilation_state: Option<CompilationState>,
    diagnostics_receiver: Box<DiagnosticsReceiver<'a>>,
    logger: Box<Logger<'a>>,
}

type DiagnosticsReceiver<'a> = dyn FnMut(&[Error]) + 'a;
type Logger<'a> = dyn Fn(&str) + 'a;

impl<'a> LanguageService<'a> {
    pub fn new(event_callback: impl FnMut(&[Error]) + 'a, logger: impl Fn(&str) + 'a) -> Self {
        LanguageService {
            compilation_state: None,
            diagnostics_receiver: Box::new(event_callback),
            logger: Box::new(logger),
        }
    }

    fn log(&self, msg: &str) {
        (self.logger)(msg);
    }

    pub fn update_code(&mut self, uri: &str, code: &str) {
        self.log(&format!("update_code enter: uri: {uri:?}"));
        let mut store = PackageStore::new(compile::core());
        let std = store.insert(compile::std(&store));

        let sources = SourceMap::new([(uri.into(), code.into())], None);
        let (compile_unit, errors) = compile::compile(&store, &[std], sources);

        // TODO: document uri with callback, and one callback per document
        (self.diagnostics_receiver)(&errors);

        self.compilation_state = Some(CompilationState {
            store,
            std,
            compile_unit: Some(compile_unit),
        });
        self.log(&format!("update_code exit: uri: {uri:?}"));
    }

    #[must_use]
    pub fn get_completions(&self, uri: &str, offset: u32) -> CompletionList {
        self.log(&format!(
            "get_completions: uri: {uri:?}, offset: {offset:?}"
        ));
        let res = completion::get_completions(
            self
                .compilation_state.as_ref()
                .expect("get_completions should not be called before compilation has been initialized with update_code"),
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
                .compilation_state.as_ref()
                .expect("get_definition should not be called before compilation has been initialized with update_code"),
                uri, offset);
        self.log(&format!("get_definition result: {res:?}"));
        res
    }

    #[must_use]
    pub fn get_hover(&self, uri: &str, offset: u32) -> Option<Hover> {
        self.log(&format!("get_hover: uri: {uri:?}, offset: {offset:?}"));
        let res = hover::get_hover(
            self
                .compilation_state.as_ref()
                .expect("get_hover should not be called before compilation has been initialized with update_code"),
                uri, offset);
        self.log(&format!("get_hover result: {res:?}"));
        res
    }
}
