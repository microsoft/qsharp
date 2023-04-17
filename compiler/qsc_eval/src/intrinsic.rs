// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::ops::ControlFlow;

use qir_backend::{
    __quantum__qis__ccx__body, __quantum__qis__cx__body, __quantum__qis__cy__body,
    __quantum__qis__cz__body, __quantum__qis__h__body, __quantum__qis__m__body,
    __quantum__qis__mresetz__body, __quantum__qis__reset__body, __quantum__qis__rx__body,
    __quantum__qis__rxx__body, __quantum__qis__ry__body, __quantum__qis__ryy__body,
    __quantum__qis__rz__body, __quantum__qis__rzz__body, __quantum__qis__s__adj,
    __quantum__qis__s__body, __quantum__qis__swap__body, __quantum__qis__t__adj,
    __quantum__qis__t__body, __quantum__qis__x__body, __quantum__qis__y__body,
    __quantum__qis__z__body, __quantum__rt__qubit_allocate, __quantum__rt__qubit_release,
    capture_quantum_state, qubit_is_zero,
    result_bool::{__quantum__rt__result_equal, __quantum__rt__result_get_one},
};
use qsc_ast::ast::Span;

use crate::{
    output::Receiver,
    val::{Qubit, Value},
    Error, Reason, WithSpan,
};

pub(crate) fn invoke_intrinsic(
    name: &str,
    name_span: Span,
    args: Value,
    args_span: Span,
    out: &mut dyn Receiver,
) -> ControlFlow<Reason, Value> {
    if name.starts_with("__quantum__qis__") {
        invoke_quantum_intrinsic(name, name_span, args, args_span)
    } else {
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
                let (state, qubit_count) = capture_quantum_state();
                match out.state(state, qubit_count) {
                    Ok(_) => ControlFlow::Continue(Value::UNIT),
                    Err(_) => ControlFlow::Break(Reason::Error(Error::Output(name_span))),
                }
            }

            "Message" => match out.message(args.try_into().with_span(args_span)?) {
                Ok(_) => ControlFlow::Continue(Value::UNIT),
                Err(_) => ControlFlow::Break(Reason::Error(Error::Output(name_span))),
            },

            "AsString" => ControlFlow::Continue(Value::String(args.to_string())),

            "CheckZero" => ControlFlow::Continue(Value::Bool(qubit_is_zero(
                args.try_into().with_span(args_span)?,
            ))),

            "ArcCos" => {
                let val: f64 = args.try_into().with_span(args_span)?;
                ControlFlow::Continue(Value::Double(val.acos()))
            }

            "ArcSin" => {
                let val: f64 = args.try_into().with_span(args_span)?;
                ControlFlow::Continue(Value::Double(val.asin()))
            }

            "ArcTan" => {
                let val: f64 = args.try_into().with_span(args_span)?;
                ControlFlow::Continue(Value::Double(val.atan()))
            }

            "ArcTan2" => {
                let mut args = args.try_into_tuple().with_span(args_span)?;
                if args.len() == 2 {
                    let (a2, a1) = (
                        args.pop().expect("tuple should have 2 entries"),
                        args.pop().expect("tuple should have 2 entries"),
                    );
                    let val: f64 = a1.try_into().with_span(args_span)?;
                    ControlFlow::Continue(Value::Double(
                        val.atan2(a2.try_into().with_span(args_span)?),
                    ))
                } else {
                    ControlFlow::Break(Reason::Error(Error::TupleArity(2, args.len(), args_span)))
                }
            }

            "Cos" => {
                let val: f64 = args.try_into().with_span(args_span)?;
                ControlFlow::Continue(Value::Double(val.cos()))
            }

            "Cosh" => {
                let val: f64 = args.try_into().with_span(args_span)?;
                ControlFlow::Continue(Value::Double(val.cosh()))
            }

            "Sin" => {
                let val: f64 = args.try_into().with_span(args_span)?;
                ControlFlow::Continue(Value::Double(val.sin()))
            }

            "Sinh" => {
                let val: f64 = args.try_into().with_span(args_span)?;
                ControlFlow::Continue(Value::Double(val.sinh()))
            }

            "Tan" => {
                let val: f64 = args.try_into().with_span(args_span)?;
                ControlFlow::Continue(Value::Double(val.tan()))
            }

            "Tanh" => {
                let val: f64 = args.try_into().with_span(args_span)?;
                ControlFlow::Continue(Value::Double(val.tanh()))
            }

            "__quantum__rt__qubit_allocate" => {
                let qubit = Qubit(__quantum__rt__qubit_allocate());
                ControlFlow::Continue(Value::Qubit(qubit))
            }

            "__quantum__rt__qubit_release" => {
                __quantum__rt__qubit_release(args.try_into().with_span(args_span)?);
                ControlFlow::Continue(Value::UNIT)
            }

            _ => ControlFlow::Break(Reason::Error(Error::UnknownIntrinsic(name_span))),
        }
    }
}

