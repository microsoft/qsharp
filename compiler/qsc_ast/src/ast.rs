// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The abstract syntax tree (AST) for Q#. The AST directly corresponds to the surface syntax of Q#.

#![warn(missing_docs)]

use indenter::{indented, Format, Indented};
use num_bigint::BigInt;
use qsc_data_structures::span::Span;
use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter, Write},
    hash::{Hash, Hasher},
    rc::Rc,
};

fn set_indentation<'a, 'b>(
    indent: Indented<'a, Formatter<'b>>,
    level: usize,
) -> Indented<'a, Formatter<'b>> {
    indent.with_format(Format::Custom {
        inserter: Box::new(move |_, f| {
            for _ in 0..level {
                write!(f, "    ")?;
            }
            Ok(())
        }),
    })
}

/// The unique identifier for an AST node.
#[derive(Clone, Copy, Debug)]
pub struct NodeId(u32);

impl NodeId {
    const DEFAULT_VALUE: u32 = u32::MAX;

    /// The ID of the first node.
    pub const FIRST: Self = Self(0);

    /// The successor of this ID.
    #[must_use]
    pub fn successor(self) -> Self {
        Self(self.0 + 1)
    }

    /// True if this is the default ID.
    #[must_use]
    pub fn is_default(self) -> bool {
        self.0 == Self::DEFAULT_VALUE
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self(Self::DEFAULT_VALUE)
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.is_default() {
            f.write_str("_id_")
        } else {
            self.0.fmt(f)
        }
    }
}

impl From<NodeId> for usize {
    fn from(value: NodeId) -> Self {
        assert!(!value.is_default(), "default node ID should be replaced");
        value.0 as usize
    }
}

impl PartialEq for NodeId {
    fn eq(&self, other: &Self) -> bool {
        assert!(!self.is_default(), "default node ID should be replaced");
        self.0 == other.0
    }
}

impl Eq for NodeId {}

impl PartialOrd for NodeId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        assert!(!self.is_default(), "default node ID should be replaced");
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for NodeId {
    fn cmp(&self, other: &Self) -> Ordering {
        assert!(!self.is_default(), "default node ID should be replaced");
        self.0.cmp(&other.0)
    }
}

impl Hash for NodeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// The root node of an AST.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Package {
    /// The node ID.
    pub id: NodeId,
    /// The namespaces in the package.
    pub namespaces: Box<[Namespace]>,
    /// The entry expression for an executable package.
    pub entry: Option<Box<Expr>>,
}

impl Display for Package {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Package {}:", self.id)?;
        indent = set_indentation(indent, 1);
        if let Some(e) = &self.entry {
            write!(indent, "\nentry expression: {e}")?;
        }
        for ns in self.namespaces.iter() {
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
    /// The documentation.
    pub doc: Rc<str>,
    /// The namespace name.
    pub name: Box<Ident>,
    /// The items in the namespace.
    pub items: Box<[Box<Item>]>,
}

impl Display for Namespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(
            indent,
            "Namespace {} {} ({}):",
            self.id, self.span, self.name
        )?;
        indent = set_indentation(indent, 1);

        if !self.doc.is_empty() {
            write!(indent, "\ndoc:")?;
            indent = set_indentation(indent, 2);
            write!(indent, "\n{}", self.doc)?;
            indent = set_indentation(indent, 1);
        }

        for i in self.items.iter() {
            write!(indent, "\n{i}")?;
        }

        Ok(())
    }
}

/// An item.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    /// The ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The documentation.
    pub doc: Rc<str>,
    /// The attributes.
    pub attrs: Box<[Box<Attr>]>,
    /// The visibility.
    pub visibility: Option<Visibility>,
    /// The item kind.
    pub kind: Box<ItemKind>,
}

