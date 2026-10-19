// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use indenter::indented;
use qsc_data_structures::{display::core::set_indentation, span::Span};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::fir::{CallableKind, FieldPath, Functor, ItemId, Res};
use std::{
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

    /// Whether this type contains no unresolved type or functor evidence.
    #[must_use]
    pub fn is_concrete(&self) -> bool {
        match self {
            Self::Array(inner) => inner.is_concrete(),
            Self::Arrow(arrow) => {
                matches!(arrow.functors, FunctorSet::Value(_))
                    && arrow.input.is_concrete()
                    && arrow.output.is_concrete()
            }
            Self::Tuple(items) => items.iter().all(Self::is_concrete),
            Self::Prim(_) | Self::Udt(Res::Item(_)) => true,
            _ => false,
        }
    }
}

/// Maximum nodes materialized by one generic type substitution.
pub const MAX_TYPE_NODES: usize = 4096;
/// Maximum depth materialized by generic type substitution.
pub const MAX_TYPE_DEPTH: usize = 128;

/// Checks expansion before constructing substituted types. Each parameter is
/// substituted once, since its argument belongs to a different declaration scope.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn type_expansion_within_limits<'a>(
    types: impl IntoIterator<Item = &'a Ty>,
    args: &FxHashMap<ParamId, GenericArg>,
) -> bool {
    let mut nodes = 0;
    for ty in types {
        let mut pending = vec![(ty, 1, true)];
        while let Some((ty, depth, substitute)) = pending.pop() {
            if substitute
                && let Ty::Param(param) = ty
                && let Some(GenericArg::Ty(concrete)) = args.get(param)
            {
                pending.push((concrete, depth, false));
                continue;
            }
            nodes += 1;
            if nodes > MAX_TYPE_NODES || depth > MAX_TYPE_DEPTH {
                return false;
            }
            match ty {
                Ty::Array(inner) => pending.push((inner, depth + 1, substitute)),
                Ty::Arrow(arrow) => {
                    pending.push((&arrow.input, depth + 1, substitute));
                    pending.push((&arrow.output, depth + 1, substitute));
                }
                Ty::Tuple(items) => {
                    if items.len() > MAX_TYPE_NODES - nodes {
                        return false;
                    }
                    pending.extend(items.iter().map(|ty| (ty, depth + 1, substitute)));
                }
                _ => {}
            }
        }
    }
    true
}

