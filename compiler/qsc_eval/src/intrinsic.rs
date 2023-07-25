// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::{
    backend::Backend,
    output::Receiver,
    val::{Qubit, Value},
    Error,
};
use num_bigint::BigInt;
use qsc_data_structures::span::Span;
use rand::Rng;
use std::array;

pub(crate) fn call(
    name: &str,
    name_span: Span,
    arg: Value,
    arg_span: Span,
    sim: &mut dyn Backend,
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
            let (state, qubit_count) = sim.capture_quantum_state();
            match out.state(state, qubit_count) {
                Ok(_) => Ok(Value::unit()),
                Err(_) => Err(Error::OutputFail(name_span)),
            }
        }
        "Message" => match out.message(&arg.unwrap_string()) {
            Ok(_) => Ok(Value::unit()),
            Err(_) => Err(Error::OutputFail(name_span)),
        },
        "CheckZero" => Ok(Value::Bool(sim.qubit_is_zero(arg.unwrap_qubit().0))),
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
        "DrawRandomDouble" => {
            let [lo, hi] = unwrap_tuple(arg);
            let lo = lo.unwrap_double();
            let hi = hi.unwrap_double();
            if lo > hi {
                Err(Error::EmptyRange(arg_span))
            } else {
                Ok(Value::Double(rand::thread_rng().gen_range(lo..=hi)))
            }
        }
        #[allow(clippy::cast_possible_truncation)]
        "Truncate" => Ok(Value::Int(arg.unwrap_double() as i64)),
        "__quantum__rt__qubit_allocate" => Ok(Value::Qubit(Qubit(sim.qubit_allocate()))),
        "__quantum__rt__qubit_release" => {
            let qubit = arg.unwrap_qubit().0;
            if sim.qubit_is_zero(qubit) {
                sim.qubit_release(qubit);
                Ok(Value::unit())
            } else {
                Err(Error::ReleasedQubitNotZero(qubit))
            }
        }
        "__quantum__qis__ccx__body" => {
            three_qubit_gate(|ctl0, ctl1, q| sim.ccx(ctl0, ctl1, q), arg, arg_span)
        }
        "__quantum__qis__cx__body" => two_qubit_gate(|ctl, q| sim.cx(ctl, q), arg, arg_span),
        "__quantum__qis__cy__body" => two_qubit_gate(|ctl, q| sim.cy(ctl, q), arg, arg_span),
        "__quantum__qis__cz__body" => two_qubit_gate(|ctl, q| sim.cz(ctl, q), arg, arg_span),
        "__quantum__qis__rx__body" => Ok(one_qubit_rotation(|theta, q| sim.rx(theta, q), arg)),
        "__quantum__qis__rxx__body" => {
            two_qubit_rotation(|theta, q0, q1| sim.rxx(theta, q0, q1), arg, arg_span)
        }
        "__quantum__qis__ry__body" => Ok(one_qubit_rotation(|theta, q| sim.ry(theta, q), arg)),
        "__quantum__qis__ryy__body" => {
            two_qubit_rotation(|theta, q0, q1| sim.ryy(theta, q0, q1), arg, arg_span)
        }
        "__quantum__qis__rz__body" => Ok(one_qubit_rotation(|theta, q| sim.rz(theta, q), arg)),
        "__quantum__qis__rzz__body" => {
            two_qubit_rotation(|theta, q0, q1| sim.rzz(theta, q0, q1), arg, arg_span)
        }
        "__quantum__qis__h__body" => Ok(one_qubit_gate(|q| sim.h(q), arg)),
        "__quantum__qis__s__body" => Ok(one_qubit_gate(|q| sim.s(q), arg)),
        "__quantum__qis__s__adj" => Ok(one_qubit_gate(|q| sim.sadj(q), arg)),
        "__quantum__qis__t__body" => Ok(one_qubit_gate(|q| sim.t(q), arg)),
        "__quantum__qis__t__adj" => Ok(one_qubit_gate(|q| sim.tadj(q), arg)),
        "__quantum__qis__x__body" => Ok(one_qubit_gate(|q| sim.x(q), arg)),
        "__quantum__qis__y__body" => Ok(one_qubit_gate(|q| sim.y(q), arg)),
        "__quantum__qis__z__body" => Ok(one_qubit_gate(|q| sim.z(q), arg)),
        "__quantum__qis__swap__body" => two_qubit_gate(|q0, q1| sim.swap(q0, q1), arg, arg_span),
        "__quantum__qis__reset__body" => Ok(one_qubit_gate(|q| sim.reset(q), arg)),
        "__quantum__qis__m__body" => Ok(Value::Result(sim.m(arg.unwrap_qubit().0))),
        "__quantum__qis__mresetz__body" => Ok(Value::Result(sim.mresetz(arg.unwrap_qubit().0))),
        _ => Err(Error::UnknownIntrinsic(name.to_string(), name_span)),
    }
}

fn one_qubit_gate(mut gate: impl FnMut(usize), arg: Value) -> Value {
    gate(arg.unwrap_qubit().0);
    Value::unit()
}

fn two_qubit_gate(
    mut gate: impl FnMut(usize, usize),
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

fn one_qubit_rotation(mut gate: impl FnMut(f64, usize), arg: Value) -> Value {
    let [x, y] = unwrap_tuple(arg);
    gate(x.unwrap_double(), y.unwrap_qubit().0);
    Value::unit()
}

fn three_qubit_gate(
    mut gate: impl FnMut(usize, usize, usize),
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
    mut gate: impl FnMut(f64, usize, usize),
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
