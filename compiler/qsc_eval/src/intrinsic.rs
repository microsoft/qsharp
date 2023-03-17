// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::{ops::ControlFlow, ptr::null_mut};

use qir_backend::{
    __quantum__qis__ccx__body, __quantum__qis__cx__body, __quantum__qis__cy__body,
    __quantum__qis__cz__body, __quantum__qis__dumpmachine__body, __quantum__qis__h__body,
    __quantum__qis__m__body, __quantum__qis__reset__body, __quantum__qis__rx__body,
    __quantum__qis__rxx__body, __quantum__qis__ry__body, __quantum__qis__ryy__body,
    __quantum__qis__rz__body, __quantum__qis__rzz__body, __quantum__qis__s__adj,
    __quantum__qis__s__body, __quantum__qis__swap__body, __quantum__qis__t__adj,
    __quantum__qis__t__body, __quantum__qis__x__body, __quantum__qis__y__body,
    __quantum__qis__z__body, qubit_is_zero,
    result_bool::{__quantum__rt__result_equal, __quantum__rt__result_get_one},
};
use qsc_ast::ast::Span;

use crate::{val::Value, Error, Reason, WithSpan};

#[allow(clippy::too_many_lines)]
pub(crate) fn invoke_intrinsic(
    name: &str,
    name_span: Span,
    args: Value,
    args_span: Span,
) -> ControlFlow<Reason, Value> {
    match name {
        "Length" => match args.try_into_array().with_span(args_span)?.len().try_into() {
            Ok(len) => ControlFlow::Continue(Value::Int(len)),
            Err(_) => ControlFlow::Break(Reason::Error(Error::ArrayTooLarge(args_span))),
        },

        #[allow(clippy::cast_precision_loss)]
        "IntAsDouble" => {
            let val: i64 = args.try_into().with_span(args_span)?;
            ControlFlow::Continue(Value::Double(val as f64))
        }

        "DumpMachine" => {
            // TODO(swernli): Replace with Rust state dump call when added to qir-backend.
            __quantum__qis__dumpmachine__body(null_mut());
            ControlFlow::Continue(Value::UNIT)
        }

        "CheckZero" => ControlFlow::Continue(Value::Bool(qubit_is_zero(
            args.try_into().with_span(args_span)?,
        ))),

        "__quantum__qis__ccx__body" => {
            let mut args = args.try_into_tuple().with_span(args_span)?;
            if args.len() == 3 {
                let (q3, q2, q1) = (
                    args.pop().expect("tuple should have 3 entries"),
                    args.pop().expect("tuple should have 3 entries"),
                    args.pop().expect("tuple should have 3 entries"),
                );
                __quantum__qis__ccx__body(
                    q1.try_into().with_span(args_span)?,
                    q2.try_into().with_span(args_span)?,
                    q3.try_into().with_span(args_span)?,
                );
                ControlFlow::Continue(Value::UNIT)
            } else {
                ControlFlow::Break(Reason::Error(Error::TupleArity(3, args.len(), args_span)))
            }
        }

        "__quantum__qis__cx__body" => {
            let mut args = args.try_into_tuple().with_span(args_span)?;
            if args.len() == 2 {
                let (q2, q1) = (
                    args.pop().expect("tuple should have 2 entries"),
                    args.pop().expect("tuple should have 2 entries"),
                );
                __quantum__qis__cx__body(
                    q1.try_into().with_span(args_span)?,
                    q2.try_into().with_span(args_span)?,
                );
                ControlFlow::Continue(Value::UNIT)
            } else {
                ControlFlow::Break(Reason::Error(Error::TupleArity(2, args.len(), args_span)))
            }
        }

        "__quantum__qis__cy__body" => {
            let mut args = args.try_into_tuple().with_span(args_span)?;
            if args.len() == 2 {
                let (q2, q1) = (
                    args.pop().expect("tuple should have 2 entries"),
                    args.pop().expect("tuple should have 2 entries"),
                );
                __quantum__qis__cy__body(
                    q1.try_into().with_span(args_span)?,
                    q2.try_into().with_span(args_span)?,
                );
                ControlFlow::Continue(Value::UNIT)
            } else {
                ControlFlow::Break(Reason::Error(Error::TupleArity(2, args.len(), args_span)))
            }
        }

        "__quantum__qis__cz__body" => {
            let mut args = args.try_into_tuple().with_span(args_span)?;
            if args.len() == 2 {
                let (q2, q1) = (
                    args.pop().expect("tuple should have 2 entries"),
                    args.pop().expect("tuple should have 2 entries"),
                );
                __quantum__qis__cz__body(
                    q1.try_into().with_span(args_span)?,
                    q2.try_into().with_span(args_span)?,
                );
                ControlFlow::Continue(Value::UNIT)
            } else {
                ControlFlow::Break(Reason::Error(Error::TupleArity(2, args.len(), args_span)))
            }
        }

        "__quantum__qis__rx__body" => {
            let mut args = args.try_into_tuple().with_span(args_span)?;
            if args.len() == 2 {
                let (q, angle) = (
                    args.pop().expect("tuple should have 2 entries"),
                    args.pop().expect("tuple should have 2 entries"),
                );
                __quantum__qis__rx__body(
                    angle.try_into().with_span(args_span)?,
                    q.try_into().with_span(args_span)?,
                );
                ControlFlow::Continue(Value::UNIT)
            } else {
                ControlFlow::Break(Reason::Error(Error::TupleArity(2, args.len(), args_span)))
            }
        }

        "__quantum__qis__rxx__body" => {
            let mut args = args.try_into_tuple().with_span(args_span)?;
            if args.len() == 3 {
                let (q2, q1, angle) = (
                    args.pop().expect("tuple should have 3 entries"),
                    args.pop().expect("tuple should have 3 entries"),
                    args.pop().expect("tuple should have 3 entries"),
                );
                __quantum__qis__rxx__body(
                    angle.try_into().with_span(args_span)?,
                    q1.try_into().with_span(args_span)?,
                    q2.try_into().with_span(args_span)?,
                );
                ControlFlow::Continue(Value::UNIT)
            } else {
                ControlFlow::Break(Reason::Error(Error::TupleArity(3, args.len(), args_span)))
            }
        }

        "__quantum__qis__ry__body" => {
            let mut args = args.try_into_tuple().with_span(args_span)?;
            if args.len() == 2 {
                let (q, angle) = (
                    args.pop().expect("tuple should have 2 entries"),
                    args.pop().expect("tuple should have 2 entries"),
                );
                __quantum__qis__ry__body(
                    angle.try_into().with_span(args_span)?,
                    q.try_into().with_span(args_span)?,
                );
                ControlFlow::Continue(Value::UNIT)
            } else {
                ControlFlow::Break(Reason::Error(Error::TupleArity(2, args.len(), args_span)))
            }
        }

        "__quantum__qis__ryy__body" => {
            let mut args = args.try_into_tuple().with_span(args_span)?;
            if args.len() == 3 {
                let (q2, q1, angle) = (
                    args.pop().expect("tuple should have 3 entries"),
                    args.pop().expect("tuple should have 3 entries"),
                    args.pop().expect("tuple should have 3 entries"),
                );
                __quantum__qis__ryy__body(
                    angle.try_into().with_span(args_span)?,
                    q1.try_into().with_span(args_span)?,
                    q2.try_into().with_span(args_span)?,
                );
                ControlFlow::Continue(Value::UNIT)
            } else {
                ControlFlow::Break(Reason::Error(Error::TupleArity(3, args.len(), args_span)))
            }
        }

        "__quantum__qis__rz__body" => {
            let mut args = args.try_into_tuple().with_span(args_span)?;
            if args.len() == 2 {
                let (q, angle) = (
                    args.pop().expect("tuple should have 2 entries"),
                    args.pop().expect("tuple should have 2 entries"),
                );
                __quantum__qis__rz__body(
                    angle.try_into().with_span(args_span)?,
                    q.try_into().with_span(args_span)?,
                );
                ControlFlow::Continue(Value::UNIT)
            } else {
                ControlFlow::Break(Reason::Error(Error::TupleArity(2, args.len(), args_span)))
            }
        }

        "__quantum__qis__rzz__body" => {
            let mut args = args.try_into_tuple().with_span(args_span)?;
            if args.len() == 3 {
                let (q2, q1, angle) = (
                    args.pop().expect("tuple should have 3 entries"),
                    args.pop().expect("tuple should have 3 entries"),
                    args.pop().expect("tuple should have 3 entries"),
                );
                __quantum__qis__rzz__body(
                    angle.try_into().with_span(args_span)?,
                    q1.try_into().with_span(args_span)?,
                    q2.try_into().with_span(args_span)?,
                );
                ControlFlow::Continue(Value::UNIT)
            } else {
                ControlFlow::Break(Reason::Error(Error::TupleArity(3, args.len(), args_span)))
            }
        }

        "__quantum__qis__h__body" => {
            __quantum__qis__h__body(args.try_into().with_span(args_span)?);
            ControlFlow::Continue(Value::UNIT)
        }

        "__quantum__qis__s__body" => {
            __quantum__qis__s__body(args.try_into().with_span(args_span)?);
            ControlFlow::Continue(Value::UNIT)
        }

        "__quantum__qis__s__adj" => {
            __quantum__qis__s__adj(args.try_into().with_span(args_span)?);
            ControlFlow::Continue(Value::UNIT)
        }

        "__quantum__qis__t__body" => {
            __quantum__qis__t__body(args.try_into().with_span(args_span)?);
            ControlFlow::Continue(Value::UNIT)
        }

        "__quantum__qis__t__adj" => {
            __quantum__qis__t__adj(args.try_into().with_span(args_span)?);
            ControlFlow::Continue(Value::UNIT)
        }

        "__quantum__qis__x__body" => {
            __quantum__qis__x__body(args.try_into().with_span(args_span)?);
            ControlFlow::Continue(Value::UNIT)
        }

        "__quantum__qis__y__body" => {
            __quantum__qis__y__body(args.try_into().with_span(args_span)?);
            ControlFlow::Continue(Value::UNIT)
        }

        "__quantum__qis__z__body" => {
            __quantum__qis__z__body(args.try_into().with_span(args_span)?);
            ControlFlow::Continue(Value::UNIT)
        }

        "__quantum__qis__swap__body" => {
            let mut args = args.try_into_tuple().with_span(args_span)?;
            if args.len() == 2 {
                let (q2, q1) = (
                    args.pop().expect("tuple should have 2 entries"),
                    args.pop().expect("tuple should have 2 entries"),
                );
                __quantum__qis__swap__body(
                    q1.try_into().with_span(args_span)?,
                    q2.try_into().with_span(args_span)?,
                );
                ControlFlow::Continue(Value::UNIT)
            } else {
                ControlFlow::Break(Reason::Error(Error::TupleArity(2, args.len(), args_span)))
            }
        }

        "__quantum__qis__m__body" => {
            let res = __quantum__qis__m__body(args.try_into().with_span(args_span)?);
            ControlFlow::Continue(Value::Result(__quantum__rt__result_equal(
                res,
                __quantum__rt__result_get_one(),
            )))
        }

        "__quantum__qis__reset__body" => {
            __quantum__qis__reset__body(args.try_into().with_span(args_span)?);
            ControlFlow::Continue(Value::UNIT)
        }

        "__quantum__qis__mresetz__body" => {
            // TODO(swernli): Requires update to qir-backend to support intrinsic measure and reset.
            // let res = __quantum__qis__mresetz__body(args.try_into().with_span(args_span)?);
            // ControlFlow::Continue(Value::Result(__quantum__rt__result_equal(
            //     res,
            //     __quantum__rt__result_get_one(),
            // )))
            ControlFlow::Break(Reason::Error(Error::Unimplemented(name_span)))
        }

        _ => ControlFlow::Break(Reason::Error(Error::UnknownIntrinsic(name_span))),
    }
}
