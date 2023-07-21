// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The high-level intermediate representation for Q#. HIR is lowered from the AST.

#![warn(missing_docs)]

use crate::ty::{Arrow, FunctorSet, FunctorSetValue, GenericArg, GenericParam, Scheme, Ty, Udt};
use indenter::{indented, Format, Indented};
use num_bigint::BigInt;
use qsc_data_structures::{index_map::IndexMap, span::Span};
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter, Write},
    hash::{Hash, Hasher},
    rc::Rc,
    result,
    str::FromStr,
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

/// A unique identifier for an HIR node.
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
            Display::fmt(&self.0, f)
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

/// A unique identifier for a package within a package store.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackageId(usize);

impl PackageId {
    /// The package ID of the core library.
    pub const CORE: Self = Self(0);

    /// The successor of this ID.
    #[must_use]
    pub fn successor(self) -> Self {
        Self(self.0 + 1)
    }
}

impl Display for PackageId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl From<PackageId> for usize {
    fn from(value: PackageId) -> Self {
        value.0
    }
}

impl From<usize> for PackageId {
    fn from(value: usize) -> Self {
        PackageId(value)
    }
}

/// A unique identifier for an item within a package.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LocalItemId(usize);

impl LocalItemId {
    /// The successor of this ID.
    #[must_use]
    pub fn successor(self) -> Self {
        Self(self.0 + 1)
    }
}

impl Display for LocalItemId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl From<usize> for LocalItemId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<LocalItemId> for usize {
    fn from(value: LocalItemId) -> Self {
        value.0
    }
}

/// A unique identifier for an item within a package store.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ItemId {
    /// The package ID or `None` for the local package.
    pub package: Option<PackageId>,
    /// The item ID.
    pub item: LocalItemId,
}

impl Display for ItemId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.package {
            None => write!(f, "Item {}", self.item),
            Some(package) => write!(f, "Item {} (Package {package})", self.item),
        }
    }
}

/// A resolution. This connects a usage of a name with the declaration of that name by uniquely
/// identifying the node that declared it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Res {
    /// An invalid resolution.
    Err,
    /// A global item.
    Item(ItemId),
    /// A local variable.
    Local(NodeId),
}

impl Display for Res {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Res::Err => f.write_str("Err"),
            Res::Item(item) => Display::fmt(item, f),
            Res::Local(node) => write!(f, "Local {node}"),
        }
    }
}

/// The root node of the HIR.
#[derive(Clone, Debug, Default)]
pub struct Package {
    /// The items in the package.
    pub items: IndexMap<LocalItemId, Item>,
    /// The entry expression for an executable package.
    pub entry: Option<Expr>,
}

impl Display for Package {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Package:")?;
        indent = set_indentation(indent, 1);
        if let Some(e) = &self.entry {
            write!(indent, "\nentry expression: {e}")?;
        }
        for item in self.items.values() {
            write!(indent, "\n{item}")?;
        }
        Ok(())
    }
}

/// An item.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    /// The ID.
    pub id: LocalItemId,
    /// The span.
    pub span: Span,
    /// The parent item.
    pub parent: Option<LocalItemId>,
    /// The documentation.
    pub doc: Rc<str>,
    /// The attributes.
    pub attrs: Vec<Attr>,
    /// The visibility.
    pub visibility: Visibility,
    /// The item kind.
    pub kind: ItemKind,
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(
            indent,
            "Item {} {} ({:?}):",
            self.id, self.span, self.visibility
        )?;

        indent = set_indentation(indent, 1);
        if let Some(parent) = self.parent {
            write!(indent, "\nParent: {parent}")?;
        }

        if !self.doc.is_empty() {
            write!(indent, "\nDoc:")?;
            indent = set_indentation(indent, 2);
            write!(indent, "\n{}", self.doc)?;
            indent = set_indentation(indent, 1);
        }

        for attr in &self.attrs {
            write!(indent, "\n{attr:?}")?;
        }

        write!(indent, "\n{}", self.kind)?;
        Ok(())
    }
}

/// An item kind.
#[derive(Clone, Debug, PartialEq)]
pub enum ItemKind {
    /// A `function` or `operation` declaration.
    Callable(CallableDecl),
    /// A `namespace` declaration.
    Namespace(Ident, Vec<LocalItemId>),
    /// A `newtype` declaration.
    Ty(Ident, Udt),
}

