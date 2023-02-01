// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::pedantic)]

pub mod visitor;

use derivative::Derivative;
use num_bigint::BigInt;
use std::fmt::Debug;
use void::Void;

pub trait Staged: Clone + Debug {}

impl<T: Clone + Debug> Staged for T {}

pub trait Stage {
    type Attribute: Staged;
    type Block: Staged;
    type CallBodyFull: Staged;
    type CallBodySingle: Staged;
    type CallBodyX: Staged;
    type CallHeader: Staged;
    type DeclInfo: Staged;
    type Expr: Staged;
    type ExprArray: Staged;
    type ExprArrayRepeat: Staged;
    type ExprAssign: Staged;
    type ExprAssignOp: Staged;
    type ExprAssignUpdate: Staged;
    type ExprBinOp: Staged;
    type ExprBlock: Staged;
    type ExprCall: Staged;
    type ExprConjugate: Staged;
    type ExprFail: Staged;
    type ExprField: Staged;
    type ExprFor: Staged;
    type ExprHole: Staged;
    type ExprIf: Staged;
    type ExprIndex: Staged;
    type ExprInterp: Staged;
    type ExprLambda: Staged;
    type ExprLet: Staged;
    type ExprLit: Staged;
    type ExprPath: Staged;
    type ExprQubit: Staged;
    type ExprRange: Staged;
    type ExprRepeat: Staged;
    type ExprReturn: Staged;
    type ExprTernOp: Staged;
    type ExprTuple: Staged;
    type ExprUnOp: Staged;
    type ExprWhile: Staged;
    type ExprX: Staged;
    type FunctorExprBinOp: Staged;
    type FunctorExprLit: Staged;
    type FunctorExprNull: Staged;
    type FunctorExprX: Staged;
    type Ident: Staged;
    type ItemCallable: Staged;
    type ItemOpen: Staged;
    type ItemType: Staged;
    type ItemX: Staged;
    type Namespace: Staged;
    type PatBind: Staged;
    type PatDiscard: Staged;
    type Path: Staged;
    type PatOmit: Staged;
    type PatTuple: Staged;
    type PatX: Staged;
    type Project: Staged;
    type QubitInitArray: Staged;
    type QubitInitSingle: Staged;
    type QubitInitTuple: Staged;
    type QubitInitX: Staged;
    type SpecBodyGen: Staged;
    type SpecBodyImpl: Staged;
    type SpecBodyX: Staged;
    type SpecDecl: Staged;
    type TyApp: Staged;
    type TyArrow: Staged;
    type TyHole: Staged;
    type TyPath: Staged;
    type TypeDefField: Staged;
    type TypeDefTuple: Staged;
    type TypeDefX: Staged;
    type TyPrim: Staged;
    type TyTuple: Staged;
    type TyVar: Staged;
    type TyX: Staged;
}

/// The undecorated stage.
pub struct UD;

