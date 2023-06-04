// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{language_service, VSDiagnostic};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct QSharpLanguageService(language_service::QSharpLanguageService);

#[wasm_bindgen]
impl QSharpLanguageService {
    #[wasm_bindgen(constructor)]
    pub fn new(event_callback: &js_sys::Function) -> Self {
        let event_callback = event_callback.clone();
        let inner = language_service::QSharpLanguageService::new(move || {
            let _ = event_callback.call0(&JsValue::null());
        });
        QSharpLanguageService(inner)
    }

    pub fn update_code(&mut self, uri: &str, code: &str) {
        self.0.update_code(uri, code);
    }

    pub fn check_code(&mut self, _uri: &str) -> Result<JsValue, JsValue> {
        let diags = self
            .0
            .check_code(_uri)
            .iter()
            .map(VSDiagnostic::from)
            .collect::<Vec<_>>();
        Ok(serde_wasm_bindgen::to_value(&diags)?)
    }
}