fn invoke_quantum_intrinsic(
    name: &str,
    name_span: Span,
    args: Value,
    args_span: Span,
) -> ControlFlow<Reason, Value> {
    macro_rules! match_intrinsic {
        ($chosen_op:ident, $chosen_op_span:ident, $(($(1, $op1:ident),* $(2, $op2:ident),* $(3, $op3:ident),*)),* ) => {
            match $chosen_op {
                $($(stringify!($op1) => {
                    $op1(args.try_into().with_span(args_span)?);
                    ControlFlow::Continue(Value::UNIT)
                })*
                $(stringify!($op2) => {
                    let mut args = args.try_into_tuple().with_span(args_span)?;
                    if args.len() == 2 {
                        let (a2, a1) = (
                            args.pop().expect("tuple should have 2 entries"),
                            args.pop().expect("tuple should have 2 entries"),
                        );
                        $op2(
                            a1.try_into().with_span(args_span)?,
                            a2.try_into().with_span(args_span)?,
                        );
                        ControlFlow::Continue(Value::UNIT)
                    } else {
                        ControlFlow::Break(Reason::Error(Error::TupleArity(2, args.len(), args_span)))
                    }
                })*
                $(stringify!($op3) => {
                    let mut args = args.try_into_tuple().with_span(args_span)?;
                    if args.len() == 3 {
                        let (a3, a2, a1) = (
                            args.pop().expect("tuple should have 3 entries"),
                            args.pop().expect("tuple should have 3 entries"),
                            args.pop().expect("tuple should have 3 entries"),
                        );
                        $op3(
                            a1.try_into().with_span(args_span)?,
                            a2.try_into().with_span(args_span)?,
                            a3.try_into().with_span(args_span)?,
                        );
                        ControlFlow::Continue(Value::UNIT)
                    } else {
                        ControlFlow::Break(Reason::Error(Error::TupleArity(3, args.len(), args_span)))
                    }
                })*)*

                "__quantum__qis__m__body" => {
                    let res = __quantum__qis__m__body(args.try_into().with_span(args_span)?);
                    ControlFlow::Continue(Value::Result(__quantum__rt__result_equal(
                        res,
                        __quantum__rt__result_get_one(),
                    )))
                }

                "__quantum__qis__mresetz__body" => {
                    let res = __quantum__qis__mresetz__body(args.try_into().with_span(args_span)?);
                    ControlFlow::Continue(Value::Result(__quantum__rt__result_equal(
                        res,
                        __quantum__rt__result_get_one(),
                    )))
                }

                _ => ControlFlow::Break(Reason::Error(Error::UnknownIntrinsic($chosen_op_span))),
            }
        };
    }

    match_intrinsic!(
        name,
        name_span,
        (3, __quantum__qis__ccx__body),
        (2, __quantum__qis__cx__body),
        (2, __quantum__qis__cy__body),
        (2, __quantum__qis__cz__body),
        (2, __quantum__qis__rx__body),
        (3, __quantum__qis__rxx__body),
        (2, __quantum__qis__ry__body),
        (3, __quantum__qis__ryy__body),
        (2, __quantum__qis__rz__body),
        (3, __quantum__qis__rzz__body),
        (1, __quantum__qis__h__body),
        (1, __quantum__qis__s__body),
        (1, __quantum__qis__s__adj),
        (1, __quantum__qis__t__body),
        (1, __quantum__qis__t__adj),
        (1, __quantum__qis__x__body),
        (1, __quantum__qis__y__body),
        (1, __quantum__qis__z__body),
        (2, __quantum__qis__swap__body),
        (1, __quantum__qis__reset__body)
    )
}