impl Default for Item {
    fn default() -> Self {
        Self {
            id: NodeId::default(),
            span: Span::default(),
            doc: "".into(),
            attrs: Box::default(),
            visibility: None,
            kind: Box::default(),
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Item {} {}:", self.id, self.span)?;
        indent = set_indentation(indent, 1);

        if !self.doc.is_empty() {
            write!(indent, "\ndoc:")?;
            indent = set_indentation(indent, 2);
            write!(indent, "\n{}", self.doc)?;
            indent = set_indentation(indent, 1);
        }

        for attr in self.attrs.iter() {
            write!(indent, "\n{attr}")?;
        }

        if let Some(visibility) = &self.visibility {
            write!(indent, "\n{visibility}")?;
        }

        write!(indent, "\n{}", self.kind)?;
        Ok(())
    }
}

/// An item kind.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ItemKind {
    /// A `function` or `operation` declaration.
    Callable(Box<CallableDecl>),
    /// Default item when nothing has been parsed.
    #[default]
    Err,
    /// An `open` item for a namespace with an optional alias.
    Open(Box<Ident>, Option<Box<Ident>>),
    /// A `newtype` declaration.
    Ty(Box<Ident>, Box<TyDef>),
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
    pub name: Box<Ident>,
    /// The argument to the attribute.
    pub arg: Box<Expr>,
}

impl Display for Attr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Attr {} {} ({}):", self.id, self.span, self.name)?;
        indent = set_indentation(indent, 1);
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
    pub kind: Box<TyDefKind>,
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
    Field(Option<Box<Ident>>, Box<Ty>),
    /// A parenthesized type definition.
    Paren(Box<TyDef>),
    /// A tuple.
    Tuple(Box<[Box<TyDef>]>),
}

impl Display for TyDefKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        match &self {
            TyDefKind::Field(name, t) => {
                write!(indent, "Field:")?;
                indent = set_indentation(indent, 1);
                if let Some(n) = name {
                    write!(indent, "\n{n}")?;
                }
                write!(indent, "\n{t}")?;
            }
            TyDefKind::Paren(t) => {
                write!(indent, "Paren:")?;
                indent = set_indentation(indent, 1);
                write!(indent, "\n{t}")?;
            }
            TyDefKind::Tuple(ts) => {
                if ts.is_empty() {
                    write!(indent, "Unit")?;
                } else {
                    write!(indent, "Tuple:")?;
                    indent = set_indentation(indent, 1);
                    for t in ts.iter() {
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
    pub name: Box<Ident>,
    /// The generic parameters to the callable.
    pub generics: Box<[Box<Ident>]>,
    /// The input to the callable.
    pub input: Box<Pat>,
    /// The return type of the callable.
    pub output: Box<Ty>,
    /// The functors supported by the callable.
    pub functors: Option<Box<FunctorExpr>>,
    /// The body of the callable.
    pub body: Box<CallableBody>,
}

impl Display for CallableDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(
            indent,
            "Callable {} {} ({:?}):",
            self.id, self.span, self.kind
        )?;
        indent = set_indentation(indent, 1);
        write!(indent, "\nname: {}", self.name)?;
        if !self.generics.is_empty() {
            write!(indent, "\ngenerics:")?;
            indent = set_indentation(indent, 2);
            for param in self.generics.iter() {
                write!(indent, "\n{param}")?;
            }
            indent = set_indentation(indent, 1);
        }
        write!(indent, "\ninput: {}", self.input)?;
        write!(indent, "\noutput: {}", self.output)?;
        if let Some(f) = &self.functors {
            write!(indent, "\nfunctors: {}", f.as_ref())?;
        }
        write!(indent, "\nbody: {}", self.body)?;
        Ok(())
    }
}

/// The body of a callable.
#[derive(Clone, Debug, PartialEq)]
pub enum CallableBody {
    /// A block for the callable's body specialization.
    Block(Box<Block>),
    /// One or more explicit specializations.
    Specs(Box<[Box<SpecDecl>]>),
}

impl Display for CallableBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CallableBody::Block(body) => write!(f, "Block: {body}")?,
            CallableBody::Specs(specs) => {
                let mut indent = set_indentation(indented(f), 0);
                write!(indent, "Specializations:")?;
                indent = set_indentation(indent, 1);
                for spec in specs.iter() {
                    write!(indent, "\n{spec}")?;
                }
            }
        }
        Ok(())
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

impl Display for SpecDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SpecDecl {} {} ({:?}): {}",
            self.id, self.span, self.spec, self.body
        )
    }
}

