// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::{
    compile::{self, Error},
    PackageStore, SourceMap,
};

use qsc_hir::hir::PackageId;

pub struct QSharpLanguageService {
    store: PackageStore,
    std: PackageId,
    code: String,
    //    compile_unit: Option<CompileUnit>,
    errors: Vec<Error>,
    event_callback: Box<dyn FnMut()>,
}

impl QSharpLanguageService {
    pub fn new(event_callback: impl FnMut() + 'static) -> Self {
        let mut store = PackageStore::new(compile::core());
        let std = store.insert(compile::std(&store));

        QSharpLanguageService {
            code: String::new(),
            store,
            std,
            errors: Vec::new(),
            event_callback: Box::new(event_callback),
        }
    }

    pub fn update_code(&mut self, uri: &str, code: &str) {
        self.code = code.to_string();

        let sources = SourceMap::new([(uri.into(), code.into())], None);
        let (_compile_unit, errors) = compile::compile(&self.store, &[self.std], sources);

        self.errors = errors;

        // TODO: check the code
    }

    pub fn check_code(&mut self, _uri: &str) -> Vec<Error> {
        (self.event_callback)();
        self.errors.clone()
    }
}
