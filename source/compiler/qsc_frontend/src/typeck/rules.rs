// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines type system rules for Q#. The checker calls these rules on the AST.
//! These rules use the inferrer to know what types to apply constraints to.

use super::{
    Error, Table, convert,
    infer::{ArgTy, Class, Inferrer, TySource},
};
use crate::resolve::{self, Names, Res};
use qsc_ast::ast::{
    self, BinOp, Block, Expr, ExprKind, FieldAccess, Functor, Ident, Idents, Lit, NodeId, Pat,
    PatKind, Path, PathKind, QubitInit, QubitInitKind, Spec, Stmt, StmtKind, StringComponent,
    TernOp, TyKind, TypeParameter, UnOp,
};
use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{self, ItemId},
    ty::{Arrow, FunctorSet, FunctorSetValue, GenericArg, Prim, Scheme, Ty},
};
use rustc_hash::FxHashMap;
use std::{cell::RefCell, rc::Rc};

/// An inferred partial term has a type, but may be the result of a diverging (non-terminating)
/// computation.
struct Partial<T> {
    ty: T,
    diverges: bool,
}

impl<T> Partial<T> {
    fn map<U>(self, f: impl FnOnce(T) -> U) -> Partial<U> {
        Partial {
            ty: f(self.ty),
            diverges: self.diverges,
        }
    }

    fn diverge_if(self, diverges: bool) -> Partial<T> {
        Partial {
            ty: self.ty,
            diverges: diverges || self.diverges,
        }
    }
}

/// Contexts are currently only generated for exprs, stmts, and specs,
/// They provide a context within which types are solved for.
#[derive(Debug)]
struct Context<'a> {
    names: &'a Names,
    globals: &'a FxHashMap<ItemId, Scheme>,
    table: &'a mut Table,
    return_ty: Option<Ty>,
    typed_holes: Vec<(NodeId, Span)>,
    /// New nodes that will be introduced into the parent `Context` after this context terminates
    new: Vec<NodeId>,
    inferrer: &'a mut Inferrer,
}

impl<'a> Context<'a> {
    fn new(
        names: &'a Names,
        globals: &'a FxHashMap<ItemId, Scheme>,
        table: &'a mut Table,
        inferrer: &'a mut Inferrer,
        new: Vec<NodeId>,
    ) -> Self {
        Self {
            names,
            globals,
            table,
            return_ty: None,
            typed_holes: Vec::new(),
            new,
            inferrer,
        }
    }

    fn infer_spec(&mut self, spec: SpecImpl<'a>) {
        let callable_input = self.infer_pat(spec.callable_input);
        if let Some(input) = spec.spec_input {
            let expected = match spec.spec {
                Spec::Body | Spec::Adj => callable_input,
                Spec::Ctl | Spec::CtlAdj => Ty::Tuple(vec![
                    Ty::Array(Box::new(Ty::Prim(Prim::Qubit))),
                    callable_input,
                ]),
            };
            let actual = self.infer_pat(input);
            self.inferrer.eq(input.span, expected, actual);
        }

        self.return_ty = Some(spec.output.clone());
        let mut block = self.infer_block(spec.block);
        if block.diverges {
            block = self.diverge();
            self.record(spec.block.id, block.ty.clone());
        }
        if let Some(return_ty) = self.return_ty.take() {
            if block.ty == Ty::UNIT {
                self.inferrer.eq(spec.output_span, block.ty, return_ty);
            } else {
                let span = spec.block.stmts.last().map_or(spec.block.span, |s| s.span);
                self.inferrer.eq(span, return_ty, block.ty);
            }
        }
    }