/// The body of a specialization.
#[derive(Clone, Debug, PartialEq)]
pub enum SpecBody {
    /// The strategy to use to automatically generate the specialization.
    Gen(SpecGen),
    /// A manual implementation of the specialization.
    Impl(Box<Pat>, Box<Block>),
}

impl Display for SpecBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        match self {
            SpecBody::Gen(sg) => write!(indent, "Gen: {sg:?}")?,
            SpecBody::Impl(p, b) => {
                write!(indent, "Impl:")?;
                indent = set_indentation(indent, 1);
                write!(indent, "\n{p}")?;
                write!(indent, "\n{b}")?;
            }
        }
        Ok(())
    }
}

/// An expression that describes a set of functors.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FunctorExpr {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The functor expression kind.
    pub kind: Box<FunctorExprKind>,
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
        match self {
            FunctorExprKind::BinOp(op, l, r) => write!(f, "BinOp {op:?}: ({l}) ({r})"),
            FunctorExprKind::Lit(func) => write!(f, "{func:?}"),
            FunctorExprKind::Paren(func) => write!(f, "Paren: {func}"),
        }
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
    pub kind: Box<TyKind>,
}

impl Display for Ty {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Type {} {}: {}", self.id, self.span, self.kind)
    }
}

/// A type kind.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TyKind {
    /// An array type.
    Array(Box<Ty>),
    /// An arrow type: `->` for a function or `=>` for an operation.
    Arrow(CallableKind, Box<Ty>, Box<Ty>, Option<Box<FunctorExpr>>),
    /// An unspecified type, `_`, which may be inferred.
    Hole,
    /// A type wrapped in parentheses.
    Paren(Box<Ty>),
    /// A named type.
    Path(Box<Path>),
    /// A type parameter.
    Param(Box<Ident>),
    /// A tuple type.
    Tuple(Box<[Ty]>),
}

impl Display for TyKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        match self {
            TyKind::Array(item) => write!(indent, "Array: {item}")?,
            TyKind::Arrow(ck, param, rtrn, functors) => {
                write!(indent, "Arrow ({ck:?}):")?;
                indent = set_indentation(indent, 1);
                write!(indent, "\nparam: {param}")?;
                write!(indent, "\nreturn: {rtrn}")?;
                if let Some(f) = functors {
                    write!(indent, "\nfunctors: {f}")?;
                }
            }
            TyKind::Hole => write!(indent, "Hole")?,
            TyKind::Paren(t) => write!(indent, "Paren: {t}")?,
            TyKind::Path(p) => write!(indent, "Path: {p}")?,
            TyKind::Param(name) => write!(indent, "\nType Param {name}")?,
            TyKind::Tuple(ts) => {
                if ts.is_empty() {
                    write!(indent, "Unit")?;
                } else {
                    write!(indent, "Tuple:")?;
                    indent = indent.with_format(Format::Uniform {
                        indentation: "    ",
                    });
                    for t in ts.iter() {
                        write!(indent, "\n{t}")?;
                    }
                }
            }
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
    pub stmts: Box<[Box<Stmt>]>,
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.stmts.is_empty() {
            write!(f, "Block {} {}: <empty>", self.id, self.span)?;
        } else {
            let mut indent = set_indentation(indented(f), 0);
            write!(indent, "Block {} {}:", self.id, self.span)?;
            indent = set_indentation(indent, 1);
            for s in self.stmts.iter() {
                write!(indent, "\n{s}")?;
            }
        }
        Ok(())
    }
}

/// A statement.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stmt {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The statement kind.
    pub kind: Box<StmtKind>,
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Stmt {} {}: {}", self.id, self.span, self.kind)
    }
}

/// A statement kind.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum StmtKind {
    /// An empty statement.
    Empty,
    /// An expression without a trailing semicolon.
    Expr(Box<Expr>),
    /// A let or mutable binding: `let a = b;` or `mutable x = b;`.
    Local(Mutability, Box<Pat>, Box<Expr>),
    /// An item.
    Item(Box<Item>),
    /// A use or borrow qubit allocation: `use a = b;` or `borrow a = b;`.
    Qubit(QubitSource, Box<Pat>, Box<QubitInit>, Option<Box<Block>>),
    /// An expression with a trailing semicolon.
    Semi(Box<Expr>),
    /// An invalid statement.
    #[default]
    Err,
}

