// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This module contains the types and functions used to build the
//! data-interop layer between Python and Q#.

use super::{Pauli, Result};
use crate::interpreter::QSharpError;
use num_bigint::BigInt;
use pyo3::{IntoPyObjectExt, prelude::*};
use qsc::{
    fir::{self},
    hir::ty::{Prim, Ty},
    interpret::{self, Value},
};
use rustc_hash::FxHashMap;
use std::{fmt::Write, rc::Rc};

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
            Err(QSharpError::new_err(
                "ValueError: type is not a primitive".to_string(),
            ))
        }
    }

    fn unwrap_tuple(&self) -> PyResult<Vec<TypeIR>> {
        if let Self::Tuple(ty) = self {
            Ok(ty.clone())
        } else {
            Err(QSharpError::new_err(
                "ValueError: type is not a tuple".to_string(),
            ))
        }
    }

    fn unwrap_array(&self) -> PyResult<Vec<TypeIR>> {
        if let Self::Tuple(ty) = self {
            Ok(ty.clone())
        } else {
            Err(QSharpError::new_err(
                "ValueError: type is not an array".to_string(),
            ))
        }
    }

    fn unwrap_udt(&self) -> PyResult<UdtIR> {
        if let Self::Udt(ty) = self {
            Ok(ty.clone())
        } else {
            Err(QSharpError::new_err(
                "ValueError: type is not a UDT".to_string(),
            ))
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

#[pyclass]
#[derive(Clone)]
pub(super) enum ValueIR {
    Primitive(PrimitiveValue),
    Tuple(Vec<ValueIR>),
    Array(Vec<ValueIR>),
    Udt(UdtValue),
}

impl ValueIR {
    fn is_complex(&self) -> bool {
        matches!(self, Self::Primitive(PrimitiveValue::Complex(_)))
    }

    fn ty_name(&self) -> String {
        fn ty_name_rec(val: &ValueIR, f: &mut String) -> std::fmt::Result {
            match val {
                ValueIR::Primitive(primitive) => match primitive {
                    PrimitiveValue::Bool(_) => write!(f, "Bool"),
                    PrimitiveValue::Int(_) => write!(f, "Int"),
                    PrimitiveValue::BigInt(_) => write!(f, "BigInt"),
                    PrimitiveValue::Double(_) => write!(f, "Double"),
                    PrimitiveValue::Complex(_) => write!(f, "Complex"),
                    PrimitiveValue::String(_) => write!(f, "String"),
                    PrimitiveValue::Result(_) => write!(f, "Result"),
                    PrimitiveValue::Pauli(_) => write!(f, "Pauli"),
                },
                ValueIR::Tuple(tuple) => {
                    write!(f, "(")?;
                    for value in tuple {
                        ty_name_rec(value, f)?;
                    }
                    write!(f, ")")
                }
                ValueIR::Array(array) => {
                    write!(f, "[")?;
                    for value in array {
                        ty_name_rec(value, f)?;
                    }
                    write!(f, "]")
                }
                ValueIR::Udt(_) => write!(f, "Udt"),
            }
        }
        let mut buffer = String::new();
        ty_name_rec(self, &mut buffer).expect("writing to String should succeed");
        buffer
    }
}

#[pymethods]
impl ValueIR {
    fn kind(&self) -> TypeKind {
        match self {
            Self::Primitive(_) => TypeKind::Primitive,
            Self::Tuple(_) => TypeKind::Tuple,
            Self::Array(_) => TypeKind::Array,
            Self::Udt(_) => TypeKind::Udt,
        }
    }

    fn unwrap_primitive(&self, py: Python) -> PyResult<PyObject> {
        if let Self::Primitive(prim) = self {
            match prim {
                PrimitiveValue::Bool(val) => val.into_py_any(py),
                PrimitiveValue::Int(val) => val.into_py_any(py),
                PrimitiveValue::BigInt(val) => val.into_py_any(py),
                PrimitiveValue::Double(val) => val.into_py_any(py),
                PrimitiveValue::Complex(val) => val.into_py_any(py),
                PrimitiveValue::String(val) => val.into_py_any(py),
                PrimitiveValue::Result(val) => val.into_py_any(py),
                PrimitiveValue::Pauli(val) => val.into_py_any(py),
            }
        } else {
            Err(QSharpError::new_err(
                "ValueError: value is not a primitive".to_string(),
            ))
        }
    }

    fn unwrap_tuple(&self, py: Python) -> PyResult<PyObject> {
        if let Self::Tuple(tuple) = self {
            tuple.clone().into_py_any(py)
        } else {
            Err(QSharpError::new_err(
                "ValueError: value is not a tuple".to_string(),
            ))
        }
    }

    fn unwrap_array(&self, py: Python) -> PyResult<PyObject> {
        if let Self::Array(tuple) = self {
            tuple.clone().into_py_any(py)
        } else {
            Err(QSharpError::new_err(
                "ValueError: value is not an array".to_string(),
            ))
        }
    }

    fn unwrap_udt(&self) -> PyResult<UdtValue> {
        if let Self::Udt(udt) = self {
            Ok(udt.clone())
        } else {
            Err(QSharpError::new_err(
                "ValueError: value is not a UDT".to_string(),
            ))
        }
    }

    #[staticmethod]
    fn udt(fields: FxHashMap<String, Self>) -> Self {
        Self::Udt(UdtValue { name: None, fields })
    }

    #[staticmethod]
    fn tuple(values: Vec<ValueIR>) -> Self {
        Self::Tuple(values)
    }

    #[staticmethod]
    fn array(values: Vec<ValueIR>) -> Self {
        Self::Array(values)
    }

    #[staticmethod]
    fn bool(value: bool) -> Self {
        Self::Primitive(PrimitiveValue::Bool(value))
    }

    #[staticmethod]
    fn int(value: i64) -> Self {
        Self::Primitive(PrimitiveValue::Int(value))
    }

    #[staticmethod]
    fn bigint(value: BigInt) -> Self {
        Self::Primitive(PrimitiveValue::BigInt(value))
    }

    #[staticmethod]
    fn double(value: f64) -> Self {
        Self::Primitive(PrimitiveValue::Double(value))
    }

    #[staticmethod]
    fn complex(value: num_complex::Complex64) -> Self {
        Self::Primitive(PrimitiveValue::Complex(value))
    }

    #[staticmethod]
    fn str(value: String) -> Self {
        Self::Primitive(PrimitiveValue::String(value))
    }

    #[staticmethod]
    fn result(value: Result) -> Self {
        Self::Primitive(PrimitiveValue::Result(value))
    }

    #[staticmethod]
    fn pauli(value: Pauli) -> Self {
        Self::Primitive(PrimitiveValue::Pauli(value))
    }
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

#[pyclass]
#[derive(Clone)]
pub(super) struct UdtValue {
    #[pyo3(get)]
    name: Option<String>,
    #[pyo3(get)]
    fields: FxHashMap<String, ValueIR>,
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
                Err(QSharpError::new_err(format!(
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

/// A helper macro for converting a primitive `ValueIR` to a primitive `Value`
/// returning an error if the convertion fails.
macro_rules! convert_prim {
    ($val:expr, $prim:ident) => {
        if let ValueIR::Primitive(PrimitiveValue::$prim(val)) = $val {
            Ok(Value::$prim(val.into()))
        } else {
            return Err(QSharpError::new_err(format!(
                "mismatched types: expected {}, found {}",
                stringify!($prim),
                $val.ty_name()
            )));
        }
    };
}

/// Given a type, convert a Python object into a Q# value of that type. This will recur through tuples and arrays,
/// and will return an error if the type is not supported or the object cannot be converted.
pub(super) fn convert_value_ir_with_ty(
    ctx: &interpret::Interpreter,
    value_ir: ValueIR,
    ty: &Ty,
) -> PyResult<Value> {
    match ty {
        Ty::Prim(prim_ty) => match prim_ty {
            Prim::Bool => convert_prim!(value_ir, Bool),
            Prim::Int => convert_prim!(value_ir, Int),
            Prim::BigInt => convert_prim!(value_ir, BigInt),
            Prim::Double => convert_prim!(value_ir, Double),
            Prim::Result => convert_prim!(value_ir, Result),
            Prim::Pauli => convert_prim!(value_ir, Pauli),
            Prim::String => convert_prim!(value_ir, String),
            Prim::Qubit | Prim::Range | Prim::RangeTo | Prim::RangeFrom | Prim::RangeFull => {
                unimplemented!("primitive input type: {prim_ty:?}")
            }
        },
        Ty::Tuple(tup) => {
            if let ValueIR::Tuple(values) = value_ir {
                if tup.len() != values.len() {
                    return Err(QSharpError::new_err(format!(
                        "mismatched tuple arity: expected {}, got {}",
                        tup.len(),
                        values.len()
                    )));
                }

                if values.len() == 1 {
                    let val = values
                        .into_iter()
                        .next()
                        .expect("there is exactly one element");
                    convert_value_ir_with_ty(ctx, val, &tup[0])
                } else {
                    let mut tuple = Vec::new();
                    for (val, ty) in values.into_iter().zip(tup) {
                        tuple.push(convert_value_ir_with_ty(ctx, val, ty)?);
                    }
                    Ok(Value::Tuple(tuple.into()))
                }
            } else {
                Err(QSharpError::new_err(format!(
                    "mismatched types: expected {}, found {}",
                    ty,
                    value_ir.ty_name()
                )))
            }
        }
        Ty::Array(ty) => {
            if let ValueIR::Array(values) = value_ir {
                let ty = &**ty;
                let mut array = Vec::new();
                for val in values {
                    array.push(convert_value_ir_with_ty(ctx, val, ty)?);
                }
                Ok(Value::Array(array.into()))
            } else {
                Err(QSharpError::new_err(format!(
                    "mismatched types: expected {}, found {}",
                    ty,
                    value_ir.ty_name()
                )))
            }
        }
        Ty::Udt(_, res) => {
            let ValueIR::Udt(udt_value) = &value_ir else {
                // Handle `Complex` special case.
                if value_ir.is_complex() {
                    let ValueIR::Primitive(PrimitiveValue::Complex(v)) = value_ir else {
                        unreachable!("we checked the value is complex");
                    };
                    let tuple = Value::Tuple(Rc::new([Value::Double(v.re), Value::Double(v.im)]));
                    return Ok(tuple);
                }

                return Err(QSharpError::new_err(format!(
                    "mismatched types: expected {}, found {}",
                    ty,
                    value_ir.ty_name()
                )));
            };

            let qsc::hir::Res::Item(item_id) = res else {
                panic!("Udt should be an item");
            };
            let Some(udt) = ctx.udt_ty(item_id) else {
                unreachable!(
                    "we verified that the udt is defined in `first_unsupported_interop_ty`"
                );
            };

            let mut tuple = Vec::new();
            for (name, ty) in collect_udt_fields(udt)? {
                let Some(value) = udt_value.fields.get(&*name) else {
                    return Err(QSharpError::new_err(format!(
                        "mismatched types: missing field {} in {}",
                        name, udt.name,
                    )));
                };
                verify_that_field_type_matches_field_value(ctx, ty, value)?;
                tuple.push(convert_value_ir_with_ty(ctx, value.clone(), ty)?);
            }
            Ok(Value::Tuple(tuple.into()))
        }
        _ => unimplemented!("input type: {ty}"),
    }
}

fn verify_that_udt_matches_value(
    ctx: &interpret::Interpreter,
    udt: &qsc::hir::ty::Udt,
    value: &UdtValue,
) -> PyResult<()> {
    verify_that_udt_def_matches_fields(ctx, &udt.name, &udt.definition, &value.fields)
}

fn verify_that_udt_def_matches_fields(
    ctx: &interpret::Interpreter,
    udt_name: &str,
    udt_def: &qsc::hir::ty::UdtDef,
    fields: &FxHashMap<String, ValueIR>,
) -> PyResult<()> {
    match &udt_def.kind {
        qsc::hir::ty::UdtDefKind::Field(udt_field) => {
            let Some(udt_field_name) = udt_field.name.clone() else {
                return Err(QSharpError::new_err(format!(
                    "unsupported: {udt_name} has anonymous fields",
                )));
            };

            let Some(field_value) = fields.get(&*udt_field_name) else {
                return Err(QSharpError::new_err(format!(
                    "mismatched types: missing field {udt_field_name} in {udt_name}",
                )));
            };

            verify_that_field_type_matches_field_value(ctx, &udt_field.ty, field_value)?;
        }
        qsc::hir::ty::UdtDefKind::Tuple(udt_defs) => {
            for udt_def in udt_defs {
                verify_that_udt_def_matches_fields(ctx, udt_name, udt_def, fields)?;
            }
        }
    }

    Ok(())
}

fn verify_that_field_type_matches_field_value(
    ctx: &interpret::Interpreter,
    ty: &Ty,
    value: &ValueIR,
) -> PyResult<()> {
    match ty {
        Ty::Arrow(..) | Ty::Infer(..) | Ty::Param { .. } | Ty::Err => {
            unreachable!("we verified unsupported types in `first_unsupported_interop_ty`");
        }
        Ty::Prim(prim) => match (prim, value) {
            (Prim::Pauli, ValueIR::Primitive(PrimitiveValue::Pauli(..)))
            | (Prim::Bool, ValueIR::Primitive(PrimitiveValue::Bool(..)))
            | (Prim::Double, ValueIR::Primitive(PrimitiveValue::Double(..)))
            | (Prim::Int, ValueIR::Primitive(PrimitiveValue::Int(..)))
            | (Prim::BigInt, ValueIR::Primitive(PrimitiveValue::BigInt(..)))
            | (Prim::String, ValueIR::Primitive(PrimitiveValue::String(..)))
            | (Prim::Result, ValueIR::Primitive(PrimitiveValue::Result(..))) => (),
            _ => {
                return Err(QSharpError::new_err(format!(
                    "mismatched types: expected {}, found {}",
                    ty,
                    value.ty_name()
                )));
            }
        },
        Ty::Array(ty) => {
            if let ValueIR::Array(values) = value {
                for value in values {
                    verify_that_field_type_matches_field_value(ctx, ty, value)?;
                }
            } else {
                return Err(QSharpError::new_err(format!(
                    "mismatched types: expected {}, found {}",
                    ty,
                    value.ty_name()
                )));
            }
        }
        Ty::Tuple(items) => {
            if let ValueIR::Tuple(values) = value
                && items.len() == values.len()
            {
                for (ty, value) in items.iter().zip(values) {
                    verify_that_field_type_matches_field_value(ctx, ty, value)?;
                }
            } else {
                return Err(QSharpError::new_err(format!(
                    "mismatched types: expected {}, found {}",
                    ty,
                    value.ty_name()
                )));
            }
        }
        Ty::Udt(_, res) => {
            let ValueIR::Udt(udt_value) = &value else {
                // Handle `Complex` special case.
                if value.is_complex() {
                    return Ok(());
                }

                return Err(QSharpError::new_err(format!(
                    "mismatched types: expected {}, found {}",
                    ty,
                    value.ty_name()
                )));
            };
            let qsc::hir::Res::Item(item_id) = res else {
                panic!("Udt should be an item");
            };
            let Some(udt) = ctx.udt_ty(item_id) else {
                unreachable!(
                    "we verified that the udt is defined in `first_unsupported_interop_ty`"
                );
            };
            verify_that_udt_matches_value(ctx, udt, udt_value)?;
        }
    }

    Ok(())
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
                    return Err(QSharpError::new_err(format!(
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
            let Some(udt) = ctx.udt_ty(item_id) else {
                unreachable!(
                    "we verified that the udt is defined in `first_unsupported_interop_ty`"
                );
            };

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
        Ty::Param { .. } | Ty::Infer(..) | Ty::Arrow(..) | Ty::Err => Err(QSharpError::new_err(
            format!("unsupported interop type: `{ty}`"),
        )),
    }
}

pub(super) fn typed_value_to_value_ir(
    ctx: &interpret::Interpreter,
    value: &Value,
    ty: &Ty,
) -> PyResult<ValueIR> {
    match value {
        Value::Int(val) => Ok(ValueIR::int(*val)),
        Value::BigInt(val) => Ok(ValueIR::bigint(val.clone())),
        Value::Double(val) => Ok(ValueIR::double(*val)),
        Value::Bool(val) => Ok(ValueIR::bool(*val)),
        Value::String(val) => Ok(ValueIR::str(val.to_string())),
        Value::Result(val) => match val {
            qsc::interpret::Result::Id(_) => {
                panic!("unexpected Result::Id in typed_value_to_value_ir")
            }
            qsc::interpret::Result::Val(true) => Ok(ValueIR::result(Result::One)),
            qsc::interpret::Result::Val(false) => Ok(ValueIR::result(Result::Zero)),
            qsc::interpret::Result::Loss => Ok(ValueIR::result(Result::Loss)),
        },
        Value::Pauli(val) => Ok(match val {
            fir::Pauli::I => ValueIR::pauli(Pauli::I),
            fir::Pauli::X => ValueIR::pauli(Pauli::X),
            fir::Pauli::Y => ValueIR::pauli(Pauli::Y),
            fir::Pauli::Z => ValueIR::pauli(Pauli::Z),
        }),
        Value::Tuple(values) => match ty {
            Ty::Tuple(items) => {
                let mut tuple = Vec::new();
                for (val, ty) in values.iter().zip(items) {
                    tuple.push(typed_value_to_value_ir(ctx, val, ty)?);
                }
                Ok(ValueIR::tuple(tuple))
            }
            Ty::Udt(_, res) => {
                let qsc::hir::Res::Item(item_id) = res else {
                    panic!("Udt should be an item");
                };
                let Some(udt) = ctx.udt_ty(item_id) else {
                    unreachable!("output type should be defined");
                };
                if is_complex_udt(udt) {
                    let re = values[0].clone().unwrap_double();
                    let im = values[1].clone().unwrap_double();
                    return Ok(ValueIR::complex(num_complex::Complex { re, im }));
                }
                let ty_fields = collect_udt_fields(udt)?;
                let mut fields = Vec::new();
                for (value, (name, ty)) in values.iter().zip(ty_fields) {
                    fields.push((name.to_string(), typed_value_to_value_ir(ctx, value, ty)?));
                }
                let fields = fields.into_iter().collect();
                Ok(ValueIR::Udt(UdtValue {
                    name: Some(udt.name.to_string()),
                    fields,
                }))
            }
            _ => unreachable!(),
        },
        Value::Array(values) => {
            let Ty::Array(ty) = ty else {
                unreachable!();
            };
            let mut array = Vec::new();
            for val in values.iter() {
                array.push(typed_value_to_value_ir(ctx, val, ty)?);
            }
            Ok(ValueIR::array(array))
        }
        _ => Err(QSharpError::new_err(format!(
            "unsupported interop type: `{}`",
            value.type_name(),
        ))),
    }
}