impl Stage for UD {
    type Attribute = ();
    type Block = ();
    type CallBodyFull = ();
    type CallBodySingle = ();
    type CallBodyX = Void;
    type CallHeader = ();
    type DeclInfo = ();
    type Expr = ();
    type ExprArray = ();
    type ExprArrayRepeat = ();
    type ExprAssign = ();
    type ExprAssignOp = ();
    type ExprAssignUpdate = ();
    type ExprBinOp = ();
    type ExprBlock = ();
    type ExprCall = ();
    type ExprConjugate = ();
    type ExprFail = ();
    type ExprField = ();
    type ExprFor = ();
    type ExprHole = ();
    type ExprIf = ();
    type ExprIndex = ();
    type ExprInterp = ();
    type ExprLambda = ();
    type ExprLet = ();
    type ExprLit = ();
    type ExprPath = ();
    type ExprQubit = ();
    type ExprRange = ();
    type ExprRepeat = ();
    type ExprReturn = ();
    type ExprTernOp = ();
    type ExprTuple = ();
    type ExprUnOp = ();
    type ExprWhile = ();
    type ExprX = Void;
    type FunctorExprBinOp = ();
    type FunctorExprLit = ();
    type FunctorExprNull = ();
    type FunctorExprX = Void;
    type Ident = ();
    type ItemCallable = ();
    type ItemOpen = ();
    type ItemType = ();
    type ItemX = Void;
    type Namespace = ();
    type PatBind = ();
    type PatDiscard = ();
    type Path = ();
    type PatOmit = ();
    type PatTuple = ();
    type PatX = Void;
    type Project = ();
    type QubitInitArray = ();
    type QubitInitSingle = ();
    type QubitInitTuple = ();
    type QubitInitX = Void;
    type SpecBodyGen = ();
    type SpecBodyImpl = ();
    type SpecBodyX = Void;
    type SpecDecl = ();
    type TyApp = ();
    type TyArrow = ();
    type TyHole = ();
    type TyPath = ();
    type TypeDefField = ();
    type TypeDefTuple = ();
    type TypeDefX = Void;
    type TyPrim = ();
    type TyTuple = ();
    type TyVar = ();
    type TyX = Void;
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct Project<S: Stage> {
    pub stage: S::Project,
    pub namespaces: Vec<Namespace<S>>,
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct Namespace<S: Stage> {
    pub stage: S::Namespace,
    pub name: Path<S>,
    pub items: Vec<Item<S>>,
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub enum Item<S: Stage> {
    Open(S::ItemOpen, Path<S>, Ident<S>),
    Type(S::ItemType, DeclInfo<S>, Ident<S>, TypeDef<S>),
    Callable(S::ItemCallable, DeclInfo<S>, CallHeader<S>, CallBody<S>),
    X(S::ItemX),
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct DeclInfo<S: Stage> {
    pub stage: S::DeclInfo,
    pub attributes: Vec<Attribute<S>>,
    pub visibility: Visibility,
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct Attribute<S: Stage> {
    pub stage: S::Attribute,
    pub name: Path<S>,
    pub arg: Expr<S>,
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub enum TypeDef<S: Stage> {
    Field(S::TypeDefField, Option<Ident<S>>, Ty<S>),
    Tuple(S::TypeDefTuple, Vec<TypeDef<S>>),
    X(S::TypeDefX),
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct CallHeader<S: Stage> {
    pub stage: S::CallHeader,
    pub kind: CallKind,
    pub name: Ident<S>,
    pub ty_params: Vec<Ident<S>>,
    pub input: Pat<S>,
    pub output: Ty<S>,
    pub functors: FunctorExpr<S>,
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub enum CallBody<S: Stage> {
    Single(S::CallBodySingle, SpecBody<S>),
    Full(S::CallBodyFull, Vec<SpecDecl<S>>),
    X(S::CallBodyX),
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct SpecDecl<S: Stage> {
    pub stage: S::SpecDecl,
    pub spec: Spec,
    pub body: SpecBody<S>,
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub enum SpecBody<S: Stage> {
    Gen(S::SpecBodyGen, SpecGen),
    Impl(S::SpecBodyImpl, Pat<S>, Block<S>),
    X(S::SpecBodyX),
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub enum FunctorExpr<S: Stage> {
    BinOp(
        S::FunctorExprBinOp,
        SetOp,
        Box<FunctorExpr<S>>,
        Box<FunctorExpr<S>>,
    ),
    Lit(S::FunctorExprLit, Functor),
    Null(S::FunctorExprNull),
    X(S::FunctorExprX),
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub enum Ty<S: Stage> {
    App(S::TyApp, Box<Ty<S>>, Vec<Ty<S>>),
    Arrow(S::TyArrow, CallKind, Box<Ty<S>>, Box<Ty<S>>, FunctorExpr<S>),
    Hole(S::TyHole),
    Path(S::TyPath, Path<S>),
    Prim(S::TyPrim, TyPrim),
    Tuple(S::TyTuple, Vec<Ty<S>>),
    Var(S::TyVar, TyVar),
    X(S::TyX),
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct Expr<S: Stage> {
    pub stage: S::Expr,
    pub kind: ExprKind<S>,
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub enum ExprKind<S: Stage> {
    Array(S::ExprArray, Vec<Expr<S>>),
    ArrayRepeat(S::ExprArrayRepeat, Box<Expr<S>>, Box<Expr<S>>),
    Assign(S::ExprAssign, Box<Expr<S>>, Box<Expr<S>>),
    AssignOp(S::ExprAssignOp, BinOp, Box<Expr<S>>, Box<Expr<S>>),
    AssignUpdate(
        S::ExprAssignUpdate,
        Box<Expr<S>>,
        Box<Expr<S>>,
        Box<Expr<S>>,
    ),
    BinOp(S::ExprBinOp, BinOp, Box<Expr<S>>, Box<Expr<S>>),
    Block(S::ExprBlock, Block<S>),
    Call(S::ExprCall, Box<Expr<S>>, Box<Expr<S>>),
    Conjugate(S::ExprConjugate, Block<S>, Block<S>),
    Fail(S::ExprFail, Box<Expr<S>>),
    Field(S::ExprField, Box<Expr<S>>, Ident<S>),
    For(S::ExprFor, Pat<S>, Box<Expr<S>>, Block<S>),
    Hole(S::ExprHole),
    If(S::ExprIf, Vec<(Expr<S>, Block<S>)>, Option<Block<S>>),
    Index(S::ExprIndex, Box<Expr<S>>, Box<Expr<S>>),
    Interp(S::ExprInterp, String, Vec<Expr<S>>),
    Lambda(S::ExprLambda, CallKind, Pat<S>, Box<Expr<S>>),
    Let(S::ExprLet, Pat<S>, Box<Expr<S>>),
    Lit(S::ExprLit, Lit),
    Path(S::ExprPath, Path<S>),
    Qubit(
        S::ExprQubit,
        QubitKind,
        Pat<S>,
        QubitInit<S>,
        Option<Block<S>>,
    ),
    Range(S::ExprRange, Box<Expr<S>>, Box<Expr<S>>, Box<Expr<S>>),
    Repeat(S::ExprRepeat, Block<S>, Box<Expr<S>>, Option<Block<S>>),
    Return(S::ExprReturn, Box<Expr<S>>),
    TernOp(
        S::ExprTernOp,
        TernOp,
        Box<Expr<S>>,
        Box<Expr<S>>,
        Box<Expr<S>>,
    ),
    Tuple(S::ExprTuple, Vec<Expr<S>>),
    UnOp(S::ExprUnOp, UnOp, Box<Expr<S>>),
    While(S::ExprWhile, Box<Expr<S>>, Block<S>),
    X(S::ExprX),
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct Block<S: Stage> {
    pub stage: S::Block,
    pub exprs: Vec<Expr<S>>,
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct Ident<S: Stage> {
    pub stage: S::Ident,
    pub name: String,
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct Path<S: Stage> {
    pub stage: S::Path,
    pub parts: Vec<String>,
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub enum Pat<S: Stage> {
    Bind(S::PatBind, Mut, Ident<S>, Ty<S>),
    Discard(S::PatDiscard, Ty<S>),
    Omit(S::PatOmit),
    Tuple(S::PatTuple, Vec<Pat<S>>),
    X(S::PatX),
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub enum QubitInit<S: Stage> {
    Single(S::QubitInitSingle),
    Tuple(S::QubitInitTuple, Vec<QubitInit<S>>),
    Array(S::QubitInitArray, Box<Expr<S>>),
    X(S::QubitInitX),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Visibility {
    Public,
    Internal,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CallKind {
    Function,
    Operation,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Mut {
    Immutable,
    Mutable,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TyPrim {
    Array,
    BigInt,
    Bool,
    Double,
    Int,
    Pauli,
    Qubit,
    Range,
    Result,
    String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TyVar {
    Name(String),
    Id(u32),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Lit {
    BigInt(BigInt),
    Bool(bool),
    Double(f64),
    Int(u64),
    Pauli(Pauli),
    Result(Result),
    String(String),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Result {
    Zero,
    One,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Pauli {
    I,
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum QubitKind {
    Fresh,
    Dirty,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Functor {
    Adj,
    Ctl,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Spec {
    Body,
    Adj,
    Ctl,
    CtlAdj,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SpecGen {
    Auto,
    Distribute,
    Intrinsic,
    Invert,
    Slf,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UnOp {
    Functor(Functor),
    Neg,
    NotB,
    NotL,
    Pos,
    Unwrap,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BinOp {
    Add,
    AndB,
    AndL,
    Div,
    Eq,
    Exp,
    Gt,
    Gte,
    Lt,
    Lte,
    Mod,
    Mul,
    Neq,
    OrB,
    OrL,
    Shl,
    Shr,
    Sub,
    XorB,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TernOp {
    Cond,
    Update,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SetOp {
    Union,
    Intersect,
}