    fn infer_ty(&mut self, ty: &ast::Ty) -> Ty {
        match &*ty.kind {
            TyKind::Array(item) => Ty::Array(Box::new(self.infer_ty(item))),
            TyKind::Arrow(kind, input, output, functors) => Ty::Arrow(Rc::new(Arrow {
                kind: convert::callable_kind_from_ast(*kind),
                input: RefCell::new(self.infer_ty(input)),
                output: RefCell::new(self.infer_ty(output)),
                functors: RefCell::new(FunctorSet::Value(
                    functors.as_ref().map_or(FunctorSetValue::Empty, |f| {
                        convert::eval_functor_expr(f.as_ref())
                    }),
                )),
            })),
            TyKind::Hole => self.inferrer.fresh_ty(TySource::not_divergent(ty.span)),
            TyKind::Paren(inner) => self.infer_ty(inner),
            TyKind::Path(PathKind::Ok(path)) => match self.names.get(path.id) {
                Some(&Res::Item(item, _)) => Ty::Udt(path.name.name.clone(), hir::Res::Item(item)),
                Some(&Res::PrimTy(prim)) => Ty::Prim(prim),
                Some(Res::UnitTy) => Ty::Tuple(Vec::new()),
                None => Ty::Err,
                // a path should never resolve to a parameter,
                // as there is a syntactic difference between
                // paths and parameters.
                // So realistically, by construction, `Param` here is unreachable.
                // A path can also never resolve to an export, because in typeck/check,
                // we resolve exports to their original definition.
                Some(
                    resolve::Res::Local(_)
                    | resolve::Res::Param { .. }
                    | resolve::Res::Importable(..),
                ) => unreachable!(
                    " A path in an expression should never resolve to a local, parameter, \
            or as an importable, as there is syntactic differentiation."
                ),
            },
            TyKind::Param(TypeParameter {
                ty, constraints: _, ..
            }) => match self.names.get(ty.id) {
                Some(Res::Param { id, bounds }) => {
                    let (bounds, errs) = convert::class_constraints_from_ast(
                        self.names,
                        bounds,
                        &mut Default::default(),
                    );
                    for err in errs {
                        self.inferrer.report_error(err);
                    }
                    Ty::Param {
                        name: ty.name.clone(),
                        id: *id,
                        bounds,
                    }
                }
                None => Ty::Err,
                Some(_) => unreachable!(
                    "A parameter should never resolve to a non-parameter type, as there \
                    is syntactic differentiation"
                ),
            },
            TyKind::Tuple(items) => {
                Ty::Tuple(items.iter().map(|item| self.infer_ty(item)).collect())
            }
            TyKind::Err | TyKind::Path(PathKind::Err { .. }) => Ty::Err,
        }
    }

    fn infer_block(&mut self, block: &Block) -> Partial<Ty> {
        let mut diverges = false;
        let mut last = None;
        for stmt in &block.stmts {
            let stmt = self.infer_stmt(stmt);
            diverges = diverges || stmt.diverges;
            last = Some(stmt);
        }

        let ty = last.unwrap_or(converge(Ty::UNIT)).diverge_if(diverges);
        self.record(block.id, ty.ty.clone());
        ty
    }

    fn infer_stmt(&mut self, stmt: &Stmt) -> Partial<Ty> {
        let ty = match &*stmt.kind {
            StmtKind::Empty | StmtKind::Item(_) => converge(Ty::UNIT),
            StmtKind::Expr(expr) => self.infer_expr(expr),
            StmtKind::Local(_, pat, expr) => {
                let pat_ty = self.infer_pat(pat);
                let expr_ty = self.infer_expr(expr);
                self.inferrer.eq(expr.span, pat_ty, expr_ty.ty);
                converge(Ty::UNIT).diverge_if(expr_ty.diverges)
            }
            StmtKind::Qubit(_, pat, init, block) => {
                let pat_ty = self.infer_pat(pat);
                let init = self.infer_qubit_init(init);
                self.inferrer.eq(pat.span, init.ty, pat_ty);
                match block {
                    None => converge(Ty::UNIT).diverge_if(init.diverges),
                    Some(block) => {
                        let block_ty = self.infer_block(block);
                        block_ty.diverge_if(init.diverges)
                    }
                }
            }
            StmtKind::Semi(expr) => {
                let expr = self.infer_expr(expr);
                converge(Ty::UNIT).diverge_if(expr.diverges)
            }
            StmtKind::Err => converge(Ty::Err),
        };

        self.record(stmt.id, ty.ty.clone());
        ty
    }