impl Display for ItemKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ItemKind::Callable(decl) => write!(f, "{decl}"),
            ItemKind::Namespace(name, items) => {
                write!(f, "Namespace ({name}):")?;
                let mut items = items.iter();
                if let Some(item) = items.next() {
                    write!(f, " Item {item}")?;
                    for item in items {
                        write!(f, ", Item {item}")?;
                    }
                    Ok(())
                } else {
                    write!(f, " <empty>")
                }
            }
            ItemKind::Ty(name, udt) => write!(f, "Type ({name}): {udt}"),
        }
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
    /// The generic parameters to the callable.
    pub generics: Vec<GenericParam>,
    /// The input to the callable.
    pub input: Pat,
    /// The return type of the callable.
    pub output: Ty,
    /// The functors supported by the callable.
    pub functors: FunctorSetValue,
    /// The callable body.
    pub body: SpecDecl,
    /// The adjoint specialization.
    pub adj: Option<SpecDecl>,
    /// The controlled specialization.
    pub ctl: Option<SpecDecl>,
    /// The controlled adjoint specialization.
    pub ctl_adj: Option<SpecDecl>,
}

impl CallableDecl {
    /// The type scheme of the callable.
    #[must_use]
    pub fn scheme(&self) -> Scheme {
        Scheme::new(
            self.generics.clone(),
            Box::new(Arrow {
                kind: self.kind,
                input: Box::new(self.input.ty.clone()),
                output: Box::new(self.output.clone()),
                functors: FunctorSet::Value(self.functors),
            }),
        )
    }
}

impl Display for CallableDecl {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(
            indent,
            "Callable {} {} ({}):",
            self.id, self.span, self.kind
        )?;
        indent = set_indentation(indent, 1);
        write!(indent, "\nname: {}", self.name)?;
        if !self.generics.is_empty() {
            write!(indent, "\ngenerics:")?;
            indent = set_indentation(indent, 2);
            for (ix, param) in self.generics.iter().enumerate() {
                write!(indent, "\n{ix}: {param}")?;
            }
            indent = set_indentation(indent, 1);
        }
        write!(indent, "\ninput: {}", self.input)?;
        write!(indent, "\noutput: {}", self.output)?;
        write!(indent, "\nfunctors: {}", self.functors)?;
        write!(indent, "\nbody: {}", self.body)?;
        match &self.adj {
            Some(spec) => write!(indent, "\nadj: {spec}")?,
            None => write!(indent, "\nadj: <none>")?,
        }
        match &self.ctl {
            Some(spec) => write!(indent, "\nctl: {spec}")?,
            None => write!(indent, "\nctl: <none>")?,
        }
        match &self.ctl_adj {
            Some(spec) => write!(indent, "\nctl-adj: {spec}")?,
            None => write!(indent, "\nctl-adj: <none>")?,
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
    /// The body of the specialization.
    pub body: SpecBody,
}

impl Display for SpecDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "SpecDecl {} {}: {}", self.id, self.span, self.body)
    }
}

/// The body of a specialization.
#[derive(Clone, Debug, PartialEq)]
pub enum SpecBody {
    /// The strategy to use to automatically generate the specialization.
    Gen(SpecGen),
    /// A manual implementation of the specialization.
    Impl(Option<Pat>, Block),
}

impl Display for SpecBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        match self {
            SpecBody::Gen(sg) => write!(indent, "Gen: {sg:?}")?,
            SpecBody::Impl(p, b) => {
                write!(indent, "Impl:")?;
                indent = set_indentation(indent, 1);
                if let Some(p) = p {
                    write!(indent, "\n{p}")?;
                }
                write!(indent, "\n{b}")?;
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
    /// The block type.
    pub ty: Ty,
    /// The statements in the block.
    pub stmts: Vec<Stmt>,
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.stmts.is_empty() {
            write!(f, "Block {} {}: <empty>", self.id, self.span)?;
        } else {
            let mut indent = set_indentation(indented(f), 0);
            write!(
                indent,
                "Block {} {} [Type {}]:",
                self.id, self.span, self.ty
            )?;
            indent = set_indentation(indent, 1);
            for s in &self.stmts {
                write!(indent, "\n{s}")?;
            }
        }
        Ok(())
    }
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

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Stmt {} {}: {}", self.id, self.span, self.kind)
    }
}

