// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(super) mod ast;

macro_rules! push_lint {
    ($lint_ty:ty, $span:expr, $buffer:expr) => {
        $buffer.push(Lint {
            span: $span,
            message: <$lint_ty>::MESSAGE,
            level: <$lint_ty>::LEVEL,
        })
    };
}

pub(crate) use push_lint;
