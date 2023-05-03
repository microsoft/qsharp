// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::{
    output::Receiver,
    val::{Qubit, Value},
    Error,
};
use num_bigint::BigInt;
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
use qsc_data_structures::span::Span;
use rand::Rng;

#[allow(clippy::too_many_lines)]
pub(crate) fn invoke_intrinsic(
    name: &str,
    name_span: Span,
    args: Value,
    args_span: Span,
    out: &mut dyn Receiver,
) -> Result<Value, Error> {
    if name.starts_with("__quantum__qis__") {
        invoke_quantum_intrinsic(name, name_span, args, args_span)
    } else {
        match name {
            "Length" => match args.unwrap_array().len().try_into() {
                Ok(len) => Ok(Value::Int(len)),
                Err(_) => Err(Error::ArrayTooLarge(args_span)),
            },

            #[allow(clippy::cast_precision_loss)]
            "IntAsDouble" => {
                let val = args.unwrap_int();
                Ok(Value::Double(val as f64))
            }

            "IntAsBigInt" => {
                let val = args.unwrap_int();
                Ok(Value::BigInt(BigInt::from(val)))
            }

            "DumpMachine" => {
                let (state, qubit_count) = capture_quantum_state();
                match out.state(state, qubit_count) {
                    Ok(_) => Ok(Value::unit()),
                    Err(_) => Err(Error::Output(name_span)),
                }
            }

            "Message" => match out.message(&args.unwrap_string()) {
                Ok(_) => Ok(Value::unit()),
                Err(_) => Err(Error::Output(name_span)),
            },

            "CheckZero" => Ok(Value::Bool(qubit_is_zero(args.unwrap_qubit().0))),

            "ArcCos" => {
                let val = args.unwrap_double();
                Ok(Value::Double(val.acos()))
            }

            "ArcSin" => {
                let val = args.unwrap_double();
                Ok(Value::Double(val.asin()))
            }

            "ArcTan" => {
                let val = args.unwrap_double();
                Ok(Value::Double(val.atan()))
            }

            "ArcTan2" => {
                let tup = args.unwrap_tuple();
                match &*tup {
                    [x, y] => {
                        let x = x.clone().unwrap_double();
                        let y = y.clone().unwrap_double();
                        Ok(Value::Double(x.atan2(y)))
                    }
                    _ => Err(Error::TupleArity(2, tup.len(), args_span)),
                }
            }

            "Cos" => {
                let val = args.unwrap_double();
                Ok(Value::Double(val.cos()))
            }

            "Cosh" => {
                let val = args.unwrap_double();
                Ok(Value::Double(val.cosh()))
            }

            "Sin" => {
                let val = args.unwrap_double();
                Ok(Value::Double(val.sin()))
            }

            "Sinh" => {
                let val = args.unwrap_double();
                Ok(Value::Double(val.sinh()))
            }

            "Tan" => {
                let val = args.unwrap_double();
                Ok(Value::Double(val.tan()))
            }

            "Tanh" => {
                let val = args.unwrap_double();
                Ok(Value::Double(val.tanh()))
            }

            "Sqrt" => {
                let val = args.unwrap_double();
                Ok(Value::Double(val.sqrt()))
            }

            "Log" => {
                let val = args.unwrap_double();
                Ok(Value::Double(val.ln()))
            }

            "DrawRandomInt" => {
                let tup = args.unwrap_tuple();
                match &*tup {
                    [lo, hi] => invoke_draw_random_int(lo.clone(), hi.clone(), args_span),
                    _ => Err(Error::TupleArity(2, tup.len(), args_span)),
                }
            }

            "Truncate" => {
                let val = args.unwrap_double();
                #[allow(clippy::cast_possible_truncation)]
                Ok(Value::Int(val as i64))
            }

            "__quantum__rt__qubit_allocate" => {
                let qubit = Qubit(__quantum__rt__qubit_allocate());
                Ok(Value::Qubit(qubit))
            }

            "__quantum__rt__qubit_release" => {
                let qubit = args.unwrap_qubit().0;
                if !qubit_is_zero(qubit) {
                    return Err(Error::ReleasedQubitNotZero(qubit as usize));
                }
                __quantum__rt__qubit_release(qubit);
                Ok(Value::unit())
            }

            _ => Err(Error::UnknownIntrinsic(name_span)),
        }
    }
}

