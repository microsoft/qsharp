// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(super) mod ast;
pub(super) mod hir;

macro_rules! lint {
    ($lint:expr, $span:expr) => {
        Lint {
            span: $span,
            level: $lint.level,
            message: $lint.message,
            help: $lint.help,
        }
    };
}

pub(crate) use lint;
