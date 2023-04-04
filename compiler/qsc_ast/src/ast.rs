// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The abstract syntax tree (AST) for Q#. The AST directly corresponds to the surface syntax of Q#.

#![warn(missing_docs)]

use indenter::{indented, Format};
use miette::SourceSpan;
use num_bigint::BigInt;
use std::{
    fmt::{self, Display, Formatter, Write},
    ops::{Bound, Index, RangeBounds},
};

/// The unique identifier for an AST node.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId(u32);

impl NodeId {
    const PLACEHOLDER: Self = Self(u32::MAX);

    /// Whether this ID is a placeholder.
    #[must_use]
    pub fn is_placeholder(self) -> bool {
        self == Self::PLACEHOLDER
    }

    /// The initial node ID.
    #[must_use]
    pub fn zero() -> Self {
        NodeId(0)
    }

    /// The successor of this ID.
    #[must_use]
    pub fn successor(self) -> Self {
        Self(self.0 + 1)
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::PLACEHOLDER
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A region between two source code positions. Spans are the half-open interval `[lo, hi)`. The
/// offsets are absolute within an AST, assuming that each file has its own offset.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Span {
    /// The offset of the first byte.
    pub lo: usize,
    /// The offset immediately following the last byte.
    pub hi: usize,
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{}-{}]", self.lo, self.hi)?;
        Ok(())
    }
}

impl Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[index.lo..index.hi]
    }
}

impl Index<Span> for String {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[index.lo..index.hi]
    }
}

impl RangeBounds<usize> for &Span {
    fn start_bound(&self) -> Bound<&usize> {
        Bound::Included(&self.lo)
    }

    fn end_bound(&self) -> Bound<&usize> {
        Bound::Excluded(&self.hi)
    }
}

impl From<Span> for SourceSpan {
    fn from(value: Span) -> Self {
        Self::from(value.lo..value.hi)
    }
}

// fn set_indentation(mut indent: &mut Indented<Formatter>, level: usize) {
//     let s = "    ".repeat(level);
//     indent = &mut indent.with_format(Format::Custom {
//         inserter: Box::new(|i, f| {
//             for i in 0..level {
//                 write!(f, "****");
//             }
//             Ok(())
//         }),
//     });
// }

/// The root node of an AST.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Package {
    /// The node ID.
    pub id: NodeId,
    /// The namespaces in the package.
    pub namespaces: Vec<Namespace>,
    /// The entry expression for an executable package.
    pub entry: Option<Expr>,
}

impl Package {
    /// Creates a new package.
    #[must_use]
    pub fn new(namespaces: Vec<Namespace>, entry: Option<Expr>) -> Self {
        Self {
            id: NodeId::default(),
            namespaces,
            entry,
        }
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = indented(f).with_format(Format::Uniform { indentation: "" });
        write!(indent, "Package {}:", self.id)?;
        indent = indent.with_format(Format::Uniform {
            indentation: "    ",
        });
        if let Some(e) = &self.entry {
            write!(indent, "\nentry expression: {e}")?;
        }
        for ns in &self.namespaces {
            write!(indent, "\n{ns}")?;
        }
        Ok(())
    }
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

impl Display for Namespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = indented(f).with_format(Format::Uniform { indentation: "" });
        write!(
            indent,
            "Namespace {} {} ({}):",
            self.id, self.span, self.name
        )?;
        indent = indent.with_format(Format::Uniform {
            indentation: "    ",
        });
        for i in &self.items {
            write!(indent, "\n{i}")?;
        }
        Ok(())
    }
}

/// An item.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Item {
    /// The ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The item metadata.
    pub meta: ItemMeta,
    /// The item kind.
    pub kind: ItemKind,
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = indented(f).with_format(Format::Uniform { indentation: "" });
        write!(indent, "Item {} {}:", self.id, self.span)?;
        indent = indent.with_format(Format::Uniform {
            indentation: "    ",
        });
        if self.meta.attrs.is_empty() && self.meta.visibility.is_none() {
            write!(indent, "\nmeta: <none>")?;
        } else {
            write!(indent, "\nmeta:")?;
            indent = indent.with_format(Format::Uniform {
                indentation: "        ",
            });
            write!(indent, "{}", self.meta)?;
            indent = indent.with_format(Format::Uniform {
                indentation: "    ",
            });
        }
        write!(indent, "\n{}", self.kind)?;
        Ok(())
    }
}

/// An item kind.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ItemKind {
    /// A `function` or `operation` declaration.
    Callable(CallableDecl),
    /// Default item when nothing has been parsed.
    #[default]
    Err,
    /// An `open` item for a namespace with an optional alias.
    Open(Ident, Option<Ident>),
    /// A `newtype` declaration.
    Ty(Ident, TyDef),
}

impl Display for ItemKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            ItemKind::Callable(decl) => write!(f, "{decl}")?,
            ItemKind::Err => write!(f, "Err")?,
            ItemKind::Open(name, alias) => match alias {
                Some(a) => write!(f, "Open ({name}) ({a})")?,
                None => write!(f, "Open ({name})")?,
            },
            ItemKind::Ty(name, t) => write!(f, "New Type ({name}): {t}")?,
        }
        Ok(())
    }
}

