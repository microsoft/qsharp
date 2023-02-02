// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::pedantic)]

pub mod mut_visit;
pub mod visit;

use num_bigint::BigInt;
use std::fmt::Debug;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId(u32);

#[derive(Clone, Debug)]
pub struct Project {
    pub id: NodeId,
    pub namespaces: Vec<Namespace>,
}

#[derive(Clone, Debug)]
pub struct Namespace {
    pub id: NodeId,
    pub name: Path,
    pub items: Vec<Item>,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub id: NodeId,
    pub kind: ItemKind,
}

#[derive(Clone, Debug)]
pub enum ItemKind {
    Open(Path, Ident),
    Type(DeclInfo, Ident, TyDef),
    Callable(DeclInfo, CallHeader, CallBody),
}

#[derive(Clone, Debug)]
pub struct DeclInfo {
    pub id: NodeId,
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
}

#[derive(Clone, Debug)]
pub struct Attribute {
    pub id: NodeId,
    pub name: Path,
    pub arg: Expr,
}

#[derive(Clone, Debug)]
pub enum TyDef {
    Field(Option<Ident>, Ty),
    Tuple(Vec<TyDef>),
}

#[derive(Clone, Debug)]
pub struct CallHeader {
    pub kind: CallKind,
    pub name: Ident,
    pub ty_params: Vec<Ident>,
    pub input: Pat,
    pub output: Ty,
    pub functors: FunctorExpr,
}

#[derive(Clone, Debug)]
pub enum CallBody {
    Single(SpecBody),
    Full(Vec<SpecDecl>),
}

#[derive(Clone, Debug)]
pub struct SpecDecl {
    pub id: NodeId,
    pub spec: Spec,
    pub body: SpecBody,
}

#[derive(Clone, Debug)]
pub enum SpecBody {
    Gen(SpecGen),
    Impl(Pat, Block),
}

#[derive(Clone, Debug)]
pub enum FunctorExpr {
    BinOp(SetOp, Box<FunctorExpr>, Box<FunctorExpr>),
    Lit(Functor),
    Null,
}

#[derive(Clone, Debug)]
pub struct Ty {
    pub id: NodeId,
    pub kind: TyKind,
}

#[derive(Clone, Debug)]
pub enum TyKind {
    App(Box<Ty>, Vec<Ty>),
    Arrow(CallKind, Box<Ty>, Box<Ty>, FunctorExpr),
    Hole,
    Path(Path),
    Prim(TyPrim),
    Tuple(Vec<Ty>),
    Var(TyVar),
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Array(Vec<Expr>),
    ArrayRepeat(Box<Expr>, Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    AssignOp(BinOp, Box<Expr>, Box<Expr>),
    AssignUpdate(Box<Expr>, Box<Expr>, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    Block(Block),
    Call(Box<Expr>, Box<Expr>),
    Conjugate(Block, Block),
    Fail(Box<Expr>),
    Field(Box<Expr>, Ident),
    For(Pat, Box<Expr>, Block),
    Hole,
    If(Vec<(Expr, Block)>, Option<Block>),
    Index(Box<Expr>, Box<Expr>),
    Interp(String, Vec<Expr>),
    Lambda(CallKind, Pat, Box<Expr>),
    Let(Pat, Box<Expr>),
    Lit(Lit),
    Path(Path),
    Qubit(QubitKind, Pat, QubitInit, Option<Block>),
    Range(Box<Expr>, Box<Expr>, Box<Expr>),
    Repeat(Block, Box<Expr>, Option<Block>),
    Return(Box<Expr>),
    TernOp(TernOp, Box<Expr>, Box<Expr>, Box<Expr>),
    Tuple(Vec<Expr>),
    UnOp(UnOp, Box<Expr>),
    While(Box<Expr>, Block),
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id: NodeId,
    pub exprs: Vec<Expr>,
}

#[derive(Clone, Debug)]
pub struct Ident {
    pub id: NodeId,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Path {
    pub id: NodeId,
    pub parts: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Pat {
    pub id: NodeId,
    pub kind: PatKind,
}

#[derive(Clone, Debug)]
pub enum PatKind {
    Bind(Mut, Ident, Ty),
    Discard(Ty),
    Omit,
    Tuple(Vec<Pat>),
}

#[derive(Clone, Debug)]
pub struct QubitInit {
    pub id: NodeId,
    pub kind: QubitInitKind,
}

#[derive(Clone, Debug)]
pub enum QubitInitKind {
    Single,
    Tuple(Vec<QubitInit>),
    Array(Box<Expr>),
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