/// A statement kind.
#[derive(Clone, Debug, PartialEq)]
pub enum StmtKind {
    /// An expression without a trailing semicolon.
    Expr(Expr),
    /// An item.
    Item(LocalItemId),
    /// A let or mutable binding: `let a = b;` or `mutable x = b;`.
    Local(Mutability, Pat, Expr),
    /// A use or borrow qubit allocation: `use a = b;` or `borrow a = b;`.
    Qubit(QubitSource, Pat, QubitInit, Option<Block>),
    /// An expression with a trailing semicolon.
    Semi(Expr),
}

impl Display for StmtKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        match self {
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
    /// The expression type.
    pub ty: Ty,
    /// The expression kind.
    pub kind: ExprKind,
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Expr {} {} [Type {}]: {}",
            self.id, self.span, self.ty, self.kind
        )
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
    /// An assignment with a compound field update operator: `set a w/= B <- c`.
    AssignField(Box<Expr>, Field, Box<Expr>),
    /// An assignment with a compound index update operator: `set a w/= b <- c`.
    AssignIndex(Box<Expr>, Box<Expr>, Box<Expr>),
    /// A binary operator.
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    /// A block: `{ ... }`.
    Block(Block),
    /// A call: `a(b)`.
    Call(Box<Expr>, Box<Expr>),
    /// A closure that fixes the vector of local variables as arguments to the callable item.
    Closure(Vec<NodeId>, LocalItemId),
    /// A conjugation: `within { ... } apply { ... }`.
    Conjugate(Block, Block),
    /// A failure: `fail "message"`.
    Fail(Box<Expr>),
    /// A field accessor: `a::F`.
    Field(Box<Expr>, Field),
    /// A for loop: `for a in b { ... }`.
    For(Pat, Box<Expr>, Block),
    /// An unspecified expression, _, which may indicate partial application or a typed hole.
    Hole,
    /// An if expression with an optional else block: `if a { ... } else { ... }`.
    ///
    /// Note that, as a special case, `elif ...` is effectively parsed as `else if ...`, without a
    /// block wrapping the `if`. This distinguishes `elif ...` from `else { if ... }`, which does
    /// have a block.
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    /// An index accessor: `a[b]`.
    Index(Box<Expr>, Box<Expr>),
    /// A literal.
    Lit(Lit),
    /// A range: `start..step..end`, `start..end`, `start...`, `...end`, or `...`.
    Range(Option<Box<Expr>>, Option<Box<Expr>>, Option<Box<Expr>>),
    /// A repeat-until loop with an optional fixup: `repeat { ... } until a fixup { ... }`.
    Repeat(Block, Box<Expr>, Option<Block>),
    /// A return: `return a`.
    Return(Box<Expr>),
    /// A string.
    String(Vec<StringComponent>),
    /// Update array index: `a w/ b <- c`.
    UpdateIndex(Box<Expr>, Box<Expr>, Box<Expr>),
    /// A tuple: `(a, b, c)`.
    Tuple(Vec<Expr>),
    /// A unary operator.
    UnOp(UnOp, Box<Expr>),
    /// A record field update: `a w/ B <- c`.
    UpdateField(Box<Expr>, Field, Box<Expr>),
    /// A variable and its generic arguments.
    Var(Res, Vec<GenericArg>),
    /// A while loop: `while a { ... }`.
    While(Box<Expr>, Block),
    /// An invalid expression.
    #[default]
    Err,
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        match self {
            ExprKind::Array(exprs) => display_array(indent, exprs)?,
            ExprKind::ArrayRepeat(val, size) => display_array_repeat(indent, val, size)?,
            ExprKind::Assign(lhs, rhs) => display_assign(indent, lhs, rhs)?,
            ExprKind::AssignOp(op, lhs, rhs) => display_assign_op(indent, *op, lhs, rhs)?,
            ExprKind::AssignField(record, field, replace) => {
                display_assign_field(indent, record, field, replace)?;
            }
            ExprKind::AssignIndex(container, item, replace) => {
                display_assign_index(indent, container, item, replace)?;
            }
            ExprKind::BinOp(op, lhs, rhs) => display_bin_op(indent, *op, lhs, rhs)?,
            ExprKind::Block(block) => write!(indent, "Expr Block: {block}")?,
            ExprKind::Call(callable, arg) => display_call(indent, callable, arg)?,
            ExprKind::Closure(args, callable) => display_closure(indent, args, *callable)?,
            ExprKind::Conjugate(within, apply) => display_conjugate(indent, within, apply)?,
            ExprKind::Err => write!(indent, "Err")?,
            ExprKind::Fail(e) => write!(indent, "Fail: {e}")?,
            ExprKind::Field(expr, field) => display_field(indent, expr, field)?,
            ExprKind::For(iter, iterable, body) => display_for(indent, iter, iterable, body)?,
            ExprKind::Hole => write!(indent, "Hole")?,
            ExprKind::If(cond, body, els) => display_if(indent, cond, body, els)?,
            ExprKind::Index(array, index) => display_index(indent, array, index)?,
            ExprKind::Lit(lit) => write!(indent, "Lit: {lit}")?,
            ExprKind::Range(start, step, end) => display_range(indent, start, step, end)?,
            ExprKind::Repeat(repeat, until, fixup) => display_repeat(indent, repeat, until, fixup)?,
            ExprKind::Return(e) => write!(indent, "Return: {e}")?,
            ExprKind::String(components) => display_string(indent, components)?,
            ExprKind::UpdateIndex(expr1, expr2, expr3) => {
                display_update_index(indent, expr1, expr2, expr3)?;
            }
            ExprKind::Tuple(exprs) => display_tuple(indent, exprs)?,
            ExprKind::UnOp(op, expr) => display_un_op(indent, *op, expr)?,
            ExprKind::UpdateField(record, field, replace) => {
                display_update_field(indent, record, field, replace)?;
            }
            ExprKind::Var(res, args) => display_var(indent, *res, args)?,
            ExprKind::While(cond, block) => display_while(indent, cond, block)?,
        }
        Ok(())
    }
}

