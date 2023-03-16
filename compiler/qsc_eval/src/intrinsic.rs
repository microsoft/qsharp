// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::ops::ControlFlow;

use qsc_ast::ast::Span;

use crate::{val::Value, Error, Reason, WithSpan};

pub(crate) fn invoke_intrinsic(
    name: &str,
    name_span: Span,
    args: Value,
    args_span: Span,
) -> ControlFlow<Reason, Value> {
    match name {
        "Length" => match args.try_into_array().with_span(args_span)?.len().try_into() {
            Ok(len) => ControlFlow::Continue(Value::Int(len)),
            Err(_) => ControlFlow::Break(Reason::Error(Error::IntegerSize(args_span))),
        },

        #[allow(clippy::cast_precision_loss)]
        "IntAsDouble" => {
            let val: i64 = args.try_into().with_span(args_span)?;
            ControlFlow::Continue(Value::Double(val as f64))
        }

        _ => ControlFlow::Break(Reason::Error(Error::UnknownIntrinsic(name_span))),
    }
}