/// Metadata for an item.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemMeta {
    /// The attributes.
    pub attrs: Vec<Attr>,
    /// The visibility.
    pub visibility: Option<Visibility>,
}

impl Display for ItemMeta {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for a in &self.attrs {
            write!(f, "\n{a}")?;
        }
        if let Some(v) = &self.visibility {
            write!(f, "\n{v}")?;
        }
        Ok(())
    }
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

impl Display for Visibility {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Visibility {} {} ({:?})", self.id, self.span, self.kind)
    }
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

impl Display for Attr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = indented(f).with_format(Format::Uniform { indentation: "" });
        write!(indent, "Attr {} {} ({}):", self.id, self.span, self.name)?;
        indent = indent.with_format(Format::Uniform {
            indentation: "    ",
        });
        write!(indent, "\n{}", self.arg)?;
        Ok(())
    }
}

/// A type definition.
#[derive(Clone, Debug, PartialEq)]
pub struct TyDef {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The type definition kind.
    pub kind: TyDefKind,
}

impl Display for TyDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "TyDef {} {}: {}", self.id, self.span, self.kind)
    }
}

/// A type definition kind.
#[derive(Clone, Debug, PartialEq)]
pub enum TyDefKind {
    /// A field definition with an optional name but required type.
    Field(Option<Ident>, Ty),
    /// A parenthesized type definition.
    Paren(Box<TyDef>),
    /// A tuple.
    Tuple(Vec<TyDef>),
}

impl Display for TyDefKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = indented(f).with_format(Format::Uniform { indentation: "" });
        match &self {
            TyDefKind::Field(name, t) => {
                write!(indent, "Field:")?;
                indent = indent.with_format(Format::Uniform {
                    indentation: "    ",
                });
                if let Some(n) = name {
                    write!(indent, "\n{n}")?;
                }
                write!(indent, "\n{t}")?;
            }
            TyDefKind::Paren(t) => {
                write!(indent, "Paren:")?;
                indent = indent.with_format(Format::Uniform {
                    indentation: "    ",
                });
                write!(indent, "\n{t}")?;
            }
            TyDefKind::Tuple(ts) => {
                if ts.is_empty() {
                    write!(indent, "Unit")?;
                } else {
                    write!(indent, "Tuple:")?;
                    indent = indent.with_format(Format::Uniform {
                        indentation: "    ",
                    });
                    for t in ts {
                        write!(indent, "\n{t}")?;
                    }
                }
            }
        }
        Ok(())
    }
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

impl Display for CallableDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = indented(f).with_format(Format::Uniform { indentation: "" });
        write!(
            indent,
            "Callable {} {} ({:?}):",
            self.id, self.span, self.kind
        )?;
        indent = indent.with_format(Format::Uniform {
            indentation: "    ",
        });
        write!(indent, "\nname: {}", self.name)?;
        if self.ty_params.is_empty() {
            write!(indent, "\ntype params: <none>")?;
        } else {
            write!(indent, "\ntype params:")?;
            indent = indent.with_format(Format::Uniform {
                indentation: "        ",
            });
            for t in &self.ty_params {
                write!(indent, "\n{t}")?;
            }
            indent = indent.with_format(Format::Uniform {
                indentation: "    ",
            });
        }
        write!(indent, "\ninput: {}", self.input)?;
        write!(indent, "\noutput: {}", self.output)?;
        if let Some(f) = &self.functors {
            write!(indent, "\nfunctors: {f}")?;
        }
        write!(indent, "\nbody: {}", self.body)?;
        Ok(())
    }
}

/// The body of a callable.
#[derive(Clone, Debug, PartialEq)]
pub enum CallableBody {
    /// A block for the callable's body specialization.
    Block(Block),
    /// One or more explicit specializations.
    Specs(Vec<SpecDecl>),
}

impl Display for CallableBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "This is a callable body!")
    }
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

impl Display for FunctorExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Functor Expr {} {}: {}", self.id, self.span, self.kind)
    }
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
impl Display for FunctorExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = indented(f).with_format(Format::Uniform { indentation: "" });
        match self {
            FunctorExprKind::BinOp(op, l, r) => {
                write!(indent, "BinOp ({op:?})")?;
                indent = indent.with_format(Format::Uniform {
                    indentation: "    ",
                });
                write!(indent, "\n{l}")?;
                write!(indent, "\n{r}")?;
            }
            FunctorExprKind::Lit(f) => write!(indent, "Lit ({f:?})")?,
            FunctorExprKind::Paren(f) => write!(indent, "Paren: {f}")?,
        }
        Ok(())
    }
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

impl Display for Ty {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Type {} {}: {}", self.id, self.span, self.kind)
    }
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