fn display_array(mut indent: Indented<Formatter>, exprs: &Vec<Expr>) -> fmt::Result {
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

fn display_assign_field(
    mut indent: Indented<Formatter>,
    record: &Expr,
    field: &Field,
    replace: &Expr,
) -> fmt::Result {
    write!(indent, "AssignField:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{record}")?;
    write!(indent, "\n{field}")?;
    write!(indent, "\n{replace}")?;
    Ok(())
}

fn display_assign_index(
    mut indent: Indented<Formatter>,
    array: &Expr,
    index: &Expr,
    replace: &Expr,
) -> fmt::Result {
    write!(indent, "AssignIndex:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{array}")?;
    write!(indent, "\n{index}")?;
    write!(indent, "\n{replace}")?;
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

fn display_closure(
    mut f: Indented<Formatter>,
    args: &[NodeId],
    callable: LocalItemId,
) -> fmt::Result {
    f.write_str("Closure([")?;
    let mut args = args.iter();
    if let Some(arg) = args.next() {
        write!(f, "{arg}")?;
    }
    for arg in args {
        write!(f, ", {arg}")?;
    }
    write!(f, "], {callable})")
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

fn display_field(mut indent: Indented<Formatter>, expr: &Expr, field: &Field) -> fmt::Result {
    write!(indent, "Field:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{expr}")?;
    write!(indent, "\n{field:?}")?;
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
    body: &Expr,
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
    fixup: &Option<Block>,
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

fn display_string(mut indent: Indented<Formatter>, components: &[StringComponent]) -> fmt::Result {
    write!(indent, "String:")?;
    indent = set_indentation(indent, 1);
    for component in components {
        match component {
            StringComponent::Expr(expr) => write!(indent, "\nExpr: {expr}")?,
            StringComponent::Lit(str) => write!(indent, "\nLit: {str:?}")?,
        }
    }

    Ok(())
}

fn display_update_index(
    mut indent: Indented<Formatter>,
    expr1: &Expr,
    expr2: &Expr,
    expr3: &Expr,
) -> fmt::Result {
    write!(indent, "UpdateIndex:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{expr1}")?;
    write!(indent, "\n{expr2}")?;
    write!(indent, "\n{expr3}")?;
    Ok(())
}

fn display_tuple(mut indent: Indented<Formatter>, exprs: &Vec<Expr>) -> fmt::Result {
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

fn display_update_field(
    mut indent: Indented<Formatter>,
    record: &Expr,
    field: &Field,
    replace: &Expr,
) -> fmt::Result {
    write!(indent, "UpdateField:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{record}")?;
    write!(indent, "\n{field}")?;
    write!(indent, "\n{replace}")?;
    Ok(())
}

fn display_var(mut f: Indented<Formatter>, res: Res, args: &[GenericArg]) -> fmt::Result {
    if args.is_empty() {
        write!(f, "Var: {res}")
    } else {
        write!(f, "Var:")?;
        f = set_indentation(f, 1);
        write!(f, "\nres: {res}")?;
        write!(f, "\ngenerics:")?;
        f = set_indentation(f, 2);
        for arg in args {
            write!(f, "\n{arg}")?;
        }
        Ok(())
    }
}

fn display_while(mut indent: Indented<Formatter>, cond: &Expr, block: &Block) -> fmt::Result {
    write!(indent, "While:")?;
    indent = set_indentation(indent, 1);
    write!(indent, "\n{cond}")?;
    write!(indent, "\n{block}")?;
    Ok(())
}

/// A string component.
#[derive(Clone, Debug, PartialEq)]
pub enum StringComponent {
    /// An expression.
    Expr(Expr),
    /// A string literal.
    Lit(Rc<str>),
}

/// A pattern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pat {
    /// The node ID.
    pub id: NodeId,
    /// The span.
    pub span: Span,
    /// The pattern type.
    pub ty: Ty,
    /// The pattern kind.
    pub kind: PatKind,
}

impl Display for Pat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Pat {} {} [Type {}]: {}",
            self.id, self.span, self.ty, self.kind
        )
    }
}

/// A pattern kind.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatKind {
    /// A binding.
    Bind(Ident),
    /// A discarded binding, `_`.
    Discard,
    /// A tuple: `(a, b, c)`.
    Tuple(Vec<Pat>),
}

