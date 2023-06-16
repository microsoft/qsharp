// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::hir::{CallableKind, FieldPath, Functor, ItemId, Res};
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter, Write},
    rc::Rc,
};

/// A type.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Ty {
    /// An array type.
    Array(Box<Ty>),
    /// An arrow type: `->` for a function or `=>` for an operation.
    Arrow(Box<Arrow>),
    /// A placeholder type variable used during type inference.
    Infer(InferTyId),
    /// A type parameter.
    Param(ParamId),
    /// A primitive type.
    Prim(Prim),
    /// A tuple type.
    Tuple(Vec<Ty>),
    /// A user-defined type.
    Udt(Res),
    /// An invalid type.
    #[default]
    Err,
}

impl Ty {
    /// The unit type.
    pub const UNIT: Self = Self::Tuple(Vec::new());
}

impl Display for Ty {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Ty::Array(item) => write!(f, "({item})[]"),
            Ty::Arrow(arrow) => Display::fmt(arrow, f),
            Ty::Infer(infer) => Display::fmt(infer, f),
            Ty::Param(name) => write!(f, "'{name}"),
            Ty::Prim(prim) => Debug::fmt(prim, f),
            Ty::Tuple(items) => {
                if items.is_empty() {
                    f.write_str("Unit")
                } else {
                    f.write_str("(")?;
                    if let Some((first, rest)) = items.split_first() {
                        Display::fmt(first, f)?;
                        if rest.is_empty() {
                            f.write_str(",")?;
                        } else {
                            for item in rest {
                                f.write_str(", ")?;
                                Display::fmt(item, f)?;
                            }
                        }
                    }
                    f.write_str(")")
                }
            }
            Ty::Udt(res) => write!(f, "UDT<{res}>"),
            Ty::Err => f.write_str("?"),
        }
    }
}

/// A type scheme.
pub struct Scheme {
    params: Vec<GenericParam>,
    ty: Box<Arrow>,
}

impl Scheme {
    /// Creates a new type scheme.
    #[must_use]
    pub fn new(params: Vec<GenericParam>, ty: Box<Arrow>) -> Self {
        Self { params, ty }
    }

    /// The generic parameters to the type.
    #[must_use]
    pub fn params(&self) -> &[GenericParam] {
        &self.params
    }

    /// Instantiates this type scheme with the given arguments.
    ///
    /// # Errors
    ///
    /// Returns an error if the given arguments do not match the scheme parameters.
    pub fn instantiate(&self, args: &[GenericArg]) -> Result<Arrow, InstantiationError> {
        if args.len() == self.params.len() {
            let args: HashMap<_, _> = self
                .params
                .iter()
                .enumerate()
                .map(|(ix, _)| ParamId::from(ix))
                .zip(args)
                .collect();
            instantiate_arrow_ty(|name| args.get(name).copied(), &self.ty)
        } else {
            Err(InstantiationError::Arity)
        }
    }
}

/// A type scheme instantiation error.
#[derive(Debug)]
pub enum InstantiationError {
    /// The number of generic arguments does not match the number of generic parameters.
    Arity,
    /// A generic argument does not match the kind of its corresponding generic parameter.
    Kind(ParamId),
}

fn instantiate_ty<'a>(
    arg: impl Fn(&ParamId) -> Option<&'a GenericArg> + Copy,
    ty: &Ty,
) -> Result<Ty, InstantiationError> {
    match ty {
        Ty::Err | Ty::Infer(_) | Ty::Prim(_) | Ty::Udt(_) => Ok(ty.clone()),
        Ty::Array(item) => Ok(Ty::Array(Box::new(instantiate_ty(arg, item)?))),
        Ty::Arrow(arrow) => Ok(Ty::Arrow(Box::new(instantiate_arrow_ty(arg, arrow)?))),
        Ty::Param(param) => match arg(param) {
            Some(GenericArg::Ty(ty_arg)) => Ok(ty_arg.clone()),
            Some(_) => Err(InstantiationError::Kind(*param)),
            None => Ok(ty.clone()),
        },
        Ty::Tuple(items) => Ok(Ty::Tuple(
            items
                .iter()
                .map(|item| instantiate_ty(arg, item))
                .collect::<Result<_, _>>()?,
        )),
    }
}

