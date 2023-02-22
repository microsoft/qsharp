// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The abstract syntax tree (AST) for Q#. The AST directly corresponds to the surface syntax of Q#.

#![warn(missing_docs)]

use num_bigint::BigInt;
use std::ops::Index;

/// The unique identifier for an AST node.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId(u32);

impl NodeId {
    /// The ID for the root node in an AST.
    pub const ROOT: Self = Self(0);

    /// The ID used before unique IDs have been assigned.
    pub const PLACEHOLDER: Self = Self(u32::MAX);

    /// The successor of this ID.
    #[must_use]
    pub fn successor(self) -> Self {
        Self(self.0 + 1)
    }
}

/// A region between two source code positions. Spans are the half-open interval `[lo, hi)`. The
/// offsets are absolute within an AST, assuming that each file has its own offset.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Span {
    /// The offset of the first byte.
    pub lo: usize,
    /// The offset immediately following the last byte.
    pub hi: usize,
}

impl Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[index.lo..index.hi]
    }
}

/// The package currently being compiled and the root node of an AST.
#[derive(Clone, Debug, PartialEq)]
pub struct Package {
    /// The node ID.
    pub id: NodeId,
    /// The namespaces in the package.
    pub namespaces: Vec<Namespace>,
}

/// A namespace.
#[derive(Clone, Debug, PartialEq)]
pub struct Namespace {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The namespace name.
    pub name: Ident,
    /// The items in the namespace.
    pub items: Vec<Item>,
}

/// A namespace item.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    /// The ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The item kind.
    pub kind: ItemKind,
}

/// A namespace item kind.
#[derive(Clone, Debug, PartialEq)]
pub enum ItemKind {
    /// An `open` statement for another namespace with an optional alias.
    Open(Ident, Option<Ident>),
    /// A `newtype` declaration.
    Type(DeclMeta, Ident, TyDef),
    /// A `function` or `operation` declaration.
    Callable(DeclMeta, CallableDecl),
}

/// Metadata for a top-level declaration.
#[derive(Clone, Debug, PartialEq)]
pub struct DeclMeta {
    /// The declaration attributes.
    pub attrs: Vec<Attr>,
    /// The declaration visibility.
    pub visibility: Option<Visibility>,
}

/// A visibility modifier.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Visibility {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The visibility kind.
    pub kind: VisibilityKind,
}

/// An attribute.
#[derive(Clone, Debug, PartialEq)]
pub struct Attr {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The name of the attribute.
    pub name: Path,
    /// The argument to the attribute.
    pub arg: Expr,
}

/// A type definition.
#[derive(Clone, Debug, PartialEq)]
pub enum TyDef {
    /// A field definition with an optional name but required type.
    Field(Option<Ident>, Ty),
    /// A tuple.
    Tuple(Vec<TyDef>),
}

/// A callable declaration header.
#[derive(Clone, Debug, PartialEq)]
pub struct CallableDecl {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The callable kind.
    pub kind: CallableKind,
    /// The name of the callable.
    pub name: Ident,
    /// The type parameters to the callable.
    pub ty_params: Vec<Ident>,
    /// The input to the callable.
    pub input: Pat,
    /// The return type of the callable.
    pub output: Ty,
    /// The functors supported by the callable.
    pub functors: Option<FunctorExpr>,
    /// The body of the callable.
    pub body: CallableBody,
}

/// The body of a callable.
#[derive(Clone, Debug, PartialEq)]
pub enum CallableBody {
    /// A block for the callable's body specialization.
    Block(Block),
    /// One or more explicit specializations.
    Specs(Vec<SpecDecl>),
}

/// A specialization declaration.
#[derive(Clone, Debug, PartialEq)]
pub struct SpecDecl {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// Which specialization is being declared.
    pub spec: Spec,
    /// The body of the specialization.
    pub body: SpecBody,
}

/// The body of a specialization.
#[derive(Clone, Debug, PartialEq)]
pub enum SpecBody {
    /// The strategy to use to automatically generate the specialization.
    Gen(SpecGen),
    /// A manual implementation of the specialization.
    Impl(Pat, Block),
}

/// An expression that describes a set of functors.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FunctorExpr {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The functor expression kind.
    pub kind: FunctorExprKind,
}

/// A functor expression kind.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FunctorExprKind {
    /// A binary operation.
    BinOp(SetOp, Box<FunctorExpr>, Box<FunctorExpr>),
    /// A literal for a specific functor.
    Lit(Functor),
    /// A parenthesized group.
    Paren(Box<FunctorExpr>),
}

/// A type.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Ty {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The type kind.
    pub kind: TyKind,
}