    #[allow(clippy::too_many_lines)]
    fn infer_expr(&mut self, expr: &Expr) -> Partial<Ty> {
        let ty = match &*expr.kind {
            ExprKind::Array(items) => match items.split_first() {
                Some((first, rest)) => {
                    let first = self.infer_expr(first);
                    let mut diverges = first.diverges;
                    for item in rest {
                        let span = item.span;
                        let item = self.infer_expr(item);
                        diverges = diverges || item.diverges;
                        self.inferrer.eq(span, first.ty.clone(), item.ty);
                    }
                    converge(Ty::Array(Box::new(first.ty))).diverge_if(diverges)
                }
                None => converge(Ty::Array(Box::new(
                    self.inferrer.fresh_ty(TySource::not_divergent(expr.span)),
                ))),
            },
            ExprKind::ArrayRepeat(item, size) => {
                let item = self.infer_expr(item);
                let size_span = size.span;
                let size = self.infer_expr(size);
                self.inferrer.eq(size_span, Ty::Prim(Prim::Int), size.ty);
                converge(Ty::Array(Box::new(item.ty))).diverge_if(item.diverges || size.diverges)
            }
            ExprKind::Assign(lhs, rhs) => {
                let lhs_span = lhs.span;
                let lhs = self.infer_hole_tuple(lhs);
                let rhs = self.infer_expr(rhs);
                self.inferrer.eq(lhs_span, lhs.ty, rhs.ty);
                converge(Ty::UNIT).diverge_if(lhs.diverges || rhs.diverges)
            }
            ExprKind::AssignOp(op, lhs, rhs) => {
                let binop = self.infer_binop(expr.span, *op, lhs, rhs);
                converge(Ty::UNIT).diverge_if(binop.diverges)
            }
            ExprKind::AssignUpdate(container, index, replace) => {
                let update = self.infer_update(expr.span, container, index, replace);
                converge(Ty::UNIT).diverge_if(update.diverges)
            }
            ExprKind::BinOp(op, lhs, rhs) => self.infer_binop(expr.span, *op, lhs, rhs),
            ExprKind::Block(block) => self.infer_block(block),
            ExprKind::Call(callee, input) => {
                let callee = self.infer_expr(callee);
                let input = if has_holes(input) {
                    self.infer_hole_tuple_arg(input)
                } else {
                    self.infer_expr(input).map(ArgTy::Given)
                };
                let output_ty = self.inferrer.fresh_ty(TySource::not_divergent(expr.span));
                self.inferrer.class(
                    expr.span,
                    Class::Call {
                        callee: callee.ty,
                        input: input.ty,
                        output: output_ty.clone(),
                    },
                );
                converge(output_ty).diverge_if(callee.diverges || input.diverges)
            }
            ExprKind::Conjugate(within, apply) => {
                let within_span = within.span;
                let within = self.infer_block(within);
                self.inferrer.eq(within_span, Ty::UNIT, within.ty);
                let apply = self.infer_block(apply);
                apply.diverge_if(within.diverges)
            }
            ExprKind::Fail(message) => {
                let message_ty = self.infer_expr(message).ty;
                self.inferrer
                    .eq(message.span, Ty::Prim(Prim::String), message_ty);
                self.diverge()
            }
            ExprKind::Field(record, name) => {
                let record = self.infer_expr(record);
                if let FieldAccess::Ok(name) = name {
                    let item_ty = self.inferrer.fresh_ty(TySource::not_divergent(expr.span));
                    self.inferrer.class(
                        expr.span,
                        Class::HasField {
                            record: record.ty,
                            name: name.name.to_string(),
                            item: item_ty.clone(),
                        },
                    );
                    converge(item_ty).diverge_if(record.diverges)
                } else {
                    converge(Ty::Err)
                }
            }
            ExprKind::For(item, container, body) => {
                let item_ty = self.infer_pat(item);
                let container_span = container.span;
                let container = self.infer_expr(container);
                self.inferrer.class(
                    container_span,
                    Class::Iterable {
                        container: container.ty,
                        item: item_ty,
                    },
                );
                let body_span = body.span;
                let body = self.infer_block(body);
                self.inferrer.eq(body_span, Ty::UNIT, body.ty);
                converge(Ty::UNIT).diverge_if(container.diverges || body.diverges)
            }
            ExprKind::If(cond, if_true, if_false) => {
                let cond_span = cond.span;
                let cond = self.infer_expr(cond);
                self.inferrer.eq(cond_span, Ty::Prim(Prim::Bool), cond.ty);
                let if_true_span = if_true.span;
                let true_id = if_true.id;
                let mut if_true = self.infer_block(if_true);
                if if_true.diverges {
                    if_true = self.diverge();
                    self.record(true_id, if_true.ty.clone());
                }
                let if_false_diverges = match if_false {
                    None => {
                        self.inferrer.eq(if_true_span, Ty::UNIT, if_true.ty.clone());
                        false
                    }
                    Some(if_false_expr) => {
                        let mut if_false = self.infer_expr(if_false_expr);
                        if if_false.diverges {
                            if_false = self.diverge();
                            self.record(if_false_expr.id, if_false.ty.clone());
                        }
                        self.inferrer
                            .eq(if_true_span, if_true.ty.clone(), if_false.ty);
                        if_false.diverges
                    }
                };
                Partial {
                    diverges: cond.diverges || (if_true.diverges && if_false_diverges),
                    ..if_true
                }
            }
            ExprKind::Index(container, index) => {
                let container_span = container.span;
                let container = self.infer_expr(container);
                let index = self.infer_expr(index);
                let item_ty = self.inferrer.fresh_ty(TySource::not_divergent(expr.span));
                let container_item_ty = self
                    .inferrer
                    .fresh_ty(TySource::not_divergent(container_span));
                self.inferrer.eq(
                    container_span,
                    container.ty.clone(),
                    Ty::Array(Box::new(container_item_ty)),
                );
                self.inferrer.class(
                    expr.span,
                    Class::HasIndex {
                        container: container.ty,
                        index: index.ty,
                        item: item_ty.clone(),
                    },
                );
                converge(item_ty).diverge_if(container.diverges || index.diverges)
            }
            ExprKind::Interpolate(components) => {
                let mut diverges = false;
                for component in components {
                    match component {
                        StringComponent::Expr(expr) => {
                            let span = expr.span;
                            let expr = self.infer_expr(expr.as_ref());
                            self.inferrer.class(span, Class::Show(expr.ty));
                            diverges = diverges || expr.diverges;
                        }
                        StringComponent::Lit(_) => {}
                    }
                }

                converge(Ty::Prim(Prim::String)).diverge_if(diverges)
            }
            ExprKind::Lambda(kind, input, body) => {
                let input = self.infer_pat(input);
                let prev_ret_ty = self.return_ty.take();
                let output_ty = self.inferrer.fresh_ty(TySource::not_divergent(body.span));
                self.return_ty = Some(output_ty);
                let mut body_partial = self.infer_expr(body);
                if body_partial.diverges {
                    body_partial = self.diverge();
                    self.record(body.id, body_partial.ty.clone());
                }
                let output_ty = self
                    .return_ty
                    .take()
                    .expect("return type should be present");
                self.return_ty = prev_ret_ty;
                self.inferrer
                    .eq(body.span, body_partial.ty, output_ty.clone());
                converge(Ty::Arrow(Rc::new(Arrow {
                    kind: convert::callable_kind_from_ast(*kind),
                    input: RefCell::new(input),
                    output: RefCell::new(output_ty),
                    functors: RefCell::new(self.inferrer.fresh_functor()),
                })))
            }
            ExprKind::Lit(lit) => match lit.as_ref() {
                Lit::BigInt(_) => converge(Ty::Prim(Prim::BigInt)),
                Lit::Bool(_) => converge(Ty::Prim(Prim::Bool)),
                Lit::Double(_) => converge(Ty::Prim(Prim::Double)),
                Lit::Int(_) => converge(Ty::Prim(Prim::Int)),
                Lit::Pauli(_) => converge(Ty::Prim(Prim::Pauli)),
                Lit::Result(_) => converge(Ty::Prim(Prim::Result)),
                Lit::String(_) => converge(Ty::Prim(Prim::String)),
            },
            ExprKind::Paren(expr) => self.infer_expr(expr),
            ExprKind::Path(path) => self.infer_path_kind(expr, path),
            ExprKind::Range(start, step, end) => {
                let mut diverges = false;
                for expr in start.iter().chain(step).chain(end) {
                    let span = expr.span;
                    let expr = self.infer_expr(expr);
                    diverges = diverges || expr.diverges;
                    self.inferrer.eq(span, Ty::Prim(Prim::Int), expr.ty);
                }

                let ty = if start.is_none() && end.is_none() {
                    Prim::RangeFull
                } else if start.is_none() {
                    Prim::RangeTo
                } else if end.is_none() {
                    Prim::RangeFrom
                } else {
                    Prim::Range
                };

                converge(Ty::Prim(ty)).diverge_if(diverges)
            }
            ExprKind::Repeat(body, until, fixup) => {
                let body_span = body.span;
                let body = self.infer_block(body);
                self.inferrer.eq(body_span, Ty::UNIT, body.ty);
                let until_span = until.span;
                let until = self.infer_expr(until);
                self.inferrer.eq(until_span, Ty::Prim(Prim::Bool), until.ty);
                let fixup_diverges = match fixup {
                    None => false,
                    Some(f) => {
                        let f_span = f.span;
                        let f = self.infer_block(f);
                        self.inferrer.eq(f_span, Ty::UNIT, f.ty);
                        f.diverges
                    }
                };
                converge(Ty::UNIT).diverge_if(body.diverges || until.diverges || fixup_diverges)
            }
            ExprKind::Return(expr) => {
                let ty = self.infer_expr(expr).ty;
                if let Some(return_ty) = &self.return_ty {
                    self.inferrer.eq(expr.span, (*return_ty).clone(), ty);
                }
                self.diverge()
            }
            ExprKind::Struct(PathKind::Ok(name), copy, fields) => {
                let container = convert::ty_from_path(self.names, name);

                self.inferrer
                    .class(name.span, Class::Struct(container.clone()));

                // If the container is not a struct type, assign type Err and don't continue to process the fields.
                match &container {
                    Ty::Udt(_, hir::Res::Item(item_id)) => match self.table.udts.get(item_id) {
                        Some(udt) if udt.is_struct() => {}
                        _ => return converge(Ty::Err),
                    },
                    _ => return converge(Ty::Err),
                }

                self.inferrer.class(
                    expr.span,
                    Class::HasStructShape {
                        record: container.clone(),
                        is_copy: copy.is_some(),
                        fields: fields
                            .iter()
                            .map(|field| (field.field.name.to_string(), field.span))
                            .collect(),
                    },
                );

                // Ensure that the copy expression has the same type as the given struct.
                if let Some(copy) = copy {
                    let copy_ty = self.infer_expr(copy);
                    self.inferrer.eq(copy.span, container.clone(), copy_ty.ty);
                }

                for field in fields {
                    self.infer_field_assign(
                        field.span,
                        container.clone(),
                        &field.field,
                        &field.value,
                    );
                }

                converge(container)
            }
            ExprKind::TernOp(TernOp::Cond, cond, if_true, if_false) => {
                let cond_span = cond.span;
                let cond = self.infer_expr(cond);
                self.inferrer.eq(cond_span, Ty::Prim(Prim::Bool), cond.ty);
                let if_true = self.infer_expr(if_true);
                let if_false_span = if_false.span;
                let if_false = self.infer_expr(if_false);
                self.inferrer
                    .eq(if_false_span, if_true.ty.clone(), if_false.ty);
                Partial {
                    diverges: cond.diverges || (if_true.diverges && if_false.diverges),
                    ..if_true
                }
            }
            ExprKind::TernOp(TernOp::Update, container, index, replace) => {
                self.infer_update(expr.span, container, index, replace)
            }
            ExprKind::Tuple(items) => {
                let mut tys = Vec::new();
                let mut diverges = false;
                for item in items {
                    let item = self.infer_expr(item);
                    diverges = diverges || item.diverges;
                    tys.push(item.ty);
                }
                converge(Ty::Tuple(tys)).diverge_if(diverges)
            }
            ExprKind::UnOp(op, expr) => self.infer_unop(*op, expr),
            ExprKind::While(cond, body) => {
                let cond_span = cond.span;
                let cond = self.infer_expr(cond);
                self.inferrer.eq(cond_span, Ty::Prim(Prim::Bool), cond.ty);
                let body_span = body.span;
                let body = self.infer_block(body);
                self.inferrer.eq(body_span, Ty::UNIT, body.ty);
                converge(Ty::UNIT).diverge_if(cond.diverges || body.diverges)
            }
            ExprKind::Hole => {
                self.typed_holes.push((expr.id, expr.span));
                converge(self.inferrer.fresh_ty(TySource::not_divergent(expr.span)))
            }
            ExprKind::Err | ast::ExprKind::Struct(ast::PathKind::Err(_), ..) => converge(Ty::Err),
        };

        self.record(expr.id, ty.ty.clone());
        ty
    }