fn instantiate_arrow_ty<'a>(
    arg: impl Fn(&ParamId) -> Option<&'a GenericArg> + Copy,
    arrow: &Arrow,
) -> Result<Arrow, InstantiationError> {
    let input = instantiate_ty(arg, &arrow.input)?;
    let output = instantiate_ty(arg, &arrow.output)?;
    let functors = if let FunctorSet::Param(param) = arrow.functors {
        match arg(&param) {
            Some(GenericArg::Functor(functor_arg)) => *functor_arg,
            Some(_) => return Err(InstantiationError::Kind(param)),
            None => arrow.functors,
        }
    } else {
        arrow.functors
    };

    Ok(Arrow {
        kind: arrow.kind,
        input: Box::new(input),
        output: Box::new(output),
        functors,
    })
}

/// A generic parameter.
#[derive(Clone, Debug, PartialEq)]
pub struct GenericParam {
    /// The parameter kind.
    pub kind: ParamKind,
}

impl Display for GenericParam {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.kind {
            ParamKind::Ty => write!(f, "parameter type"),
            ParamKind::Functor(min) => write!(f, "functor ({min})"),
        }
    }
}

/// The kind of a generic parameter.
#[derive(Clone, Debug, PartialEq)]
pub enum ParamKind {
    /// A type parameter.
    Ty,
    /// A functor parameter with a lower bound.
    Functor(FunctorSetValue),
}

/// A generic parameter ID.
#[derive(Clone, Copy, Default, Debug, Eq, Hash, PartialEq)]
pub struct ParamId(u32);

impl ParamId {
    /// The successor of this ID.
    #[must_use]
    pub fn successor(self) -> Self {
        Self(self.0 + 1)
    }
}

impl From<usize> for ParamId {
    #[allow(clippy::cast_possible_truncation)]
    fn from(value: usize) -> Self {
        ParamId(value as u32)
    }
}

impl Display for ParamId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// An argument to a generic parameter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenericArg {
    /// A type argument.
    Ty(Ty),
    /// A functor argument.
    Functor(FunctorSet),
}

impl Display for GenericArg {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            GenericArg::Ty(ty) => Display::fmt(ty, f),
            GenericArg::Functor(functors) => Display::fmt(functors, f),
        }
    }
}

/// An arrow type: `->` for a function or `=>` for an operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Arrow {
    /// Whether the callable is a function or an operation.
    pub kind: CallableKind,
    /// The input type to the callable.
    pub input: Box<Ty>,
    /// The output type from the callable.
    pub output: Box<Ty>,
    /// The functors supported by the callable.
    pub functors: FunctorSet,
}

impl Display for Arrow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let arrow = match self.kind {
            CallableKind::Function => "->",
            CallableKind::Operation => "=>",
        };
        write!(f, "({} {arrow} {}", self.input, self.output)?;
        if self.functors != FunctorSet::Value(FunctorSetValue::Empty) {
            f.write_str(" is ")?;
            Display::fmt(&self.functors, f)?;
        }
        f.write_char(')')
    }
}

/// A primitive type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Prim {
    /// The big integer type.
    BigInt,
    /// The boolean type.
    Bool,
    /// The floating-point type.
    Double,
    /// The integer type.
    Int,
    /// The Pauli operator type.
    Pauli,
    /// The qubit type.
    Qubit,
    /// The range type.
    Range,
    /// The range type without a lower bound.
    RangeTo,
    /// The range type without an upper bound.
    RangeFrom,
    /// The range type without lower and upper bounds.
    RangeFull,
    /// The measurement result type.
    Result,
    /// The string type.
    String,
}

/// A set of functors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FunctorSet {
    /// An evaluated set.
    Value(FunctorSetValue),
    /// A functor parameter.
    Param(ParamId),
    /// A placeholder functor variable used during type inference.
    Infer(InferFunctorId),
}

impl FunctorSet {
    /// Returns the contained value.
    ///
    /// # Panics
    ///
    /// Panics if this set is not a value.
    #[must_use]
    pub fn expect_value(self, msg: &str) -> FunctorSetValue {
        match self {
            Self::Value(value) => value,
            Self::Param(_) | Self::Infer(_) => panic!("{msg}"),
        }
    }
}

impl Display for FunctorSet {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Value(value) => Display::fmt(value, f),
            Self::Param(param) => Display::fmt(param, f),
            Self::Infer(infer) => Display::fmt(infer, f),
        }
    }
}

/// The value of a functor set.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FunctorSetValue {
    /// The empty set.
    #[default]
    Empty,
    /// The singleton adjoint set.
    Adj,
    /// The singleton controlled set.
    Ctl,
    /// The set of controlled and adjoint.
    CtlAdj,
}