fn invoke_draw_random_int(lo: Value, hi: Value, args_span: Span) -> Result<Value, Error> {
    let lo = lo.unwrap_int();
    let hi = hi.unwrap_int();
    if lo > hi {
        Err(Error::EmptyRange(args_span))
    } else {
        Ok(Value::Int(rand::thread_rng().gen_range(lo..=hi)))
    }
}

#[allow(clippy::too_many_lines)]
fn invoke_quantum_intrinsic(
    name: &str,
    name_span: Span,
    args: Value,
    args_span: Span,
) -> Result<Value, Error> {
    macro_rules! match_intrinsic {
        ($chosen_op:ident, $chosen_op_span:ident, $(($(1, $op1:ident),* $(2, $op2:ident),* $(3, $op3:ident),* $(2.1, $op21:ident),* $(3.1, $op31:ident),*)),* ) => {
            match $chosen_op {
                $($(stringify!($op1) => {
                    $op1(args.unwrap_qubit().0);
                    Ok(Value::unit())
                })*
                $(stringify!($op2) => {
                    let tup = args.unwrap_tuple();
                    match &*tup {
                        [x, y] =>  {
                            if x == y {
                                return Err(Error::QubitUniqueness(args_span));
                            }
                            $op2(
                                x.clone().unwrap_qubit().0,
                                y.clone().unwrap_qubit().0,
                            );
                            Ok(Value::unit())
                        }
                        _ => Err(Error::TupleArity(2, tup.len(), args_span))
                    }
                })*
                $(stringify!($op21) => {
                    let tup = args.unwrap_tuple();
                    match &*tup {
                        [x, y] =>  {
                            $op21(
                                x.clone().unwrap_double(),
                                y.clone().unwrap_qubit().0,
                            );
                            Ok(Value::unit())
                        }
                        _ => Err(Error::TupleArity(2, tup.len(), args_span))
                    }
                })*
                $(stringify!($op3) => {
                    let tup = args.unwrap_tuple();
                    match &*tup {
                        [x, y, z] => {
                            if x == y || y == z || x == z {
                                return Err(Error::QubitUniqueness(args_span));
                            }
                            $op3(
                                x.clone().unwrap_qubit().0,
                                y.clone().unwrap_qubit().0,
                                z.clone().unwrap_qubit().0,
                            );
                            Ok(Value::unit())
                        }
                        _ => Err(Error::TupleArity(3, tup.len(), args_span))
                    }
                })*
                $(stringify!($op31) => {
                    let tup = args.unwrap_tuple();
                    match &*tup {
                        [x, y, z] => {
                            if y == z {
                                return Err(Error::QubitUniqueness(args_span));
                            }
                            $op31(
                                x.clone().unwrap_double(),
                                y.clone().unwrap_qubit().0,
                                z.clone().unwrap_qubit().0,
                            );
                            Ok(Value::unit())
                        }
                        _ => Err(Error::TupleArity(3, tup.len(), args_span))
                    }
                })*
            )*

                "__quantum__qis__m__body" => {
                    let res = __quantum__qis__m__body(args.unwrap_qubit().0);
                    Ok(Value::Result(__quantum__rt__result_equal(
                        res,
                        __quantum__rt__result_get_one(),
                    )))
                }

                "__quantum__qis__mresetz__body" => {
                    let res = __quantum__qis__mresetz__body(args.unwrap_qubit().0);
                    Ok(Value::Result(__quantum__rt__result_equal(
                        res,
                        __quantum__rt__result_get_one(),
                    )))
                }

                _ => Err(Error::UnknownIntrinsic($chosen_op_span)),
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
        (2.1, __quantum__qis__rx__body),
        (3.1, __quantum__qis__rxx__body),
        (2.1, __quantum__qis__ry__body),
        (3.1, __quantum__qis__ryy__body),
        (2.1, __quantum__qis__rz__body),
        (3.1, __quantum__qis__rzz__body),
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
