// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This module contains the types and functions used to build the
//! data-interop layer between Python and Q#.

use super::{Pauli, Result};
use num_bigint::BigInt;
use pyo3::{
    IntoPyObjectExt,
    exceptions::PyTypeError,
    prelude::*,
    types::{PyList, PyTuple},
};
use qsc::{
    fir::{self},
    hir::ty::{Prim, Ty},
    interpret::{self, Value},
};
use rustc_hash::FxHashMap;
use std::rc::Rc;

/// Instances of this enum represent a Q# type. This is used
/// to send the definitions of Q# UDTs defined by the user to Python
/// and creating equivalent Python dataclasses in `qsharp.code.*`.
#[pyclass]
#[derive(Clone)]
pub(super) enum TypeIR {
    Primitive(PrimitiveKind),
    Tuple(Vec<TypeIR>),
    Array(Vec<TypeIR>),
    Udt(UdtIR),
}

#[pymethods]
impl TypeIR {
    fn kind(&self) -> TypeKind {
        match self {
            Self::Primitive(_) => TypeKind::Primitive,
            Self::Tuple(_) => TypeKind::Tuple,
            Self::Array(_) => TypeKind::Array,
            Self::Udt(_) => TypeKind::Udt,
        }
    }

    fn unwrap_primitive(&self) -> PyResult<PrimitiveKind> {
        if let Self::Primitive(ty) = self {
            Ok(*ty)
        } else {
            Err(PyTypeError::new_err("type is not a primitive".to_string()))
        }
    }

    fn unwrap_tuple(&self) -> PyResult<Vec<TypeIR>> {
        if let Self::Tuple(ty) = self {
            Ok(ty.clone())
        } else {
            Err(PyTypeError::new_err("type is not a tuple".to_string()))
        }
    }

    fn unwrap_array(&self) -> PyResult<Vec<TypeIR>> {
        if let Self::Tuple(ty) = self {
            Ok(ty.clone())
        } else {
            Err(PyTypeError::new_err("type is not an array".to_string()))
        }
    }

