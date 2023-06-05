// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    completion::{self, CompletionList},
    definition::{self, Definition},
};
use qsc::{
    compile::{self, Error},
    PackageStore, SourceMap,
};
use qsc_frontend::compile::CompileUnit;
use qsc_hir::hir::PackageId;

pub struct QSharpLanguageService<'a> {
    compilation_state: Option<CompilationState>,
    diagnostics_receiver: Box<DiagnosticsReceiver<'a>>,
    logger: Box<Logger<'a>>,
}

pub struct CompilationState {
    pub store: PackageStore,
    pub std: PackageId,
    pub compile_unit: Option<CompileUnit>,
    pub errors: Vec<Error>,
}

type DiagnosticsReceiver<'a> = dyn FnMut(&[Error]) + 'a;
type Logger<'a> = dyn Fn(&str) + 'a;

impl<'a> QSharpLanguageService<'a> {
    pub fn new(event_callback: impl FnMut(&[Error]) + 'a, logger: impl Fn(&str) + 'a) -> Self {
        QSharpLanguageService {
            compilation_state: None,
            diagnostics_receiver: Box::new(event_callback),
            logger: Box::new(logger),
        }
    }

    fn log(&self, msg: &str) {
        (self.logger)(msg);
    }

    pub fn update_code(&mut self, uri: &str, code: &str) {
        self.log(&format!("update_code enter: uri: {}, code: {}", uri, code));
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
            errors,
        });
        self.log(&format!("update_code exit: uri: {}, code: {}", uri, code));
    }

    pub fn get_completions(&self, uri: &str, offset: u32) -> CompletionList {
        self.log(&format!(
            "get_completions: uri: {}, offset: {}",
            uri, offset
        ));
        completion::get_completions(
            self
                .compilation_state.as_ref()
                .expect("get_completions should not be called before compilation has been initialized with update_code"),
            uri,
            offset,
        )
    }

    pub fn get_definition(&self, uri: &str, offset: u32) -> Definition {
        self.log(&format!("get_definition: uri: {}, offset: {}", uri, offset));
        let res = definition::get_definition(
            self
                .compilation_state.as_ref()
                .expect("get_definition should not be called before compilation has been initialized with update_code"),
                uri, offset);
        self.log(&format!(
            "get_definition result: source: {}, offset: {}",
            res.source, res.offset
        ));
        res
    }
}