/// Free parameters used by a lifted callable, including its nested closures.
/// Ordinary global references do not import another declaration's parameter scope.
#[must_use]
pub fn callable_type_parameters(
    store: &impl crate::fir::PackageStoreLookup,
    target: crate::fir::StoreItemId,
) -> FxHashSet<ParamId> {
    use crate::{fir, visit};
    use visit::Visitor;

    struct Collector<'a, S> {
        store: &'a S,
        package: fir::PackageId,
        seen: FxHashSet<fir::LocalItemId>,
        params: FxHashSet<ParamId>,
    }

    impl<S: fir::PackageStoreLookup> Collector<'_, S> {
        fn ty(&mut self, ty: &Ty) {
            let mut pending = vec![ty];
            while let Some(ty) = pending.pop() {
                match ty {
                    Ty::Param(id) => {
                        self.params.insert(*id);
                    }
                    Ty::Arrow(arrow) => {
                        if let FunctorSet::Param(id) = arrow.functors {
                            self.params.insert(id);
                        }
                        pending.extend([arrow.input.as_ref(), arrow.output.as_ref()]);
                    }
                    Ty::Array(inner) => pending.push(inner),
                    Ty::Tuple(items) => pending.extend(items),
                    _ => {}
                }
            }
        }
    }

    impl<'a, S: fir::PackageStoreLookup> Visitor<'a> for Collector<'a, S> {
        fn visit_callable_decl(&mut self, decl: &'a fir::CallableDecl) {
            self.ty(&decl.output);
            visit::walk_callable_decl(self, decl);
        }
        fn visit_expr(&mut self, id: fir::ExprId) {
            let expr = self.get_expr(id);
            self.ty(&expr.ty);
            match &expr.kind {
                fir::ExprKind::Closure(_, target) if self.seen.insert(*target) => {
                    if let Some(fir::Global::Callable(decl)) =
                        self.store.get_global((self.package, *target).into())
                        && decl.generics.is_empty()
                    {
                        self.visit_callable_decl(decl);
                    }
                }
                fir::ExprKind::Var(_, args) => {
                    for arg in args {
                        match arg {
                            GenericArg::Ty(ty) => self.ty(ty),
                            GenericArg::Functor(FunctorSet::Param(id)) => {
                                self.params.insert(*id);
                            }
                            GenericArg::Functor(_) => {}
                        }
                    }
                }
                _ => {}
            }
            visit::walk_expr(self, id);
        }
        fn visit_pat(&mut self, id: fir::PatId) {
            self.ty(&self.get_pat(id).ty);
            visit::walk_pat(self, id);
        }
        fn visit_block(&mut self, id: fir::BlockId) {
            self.ty(&self.get_block(id).ty);
            visit::walk_block(self, id);
        }
        fn get_expr(&self, id: fir::ExprId) -> &'a fir::Expr {
            self.store.get_expr((self.package, id).into())
        }
        fn get_pat(&self, id: fir::PatId) -> &'a fir::Pat {
            self.store.get_pat((self.package, id).into())
        }
        fn get_block(&self, id: fir::BlockId) -> &'a fir::Block {
            self.store.get_block((self.package, id).into())
        }
        fn get_stmt(&self, id: fir::StmtId) -> &'a fir::Stmt {
            self.store.get_stmt((self.package, id).into())
        }
    }

    let mut collector = Collector {
        store,
        package: target.package,
        seen: FxHashSet::from_iter([target.item]),
        params: FxHashSet::default(),
    };
    if let Some(fir::Global::Callable(decl)) = store.get_global(target) {
        collector.visit_callable_decl(decl);
    }
    collector.params
}

impl Display for Ty {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Ty::Array(item) => write!(f, "({item})[]"),
            Ty::Arrow(arrow) => Display::fmt(arrow, f),
            Ty::Infer(infer) => Display::fmt(infer, f),
            Ty::Param(param_id) => write!(f, "Param<{param_id}>"),
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
    params: Vec<TypeParameter>,
    ty: Box<Arrow>,
}

impl Scheme {
    /// Creates a new type scheme.
    #[must_use]
    pub fn new(params: Vec<TypeParameter>, ty: Box<Arrow>) -> Self {
        Self { params, ty }
    }

    /// The generic parameters to the type.
    #[must_use]
    pub fn params(&self) -> &[TypeParameter] {
        &self.params
    }

