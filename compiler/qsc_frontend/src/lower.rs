// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::{
    closure::{self, Lambda, PartialApp},
    resolve::{self, Names},
    typeck::{self, convert},
};
use miette::Diagnostic;
use qsc_ast::ast;
use qsc_data_structures::{index_map::IndexMap, span::Span};
use qsc_hir::{
    assigner::Assigner,
    hir::{self, LocalItemId},
    mut_visit::MutVisitor,
};
use std::{clone::Clone, rc::Rc, vec};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub(super) enum Error {
    #[error("unknown attribute {0}")]
    #[diagnostic(help("supported attributes are: EntryPoint"))]
    UnknownAttr(String, #[label] Span),
    #[error("invalid attribute arguments: expected {0}")]
    InvalidAttrArgs(&'static str, #[label] Span),
    #[error("lambda closes over mutable variable")]
    MutableClosure(#[label] Span),
}

pub(super) struct Local {
    pub(super) mutability: hir::Mutability,
    pub(super) ty: hir::Ty,
}

#[derive(Clone, Copy)]
enum ItemScope {
    Global,
    Local,
}

pub(super) struct Lowerer {
    nodes: IndexMap<ast::NodeId, hir::NodeId>,
    locals: IndexMap<hir::NodeId, Local>,
    parent: Option<LocalItemId>,
    items: Vec<hir::Item>,
    errors: Vec<Error>,
}

impl Lowerer {
    pub(super) fn new() -> Self {
        Self {
            nodes: IndexMap::new(),
            locals: IndexMap::new(),
            parent: None,
            items: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub(super) fn drain_items(&mut self) -> vec::Drain<hir::Item> {
        self.items.drain(..)
    }

    pub(super) fn drain_errors(&mut self) -> vec::Drain<Error> {
        self.errors.drain(..)
    }

    pub(super) fn with<'a>(
        &'a mut self,
        assigner: &'a mut Assigner,
        names: &'a Names,
        tys: &'a typeck::Table,
    ) -> With {
        With {
            lowerer: self,
            assigner,
            names,
            tys,
        }
    }
}

pub(super) struct With<'a> {
    lowerer: &'a mut Lowerer,
    assigner: &'a mut Assigner,
    names: &'a Names,
    tys: &'a typeck::Table,
}

impl With<'_> {
    pub(super) fn lower_package(&mut self, package: &ast::Package) -> hir::Package {
        for namespace in package.namespaces.iter() {
            self.lower_namespace(namespace);
        }

        let entry = package.entry.as_ref().map(|e| self.lower_expr(e));
        let items = self.lowerer.items.drain(..).map(|i| (i.id, i)).collect();
        hir::Package { items, entry }
    }

    pub(super) fn lower_namespace(&mut self, namespace: &ast::Namespace) {
        let Some(&resolve::Res::Item(hir::ItemId {
            item: id, ..
        })) = self.names.get(namespace.name.id) else {
            panic!("namespace should have item ID");
        };

        self.lowerer.parent = Some(id);
        let items = namespace
            .items
            .iter()
            .filter_map(|i| self.lower_item(ItemScope::Global, i))
            .collect();

        let name = self.lower_ident(&namespace.name);
        self.lowerer.items.push(hir::Item {
            id,
            span: namespace.span,
            parent: None,
            attrs: Vec::new(),
            visibility: hir::Visibility::Public,
            kind: hir::ItemKind::Namespace(name, items),
        });

        self.lowerer.parent = None;
    }

    fn lower_item(&mut self, scope: ItemScope, item: &ast::Item) -> Option<LocalItemId> {
        let attrs = item
            .attrs
            .iter()
            .filter_map(|a| self.lower_attr(a))
            .collect();

        let visibility = match scope {
            ItemScope::Global => item
                .visibility
                .as_ref()
                .map_or(hir::Visibility::Public, lower_visibility),
            ItemScope::Local => hir::Visibility::Internal,
        };

        let resolve_id = |id| match self.names.get(id) {
            Some(&resolve::Res::Item(hir::ItemId { item, .. })) => item,
            _ => panic!("item should have item ID"),
        };

        let (id, kind) = match &*item.kind {
            ast::ItemKind::Err | ast::ItemKind::Open(..) => return None,
            ast::ItemKind::Callable(callable) => {
                let id = resolve_id(callable.name.id);
                let grandparent = self.lowerer.parent;
                self.lowerer.parent = Some(id);
                let callable = self.lower_callable_decl(callable);
                self.lowerer.parent = grandparent;
                (id, hir::ItemKind::Callable(callable))
            }
            ast::ItemKind::Ty(name, _) => {
                let id = resolve_id(name.id);
                let udt = self
                    .tys
                    .udts
                    .get(&hir::ItemId {
                        package: None,
                        item: id,
                    })
                    .expect("type item should have lowered UDT");

                (id, hir::ItemKind::Ty(self.lower_ident(name), udt.clone()))
            }
        };

        self.lowerer.items.push(hir::Item {
            id,
            span: item.span,
            parent: self.lowerer.parent,
            attrs,
            visibility,
            kind,
        });

        Some(id)
    }

    fn lower_attr(&mut self, attr: &ast::Attr) -> Option<hir::Attr> {
        if attr.name.name.as_ref() == "EntryPoint" {
            match &*attr.arg.kind {
                ast::ExprKind::Tuple(args) if args.is_empty() => Some(hir::Attr::EntryPoint),
                _ => {
                    self.lowerer
                        .errors
                        .push(Error::InvalidAttrArgs("()", attr.arg.span));
                    None
                }
            }
        } else {
            self.lowerer.errors.push(Error::UnknownAttr(
                attr.name.name.to_string(),
                attr.name.span,
            ));
            None
        }
    }

    pub(super) fn lower_callable_decl(&mut self, decl: &ast::CallableDecl) -> hir::CallableDecl {
        hir::CallableDecl {
            id: self.lower_id(decl.id),
            span: decl.span,
            kind: lower_callable_kind(decl.kind),
            name: self.lower_ident(&decl.name),
            ty_params: decl.ty_params.iter().map(|p| self.lower_ident(p)).collect(),
            input: self.lower_pat(ast::Mutability::Immutable, &decl.input),
            output: convert::ty_from_ast(self.names, &decl.output).0,
            functors: convert::ast_callable_functors(decl),
            body: match decl.body.as_ref() {
                ast::CallableBody::Block(block) => {
                    hir::CallableBody::Block(self.lower_block(block))
                }
                ast::CallableBody::Specs(specs) => hir::CallableBody::Specs(
                    specs.iter().map(|s| self.lower_spec_decl(s)).collect(),
                ),
            },
        }
    }

    fn lower_spec_decl(&mut self, decl: &ast::SpecDecl) -> hir::SpecDecl {
        hir::SpecDecl {
            id: self.lower_id(decl.id),
            span: decl.span,
            spec: match decl.spec {
                ast::Spec::Body => hir::Spec::Body,
                ast::Spec::Adj => hir::Spec::Adj,
                ast::Spec::Ctl => hir::Spec::Ctl,
                ast::Spec::CtlAdj => hir::Spec::CtlAdj,
            },
            body: match &decl.body {
                ast::SpecBody::Gen(gen) => hir::SpecBody::Gen(match gen {
                    ast::SpecGen::Auto => hir::SpecGen::Auto,
                    ast::SpecGen::Distribute => hir::SpecGen::Distribute,
                    ast::SpecGen::Intrinsic => hir::SpecGen::Intrinsic,
                    ast::SpecGen::Invert => hir::SpecGen::Invert,
                    ast::SpecGen::Slf => hir::SpecGen::Slf,
                }),
                ast::SpecBody::Impl(input, block) => hir::SpecBody::Impl(
                    self.lower_pat(ast::Mutability::Immutable, input),
                    self.lower_block(block),
                ),
            },
        }
    }

    fn lower_block(&mut self, block: &ast::Block) -> hir::Block {
        hir::Block {
            id: self.lower_id(block.id),
            span: block.span,
            ty: self
                .tys
                .terms
                .get(block.id)
                .map_or(hir::Ty::Err, Clone::clone),
            stmts: block
                .stmts
                .iter()
                .filter_map(|s| self.lower_stmt(s))
                .collect(),
        }
    }

    pub(super) fn lower_stmt(&mut self, stmt: &ast::Stmt) -> Option<hir::Stmt> {
        let id = self.lower_id(stmt.id);
        let kind = match &*stmt.kind {
            ast::StmtKind::Empty => return None,
            ast::StmtKind::Expr(expr) => hir::StmtKind::Expr(self.lower_expr(expr)),
            ast::StmtKind::Item(item) => {
                hir::StmtKind::Item(self.lower_item(ItemScope::Local, item)?)
            }
            ast::StmtKind::Local(mutability, lhs, rhs) => hir::StmtKind::Local(
                lower_mutability(*mutability),
                self.lower_pat(*mutability, lhs),
                self.lower_expr(rhs),
            ),
            ast::StmtKind::Qubit(source, lhs, rhs, block) => hir::StmtKind::Qubit(
                match source {
                    ast::QubitSource::Fresh => hir::QubitSource::Fresh,
                    ast::QubitSource::Dirty => hir::QubitSource::Dirty,
                },
                self.lower_pat(ast::Mutability::Immutable, lhs),
                self.lower_qubit_init(rhs),
                block.as_ref().map(|b| self.lower_block(b)),
            ),
            ast::StmtKind::Semi(expr) => hir::StmtKind::Semi(self.lower_expr(expr)),
        };

        Some(hir::Stmt {
            id,
            span: stmt.span,
            kind,
        })
    }

    #[allow(clippy::too_many_lines)]
    fn lower_expr(&mut self, expr: &ast::Expr) -> hir::Expr {
        if let ast::ExprKind::Paren(inner) = &*expr.kind {
            return self.lower_expr(inner);
        }

        let id = self.lower_id(expr.id);
        let ty = self
            .tys
            .terms
            .get(expr.id)
            .map_or(hir::Ty::Err, Clone::clone);

        let kind = match &*expr.kind {
            ast::ExprKind::Array(items) => {
                hir::ExprKind::Array(items.iter().map(|i| self.lower_expr(i)).collect())
            }
            ast::ExprKind::ArrayRepeat(value, size) => hir::ExprKind::ArrayRepeat(
                Box::new(self.lower_expr(value)),
                Box::new(self.lower_expr(size)),
            ),
            ast::ExprKind::Assign(lhs, rhs) => hir::ExprKind::Assign(
                Box::new(self.lower_expr(lhs)),
                Box::new(self.lower_expr(rhs)),
            ),
            ast::ExprKind::AssignOp(op, lhs, rhs) => hir::ExprKind::AssignOp(
                lower_binop(*op),
                Box::new(self.lower_expr(lhs)),
                Box::new(self.lower_expr(rhs)),
            ),
            ast::ExprKind::AssignUpdate(container, index, replace) => {
                if let Some(field) = resolve::extract_field_name(self.names, index) {
                    let container = self.lower_expr(container);
                    let field = self.lower_field(&container.ty, field);
                    let replace = self.lower_expr(replace);
                    hir::ExprKind::AssignField(Box::new(container), field, Box::new(replace))
                } else {
                    hir::ExprKind::AssignIndex(
                        Box::new(self.lower_expr(container)),
                        Box::new(self.lower_expr(index)),
                        Box::new(self.lower_expr(replace)),
                    )
                }
            }
            ast::ExprKind::BinOp(op, lhs, rhs) => hir::ExprKind::BinOp(
                lower_binop(*op),
                Box::new(self.lower_expr(lhs)),
                Box::new(self.lower_expr(rhs)),
            ),
            ast::ExprKind::Block(block) => hir::ExprKind::Block(self.lower_block(block)),
            ast::ExprKind::Call(callee, arg) if is_partial_app(arg) => {
                hir::ExprKind::Block(self.lower_partial_app(callee, arg, ty.clone(), expr.span))
            }
            ast::ExprKind::Call(callee, arg) => hir::ExprKind::Call(
                Box::new(self.lower_expr(callee)),
                Box::new(self.lower_expr(arg)),
            ),
            ast::ExprKind::Conjugate(within, apply) => {
                hir::ExprKind::Conjugate(self.lower_block(within), self.lower_block(apply))
            }
            ast::ExprKind::Err => hir::ExprKind::Err,
            ast::ExprKind::Fail(message) => hir::ExprKind::Fail(Box::new(self.lower_expr(message))),
            ast::ExprKind::Field(container, name) => {
                let container = self.lower_expr(container);
                let field = self.lower_field(&container.ty, &name.name);
                hir::ExprKind::Field(Box::new(container), field)
            }
            ast::ExprKind::For(pat, iter, block) => hir::ExprKind::For(
                self.lower_pat(ast::Mutability::Immutable, pat),
                Box::new(self.lower_expr(iter)),
                self.lower_block(block),
            ),
            ast::ExprKind::Hole => hir::ExprKind::Hole,
            ast::ExprKind::If(cond, if_true, if_false) => hir::ExprKind::If(
                Box::new(self.lower_expr(cond)),
                self.lower_block(if_true),
                if_false.as_ref().map(|e| Box::new(self.lower_expr(e))),
            ),
            ast::ExprKind::Index(container, index) => hir::ExprKind::Index(
                Box::new(self.lower_expr(container)),
                Box::new(self.lower_expr(index)),
            ),
            ast::ExprKind::Lambda(kind, input, body) => {
                let functors = if let hir::Ty::Arrow(_, _, _, functors) = ty {
                    functors
                } else {
                    hir::FunctorSet::Empty
                };
                let lambda = Lambda {
                    kind: lower_callable_kind(*kind),
                    functors,
                    input: self.lower_pat(ast::Mutability::Immutable, input),
                    body: self.lower_expr(body),
                };
                self.lower_lambda(lambda, expr.span)
            }
            ast::ExprKind::Lit(lit) => lower_lit(lit),
            ast::ExprKind::Paren(_) => unreachable!("parentheses should be removed earlier"),
            ast::ExprKind::Path(path) => hir::ExprKind::Var(self.lower_path(path)),
            ast::ExprKind::Range(start, step, end) => hir::ExprKind::Range(
                start.as_ref().map(|s| Box::new(self.lower_expr(s))),
                step.as_ref().map(|s| Box::new(self.lower_expr(s))),
                end.as_ref().map(|e| Box::new(self.lower_expr(e))),
            ),
            ast::ExprKind::Repeat(body, cond, fixup) => hir::ExprKind::Repeat(
                self.lower_block(body),
                Box::new(self.lower_expr(cond)),
                fixup.as_ref().map(|f| self.lower_block(f)),
            ),
            ast::ExprKind::Return(expr) => hir::ExprKind::Return(Box::new(self.lower_expr(expr))),
            ast::ExprKind::Interpolate(components) => hir::ExprKind::String(
                components
                    .iter()
                    .map(|c| self.lower_string_component(c))
                    .collect(),
            ),
            ast::ExprKind::TernOp(ast::TernOp::Cond, cond, if_true, if_false) => {
                hir::ExprKind::TernOp(
                    hir::TernOp::Cond,
                    Box::new(self.lower_expr(cond)),
                    Box::new(self.lower_expr(if_true)),
                    Box::new(self.lower_expr(if_false)),
                )
            }
            ast::ExprKind::TernOp(ast::TernOp::Update, container, index, replace) => {
                if let Some(field) = resolve::extract_field_name(self.names, index) {
                    let record = self.lower_expr(container);
                    let field = self.lower_field(&record.ty, field);
                    let replace = self.lower_expr(replace);
                    hir::ExprKind::UpdateField(Box::new(record), field, Box::new(replace))
                } else {
                    hir::ExprKind::TernOp(
                        hir::TernOp::UpdateIndex,
                        Box::new(self.lower_expr(container)),
                        Box::new(self.lower_expr(index)),
                        Box::new(self.lower_expr(replace)),
                    )
                }
            }
            ast::ExprKind::Tuple(items) => {
                hir::ExprKind::Tuple(items.iter().map(|i| self.lower_expr(i)).collect())
            }
            ast::ExprKind::UnOp(op, operand) => {
                hir::ExprKind::UnOp(lower_unop(*op), Box::new(self.lower_expr(operand)))
            }
            ast::ExprKind::While(cond, body) => {
                hir::ExprKind::While(Box::new(self.lower_expr(cond)), self.lower_block(body))
            }
        };

        hir::Expr {
            id,
            span: expr.span,
            ty,
            kind,
        }
    }

    fn lower_partial_app(
        &mut self,
        callee: &ast::Expr,
        arg: &ast::Expr,
        ty: hir::Ty,
        span: Span,
    ) -> hir::Block {
        let callee = self.lower_expr(callee);
        let (arg, app) = self.lower_partial_arg(arg);
        let close = |mut lambda: Lambda| {
            self.assigner.visit_expr(&mut lambda.body);
            self.lower_lambda(lambda, span)
        };

        let mut block = closure::partial_app_block(close, callee, arg, app, ty, span);
        self.assigner.visit_block(&mut block);
        block
    }

    fn lower_partial_arg(&mut self, arg: &ast::Expr) -> (hir::Expr, PartialApp) {
        match arg.kind.as_ref() {
            ast::ExprKind::Hole => {
                let ty = self
                    .tys
                    .terms
                    .get(arg.id)
                    .map_or(hir::Ty::Err, Clone::clone);
                closure::partial_app_hole(self.assigner, &mut self.lowerer.locals, ty, arg.span)
            }
            ast::ExprKind::Paren(inner) => self.lower_partial_arg(inner),
            ast::ExprKind::Tuple(items) => {
                let items = items.iter().map(|item| self.lower_partial_arg(item));
                let (mut arg, mut app) = closure::partial_app_tuple(items, arg.span);
                self.assigner.visit_expr(&mut arg);
                if let Some(input) = &mut app.input {
                    self.assigner.visit_pat(input);
                }
                (arg, app)
            }
            _ => {
                let arg = self.lower_expr(arg);
                closure::partial_app_given(self.assigner, &mut self.lowerer.locals, arg)
            }
        }
    }

    fn lower_lambda(&mut self, lambda: Lambda, span: Span) -> hir::ExprKind {
        let (args, callable) = closure::lift(self.assigner, &self.lowerer.locals, lambda, span);
        if args.iter().any(|&arg| self.is_mutable(arg)) {
            self.lowerer.errors.push(Error::MutableClosure(span));
        }

        let id = self.assigner.next_item();
        self.lowerer.items.push(hir::Item {
            id,
            span,
            parent: self.lowerer.parent,
            attrs: Vec::new(),
            visibility: hir::Visibility::Internal,
            kind: hir::ItemKind::Callable(callable),
        });

        hir::ExprKind::Closure(args, id)
    }

    fn lower_field(&mut self, record_ty: &hir::Ty, name: &str) -> hir::Field {
        if let hir::Ty::Udt(hir::Res::Item(id)) = record_ty {
            self.tys
                .udts
                .get(id)
                .and_then(|udt| udt.field_path(name))
                .map_or(hir::Field::Err, |f| hir::Field::Path(f.clone()))
        } else if let Ok(prim) = name.parse() {
            hir::Field::Prim(prim)
        } else {
            hir::Field::Err
        }
    }

    fn lower_string_component(&mut self, component: &ast::StringComponent) -> hir::StringComponent {
        match component {
            ast::StringComponent::Expr(expr) => hir::StringComponent::Expr(self.lower_expr(expr)),
            ast::StringComponent::Lit(str) => hir::StringComponent::Lit(Rc::clone(str)),
        }
    }

    fn lower_pat(&mut self, mutability: ast::Mutability, pat: &ast::Pat) -> hir::Pat {
        if let ast::PatKind::Paren(inner) = &*pat.kind {
            return self.lower_pat(mutability, inner);
        }

        let id = self.lower_id(pat.id);
        let ty = self
            .tys
            .terms
            .get(pat.id)
            .map_or_else(|| convert::ast_pat_ty(self.names, pat).0, Clone::clone);

        let kind = match &*pat.kind {
            ast::PatKind::Bind(name, _) => {
                let name = self.lower_ident(name);
                self.lowerer.locals.insert(
                    name.id,
                    Local {
                        mutability: lower_mutability(mutability),
                        ty: ty.clone(),
                    },
                );
                hir::PatKind::Bind(name)
            }
            ast::PatKind::Discard(_) => hir::PatKind::Discard,
            ast::PatKind::Elided => hir::PatKind::Elided,
            ast::PatKind::Paren(_) => unreachable!("parentheses should be removed earlier"),
            ast::PatKind::Tuple(items) => hir::PatKind::Tuple(
                items
                    .iter()
                    .map(|i| self.lower_pat(mutability, i))
                    .collect(),
            ),
        };

        hir::Pat {
            id,
            span: pat.span,
            ty,
            kind,
        }
    }

    fn lower_qubit_init(&mut self, init: &ast::QubitInit) -> hir::QubitInit {
        if let ast::QubitInitKind::Paren(inner) = &*init.kind {
            return self.lower_qubit_init(inner);
        }

        let id = self.lower_id(init.id);
        let ty = self
            .tys
            .terms
            .get(init.id)
            .map_or(hir::Ty::Err, Clone::clone);

        let kind = match &*init.kind {
            ast::QubitInitKind::Array(length) => {
                hir::QubitInitKind::Array(Box::new(self.lower_expr(length)))
            }
            ast::QubitInitKind::Paren(_) => unreachable!("parentheses should be removed earlier"),
            ast::QubitInitKind::Single => hir::QubitInitKind::Single,
            ast::QubitInitKind::Tuple(items) => {
                hir::QubitInitKind::Tuple(items.iter().map(|i| self.lower_qubit_init(i)).collect())
            }
        };

        hir::QubitInit {
            id,
            span: init.span,
            ty,
            kind,
        }
    }

    fn lower_path(&mut self, path: &ast::Path) -> hir::Res {
        match self.names.get(path.id) {
            Some(&resolve::Res::Item(item)) => hir::Res::Item(item),
            Some(&resolve::Res::Local(node)) => hir::Res::Local(self.lower_id(node)),
            Some(resolve::Res::PrimTy(_) | resolve::Res::UnitTy) | None => hir::Res::Err,
        }
    }

    fn lower_ident(&mut self, ident: &ast::Ident) -> hir::Ident {
        hir::Ident {
            id: self.lower_id(ident.id),
            span: ident.span,
            name: ident.name.clone(),
        }
    }

    fn lower_id(&mut self, id: ast::NodeId) -> hir::NodeId {
        self.lowerer.nodes.get(id).copied().unwrap_or_else(|| {
            let new_id = self.assigner.next_node();
            self.lowerer.nodes.insert(id, new_id);
            new_id
        })
    }

    fn is_mutable(&mut self, id: hir::NodeId) -> bool {
        self.lowerer
            .locals
            .get(id)
            .expect("node ID should be a local")
            .mutability
            == hir::Mutability::Mutable
    }
}

fn lower_visibility(visibility: &ast::Visibility) -> hir::Visibility {
    match visibility.kind {
        ast::VisibilityKind::Public => hir::Visibility::Public,
        ast::VisibilityKind::Internal => hir::Visibility::Internal,
    }
}

fn lower_callable_kind(kind: ast::CallableKind) -> hir::CallableKind {
    match kind {
        ast::CallableKind::Function => hir::CallableKind::Function,
        ast::CallableKind::Operation => hir::CallableKind::Operation,
    }
}

fn lower_mutability(mutability: ast::Mutability) -> hir::Mutability {
    match mutability {
        ast::Mutability::Immutable => hir::Mutability::Immutable,
        ast::Mutability::Mutable => hir::Mutability::Mutable,
    }
}

fn lower_unop(op: ast::UnOp) -> hir::UnOp {
    match op {
        ast::UnOp::Functor(f) => hir::UnOp::Functor(lower_functor(f)),
        ast::UnOp::Neg => hir::UnOp::Neg,
        ast::UnOp::NotB => hir::UnOp::NotB,
        ast::UnOp::NotL => hir::UnOp::NotL,
        ast::UnOp::Pos => hir::UnOp::Pos,
        ast::UnOp::Unwrap => hir::UnOp::Unwrap,
    }
}

fn lower_binop(op: ast::BinOp) -> hir::BinOp {
    match op {
        ast::BinOp::Add => hir::BinOp::Add,
        ast::BinOp::AndB => hir::BinOp::AndB,
        ast::BinOp::AndL => hir::BinOp::AndL,
        ast::BinOp::Div => hir::BinOp::Div,
        ast::BinOp::Eq => hir::BinOp::Eq,
        ast::BinOp::Exp => hir::BinOp::Exp,
        ast::BinOp::Gt => hir::BinOp::Gt,
        ast::BinOp::Gte => hir::BinOp::Gte,
        ast::BinOp::Lt => hir::BinOp::Lt,
        ast::BinOp::Lte => hir::BinOp::Lte,
        ast::BinOp::Mod => hir::BinOp::Mod,
        ast::BinOp::Mul => hir::BinOp::Mul,
        ast::BinOp::Neq => hir::BinOp::Neq,
        ast::BinOp::OrB => hir::BinOp::OrB,
        ast::BinOp::OrL => hir::BinOp::OrL,
        ast::BinOp::Shl => hir::BinOp::Shl,
        ast::BinOp::Shr => hir::BinOp::Shr,
        ast::BinOp::Sub => hir::BinOp::Sub,
        ast::BinOp::XorB => hir::BinOp::XorB,
    }
}

fn lower_lit(lit: &ast::Lit) -> hir::ExprKind {
    match lit {
        ast::Lit::BigInt(value) => hir::ExprKind::Lit(hir::Lit::BigInt(value.as_ref().clone())),
        &ast::Lit::Bool(value) => hir::ExprKind::Lit(hir::Lit::Bool(value)),
        &ast::Lit::Double(value) => hir::ExprKind::Lit(hir::Lit::Double(value)),
        &ast::Lit::Int(value) => hir::ExprKind::Lit(hir::Lit::Int(value)),
        ast::Lit::Pauli(ast::Pauli::I) => hir::ExprKind::Lit(hir::Lit::Pauli(hir::Pauli::I)),
        ast::Lit::Pauli(ast::Pauli::X) => hir::ExprKind::Lit(hir::Lit::Pauli(hir::Pauli::X)),
        ast::Lit::Pauli(ast::Pauli::Y) => hir::ExprKind::Lit(hir::Lit::Pauli(hir::Pauli::Y)),
        ast::Lit::Pauli(ast::Pauli::Z) => hir::ExprKind::Lit(hir::Lit::Pauli(hir::Pauli::Z)),
        ast::Lit::Result(ast::Result::One) => {
            hir::ExprKind::Lit(hir::Lit::Result(hir::Result::One))
        }
        ast::Lit::Result(ast::Result::Zero) => {
            hir::ExprKind::Lit(hir::Lit::Result(hir::Result::Zero))
        }
        ast::Lit::String(value) => {
            hir::ExprKind::String(vec![hir::StringComponent::Lit(Rc::clone(value))])
        }
    }
}

fn lower_functor(functor: ast::Functor) -> hir::Functor {
    match functor {
        ast::Functor::Adj => hir::Functor::Adj,
        ast::Functor::Ctl => hir::Functor::Ctl,
    }
}

fn is_partial_app(arg: &ast::Expr) -> bool {
    match arg.kind.as_ref() {
        ast::ExprKind::Hole => true,
        ast::ExprKind::Paren(inner) => is_partial_app(inner),
        ast::ExprKind::Tuple(items) => items.iter().any(|i| is_partial_app(i)),
        _ => false,
    }
}