    fn unwrap_udt(&self) -> PyResult<UdtIR> {
        if let Self::Udt(ty) = self {
            Ok(ty.clone())
        } else {
            Err(PyTypeError::new_err("type is not a UDT".to_string()))
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[pyclass(eq, eq_int, ord)]
pub(super) enum TypeKind {
    Primitive,
    Tuple,
    Array,
    Udt,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[pyclass(eq, eq_int, ord)]
pub(super) enum PrimitiveKind {
    Bool,
    Int,
    Double,
    Complex,
    String,
    Pauli,
    Result,
}

#[pyclass]
#[derive(Clone)]
pub(super) struct UdtIR {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    fields: Vec<(String, TypeIR)>,
}

fn is_complex_udt(udt: &qsc::hir::ty::Udt) -> bool {
    if let qsc::hir::ty::UdtDefKind::Tuple(fields) = &udt.definition.kind {
        if fields.len() != 2 {
            return false;
        }
        let qsc::hir::ty::UdtDefKind::Field(real) = &fields[0].kind else {
            return false;
        };
        let qsc::hir::ty::UdtDefKind::Field(imag) = &fields[1].kind else {
            return false;
        };
        return matches!(real.ty, Ty::Prim(Prim::Double))
            && matches!(imag.ty, Ty::Prim(Prim::Double))
            && &*udt.name == "Complex";
    }
    false
}

/// This type is used to send objects from Python to Q#.
/// It is a `HashMap` to make it simple checking that the
/// objects have all the required fields to match the UDTs
/// they represent, without considering the order of the fields.
pub(super) type UdtFields = FxHashMap<String, PyObject>;

/// This type is used to send instances of UDTs from Q# to Python.
/// It is a `Vec` and not a `HashMap` to preserve the order of the fields,
/// since that results in a better user experience when printing the
/// objects in Python.
#[pyclass]
pub(super) struct UdtValue {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    fields: Vec<(String, PyObject)>,
}

#[pyclass]
#[derive(Clone)]
pub(super) enum PrimitiveValue {
    Bool(bool),
    Int(i64),
    BigInt(BigInt),
    Double(f64),
    Complex(num_complex::Complex64),
    String(String),
    Result(Result),
    Pauli(Pauli),
}

pub(super) fn collect_udt_fields<'ctx, 'udt_def>(
    udt: &'udt_def qsc::hir::ty::Udt,
) -> PyResult<Vec<(Rc<str>, &'ctx Ty)>>
where
    'udt_def: 'ctx,
{
    let mut fields = Vec::new();
    collect_udt_fields_rec(&udt.name, &udt.definition, &mut fields)?;
    Ok(fields)
}

fn collect_udt_fields_rec<'ctx, 'udt_def>(
    udt_name: &str,
    udt_def: &'udt_def qsc::hir::ty::UdtDef,
    buffer: &mut Vec<(Rc<str>, &'ctx Ty)>,
) -> PyResult<()>
where
    'udt_def: 'ctx,
{
    match &udt_def.kind {
        qsc::hir::ty::UdtDefKind::Field(udt_field) => {
            if let Some(name) = udt_field.name.as_ref() {
                buffer.push((name.clone(), &udt_field.ty));
                Ok(())
            } else {
                Err(PyTypeError::new_err(format!(
                    "structs with anonymous fields are not supported: {udt_name}"
                )))
            }
        }
        qsc::hir::ty::UdtDefKind::Tuple(udt_defs) => {
            for udt_def in udt_defs {
                collect_udt_fields_rec(udt_name, udt_def, buffer)?;
            }
            Ok(())
        }
    }
}

/// Given a type, convert a Python object into a Q# value of that type. This will recur through tuples and arrays,
/// and will return an error if the type is not supported or the object cannot be converted.
pub(super) fn convert_value_ir_with_ty(
    ctx: &interpret::Interpreter,
    py: Python,
    obj: &PyObject,
    ty: &Ty,
) -> PyResult<Value> {
    match ty {
        Ty::Prim(prim_ty) => match prim_ty {
            Prim::Bool => Ok(Value::Bool(obj.extract::<bool>(py)?)),
            Prim::Int => Ok(Value::Int(obj.extract::<i64>(py)?)),
            Prim::BigInt => Ok(Value::BigInt(obj.extract::<BigInt>(py)?)),
            Prim::Double => Ok(Value::Double(obj.extract::<f64>(py)?)),
            Prim::String => Ok(Value::String(obj.extract::<String>(py)?.into())),
            Prim::Result => Ok(Value::Result(obj.extract::<Result>(py)?.into())),
            Prim::Pauli => Ok(Value::Pauli(obj.extract::<Pauli>(py)?.into())),
            Prim::Qubit | Prim::Range | Prim::RangeTo | Prim::RangeFrom | Prim::RangeFull => {
                unimplemented!("primitive input type: {prim_ty:?}")
            }
        },
        Ty::Tuple(tup) => {
            let objs = obj.extract::<Vec<PyObject>>(py)?;

            if tup.len() != objs.len() {
                return Err(PyTypeError::new_err(format!(
                    "mismatched tuple arity: expected {}, got {}",
                    tup.len(),
                    objs.len()
                )));
            }
            if objs.len() == 1 {
                convert_value_ir_with_ty(ctx, py, &objs[0], &tup[0])
            } else {
                let mut tuple = Vec::new();
                for (obj, ty) in objs.iter().zip(tup) {
                    tuple.push(convert_value_ir_with_ty(ctx, py, obj, ty)?);
                }
                Ok(Value::Tuple(tuple.into()))
            }
        }
        Ty::Array(ty) => {
            let objs = obj.extract::<Vec<PyObject>>(py)?;

            let ty = &**ty;
            let mut array = Vec::new();
            for obj in &objs {
                array.push(convert_value_ir_with_ty(ctx, py, obj, ty)?);
            }
            Ok(Value::Array(array.into()))
        }
        Ty::Udt(_, res) => {
            let qsc::hir::Res::Item(item_id) = res else {
                panic!("Udt should be an item");
            };
            let udt = ctx.udt_ty(item_id);

            // Handle `Complex` special case.
            if let Ok(v) = obj.extract::<num_complex::Complex64>(py) {
                if is_complex_udt(udt) {
                    let tuple = Value::Tuple(Rc::new([Value::Double(v.re), Value::Double(v.im)]));
                    return Ok(tuple);
                }
            }

            let udt_fields = obj.extract::<UdtFields>(py)?;

            let mut tuple = Vec::new();
            for (name, ty) in collect_udt_fields(udt)? {
                let Some(value) = udt_fields.get(&*name) else {
                    return Err(PyTypeError::new_err(format!(
                        "missing field {} in {}",
                        name, udt.name,
                    )));
                };
                tuple.push(convert_value_ir_with_ty(ctx, py, value, ty)?);
            }
            Ok(Value::Tuple(tuple.into()))
        }
        _ => unimplemented!("input type: {ty}"),
    }
}

pub(super) fn type_ir_from_qsharp_ty(ctx: &interpret::Interpreter, ty: &Ty) -> PyResult<TypeIR> {
    match ty {
        Ty::Prim(prim) => {
            let prim = match prim {
                Prim::Bool => PrimitiveKind::Bool,
                Prim::Int | Prim::BigInt => PrimitiveKind::Int,
                Prim::Double => PrimitiveKind::Double,
                Prim::String => PrimitiveKind::String,
                Prim::Pauli => PrimitiveKind::Pauli,
                Prim::Result => PrimitiveKind::Result,

                Prim::Qubit | Prim::Range | Prim::RangeTo | Prim::RangeFrom | Prim::RangeFull => {
                    return Err(PyTypeError::new_err(format!(
                        "unsupported interop type: `{ty}`"
                    )));
                }
            };
            Ok(TypeIR::Primitive(prim))
        }
        Ty::Array(ty) => Ok(TypeIR::Array(vec![type_ir_from_qsharp_ty(ctx, ty)?])),
        Ty::Tuple(items) => {
            let mut tuple = Vec::new();
            for item in items {
                tuple.push(type_ir_from_qsharp_ty(ctx, item)?);
            }
            Ok(TypeIR::Tuple(tuple))
        }
        Ty::Udt(name, res) => {
            let qsc::hir::Res::Item(item_id) = res else {
                panic!("Udt should be an item");
            };
            let udt = ctx.udt_ty(item_id);

            // Handle `Complex` special case.
            if is_complex_udt(udt) {
                return Ok(TypeIR::Primitive(PrimitiveKind::Complex));
            }

            let udt_fields = collect_udt_fields(udt)?;
            let mut fields = Vec::new();

            for (name, ty) in udt_fields {
                fields.push((name.to_string(), type_ir_from_qsharp_ty(ctx, ty)?));
            }

            Ok(TypeIR::Udt(UdtIR {
                name: name.to_string(),
                fields,
            }))
        }
        Ty::Param { .. } | Ty::Infer(..) | Ty::Arrow(..) | Ty::Err => Err(PyTypeError::new_err(
            format!("unsupported interop type: `{ty}`"),
        )),
    }
}

pub(super) fn typed_value_to_python_obj(
    ctx: &interpret::Interpreter,
    py: Python,
    value: &Value,
    ty: &Ty,
) -> PyResult<PyObject> {
    match value {
        Value::Int(val) => val.into_py_any(py),
        Value::BigInt(val) => val.into_py_any(py),
        Value::Double(val) => val.into_py_any(py),
        Value::Bool(val) => val.into_py_any(py),
        Value::String(val) => val.into_py_any(py),
        Value::Result(val) => {
            let val = match val {
                qsc::interpret::Result::Id(_) => {
                    panic!("unexpected Result::Id in typed_value_to_value_ir")
                }
                qsc::interpret::Result::Val(true) => Result::One,
                qsc::interpret::Result::Val(false) => Result::Zero,
                qsc::interpret::Result::Loss => Result::Loss,
            };
            val.into_py_any(py)
        }
        Value::Pauli(val) => {
            let val = match val {
                fir::Pauli::I => Pauli::I,
                fir::Pauli::X => Pauli::X,
                fir::Pauli::Y => Pauli::Y,
                fir::Pauli::Z => Pauli::Z,
            };
            val.into_py_any(py)
        }
        Value::Tuple(values) => match ty {
            Ty::Tuple(items) => {
                let mut tuple = Vec::new();
                for (val, ty) in values.iter().zip(items) {
                    tuple.push(typed_value_to_python_obj(ctx, py, val, ty)?);
                }

                // Special case Value::UNIT maps to None.
                if tuple.is_empty() {
                    Ok(py.None())
                } else {
                    PyTuple::new(py, tuple)?.into_py_any(py)
                }
            }
            Ty::Udt(_, res) => {
                let qsc::hir::Res::Item(item_id) = res else {
                    panic!("Udt should be an item");
                };
                let udt = ctx.udt_ty(item_id);
                if is_complex_udt(udt) {
                    let re = values[0].clone().unwrap_double();
                    let im = values[1].clone().unwrap_double();
                    let val = num_complex::Complex { re, im };
                    return val.into_py_any(py);
                }
                let ty_fields = collect_udt_fields(udt)?;
                let mut fields = Vec::new();
                for (value, (name, ty)) in values.iter().zip(ty_fields) {
                    fields.push((
                        name.to_string(),
                        typed_value_to_python_obj(ctx, py, value, ty)?,
                    ));
                }
                UdtValue {
                    name: udt.name.to_string(),
                    fields,
                }
                .into_py_any(py)
            }
            _ => unreachable!(),
        },
        Value::Array(values) => {
            let Ty::Array(ty) = ty else {
                unreachable!();
            };
            let mut array = Vec::new();
            for val in values.iter() {
                array.push(typed_value_to_python_obj(ctx, py, val, ty)?);
            }

            PyList::new(py, array)?.into_py_any(py)
        }
        _ => format!("<{}> {}", value.type_name(), value).into_py_any(py),
    }
}