    /// Instantiates this type scheme with the given arguments.
    ///
    /// # Errors
    ///
    /// Returns an error if the given arguments do not match the scheme parameters.
    pub fn instantiate(&self, args: &[GenericArg]) -> Result<Arrow, InstantiationError> {
        if args.len() == self.params.len() {
            let args: FxHashMap<_, _> = self
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

/// Substitutes parameters once, preserving unbound parameters and nominal types.
///
/// # Errors
/// Returns a kind error when a type parameter is bound to a functor or vice versa.
pub fn instantiate_ty<'a>(
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

/// Matches a declaration type against a checked type in another scope.
/// Bindings belong only to the declaration; substitutions in `actual` must
/// already have been resolved in the caller's scope.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn infer_generic_ty_args(
    formal: &Ty,
    actual: &Ty,
    arg_map: &mut FxHashMap<ParamId, GenericArg>,
) -> bool {
    match (formal, actual) {
        (Ty::Param(param), _) => {
            record_inferred_arg(*param, GenericArg::Ty(actual.clone()), arg_map)
        }
        (Ty::Array(formal), Ty::Array(actual)) => infer_generic_ty_args(formal, actual, arg_map),
        (Ty::Arrow(formal), Ty::Arrow(actual)) => {
            formal.kind == actual.kind
                && infer_generic_ty_args(&formal.input, &actual.input, arg_map)
                && infer_generic_ty_args(&formal.output, &actual.output, arg_map)
                && infer_generic_functor_args(formal.functors, actual.functors, arg_map)
        }
        (Ty::Tuple(formal), Ty::Tuple(actual)) if formal.len() == actual.len() => formal
            .iter()
            .zip(actual)
            .all(|(formal, actual)| infer_generic_ty_args(formal, actual, arg_map)),
        (Ty::Prim(formal), Ty::Prim(actual)) => formal == actual,
        (Ty::Udt(formal), Ty::Udt(actual)) => formal == actual,
        (Ty::Infer(formal), Ty::Infer(actual)) => formal == actual,
        (Ty::Err, Ty::Err) => true,
        _ => false,
    }
}

/// Matches a declaration's functor set against a checked functor set.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn infer_generic_functor_args(
    formal: FunctorSet,
    actual: FunctorSet,
    arg_map: &mut FxHashMap<ParamId, GenericArg>,
) -> bool {
    match formal {
        FunctorSet::Param(param) => {
            record_inferred_arg(param, GenericArg::Functor(actual), arg_map)
        }
        _ => formal == actual,
    }
}

fn record_inferred_arg(
    param: ParamId,
    arg: GenericArg,
    arg_map: &mut FxHashMap<ParamId, GenericArg>,
) -> bool {
    if let Some(existing) = arg_map.get(&param) {
        existing == &arg
    } else {
        arg_map.insert(param, arg);
        true
    }
}

impl Display for TypeParameter {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TypeParameter::Ty { name, bounds } => {
                write!(f, "type ({name}){bounds}")
            }
            TypeParameter::Functor(min) => write!(f, "functor ({min})"),
        }
    }
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ClassConstraints(pub Box<[ClassConstraint]>);

impl std::fmt::Display for ClassConstraints {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            Ok(())
        } else {
            let bounds = self
                .0
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            write!(f, "{bounds}")
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClassConstraint {
    /// Whether or not 'T can be compared via Eq to values of the same domain.
    Eq,
    /// Whether or not 'T can be added to values of the same domain via the + operator.
    Add,
    Exp {
        // `base` is inferred to be the self type
        power: Ty,
    },
    /// If 'T is iterable, then it can be iterated over and the items inside are yielded (of type `item`).
    Iterable { item: Ty },
    /// Whether or not 'T can be divided by values of the same domain via the / operator.
    Div,
    /// Whether or not 'T can be subtracted from values of the same domain via the - operator.
    Sub,
    /// Whether or not 'T can be multiplied by values of the same domain via the * operator.
    Mul,
    /// Whether or not 'T can be taken modulo values of the same domain via the % operator.
    Mod,
    /// Whether or not 'T can be compared via Ord to values of the same domain.
    Ord,
    /// Whether or not 'T can be signed.
    Signed,
    /// Whether or not 'T is an integral type (can be used in bit shifting operators).
    Integral,
    /// Whether or not 'T can be displayed as a string (converted to a string).
    Show,
    /// A class that is not built-in to the compiler.
    NonNativeClass(Rc<str>),
}

impl std::fmt::Display for ClassConstraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ClassConstraint::Eq => write!(f, "Eq"),
            ClassConstraint::NonNativeClass(name) => write!(f, "{name}"),
            ClassConstraint::Exp { power } => write!(f, "Exp<{power}>"),
            ClassConstraint::Iterable { item } => write!(f, "Iterable<{item}>"),
            ClassConstraint::Add => write!(f, "Add"),
            ClassConstraint::Integral => write!(f, "Integral"),
            ClassConstraint::Show => write!(f, "Show"),
            ClassConstraint::Div => write!(f, "Div"),
            ClassConstraint::Sub => write!(f, "Sub"),
            ClassConstraint::Mul => write!(f, "Mul"),
            ClassConstraint::Mod => write!(f, "Mod"),
            ClassConstraint::Ord => write!(f, "Ord"),
            ClassConstraint::Signed => write!(f, "Signed"),
        }
    }
}

