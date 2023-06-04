// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{language_service, VSDiagnostic};

use qsc::compile::Error;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct QSharpLanguageService(language_service::QSharpLanguageService<'static>);

#[wasm_bindgen]
impl QSharpLanguageService {
    #[wasm_bindgen(constructor)]
    pub fn new(event_callback: &js_sys::Function) -> Self {
        let event_callback = event_callback.clone();
        let inner = language_service::QSharpLanguageService::new(move |errors: &[Error]| {
            let diags = errors.iter().map(VSDiagnostic::from).collect::<Vec<_>>();
            let value = serde_wasm_bindgen::to_value(&diags)
                .expect("conversion to VSDiagnostic should succeed");
            event_callback
                .call1(&JsValue::null(), &value)
                .expect("callback should succeed");
        });
        QSharpLanguageService(inner)
    }

    pub fn update_code(&mut self, uri: &str, code: &str) {
        self.0.update_code(uri, code);
    }
}