impl Display for TyKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = indented(f).with_format(Format::Uniform { indentation: "" });
        match &self {
            TyKind::App(base, args) => {
                write!(indent, "App:")?;
                indent = indent.with_format(Format::Uniform {
                    indentation: "    ",
                });
                write!(indent, "\nbase type: {base}")?;
                write!(indent, "\narg types:")?;
                indent = indent.with_format(Format::Uniform {
                    indentation: "        ",
                });
                for a in args {
                    write!(indent, "\n{a}")?;
                }
            }
            TyKind::Arrow(ck, param, rtrn, functors) => {
                write!(indent, "Arrow ({ck:?}):")?;
                indent = indent.with_format(Format::Uniform {
                    indentation: "    ",
                });
                write!(indent, "\nparam: {param}")?;
                write!(indent, "\nreturn: {rtrn}")?;
                if let Some(f) = functors {
                    write!(indent, "\nfunctors: {f}")?;
                }
            }
            TyKind::Hole => write!(indent, "Hole")?,
            TyKind::Paren(t) => write!(indent, "Paren: {t}")?,
            TyKind::Path(p) => write!(indent, "Path: {p}")?,
            TyKind::Prim(t) => write!(indent, "Prim ({t:?})")?,
            TyKind::Tuple(ts) => {
                if ts.is_empty() {
                    write!(indent, "Unit")?;
                } else {
                    write!(indent, "Tuple:")?;
                    indent = indent.with_format(Format::Uniform {
                        indentation: "    ",
                    });
                    for t in ts {
                        write!(indent, "\n{t}")?;
                    }
                }
            }
            TyKind::Var(t) => match t {
                TyVar::Name(n) => write!(indent, "\nType Var {n}")?,
                TyVar::Id(id) => write!(indent, "\nType Var {id}")?,
            },
        }
        Ok(())
    }
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
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stmt {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The statement kind.
    pub kind: StmtKind,
}

/// A statement kind.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum StmtKind {
    /// An empty statement.
    #[default]
    Empty,
    /// An expression without a trailing semicolon.
    Expr(Expr),
    /// A let or mutable binding: `let a = b;` or `mutable x = b;`.
    Local(Mutability, Pat, Expr),
    /// A use or borrow qubit allocation: `use a = b;` or `borrow a = b;`.
    Qubit(QubitSource, Pat, QubitInit, Option<Block>),
    /// An expression with a trailing semicolon.
    Semi(Expr),
}

/// An expression.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Expr {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The expression kind.
    pub kind: ExprKind,
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "This is an expression!")
    }
}

/// An expression kind.
#[derive(Clone, Debug, Default, PartialEq)]
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
    /// An expression with invalid syntax that can't be parsed.
    #[default]
    Err,
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

impl Display for Pat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Pat {} {}: {}", self.id, self.span, self.kind)
    }
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

impl Display for PatKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = indented(f).with_format(Format::Uniform { indentation: "" });
        match self {
            PatKind::Bind(id, ty) => {
                write!(indent, "Bind:")?;
                indent = indent.with_format(Format::Uniform {
                    indentation: "    ",
                });
                write!(indent, "\n{id}")?;
                if let Some(t) = ty {
                    write!(indent, "\n{t}")?;
                }
            }
            PatKind::Discard(d) => match d {
                Some(t) => {
                    write!(indent, "Discard:")?;
                    indent = indent.with_format(Format::Uniform {
                        indentation: "    ",
                    });
                    write!(indent, "\n{t}")?;
                }
                None => write!(indent, "Discard")?,
            },
            PatKind::Elided => write!(indent, "Elided")?,
            PatKind::Paren(p) => {
                write!(indent, "Paren:")?;
                indent = indent.with_format(Format::Uniform {
                    indentation: "    ",
                });
                write!(indent, "\n{p}")?;
            }
            PatKind::Tuple(ps) => {
                write!(indent, "Tuple:")?;
                indent = indent.with_format(Format::Uniform {
                    indentation: "    ",
                });
                for p in ps {
                    write!(indent, "\n{p}")?;
                }
            }
        }
        Ok(())
    }
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

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(ns) = &self.namespace {
            write!(f, "Path {} {} ({}) ({})", self.id, self.span, ns, self.name)?;
        } else {
            write!(f, "Path {} {} ({})", self.id, self.span, self.name)?;
        }
        Ok(())
    }
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

impl Display for Ident {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Ident {} {} \"{}\"", self.id, self.span, self.name)
    }
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

/// The mutability of a binding.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Mutability {
    /// An immutable binding.
    Immutable,
    /// A mutable binding.
    Mutable,
}

/// The source of an allocated qubit.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum QubitSource {
    /// A qubit initialized to the zero state.
    Fresh,
    /// A qubit borrowed from another part of the program that may be in any state, and is expected
    /// to be returned to that state before being released.
    Dirty,
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
    Int(i64),
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

impl Display for Spec {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Spec::Body => f.write_str("body"),
            Spec::Adj => f.write_str("adjoint"),
            Spec::Ctl => f.write_str("controlled"),
            Spec::CtlAdj => f.write_str("controlled adjoint"),
        }
    }
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
