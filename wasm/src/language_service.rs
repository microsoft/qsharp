// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::{
    compile::{self, Error},
    PackageStore, SourceMap,
};

use qsc_frontend::compile::CompileUnit;
use qsc_hir::hir::PackageId;

use crate::completion::{self, CompletionList};

pub struct QSharpLanguageService<'a> {
    compilation_state: Option<CompilationState>,
    diagnostics_receiver: Box<DiagnosticsReceiver<'a>>,
}

pub struct CompilationState {
    pub store: PackageStore,
    pub std: PackageId,
    pub compile_unit: Option<CompileUnit>,
    pub errors: Vec<Error>,
}

type DiagnosticsReceiver<'a> = dyn FnMut(&[Error]) + 'a;

impl<'a> QSharpLanguageService<'a> {
    pub fn new(event_callback: impl FnMut(&[Error]) + 'a) -> Self {
        QSharpLanguageService {
            compilation_state: None,
            diagnostics_receiver: Box::new(event_callback),
        }
    }

    pub fn update_code(&mut self, uri: &str, code: &str) {
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
    }

    pub fn get_completions(&self, source_path: &str, offset: u32) -> CompletionList {
        completion::get_completions(
            self
                .compilation_state.as_ref()
                .expect("get_completions should not be called before compilation has been initialized with update_code"),
            source_path,
            offset,
        )
    }
}