    fn infer_path_parts(
        &mut self,
        init_record: Partial<Ty>,
        rest: &[&Ident],
        lo: u32,
    ) -> Partial<Ty> {
        let mut record = init_record;
        for part in rest {
            let span = Span {
                lo,
                hi: part.span.hi,
            };
            let item_ty = self.inferrer.fresh_ty(TySource::not_divergent(span));
            self.inferrer.class(
                span,
                Class::HasField {
                    record: record.ty.clone(),
                    name: part.name.to_string(),
                    item: item_ty.clone(),
                },
            );
            // The ids of the segments are mapped specially because they will become the
            // types of the field expressions that these Ident segments will be lowered into.
            self.record(part.id, item_ty.clone());
            record = converge(item_ty).diverge_if(record.diverges);
        }
        record
    }

    fn infer_path_kind(&mut self, expr: &Expr, path: &PathKind) -> Partial<Ty> {
        match path {
            PathKind::Ok(path) => self.infer_path(expr, path),
            PathKind::Err(incomplete_path) => {
                if let Some(incomplete_path) = incomplete_path {
                    // If this is a field access, infer the fields,
                    // but leave the whole expression as `Err`.
                    let _ = self.infer_path_as_field_access(&incomplete_path.segments, expr);
                }
                converge(Ty::Err)
            }
        }
    }

