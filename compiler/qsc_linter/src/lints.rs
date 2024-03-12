// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(super) mod ast;
pub(super) mod hir;

macro_rules! push_lint {
    ($lint:expr, $span:expr, $buffer:expr) => {
        $buffer.push(Lint {
            span: $span,
            message: $lint.message,
            level: $lint.level,
        })
    };
}

pub(crate) use push_lint;