impl Display for StmtKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        match self {
            StmtKind::Empty => write!(indent, "Empty")?,
            StmtKind::Expr(e) => write!(indent, "Expr: {e}")?,
            StmtKind::Item(item) => write!(indent, "Item: {item}")?,
            StmtKind::Local(m, lhs, rhs) => {
                write!(indent, "Local ({m:?}):")?;
                indent = set_indentation(indent, 1);
                write!(indent, "\n{lhs}")?;
                write!(indent, "\n{rhs}")?;
            }
            StmtKind::Qubit(s, lhs, rhs, block) => {
                write!(indent, "Qubit ({s:?})")?;
                indent = set_indentation(indent, 1);
                write!(indent, "\n{lhs}")?;
                write!(indent, "\n{rhs}")?;
                if let Some(b) = block {
                    write!(indent, "\n{b}")?;
                }
            }
            StmtKind::Semi(e) => write!(indent, "Semi: {e}")?,
            StmtKind::Err => indent.write_str("Err")?,
        }
        Ok(())
    }
}

/// An expression.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Expr {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The expression kind.
    pub kind: Box<ExprKind>,
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Expr {} {}: {}", self.id, self.span, self.kind)
    }
}

/// An expression kind.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ExprKind {
    /// An array: `[a, b, c]`.
    Array(Box<[Box<Expr>]>),
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
    Block(Box<Block>),
    /// A call: `a(b)`.
    Call(Box<Expr>, Box<Expr>),
    /// A conjugation: `within { ... } apply { ... }`.
    Conjugate(Box<Block>, Box<Block>),
    /// An expression with invalid syntax that can't be parsed.
    #[default]
    Err,
    /// A failure: `fail "message"`.
    Fail(Box<Expr>),
    /// A field accessor: `a::F`.
    Field(Box<Expr>, Box<Ident>),
    /// A for loop: `for a in b { ... }`.
    For(Box<Pat>, Box<Expr>, Box<Block>),
    /// An unspecified expression, _, which may indicate partial application or a typed hole.
    Hole,
    /// An if expression with an optional else block: `if a { ... } else { ... }`.
    ///
    /// Note that, as a special case, `elif ...` is effectively parsed as `else if ...`, without a
    /// block wrapping the `if`. This distinguishes `elif ...` from `else { if ... }`, which does
    /// have a block.
    If(Box<Expr>, Box<Block>, Option<Box<Expr>>),
    /// An index accessor: `a[b]`.
    Index(Box<Expr>, Box<Expr>),
    /// An interpolated string.
    Interpolate(Box<[StringComponent]>),
    /// A lambda: `a -> b` for a function and `a => b` for an operation.
    Lambda(CallableKind, Box<Pat>, Box<Expr>),
    /// A literal.
    Lit(Box<Lit>),
    /// Parentheses: `(a)`.
    Paren(Box<Expr>),
    /// A path: `a` or `a.b`.
    Path(Box<Path>),
    /// A range: `start..step..end`, `start..end`, `start...`, `...end`, or `...`.
    Range(Option<Box<Expr>>, Option<Box<Expr>>, Option<Box<Expr>>),
    /// A repeat-until loop with an optional fixup: `repeat { ... } until a fixup { ... }`.
    Repeat(Box<Block>, Box<Expr>, Option<Box<Block>>),
    /// A return: `return a`.
    Return(Box<Expr>),
    /// A ternary operator.
    TernOp(TernOp, Box<Expr>, Box<Expr>, Box<Expr>),
    /// A tuple: `(a, b, c)`.
    Tuple(Box<[Box<Expr>]>),
    /// A unary operator.
    UnOp(UnOp, Box<Expr>),
    /// A while loop: `while a { ... }`.
    While(Box<Expr>, Box<Block>),
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        match self {
            ExprKind::Array(exprs) => display_array(indent, exprs)?,
            ExprKind::ArrayRepeat(val, size) => display_array_repeat(indent, val, size)?,
            ExprKind::Assign(lhs, rhs) => display_assign(indent, lhs, rhs)?,
            ExprKind::AssignOp(op, lhs, rhs) => display_assign_op(indent, *op, lhs, rhs)?,
            ExprKind::AssignUpdate(container, item, val) => {
                display_assign_update(indent, container, item, val)?;
            }
            ExprKind::BinOp(op, lhs, rhs) => display_bin_op(indent, *op, lhs, rhs)?,
            ExprKind::Block(block) => write!(indent, "Expr Block: {block}")?,
            ExprKind::Call(callable, arg) => display_call(indent, callable, arg)?,
            ExprKind::Conjugate(within, apply) => display_conjugate(indent, within, apply)?,
            ExprKind::Err => write!(indent, "Err")?,
            ExprKind::Fail(e) => write!(indent, "Fail: {e}")?,
            ExprKind::Field(expr, id) => display_field(indent, expr, id)?,
            ExprKind::For(iter, iterable, body) => display_for(indent, iter, iterable, body)?,
            ExprKind::Hole => write!(indent, "Hole")?,
            ExprKind::If(cond, body, els) => display_if(indent, cond, body, els)?,
            ExprKind::Index(array, index) => display_index(indent, array, index)?,
            ExprKind::Interpolate(components) => display_interpolate(indent, components)?,
            ExprKind::Lambda(kind, param, expr) => display_lambda(indent, *kind, param, expr)?,
            ExprKind::Lit(lit) => write!(indent, "Lit: {lit}")?,
            ExprKind::Paren(e) => write!(indent, "Paren: {e}")?,
            ExprKind::Path(p) => write!(indent, "Path: {p}")?,
            ExprKind::Range(start, step, end) => display_range(indent, start, step, end)?,
            ExprKind::Repeat(repeat, until, fixup) => display_repeat(indent, repeat, until, fixup)?,
            ExprKind::Return(e) => write!(indent, "Return: {e}")?,
            ExprKind::TernOp(op, expr1, expr2, expr3) => {
                display_tern_op(indent, *op, expr1, expr2, expr3)?;
            }
            ExprKind::Tuple(exprs) => display_tuple(indent, exprs)?,
            ExprKind::UnOp(op, expr) => display_un_op(indent, *op, expr)?,
            ExprKind::While(cond, block) => display_while(indent, cond, block)?,
        }
        Ok(())
    }
}