    fn infer_path(&mut self, expr: &Expr, path: &Path) -> Partial<Ty> {
        match self.infer_path_as_field_access(path, expr) {
            Some(record) => record,
            // Otherwise we infer the path as a namespace path.
            None => match self.names.get(path.id) {
                None => converge(Ty::Err),
                Some(Res::Item(item, _)) => {
                    let Some(scheme) = self.globals.get(item) else {
                        return converge(Ty::Err);
                    };
                    let (ty, args) = self.inferrer.instantiate(scheme, expr.span);
                    self.table.generics.insert(expr.id, args);
                    converge(Ty::Arrow(Rc::new(ty)))
                }
                Some(&Res::Local(node)) => converge(
                    self.table
                        .terms
                        .get(node)
                        .expect("local should have type")
                        .clone(),
                ),
                Some(Res::PrimTy(_) | Res::UnitTy | Res::Param { .. } | Res::Importable(..)) => {
                    unreachable!("expression should not resolve to type reference or importable")
                }
            },
        }
    }

    fn infer_path_as_field_access(
        &mut self,
        path: &impl Idents,
        expr: &Expr,
    ) -> Option<Partial<Ty>> {
        // If the path is a field accessor, we infer the type of first segment
        // as an expr, and the rest as subsequent fields.
        if let Some((first_id, parts)) = resolve::path_as_field_accessor(self.names, path) {
            let record = converge(
                self.table
                    .terms
                    .get(first_id)
                    .expect("local should have type")
                    .clone(),
            );
            let (first, rest) = parts
                .split_first()
                .expect("path should have at least one part");
            self.record(first.id, record.ty.clone());
            Some(self.infer_path_parts(record, rest, expr.span.lo))
        } else {
            None
        }
    }

