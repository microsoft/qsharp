// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::lex::{RawLexer, RawToken};

pub struct Formatter {
    pub tokens: Vec<RawToken>,
}

impl Formatter {
    pub fn new(input: &str) -> Self {
        Self {
            tokens: RawLexer::new(input).collect::<Vec<_>>(),
        }
    }
}