fn display_array(mut indent: Indented<Formatter>, exprs: &[Box<Expr>]) -> fmt::Result {
    write!(indent, "Array:")?;
    indent = set_indentation(indent, 1);
    for e in exprs {
        write!(indent, "\n{e}")?;
    }
    Ok(())
}

fn display_array_repeat(mut indent: Indented<Formatter>, val: &Expr, size: &Expr) -> fmt::Result {
    write!(indent, "ArrayRepeat:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{val}")?;
    write!(indent, "\n{size}")?;
    Ok(())
}

fn display_assign(mut indent: Indented<Formatter>, lhs: &Expr, rhs: &Expr) -> fmt::Result {
    write!(indent, "Assign:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{lhs}")?;
    write!(indent, "\n{rhs}")?;
    Ok(())
}

fn display_assign_op(
    mut indent: Indented<Formatter>,
    op: BinOp,
    lhs: &Expr,
    rhs: &Expr,
) -> fmt::Result {
    write!(indent, "AssignOp ({op:?}):")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{lhs}")?;
    write!(indent, "\n{rhs}")?;
    Ok(())
}

fn display_assign_update(
    mut indent: Indented<Formatter>,
    container: &Expr,
    item: &Expr,
    val: &Expr,
) -> fmt::Result {
    write!(indent, "AssignUpdate:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{container}")?;
    write!(indent, "\n{item}")?;
    write!(indent, "\n{val}")?;
    Ok(())
}

fn display_bin_op(
    mut indent: Indented<Formatter>,
    op: BinOp,
    lhs: &Expr,
    rhs: &Expr,
) -> fmt::Result {
    write!(indent, "BinOp ({op:?}):")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{lhs}")?;
    write!(indent, "\n{rhs}")?;
    Ok(())
}

fn display_call(mut indent: Indented<Formatter>, callable: &Expr, arg: &Expr) -> fmt::Result {
    write!(indent, "Call:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{callable}")?;
    write!(indent, "\n{arg}")?;
    Ok(())
}