    fn infer_hole_tuple(&mut self, expr: &Expr) -> Partial<Ty> {
        match expr.kind.as_ref() {
            ExprKind::Hole => {
                let ty = self.inferrer.fresh_ty(TySource::not_divergent(expr.span));
                self.record(expr.id, ty.clone());
                converge(ty)
            }
            ExprKind::Paren(inner) => {
                let inner = self.infer_hole_tuple(inner);
                self.record(expr.id, inner.ty.clone());
                inner
            }
            ExprKind::Tuple(items) => {
                let mut tys = Vec::new();
                let mut diverges = false;
                for item in items {
                    let item = self.infer_hole_tuple(item);
                    diverges = diverges || item.diverges;
                    tys.push(item.ty);
                }
                self.record(expr.id, Ty::Tuple(tys.clone()));
                converge(Ty::Tuple(tys)).diverge_if(diverges)
            }
            _ => self.infer_expr(expr),
        }
    }

    fn infer_hole_tuple_arg(&mut self, expr: &Expr) -> Partial<ArgTy> {
        match expr.kind.as_ref() {
            ExprKind::Hole => {
                let ty = self.inferrer.fresh_ty(TySource::not_divergent(expr.span));
                self.record(expr.id, ty.clone());
                converge(ArgTy::Hole(ty))
            }
            ExprKind::Paren(inner) => {
                let inner = self.infer_hole_tuple_arg(inner);
                self.record(expr.id, ArgTy::to_ty(&inner.ty));
                inner
            }
            ExprKind::Tuple(items) => {
                let mut tys = Vec::new();
                let mut diverges = false;
                for item in items {
                    let item = self.infer_hole_tuple_arg(item);
                    diverges = diverges || item.diverges;
                    tys.push(item.ty);
                }
                self.record(expr.id, Ty::Tuple(tys.iter().map(ArgTy::to_ty).collect()));
                self.diverge_if_map(ArgTy::Given, diverges, converge(ArgTy::Tuple(tys)))
            }
            _ => self.infer_expr(expr).map(ArgTy::Given),
        }
    }

    fn infer_unop(&mut self, op: UnOp, operand: &Expr) -> Partial<Ty> {
        let span = operand.span;
        let operand = self.infer_expr(operand);
        let diverges = operand.diverges;
        let ty = match op {
            UnOp::Functor(Functor::Adj) => {
                self.inferrer.class(span, Class::Adj(operand.ty.clone()));
                operand
            }
            UnOp::Functor(Functor::Ctl) => {
                let with_ctls = self.inferrer.fresh_ty(TySource::not_divergent(span));
                self.inferrer.class(
                    span,
                    Class::Ctl {
                        op: operand.ty,
                        with_ctls: with_ctls.clone(),
                    },
                );
                converge(with_ctls)
            }
            UnOp::Neg | UnOp::Pos => {
                self.inferrer.class(span, Class::Signed(operand.ty.clone()));
                operand
            }
            UnOp::NotB => {
                self.inferrer
                    .class(span, Class::Integral(operand.ty.clone()));
                operand
            }
            UnOp::NotL => {
                self.inferrer
                    .eq(span, Ty::Prim(Prim::Bool), operand.ty.clone());
                operand
            }
            UnOp::Unwrap => {
                let base = self.inferrer.fresh_ty(TySource::not_divergent(span));
                self.inferrer.class(
                    span,
                    Class::Unwrap {
                        wrapper: operand.ty,
                        base: base.clone(),
                    },
                );
                converge(base)
            }
        };

        ty.diverge_if(diverges)
    }

