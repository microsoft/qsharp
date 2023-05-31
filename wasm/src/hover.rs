// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{Hover, Span};
use wasm_bindgen::prelude::*;

pub fn get_hover(_: &str, offset: u32) -> Result<JsValue, JsValue> {
    let hover = Hover {
        contents: "Hello, world!".to_string(),
        span: Span {
            start: offset,
            end: offset + 1,
        },
    };
    Ok(serde_wasm_bindgen::to_value(&hover)?)
}
