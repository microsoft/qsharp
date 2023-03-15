// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::ops::ControlFlow;

use qsc_ast::ast::Span;

use crate::{val::Value, ErrorKind, Reason, WithSpan};

pub(crate) fn invoke_intrinsic(
    name: &str,
    name_span: Span,
    args: Value,
    args_span: Span,
) -> ControlFlow<Reason, Value> {
    match (name, args) {
        ("Length", arr) => match arr.try_into_array().with_span(args_span)?.len().try_into() {
            Ok(len) => ControlFlow::Continue(Value::Int(len)),
            Err(_) => ControlFlow::Break(Reason::Error(args_span, ErrorKind::IntegerSize)),
        },

        ("IntAsDouble", val) => {
            let val: i64 = val.try_into().with_span(args_span)?;
            let val: i32 = match val.try_into() {
                Ok(i) => ControlFlow::Continue(i),
                Err(_) => ControlFlow::Break(Reason::Error(args_span, ErrorKind::IntegerSize)),
            }?;
            match val.try_into() {
                Ok(d) => ControlFlow::Continue(Value::Double(d)),
                Err(_) => ControlFlow::Break(Reason::Error(args_span, ErrorKind::IntegerSize)),
            }
        }

        _ => ControlFlow::Break(Reason::Error(name_span, ErrorKind::UnknownIntrinsic)),
    }
}