    fn infer_binop(&mut self, span: Span, op: BinOp, lhs: &Expr, rhs: &Expr) -> Partial<Ty> {
        let lhs_span = lhs.span;
        let lhs = self.infer_expr(lhs);
        let rhs_span = rhs.span;
        let rhs = self.infer_expr(rhs);
        let diverges = lhs.diverges || rhs.diverges;

        let ty = match op {
            BinOp::AndL | BinOp::OrL => {
                self.inferrer.eq(rhs_span, lhs.ty.clone(), rhs.ty);
                self.inferrer
                    .eq(lhs_span, Ty::Prim(Prim::Bool), lhs.ty.clone());
                lhs
            }
            BinOp::Eq | BinOp::Neq => {
                self.inferrer.eq(rhs_span, lhs.ty.clone(), rhs.ty);
                self.inferrer.class(lhs_span, Class::Eq(lhs.ty));
                converge(Ty::Prim(Prim::Bool))
            }
            BinOp::Add => {
                self.inferrer.eq(rhs_span, lhs.ty.clone(), rhs.ty);
                self.inferrer.class(lhs_span, Class::Add(lhs.ty.clone()));
                lhs
            }
            BinOp::Gt | BinOp::Gte | BinOp::Lt | BinOp::Lte => {
                self.inferrer.eq(rhs_span, lhs.ty.clone(), rhs.ty);
                self.inferrer.class(lhs_span, Class::Ord(lhs.ty));
                converge(Ty::Prim(Prim::Bool))
            }
            BinOp::AndB | BinOp::OrB | BinOp::XorB => {
                self.inferrer.eq(rhs_span, lhs.ty.clone(), rhs.ty);
                self.inferrer
                    .class(lhs_span, Class::Integral(lhs.ty.clone()));
                lhs
            }
            BinOp::Div => {
                self.inferrer.eq(rhs_span, lhs.ty.clone(), rhs.ty);
                self.inferrer.class(lhs_span, Class::Div(lhs.ty.clone()));
                lhs
            }
            BinOp::Mul => {
                self.inferrer.eq(rhs_span, lhs.ty.clone(), rhs.ty);
                self.inferrer.class(lhs_span, Class::Mul(lhs.ty.clone()));
                lhs
            }
            BinOp::Sub => {
                self.inferrer.eq(rhs_span, lhs.ty.clone(), rhs.ty);
                self.inferrer.class(lhs_span, Class::Sub(lhs.ty.clone()));
                lhs
            }
            BinOp::Mod => {
                self.inferrer.eq(rhs_span, lhs.ty.clone(), rhs.ty);
                self.inferrer.class(lhs_span, Class::Mod(lhs.ty.clone()));
                lhs
            }
            BinOp::Exp => {
                self.inferrer.class(
                    span,
                    Class::Exp {
                        base: lhs.ty.clone(),
                        power: rhs.ty,
                    },
                );
                lhs
            }
            BinOp::Shl | BinOp::Shr => {
                self.inferrer
                    .class(lhs_span, Class::Integral(lhs.ty.clone()));
                self.inferrer.eq(rhs_span, Ty::Prim(Prim::Int), rhs.ty);
                lhs
            }
        };

        ty.diverge_if(diverges)
    }

    fn infer_update(
        &mut self,
        span: Span,
        container: &Expr,
        index: &Expr,
        item: &Expr,
    ) -> Partial<Ty> {
        let container = self.infer_expr(container);
        let item = self.infer_expr(item);
        if let Some(field) = resolve::extract_field_name(self.names, index) {
            self.inferrer.class(
                span,
                Class::HasField {
                    record: container.ty.clone(),
                    name: field.to_string(),
                    item: item.ty.clone(),
                },
            );
            container.diverge_if(item.diverges)
        } else {
            let index = self.infer_expr(index);
            self.inferrer.class(
                span,
                Class::HasIndex {
                    container: container.ty.clone(),
                    index: index.ty,
                    item: item.ty,
                },
            );
            container.diverge_if(index.diverges || item.diverges)
        }
    }

    fn infer_field_assign(
        &mut self,
        span: Span,
        container_ty: Ty,
        field_name: &Ident,
        value: &Expr,
    ) -> Partial<Ty> {
        let value = self.infer_expr(value);
        let field = field_name.name.to_string();
        self.inferrer.class(
            span,
            Class::HasField {
                record: container_ty.clone(),
                name: field,
                item: value.ty.clone(),
            },
        );

        converge(container_ty).diverge_if(value.diverges)
    }

    fn infer_pat(&mut self, pat: &Pat) -> Ty {
        let ty = match &*pat.kind {
            PatKind::Bind(name, None) => {
                let ty = self.inferrer.fresh_ty(TySource::not_divergent(pat.span));
                self.record(name.id, ty.clone());
                ty
            }
            PatKind::Bind(name, Some(ty)) => {
                let ty = self.infer_ty(ty);
                self.record(name.id, ty.clone());
                ty
            }
            PatKind::Discard(None) | PatKind::Elided => {
                self.inferrer.fresh_ty(TySource::not_divergent(pat.span))
            }
            PatKind::Discard(Some(ty)) => self.infer_ty(ty),
            PatKind::Paren(inner) => self.infer_pat(inner),
            PatKind::Tuple(items) => {
                Ty::Tuple(items.iter().map(|item| self.infer_pat(item)).collect())
            }
            PatKind::Err => Ty::Err,
        };

        self.record(pat.id, ty.clone());
        ty
    }