impl Display for PatKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        match self {
            PatKind::Bind(id) => {
                write!(indent, "Bind: {id}")?;
            }
            PatKind::Discard => write!(indent, "Discard")?,
            PatKind::Tuple(ps) => {
                if ps.is_empty() {
                    write!(indent, "Unit")?;
                } else {
                    write!(indent, "Tuple:")?;
                    indent = set_indentation(indent, 1);
                    for p in ps {
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
    /// The qubit initializer type.
    pub ty: Ty,
    /// The qubit initializer kind.
    pub kind: QubitInitKind,
}

impl Display for QubitInit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "QubitInit {} {} [Type {}]: {}",
            self.id, self.span, self.ty, self.kind
        )
    }
}

/// A qubit initializer kind.
#[derive(Clone, Debug, PartialEq)]
pub enum QubitInitKind {
    /// An array of qubits: `Qubit[a]`.
    Array(Box<Expr>),
    /// A single qubit: `Qubit()`.
    Single,
    /// A tuple: `(a, b, c)`.
    Tuple(Vec<QubitInit>),
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
            QubitInitKind::Single => write!(indent, "Single")?,
            QubitInitKind::Tuple(qis) => {
                if qis.is_empty() {
                    write!(indent, "Unit")?;
                } else {
                    write!(indent, "Tuple:")?;
                    indent = set_indentation(indent, 1);
                    for qi in qis {
                        write!(indent, "\n{qi}")?;
                    }
                }
            }
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

/// An attribute.
#[derive(Clone, Debug, PartialEq)]
pub enum Attr {
    /// Indicates that a callable is an entry point to a program.
    EntryPoint,
}

/// A field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Field {
    /// A field path.
    Path(FieldPath),
    /// A primitive field for a built-in type.
    Prim(PrimField),
    /// An invalid field.
    Err,
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Field::Path(path) => write!(f, "Path({:?})", path.indices),
            Field::Prim(prim) => write!(f, "Prim({prim:?}"),
            Field::Err => f.write_str("Err"),
        }
    }
}

/// A path to a field in a tuple or user-defined type.
#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct FieldPath {
    /// The tuple item indices to follow in order from top to bottom.
    pub indices: Vec<usize>,
}

/// A primitive field for a built-in type.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PrimField {
    /// The start of a range.
    Start,
    /// The step of a range.
    Step,
    /// The end of a range.
    End,
}

impl FromStr for PrimField {
    type Err = ();

    fn from_str(s: &str) -> result::Result<Self, <Self as FromStr>::Err> {
        match s {
            "Start" => Ok(Self::Start),
            "Step" => Ok(Self::Step),
            "End" => Ok(Self::End),
            _ => Err(()),
        }
    }
}

/// The visibility of a declaration.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Visibility {
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

impl Display for CallableKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            CallableKind::Function => f.write_str("function"),
            CallableKind::Operation => f.write_str("operation"),
        }
    }
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

impl Display for Functor {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Functor::Adj => f.write_str("Adj"),
            Functor::Ctl => f.write_str("Ctl"),
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