/// The kind of a generic parameter.
#[derive(Clone, Debug, PartialEq)]
pub enum TypeParameter {
    /// A type parameter.
    Ty {
        name: Rc<str>,
        bounds: ClassConstraints,
    },
    /// A functor parameter with a lower bound.
    Functor(FunctorSetValue),
}

/// A generic parameter ID.
#[derive(Clone, Copy, Default, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ParamId(u32);

impl ParamId {
    /// The successor of this ID.
    #[must_use]
    pub fn successor(self) -> Self {
        Self(self.0 + 1)
    }
}

impl From<usize> for ParamId {
    fn from(value: usize) -> Self {
        ParamId(
            value
                .try_into()
                .expect("Type Parameter ID does not fit into u32"),
        )
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
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq)]
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

impl FunctorSetValue {
    /// Returns a compact identifier suitable for name mangling.
    #[must_use]
    pub fn mangle_name(&self) -> &'static str {
        match self {
            Self::Empty => "Empty",
            Self::Adj => "Adj",
            Self::Ctl => "Ctl",
            Self::CtlAdj => "AdjCtl",
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
    /// The span.
    pub span: Span,
    /// The name.
    pub name: Rc<str>,
    // The definition.
    pub definition: UdtDef,
}

impl Udt {
    #[must_use]
    pub fn get_pure_ty(&self) -> Ty {
        fn get_pure_ty(def: &UdtDef) -> Ty {
            match &def.kind {
                UdtDefKind::Field(field) => field.ty.clone(),
                UdtDefKind::Tuple(tup) => Ty::Tuple(tup.iter().map(get_pure_ty).collect()),
            }
        }
        get_pure_ty(&self.definition)
    }

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
                input: Box::new(self.get_pure_ty()),
                output: Box::new(Ty::Udt(Res::Item(id))),
                functors: FunctorSet::Value(FunctorSetValue::Empty),
            }),
        }
    }

    /// The path to the field with the given name. Returns [None] if this user-defined type does not
    /// have a field with the given name.
    #[must_use]
    pub fn field_path(&self, name: &str) -> Option<FieldPath> {
        Self::find_field_path(&self.definition, name)
    }

    fn find_field_path(def: &UdtDef, name: &str) -> Option<FieldPath> {
        match &def.kind {
            UdtDefKind::Field(field) => field.name.as_ref().and_then(|field_name| {
                if field_name.as_ref() == name {
                    Some(FieldPath::default())
                } else {
                    None
                }
            }),
            UdtDefKind::Tuple(defs) => defs.iter().enumerate().find_map(|(i, def)| {
                Self::find_field_path(def, name).map(|mut path| {
                    path.indices.insert(0, i);
                    path
                })
            }),
        }
    }

    fn find_field(&self, path: &FieldPath) -> Option<&UdtField> {
        let mut udt_def = &self.definition;
        for &index in &path.indices {
            let UdtDefKind::Tuple(items) = &udt_def.kind else {
                return None;
            };
            udt_def = &items[index];
        }
        let UdtDefKind::Field(field) = &udt_def.kind else {
            return None;
        };
        Some(field)
    }

    /// The type of the field at the given path. Returns [None] if the path is not valid for this
    /// user-defined type.
    #[must_use]
    pub fn field_ty(&self, path: &FieldPath) -> Option<&Ty> {
        self.find_field(path).map(|field| &field.ty)
    }

    /// The type of the field with the given name. Returns [None] if this user-defined type does not
    /// have a field with the given name.
    #[must_use]
    pub fn field_ty_by_name(&self, name: &str) -> Option<&Ty> {
        let path = self.field_path(name)?;
        self.field_ty(&path)
    }
}