    fn infer_qubit_init(&mut self, init: &QubitInit) -> Partial<Ty> {
        let ty = match &*init.kind {
            QubitInitKind::Array(length) => {
                let length_span = length.span;
                let length = self.infer_expr(length);
                self.inferrer
                    .eq(length_span, Ty::Prim(Prim::Int), length.ty);
                converge(Ty::Array(Box::new(Ty::Prim(Prim::Qubit)))).diverge_if(length.diverges)
            }
            QubitInitKind::Paren(inner) => self.infer_qubit_init(inner),
            QubitInitKind::Single => converge(Ty::Prim(Prim::Qubit)),
            QubitInitKind::Tuple(items) => {
                let mut diverges = false;
                let mut tys = Vec::new();
                for item in items {
                    let item = self.infer_qubit_init(item);
                    diverges = diverges || item.diverges;
                    tys.push(item.ty);
                }
                converge(Ty::Tuple(tys)).diverge_if(diverges)
            }
            QubitInitKind::Err => converge(Ty::Err),
        };

        self.record(init.id, ty.ty.clone());
        ty
    }

    fn diverge(&mut self) -> Partial<Ty> {
        Partial {
            ty: self.inferrer.fresh_ty(TySource::divergent()),
            diverges: true,
        }
    }

    fn diverge_if_map<T>(
        &mut self,
        f: impl FnOnce(Ty) -> T,
        diverges: bool,
        partial: Partial<T>,
    ) -> Partial<T> {
        if !diverges || partial.diverges {
            partial
        } else {
            self.diverge().map(f)
        }
    }

    fn record(&mut self, id: NodeId, ty: Ty) {
        self.new.push(id);
        self.table.terms.insert(id, ty);
    }

    pub(crate) fn solve(self) -> Vec<Error> {
        let mut errs = self.inferrer.solve(&self.table.udts);

        for id in self.new {
            let ty = self.table.terms.get_mut(id).expect("node should have type");
            self.inferrer.substitute_ty(ty);

            if let Some(args) = self.table.generics.get_mut(id) {
                for arg in args {
                    match arg {
                        GenericArg::Ty(ty) => self.inferrer.substitute_ty(ty),
                        GenericArg::Functor(functors) => {
                            self.inferrer.substitute_functor(functors);
                        }
                    }
                }
            }
        }

        for (id, span) in self.typed_holes {
            let ty = self.table.terms.get_mut(id).expect("node should have type");
            errs.push(Error(super::ErrorKind::TyHole(ty.display(), span)));
        }

        errs
    }
}

#[derive(Clone, Copy)]
pub(super) struct SpecImpl<'a> {
    pub(super) spec: Spec,
    pub(super) callable_input: &'a Pat,
    pub(super) spec_input: Option<&'a Pat>,
    pub(super) output: &'a Ty,
    pub(super) output_span: Span,
    pub(super) block: &'a Block,
}

pub(super) fn spec(
    names: &Names,
    globals: &FxHashMap<ItemId, Scheme>,
    table: &mut Table,
    spec: SpecImpl,
) -> Vec<Error> {
    let mut inferrer = Inferrer::new();
    let mut context = Context::new(names, globals, table, &mut inferrer, Vec::new());
    context.infer_spec(spec);
    context.solve()
}

pub(super) fn expr(
    names: &Names,
    globals: &FxHashMap<ItemId, Scheme>,
    table: &mut Table,
    expr: &Expr,
) -> Vec<Error> {
    let mut inferrer = Inferrer::new();
    let mut context = Context::new(names, globals, table, &mut inferrer, Vec::new());
    context.infer_expr(expr);
    context.solve()
}

pub(super) fn stmt(
    names: &Names,
    globals: &FxHashMap<ItemId, Scheme>,
    table: &mut Table,
    inferrer: &mut Inferrer,
    stmt: &Stmt,
) -> Vec<NodeId> {
    let mut context = Context::new(names, globals, table, inferrer, Vec::new());
    context.infer_stmt(stmt);
    context.new
}

pub(super) fn solve(
    names: &Names,
    globals: &FxHashMap<ItemId, Scheme>,
    table: &mut Table,
    inferrer: &mut Inferrer,
    new_nodes: Vec<NodeId>,
) -> Vec<Error> {
    let context = Context::new(names, globals, table, inferrer, new_nodes);
    context.solve()
}

fn converge<T>(ty: T) -> Partial<T> {
    Partial {
        ty,
        diverges: false,
    }
}

fn has_holes(expr: &Expr) -> bool {
    match &*expr.kind {
        ExprKind::Hole => true,
        ExprKind::Paren(inner) => has_holes(inner),
        ExprKind::Tuple(items) => items.iter().any(|e| has_holes(e)),
        _ => false,
    }
}