/// A type kind.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TyKind {
    /// One or more type arguments applied to a type constructor.
    App(Box<Ty>, Vec<Ty>),
    /// An arrow type: `->` for a function or `=>` for an operation.
    Arrow(CallableKind, Box<Ty>, Box<Ty>, Option<FunctorExpr>),
    /// An unspecified type, `_`, which may be inferred.
    Hole,
    /// A type wrapped in parentheses.
    Paren(Box<Ty>),
    /// A named type.
    Path(Path),
    /// A primitive type.
    Prim(TyPrim),
    /// A tuple type.
    Tuple(Vec<Ty>),
    /// A type variable.
    Var(TyVar),
}

/// A sequenced block of statements.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The statements in the block.
    pub stmts: Vec<Stmt>,
}

/// A statement.
#[derive(Clone, Debug, PartialEq)]
pub struct Stmt {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The statement kind.
    pub kind: StmtKind,
}

/// A statement kind.
#[derive(Clone, Debug, PartialEq)]
pub enum StmtKind {
    /// A borrowed qubit binding: `borrow a = b;`.
    Borrow(Pat, QubitInit, Option<Block>),
    /// An expression without a trailing semicolon.
    Expr(Expr),
    /// A let binding: `let a = b;`.
    Let(Pat, Expr),
    /// A mutable binding: `mutable a = b;`.
    Mutable(Pat, Expr),
    /// An expression with a trailing semicolon.
    Semi(Expr),
    /// A fresh qubit binding: `use a = b;`.
    Use(Pat, QubitInit, Option<Block>),
}

/// An expression.
#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The expression kind.
    pub kind: ExprKind,
}

/// An expression kind.
#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    /// An array: `[a, b, c]`.
    Array(Vec<Expr>),
    /// An array constructed by repeating a value: `[a, size = b]`.
    ArrayRepeat(Box<Expr>, Box<Expr>),
    /// An assignment: `set a = b`.
    Assign(Box<Expr>, Box<Expr>),
    /// An assignment with a compound operator. For example: `set a += b`.
    AssignOp(BinOp, Box<Expr>, Box<Expr>),
    /// An assignment with a compound update operator: `set a w/= b <- c`.
    AssignUpdate(Box<Expr>, Box<Expr>, Box<Expr>),
    /// A binary operator.
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    /// A block: `{ ... }`.
    Block(Block),
    /// A call: `a(b)`.
    Call(Box<Expr>, Box<Expr>),
    /// A conjugation: `within { ... } apply { ... }`.
    Conjugate(Block, Block),
    /// A failure: `fail "message"`.
    Fail(Box<Expr>),
    /// A field accessor: `a::F`.
    Field(Box<Expr>, Ident),
    /// A for loop: `for a in b { ... }`.
    For(Pat, Box<Expr>, Block),
    /// An unspecified expression, _, which may indicate partial application or a typed hole.
    Hole,
    /// An if expression with an optional else block: `if a { ... } else { ... }`.
    ///
    /// Note that, as a special case, `elif ...` is effectively parsed as `else if ...`, without a
    /// block wrapping the `if`. This distinguishes `elif ...` from `else { if ... }`, which does
    /// have a block.
    If(Box<Expr>, Block, Option<Box<Expr>>),
    /// An index accessor: `a[b]`.
    Index(Box<Expr>, Box<Expr>),
    /// A lambda: `a -> b` for a function and `a => b` for an operation.
    Lambda(CallableKind, Pat, Box<Expr>),
    /// A literal.
    Lit(Lit),
    /// Parentheses: `(a)`.
    Paren(Box<Expr>),
    /// A path: `a` or `a.b`.
    Path(Path),
    /// A range: `start..step..stop`, `start..stop`, `start...`, `...stop`, or `...`.
    Range(Option<Box<Expr>>, Option<Box<Expr>>, Option<Box<Expr>>),
    /// A repeat-until loop with an optional fixup: `repeat { ... } until a fixup { ... }`.
    Repeat(Block, Box<Expr>, Option<Block>),
    /// A return: `return a`.
    Return(Box<Expr>),
    /// A ternary operator.
    TernOp(TernOp, Box<Expr>, Box<Expr>, Box<Expr>),
    /// A tuple: `(a, b, c)`.
    Tuple(Vec<Expr>),
    /// A unary operator.
    UnOp(UnOp, Box<Expr>),
    /// A while loop: `while a { ... }`.
    While(Box<Expr>, Block),
}

/// A pattern.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Pat {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The pattern kind.
    pub kind: PatKind,
}

/// A pattern kind.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PatKind {
    /// A binding with an optional type annotation.
    Bind(Ident, Option<Ty>),
    /// A discarded binding, `_`, with an optional type annotation.
    Discard(Option<Ty>),
    /// An elided pattern, `...`, used by specializations.
    Elided,
    /// Parentheses: `(a)`.
    Paren(Box<Pat>),
    /// A tuple: `(a, b, c)`.
    Tuple(Vec<Pat>),
}

