// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(super) mod ast;
pub(super) mod hir;

macro_rules! push_lint {
    ($lint:expr, $span:expr, $buffer:expr) => {
        $buffer.push(Lint {
            span: $span,
            level: $lint.level,
            message: $lint.message,
            help: $lint.help,
        })
    };
}

pub(crate) use push_lint;