impl FunctorSetValue {
    /// True if this set contains the functor.
    #[must_use]
    pub fn contains(&self, functor: &Functor) -> bool {
        match self {
            Self::Empty => false,
            Self::Adj => matches!(functor, Functor::Adj),
            Self::Ctl => matches!(functor, Functor::Ctl),
            Self::CtlAdj => matches!(functor, Functor::Adj | Functor::Ctl),
        }
    }

    /// The intersection of this set and another set.
    #[must_use]
    pub fn intersect(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Empty, _)
            | (_, Self::Empty)
            | (Self::Adj, Self::Ctl)
            | (Self::Ctl, Self::Adj) => Self::Empty,
            (Self::Adj, Self::Adj) => Self::Adj,
            (Self::Ctl, Self::Ctl) => Self::Ctl,
            (Self::CtlAdj, &set) | (&set, Self::CtlAdj) => set,
        }
    }

    /// The union of this set and another set.
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Empty, &set) | (&set, Self::Empty) => set,
            (Self::Adj, Self::Adj) => Self::Adj,
            (Self::Ctl, Self::Ctl) => Self::Ctl,
            (Self::CtlAdj, _)
            | (_, Self::CtlAdj)
            | (Self::Adj, Self::Ctl)
            | (Self::Ctl, Self::Adj) => Self::CtlAdj,
        }
    }
}

impl Display for FunctorSetValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty set"),
            Self::Adj => f.write_str("Adj"),
            Self::Ctl => f.write_str("Ctl"),
            Self::CtlAdj => f.write_str("Adj + Ctl"),
        }
    }
}

/// A user-defined type.
#[derive(Clone, Debug, PartialEq)]
pub struct Udt {
    /// The basis type used as the definition of the user-defined type.
    pub base: Ty,
    /// The named fields of the user-defined type.
    pub fields: Vec<UdtField>,
}

impl Udt {
    /// The type scheme of the constructor for this type definition.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the constructed type.
    #[must_use]
    pub fn cons_scheme(&self, id: ItemId) -> Scheme {
        Scheme {
            params: Vec::new(),
            ty: Box::new(Arrow {
                kind: CallableKind::Function,
                input: Box::new(self.base.clone()),
                output: Box::new(Ty::Udt(Res::Item(id))),
                functors: FunctorSet::Value(FunctorSetValue::Empty),
            }),
        }
    }

    /// The path to the field with the given name. Returns [None] if this user-defined type does not
    /// have a field with the given name.
    #[must_use]
    pub fn field_path(&self, name: &str) -> Option<&FieldPath> {
        for field in &self.fields {
            if field.name.as_ref() == name {
                return Some(&field.path);
            }
        }

        None
    }

    /// The type of the field at the given path. Returns [None] if the path is not valid for this
    /// user-defined type.
    #[must_use]
    pub fn field_ty(&self, path: &FieldPath) -> Option<&Ty> {
        let mut ty = &self.base;
        for &index in &path.indices {
            let Ty::Tuple(items) = ty else { return None; };
            ty = &items[index];
        }
        Some(ty)
    }

    /// The type of the field with the given name. Returns [None] if this user-defined type does not
    /// have a field with the given name.
    #[must_use]
    pub fn field_ty_by_name(&self, name: &str) -> Option<&Ty> {
        let path = self.field_path(name)?;
        self.field_ty(path)
    }
}

impl Display for Udt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("Udt:")?;
        write!(f, "\n    base: {}", self.base)?;
        f.write_str("\n    fields:")?;
        for field in &self.fields {
            write!(f, "\n        {}: {:?}", field.name, field.path.indices)?;
        }
        Ok(())
    }
}

/// A named field in a user-defined type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UdtField {
    /// The field name.
    pub name: Rc<str>,
    /// The field path.
    pub path: FieldPath,
}

/// A placeholder type variable used during type inference.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct InferTyId(usize);

impl InferTyId {
    /// The successor of this ID.
    #[must_use]
    pub fn successor(self) -> Self {
        Self(self.0 + 1)
    }
}

impl Display for InferTyId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "?{}", self.0)
    }
}

impl From<usize> for InferTyId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<InferTyId> for usize {
    fn from(value: InferTyId) -> Self {
        value.0
    }
}

/// A placeholder functor variable used during type inference.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InferFunctorId(usize);

impl InferFunctorId {
    /// The successor of this ID.
    #[must_use]
    pub fn successor(self) -> Self {
        Self(self.0 + 1)
    }
}

impl Display for InferFunctorId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "f?{}", self.0)
    }
}

impl From<InferFunctorId> for usize {
    fn from(value: InferFunctorId) -> Self {
        value.0
    }
}
