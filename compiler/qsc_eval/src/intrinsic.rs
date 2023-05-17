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
use std::{array, ffi::c_void};

#[allow(clippy::too_many_lines)]
pub(crate) fn call(
    name: &str,
    name_span: Span,
    arg: Value,
    arg_span: Span,
    out: &mut dyn Receiver,
) -> Result<Value, Error> {
    match name {
        "Length" => match arg.unwrap_array().len().try_into() {
            Ok(len) => Ok(Value::Int(len)),
            Err(_) => Err(Error::ArrayTooLarge(arg_span)),
        },
        #[allow(clippy::cast_precision_loss)]
        "IntAsDouble" => Ok(Value::Double(arg.unwrap_int() as f64)),
        "IntAsBigInt" => Ok(Value::BigInt(BigInt::from(arg.unwrap_int()))),
        "DumpMachine" => {
            let (state, qubit_count) = capture_quantum_state();
            match out.state(state, qubit_count) {
                Ok(_) => Ok(Value::unit()),
                Err(_) => Err(Error::Output(name_span)),
            }
        }
        "Message" => match out.message(&arg.unwrap_string()) {
            Ok(_) => Ok(Value::unit()),
            Err(_) => Err(Error::Output(name_span)),
        },
        "CheckZero" => Ok(Value::Bool(qubit_is_zero(arg.unwrap_qubit().0))),
        "ArcCos" => Ok(Value::Double(arg.unwrap_double().acos())),
        "ArcSin" => Ok(Value::Double(arg.unwrap_double().asin())),
        "ArcTan" => Ok(Value::Double(arg.unwrap_double().atan())),
        "ArcTan2" => {
            let [x, y] = unwrap_tuple(arg);
            Ok(Value::Double(x.unwrap_double().atan2(y.unwrap_double())))
        }
        "Cos" => Ok(Value::Double(arg.unwrap_double().cos())),
        "Cosh" => Ok(Value::Double(arg.unwrap_double().cosh())),
        "Sin" => Ok(Value::Double(arg.unwrap_double().sin())),
        "Sinh" => Ok(Value::Double(arg.unwrap_double().sinh())),
        "Tan" => Ok(Value::Double(arg.unwrap_double().tan())),
        "Tanh" => Ok(Value::Double(arg.unwrap_double().tanh())),
        "Sqrt" => Ok(Value::Double(arg.unwrap_double().sqrt())),
        "Log" => Ok(Value::Double(arg.unwrap_double().ln())),
        "DrawRandomInt" => {
            let [lo, hi] = unwrap_tuple(arg);
            let lo = lo.unwrap_int();
            let hi = hi.unwrap_int();
            if lo > hi {
                Err(Error::EmptyRange(arg_span))
            } else {
                Ok(Value::Int(rand::thread_rng().gen_range(lo..=hi)))
            }
        }
        #[allow(clippy::cast_possible_truncation)]
        "Truncate" => Ok(Value::Int(arg.unwrap_double() as i64)),
        "__quantum__rt__qubit_allocate" => Ok(Value::Qubit(Qubit(__quantum__rt__qubit_allocate()))),
        "__quantum__rt__qubit_release" => {
            let qubit = arg.unwrap_qubit().0;
            if qubit_is_zero(qubit) {
                __quantum__rt__qubit_release(qubit);
                Ok(Value::unit())
            } else {
                Err(Error::ReleasedQubitNotZero(qubit as usize))
            }
        }
        "__quantum__qis__ccx__body" => three_qubit_gate(__quantum__qis__ccx__body, arg, arg_span),
        "__quantum__qis__cx__body" => two_qubit_gate(__quantum__qis__cx__body, arg, arg_span),
        "__quantum__qis__cy__body" => two_qubit_gate(__quantum__qis__cy__body, arg, arg_span),
        "__quantum__qis__cz__body" => two_qubit_gate(__quantum__qis__cz__body, arg, arg_span),
        "__quantum__qis__rx__body" => Ok(one_qubit_rotation(__quantum__qis__rx__body, arg)),
        "__quantum__qis__rxx__body" => two_qubit_rotation(__quantum__qis__rxx__body, arg, arg_span),
        "__quantum__qis__ry__body" => Ok(one_qubit_rotation(__quantum__qis__ry__body, arg)),
        "__quantum__qis__ryy__body" => two_qubit_rotation(__quantum__qis__ryy__body, arg, arg_span),
        "__quantum__qis__rz__body" => Ok(one_qubit_rotation(__quantum__qis__rz__body, arg)),
        "__quantum__qis__rzz__body" => two_qubit_rotation(__quantum__qis__rzz__body, arg, arg_span),
        "__quantum__qis__h__body" => Ok(one_qubit_gate(__quantum__qis__h__body, arg)),
        "__quantum__qis__s__body" => Ok(one_qubit_gate(__quantum__qis__s__body, arg)),
        "__quantum__qis__s__adj" => Ok(one_qubit_gate(__quantum__qis__s__adj, arg)),
        "__quantum__qis__t__body" => Ok(one_qubit_gate(__quantum__qis__t__body, arg)),
        "__quantum__qis__t__adj" => Ok(one_qubit_gate(__quantum__qis__t__adj, arg)),
        "__quantum__qis__x__body" => Ok(one_qubit_gate(__quantum__qis__x__body, arg)),
        "__quantum__qis__y__body" => Ok(one_qubit_gate(__quantum__qis__y__body, arg)),
        "__quantum__qis__z__body" => Ok(one_qubit_gate(__quantum__qis__z__body, arg)),
        "__quantum__qis__swap__body" => two_qubit_gate(__quantum__qis__swap__body, arg, arg_span),
        "__quantum__qis__reset__body" => Ok(one_qubit_gate(__quantum__qis__reset__body, arg)),
        "__quantum__qis__m__body" => {
            let res = __quantum__qis__m__body(arg.unwrap_qubit().0);
            Ok(Value::Result(__quantum__rt__result_equal(
                res,
                __quantum__rt__result_get_one(),
            )))
        }
        "__quantum__qis__mresetz__body" => {
            let res = __quantum__qis__mresetz__body(arg.unwrap_qubit().0);
            Ok(Value::Result(__quantum__rt__result_equal(
                res,
                __quantum__rt__result_get_one(),
            )))
        }
        _ => Err(Error::UnknownIntrinsic(name_span)),
    }
}