/// A qubit initializer.
#[derive(Clone, Debug, PartialEq)]
pub struct QubitInit {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The qubit initializer kind.
    pub kind: QubitInitKind,
}

/// A qubit initializer kind.
#[derive(Clone, Debug, PartialEq)]
pub enum QubitInitKind {
    /// An array of qubits: `Qubit[a]`.
    Array(Box<Expr>),
    /// A parenthesized initializer: `(a)`.
    Paren(Box<QubitInit>),
    /// A single qubit: `Qubit()`.
    Single,
    /// A tuple: `(a, b, c)`.
    Tuple(Vec<QubitInit>),
}

/// A path to a declaration.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Path {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The namespace.
    pub namespace: Option<Ident>,
    /// The declaration name.
    pub name: Ident,
}

/// An identifier.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Ident {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The identifier name.
    pub name: String,
}

/// A declaration visibility kind.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VisibilityKind {
    /// Visible everywhere.
    Public,
    /// Visible within a package.
    Internal,
}

/// A callable kind.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CallableKind {
    /// A function.
    Function,
    /// An operation.
    Operation,
}

/// A primitive type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TyPrim {
    /// The array type.
    Array,
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
    /// The measurement result type.
    Result,
    /// The string type.
    String,
}

/// A type variable.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TyVar {
    /// A named variable.
    Name(String),
    /// A numeric variable.
    Id(u32),
}

/// A literal.
#[derive(Clone, Debug, PartialEq)]
pub enum Lit {
    /// A big integer literal.
    BigInt(BigInt),
    /// A boolean literal.
    Bool(bool),
    /// A floating-point literal.
    Double(f64),
    /// An integer literal.
    Int(u64),
    /// A Pauli operator literal.
    Pauli(Pauli),
    /// A measurement result literal.
    Result(Result),
    /// A string literal.
    String(String),
}

/// A measurement result.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Result {
    /// The zero eigenvalue.
    Zero,
    /// The one eigenvalue.
    One,
}

/// A Pauli operator.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Pauli {
    /// The Pauli I operator.
    I,
    /// The Pauli X operator.
    X,
    /// The Pauli Y operator.
    Y,
    /// The Pauli Z operator.
    Z,
}

/// A functor that may be applied to an operation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Functor {
    /// The adjoint functor.
    Adj,
    /// The controlled functor.
    Ctl,
}

/// A specialization that may be implemented for an operation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Spec {
    /// The default specialization.
    Body,
    /// The adjoint specialization.
    Adj,
    /// The controlled specialization.
    Ctl,
    /// The controlled adjoint specialization.
    CtlAdj,
}

/// A strategy for generating a specialization.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SpecGen {
    /// Choose a strategy automatically.
    Auto,
    /// Distributes controlled qubits.
    Distribute,
    /// A specialization implementation is not generated, but is instead left as an opaque
    /// declaration.
    Intrinsic,
    /// Inverts the order of operations.
    Invert,
    /// Uses the body specialization without modification.
    Slf,
}

/// A unary operator.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UnOp {
    /// A functor application.
    Functor(Functor),
    /// Negation: `-`.
    Neg,
    /// Bitwise NOT: `~~~`.
    NotB,
    /// Logical NOT: `not`.
    NotL,
    /// A leading `+`.
    Pos,
    /// Unwrap a user-defined type: `!`.
    Unwrap,
}

/// A binary operator.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BinOp {
    /// Addition: `+`.
    Add,
    /// Bitwise AND: `&&&`.
    AndB,
    /// Logical AND: `and`.
    AndL,
    /// Division: `/`.
    Div,
    /// Equality: `==`.
    Eq,
    /// Exponentiation: `^`.
    Exp,
    /// Greater than: `>`.
    Gt,
    /// Greater than or equal: `>=`.
    Gte,
    /// Less than: `<`.
    Lt,
    /// Less than or equal: `<=`.
    Lte,
    /// Modulus: `%`.
    Mod,
    /// Multiplication: `*`.
    Mul,
    /// Inequality: `!=`.
    Neq,
    /// Bitwise OR: `|||`.
    OrB,
    /// Logical OR: `or`.
    OrL,
    /// Shift left: `<<<`.
    Shl,
    /// Shift right: `>>>`.
    Shr,
    /// Subtraction: `-`.
    Sub,
    /// Bitwise XOR: `^^^`.
    XorB,
}

/// A ternary operator.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TernOp {
    /// Conditional: `a ? b | c`.
    Cond,
    /// Aggregate update: `a w/ b <- c`.
    Update,
}

/// A set operator.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SetOp {
    /// The set union.
    Union,
    /// The set intersection.
    Intersect,
}
