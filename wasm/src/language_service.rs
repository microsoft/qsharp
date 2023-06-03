// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::{
    compile::{self, Error},
    PackageStore, SourceMap,
};

//use qsc_frontend::compile::CompileUnit;
use crate::VSDiagnostic;
use qsc_hir::hir::PackageId;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct QSharpLanguageService {
    store: PackageStore,
    std: PackageId,
    code: String,
    //    compile_unit: Option<CompileUnit>,
    errors: Vec<Error>,
}

#[wasm_bindgen]
impl QSharpLanguageService {
    #[wasm_bindgen(constructor)]
    pub fn new() -> QSharpLanguageService {
        let mut store = PackageStore::new(compile::core());
        let std = store.insert(compile::std(&store));

        QSharpLanguageService {
            code: String::new(),
            store,
            std,
            //            compile_unit: None,
            errors: Vec::new(),
        }
    }

    pub fn update_code(&mut self, uri: &str, code: &str) {
        self.code = code.to_string();

        let sources = SourceMap::new([(uri.into(), code.into())], None);
        let (_compile_unit, errors) = compile::compile(&self.store, &[self.std], sources);

        self.errors = errors;

        // TODO: check the code
    }

    pub fn check_code(&mut self, _uri: &str) -> Result<JsValue, JsValue> {
        let result: Vec<VSDiagnostic> = self.errors.iter().map(|error| (error).into()).collect();
        Ok(serde_wasm_bindgen::to_value(&result)?)
    }
}

impl Default for QSharpLanguageService {
    fn default() -> Self {
        Self::new()
    }
}