fn one_qubit_gate(gate: extern "C" fn(*mut c_void), arg: Value) -> Value {
    gate(arg.unwrap_qubit().0);
    Value::unit()
}

fn two_qubit_gate(
    gate: extern "C" fn(*mut c_void, *mut c_void),
    arg: Value,
    arg_span: Span,
) -> Result<Value, Error> {
    let [x, y] = unwrap_tuple(arg);
    if x == y {
        Err(Error::QubitUniqueness(arg_span))
    } else {
        gate(x.unwrap_qubit().0, y.unwrap_qubit().0);
        Ok(Value::unit())
    }
}

fn one_qubit_rotation(gate: extern "C" fn(f64, *mut c_void), arg: Value) -> Value {
    let [x, y] = unwrap_tuple(arg);
    gate(x.unwrap_double(), y.unwrap_qubit().0);
    Value::unit()
}

fn three_qubit_gate(
    gate: extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
    arg: Value,
    arg_span: Span,
) -> Result<Value, Error> {
    let [x, y, z] = unwrap_tuple(arg);
    if x == y || y == z || x == z {
        Err(Error::QubitUniqueness(arg_span))
    } else {
        gate(x.unwrap_qubit().0, y.unwrap_qubit().0, z.unwrap_qubit().0);
        Ok(Value::unit())
    }
}

fn two_qubit_rotation(
    gate: extern "C" fn(f64, *mut c_void, *mut c_void),
    arg: Value,
    arg_span: Span,
) -> Result<Value, Error> {
    let [x, y, z] = unwrap_tuple(arg);
    if y == z {
        Err(Error::QubitUniqueness(arg_span))
    } else {
        gate(x.unwrap_double(), y.unwrap_qubit().0, z.unwrap_qubit().0);
        Ok(Value::unit())
    }
}

fn unwrap_tuple<const N: usize>(value: Value) -> [Value; N] {
    let values = value.unwrap_tuple();
    array::from_fn(|i| values[i].clone())
}