impl Display for Udt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "UDT {}:", self.span)?;
        indent = set_indentation(indent, 1);
        write!(indent, "\n{}", self.definition)?;
        Ok(())
    }
}

/// A UDT type definition.
#[derive(Clone, Debug, PartialEq)]
pub struct UdtDef {
    /// The span.
    pub span: Span,
    /// The type definition kind.
    pub kind: UdtDefKind,
}

impl Display for UdtDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "TyDef {}: {}", self.span, self.kind)
    }
}

/// A UDT type definition kind.
#[derive(Clone, Debug, PartialEq)]
pub enum UdtDefKind {
    /// A field definition with an optional name but required type.
    Field(UdtField),
    /// A tuple.
    Tuple(Vec<UdtDef>),
}

impl Display for UdtDefKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        match &self {
            UdtDefKind::Field(field) => {
                write!(indent, "Field:")?;
                indent = set_indentation(indent, 1);
                write!(indent, "{field}")?;
            }
            UdtDefKind::Tuple(ts) => {
                if ts.is_empty() {
                    write!(indent, "Unit")?;
                } else {
                    write!(indent, "Tuple:")?;
                    indent = set_indentation(indent, 1);
                    for t in ts {
                        write!(indent, "\n{t}")?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// A user-defined type.
#[derive(Clone, Debug, PartialEq)]
pub struct UdtField {
    /// The span of the field name.
    pub name_span: Option<Span>,
    /// The field name.
    pub name: Option<Rc<str>>,
    // The field type.
    pub ty: Ty,
}

impl Display for UdtField {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(n) = &self.name
            && let Some(s) = &self.name_span
        {
            write!(f, "\nname: {n} {s}")?;
        }
        write!(f, "\ntype: {}", self.ty)?;
        Ok(())
    }
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

impl From<usize> for InferFunctorId {
    fn from(value: usize) -> Self {
        InferFunctorId(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instantiation_preserves_scope_and_checks_kinds() {
        let param = ParamId::from(0);
        let actual = GenericArg::Ty(Ty::Array(Box::new(Ty::Param(param))));
        assert_eq!(
            instantiate_ty(|_| Some(&actual), &Ty::Param(param)).expect("type argument"),
            Ty::Array(Box::new(Ty::Param(param))),
        );
        let wrong = GenericArg::Functor(FunctorSet::Value(FunctorSetValue::Empty));
        assert!(matches!(
            instantiate_ty(|_| Some(&wrong), &Ty::Param(param)),
            Err(InstantiationError::Kind(id)) if id == param,
        ));
        assert_eq!(
            instantiate_ty(|_| None, &Ty::Param(param)).expect("unbound"),
            Ty::Param(param),
        );
    }

    #[test]
    fn checked_type_matching_preserves_nominal_identity() {
        let a = Ty::Udt(Res::Item(ItemId {
            package: crate::fir::PackageId::from(1usize),
            item: crate::fir::LocalItemId::from(0usize),
        }));
        let b = Ty::Udt(Res::Item(ItemId {
            package: crate::fir::PackageId::from(2usize),
            item: crate::fir::LocalItemId::from(0usize),
        }));
        let mut bindings = FxHashMap::default();
        let param = Ty::Param(ParamId::from(0));
        assert!(infer_generic_ty_args(&param, &a, &mut bindings));
        assert!(!infer_generic_ty_args(&param, &b, &mut bindings));
        assert!(!infer_generic_ty_args(&a, &b, &mut FxHashMap::default()));
        assert_eq!(bindings[&ParamId::from(0)], GenericArg::Ty(a));
    }
}