fn display_conjugate(
    mut indent: Indented<Formatter>,
    within: &Block,
    apply: &Block,
) -> fmt::Result {
    write!(indent, "Conjugate:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{within}")?;
    write!(indent, "\n{apply}")?;
    Ok(())
}

fn display_field(mut indent: Indented<Formatter>, expr: &Expr, id: &Ident) -> fmt::Result {
    write!(indent, "Field:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{expr}")?;
    write!(indent, "\n{id}")?;
    Ok(())
}

fn display_for(
    mut indent: Indented<Formatter>,
    iter: &Pat,
    iterable: &Expr,
    body: &Block,
) -> fmt::Result {
    write!(indent, "For:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{iter}")?;
    write!(indent, "\n{iterable}")?;
    write!(indent, "\n{body}")?;
    Ok(())
}

fn display_if(
    mut indent: Indented<Formatter>,
    cond: &Expr,
    body: &Block,
    els: &Option<Box<Expr>>,
) -> fmt::Result {
    write!(indent, "If:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{cond}")?;
    write!(indent, "\n{body}")?;
    if let Some(e) = els {
        write!(indent, "\n{e}")?;
    }
    Ok(())
}

fn display_index(mut indent: Indented<Formatter>, array: &Expr, index: &Expr) -> fmt::Result {
    write!(indent, "Index:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{array}")?;
    write!(indent, "\n{index}")?;
    Ok(())
}

fn display_interpolate(
    mut indent: Indented<Formatter>,
    components: &[StringComponent],
) -> fmt::Result {
    write!(indent, "Interpolate:")?;
    indent = set_indentation(indent, 1);
    for component in components {
        match component {
            StringComponent::Expr(expr) => write!(indent, "\nExpr: {expr}")?,
            StringComponent::Lit(str) => write!(indent, "\nLit: {str:?}")?,
        }
    }

    Ok(())
}

fn display_lambda(
    mut indent: Indented<Formatter>,
    kind: CallableKind,
    param: &Pat,
    expr: &Expr,
) -> fmt::Result {
    write!(indent, "Lambda ({kind:?}):")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{param}")?;
    write!(indent, "\n{expr}")?;
    Ok(())
}

fn display_range(
    mut indent: Indented<Formatter>,
    start: &Option<Box<Expr>>,
    step: &Option<Box<Expr>>,
    end: &Option<Box<Expr>>,
) -> fmt::Result {
    write!(indent, "Range:")?;
    indent = set_indentation(indent, 1);
    match start {
        Some(e) => write!(indent, "\n{e}")?,
        None => write!(indent, "\n<no start>")?,
    }
    match step {
        Some(e) => write!(indent, "\n{e}")?,
        None => write!(indent, "\n<no step>")?,
    }
    match end {
        Some(e) => write!(indent, "\n{e}")?,
        None => write!(indent, "\n<no end>")?,
    }
    Ok(())
}

fn display_repeat(
    mut indent: Indented<Formatter>,
    repeat: &Block,
    until: &Expr,
    fixup: &Option<Box<Block>>,
) -> fmt::Result {
    write!(indent, "Repeat:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{repeat}")?;
    write!(indent, "\n{until}")?;
    match fixup {
        Some(b) => write!(indent, "\n{b}")?,
        None => write!(indent, "\n<no fixup>")?,
    }
    Ok(())
}

fn display_tern_op(
    mut indent: Indented<Formatter>,
    op: TernOp,
    expr1: &Expr,
    expr2: &Expr,
    expr3: &Expr,
) -> fmt::Result {
    write!(indent, "TernOp ({op:?}):")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{expr1}")?;
    write!(indent, "\n{expr2}")?;
    write!(indent, "\n{expr3}")?;
    Ok(())
}

fn display_tuple(mut indent: Indented<Formatter>, exprs: &[Box<Expr>]) -> fmt::Result {
    if exprs.is_empty() {
        write!(indent, "Unit")?;
    } else {
        write!(indent, "Tuple:")?;
        indent = set_indentation(indent, 1);
        for e in exprs {
            write!(indent, "\n{e}")?;
        }
    }
    Ok(())
}

fn display_un_op(mut indent: Indented<Formatter>, op: UnOp, expr: &Expr) -> fmt::Result {
    write!(indent, "UnOp ({op}):")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{expr}")?;
    Ok(())
}

