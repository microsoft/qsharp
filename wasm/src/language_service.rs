// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::{
    compile::{self, Error},
    PackageStore, SourceMap,
};

use qsc_hir::hir::PackageId;

pub struct QSharpLanguageService<'a> {
    store: PackageStore,
    std: PackageId,
    code: String,
    errors: Vec<Error>,
    event_callback: Box<EventCallback<'a>>,
}

type EventCallback<'a> = dyn FnMut(&[Error]) + 'a;

impl<'a> QSharpLanguageService<'a> {
    pub fn new(event_callback: impl FnMut(&[Error]) + 'a) -> Self {
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

        // TODO: document uri with callback, and one callback per document
        (self.event_callback)(&self.errors);
    }
}