fn display_while(mut indent: Indented<Formatter>, cond: &Expr, block: &Block) -> fmt::Result {
    write!(indent, "While:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{cond}")?;
    write!(indent, "\n{block}")?;
    Ok(())
}

/// An interpolated string component.
#[derive(Clone, Debug, PartialEq)]
pub enum StringComponent {
    /// An expression.
    Expr(Box<Expr>),
    /// A string literal.
    Lit(Rc<str>),
}

/// A pattern.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Pat {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The pattern kind.
    pub kind: Box<PatKind>,
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
    Bind(Box<Ident>, Option<Box<Ty>>),
    /// A discarded binding, `_`, with an optional type annotation.
    Discard(Option<Box<Ty>>),
    /// An elided pattern, `...`, used by specializations.
    Elided,
    /// Parentheses: `(a)`.
    Paren(Box<Pat>),
    /// A tuple: `(a, b, c)`.
    Tuple(Box<[Box<Pat>]>),
}

impl Display for PatKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        match self {
            PatKind::Bind(id, ty) => {
                write!(indent, "Bind:")?;
                indent = set_indentation(indent, 1);
                write!(indent, "\n{id}")?;
                if let Some(t) = ty {
                    write!(indent, "\n{t}")?;
                }
            }
            PatKind::Discard(d) => match d {
                Some(t) => {
                    write!(indent, "Discard:")?;
                    indent = set_indentation(indent, 1);
                    write!(indent, "\n{t}")?;
                }
                None => write!(indent, "Discard")?,
            },
            PatKind::Elided => write!(indent, "Elided")?,
            PatKind::Paren(p) => {
                write!(indent, "Paren:")?;
                indent = set_indentation(indent, 1);
                write!(indent, "\n{p}")?;
            }
            PatKind::Tuple(ps) => {
                if ps.is_empty() {
                    write!(indent, "Unit")?;
                } else {
                    write!(indent, "Tuple:")?;
                    indent = set_indentation(indent, 1);
                    for p in ps.iter() {
                        write!(indent, "\n{p}")?;
                    }
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
    pub kind: Box<QubitInitKind>,
}

impl Display for QubitInit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "QubitInit {} {} {}", self.id, self.span, self.kind)
    }
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
    Tuple(Box<[Box<QubitInit>]>),
}

impl Display for QubitInitKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        match self {
            QubitInitKind::Array(e) => {
                write!(indent, "Array:")?;
                indent = set_indentation(indent, 1);
                write!(indent, "\n{e}")?;
            }
            QubitInitKind::Paren(qi) => {
                write!(indent, "Parens:")?;
                indent = set_indentation(indent, 1);
                write!(indent, "\n{qi}")?;
            }
            QubitInitKind::Single => write!(indent, "Single")?,
            QubitInitKind::Tuple(qis) => {
                if qis.is_empty() {
                    write!(indent, "Unit")?;
                } else {
                    write!(indent, "Tuple:")?;
                    indent = set_indentation(indent, 1);
                    for qi in qis.iter() {
                        write!(indent, "\n{qi}")?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// A path to a declaration.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Path {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The namespace.
    pub namespace: Option<Box<Ident>>,
    /// The declaration name.
    pub name: Box<Ident>,
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
    pub name: Rc<str>,
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

/// A literal.
#[derive(Clone, Debug, PartialEq)]
pub enum Lit {
    /// A big integer literal.
    BigInt(Box<BigInt>),
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
    String(Rc<str>),
}

impl Display for Lit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Lit::BigInt(val) => write!(f, "BigInt({val})")?,
            Lit::Bool(val) => write!(f, "Bool({val})")?,
            Lit::Double(val) => write!(f, "Double({val})")?,
            Lit::Int(val) => write!(f, "Int({val})")?,
            Lit::Pauli(val) => write!(f, "Pauli({val:?})")?,
            Lit::Result(val) => write!(f, "Result({val:?})")?,
            Lit::String(val) => write!(f, "String({val:?})")?,
        }
        Ok(())
    }
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

impl Display for UnOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UnOp::Functor(func) => write!(f, "Functor {func:?}")?,
            _ => fmt::Debug::fmt(self, f)?,
        }
        Ok(())
    }
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
