// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::mem::replace;
use std::rc::Rc;

use crate::compilation::{Compilation, Lookup};
use crate::qsc_utils::{find_ident, span_contains, span_touches};
use qsc::ast::visit::{walk_expr, walk_namespace, walk_pat, walk_ty, walk_ty_def, Visitor};
use qsc::{ast, hir, resolve};

#[allow(unused_variables)]
pub(crate) trait Handler<'package> {
    fn at_callable_def(
        &mut self,
        context: &LocatorContext<'package>,
        name: &'package ast::Ident,
        decl: &'package ast::CallableDecl,
    );

    fn at_callable_ref(
        &mut self,
        path: &'package ast::Path,
        item_id: &'_ hir::ItemId,
        item: &'package hir::Item,
        package: &'package hir::Package,
        decl: &'package hir::CallableDecl,
    );

    fn at_type_param_def(
        &mut self,
        context: &LocatorContext<'package>,
        def_name: &'package ast::Ident,
        param_id: hir::ty::ParamId,
    );

    fn at_type_param_ref(
        &mut self,
        context: &LocatorContext<'package>,
        ref_name: &'package ast::Ident,
        param_id: hir::ty::ParamId,
        def_name: &'package ast::Ident,
    );

    fn at_new_type_def(&mut self, type_name: &'package ast::Ident, def: &'package ast::TyDef);

    fn at_new_type_ref(
        &mut self,
        path: &'package ast::Path,
        item_id: &'_ hir::ItemId,
        package: &'package hir::Package,
        type_name: &'package hir::Ident,
        udt: &'package hir::ty::Udt,
    );

    fn at_field_def(
        &mut self,
        context: &LocatorContext<'package>,
        field_name: &'package ast::Ident,
        ty: &'package ast::Ty,
    );

    fn at_field_ref(
        &mut self,
        field_ref: &'package ast::Ident,
        expr_id: &'package ast::NodeId,
        item_id: &'_ hir::ItemId,
        field_def: &'package hir::ty::UdtField,
    );

    fn at_local_def(
        &mut self,
        context: &LocatorContext<'package>,
        ident: &'package ast::Ident,
        pat: &'package ast::Pat,
    );

    fn at_local_ref(
        &mut self,
        context: &LocatorContext<'package>,
        path: &'package ast::Path,
        node_id: &'package ast::NodeId,
        definition: &'package ast::Ident,
    );
}

pub(crate) struct LocatorContext<'package> {
    pub(crate) current_callable: Option<&'package ast::CallableDecl>,
    pub(crate) lambda_params: Vec<&'package ast::Pat>,
    pub(crate) current_item_doc: Rc<str>,
    pub(crate) current_namespace: Rc<str>,
    pub(crate) in_params: bool,
    pub(crate) in_lambda_params: bool,
    pub(crate) current_udt_id: Option<&'package hir::ItemId>,
}

pub(crate) struct Locator<'inner, 'package, T> {
    inner: &'inner mut T,
    offset: u32,
    compilation: &'package Compilation,
    context: LocatorContext<'package>,
}

impl<'inner, 'package, T> Locator<'inner, 'package, T> {
    pub(crate) fn new(
        inner: &'inner mut T,
        offset: u32,
        compilation: &'package Compilation,
    ) -> Self {
        Self {
            inner,
            offset,
            compilation,
            context: LocatorContext {
                current_namespace: Rc::from(""),
                current_callable: None,
                in_params: false,
                lambda_params: vec![],
                in_lambda_params: false,
                current_item_doc: Rc::from(""),
                current_udt_id: None,
            },
        }
    }
}

impl<'inner, 'package, T: Handler<'package>> Visitor<'package> for Locator<'inner, 'package, T> {
    fn visit_namespace(&mut self, namespace: &'package ast::Namespace) {
        if span_contains(namespace.span, self.offset) {
            self.context.current_namespace = namespace.name.name.clone();
            walk_namespace(self, namespace);
        }
    }

    // Handles callable, UDT, and type param definitions
    fn visit_item(&mut self, item: &'package ast::Item) {
        if span_contains(item.span, self.offset) {
            let context = replace(&mut self.context.current_item_doc, item.doc.clone());
            match &*item.kind {
                ast::ItemKind::Callable(decl) => {
                    if span_touches(decl.name.span, self.offset) {
                        self.inner.at_callable_def(&self.context, &decl.name, decl);
                    } else if span_contains(decl.span, self.offset) {
                        let context = self.context.current_callable;
                        self.context.current_callable = Some(decl);

                        // walk callable decl
                        decl.generics.iter().for_each(|p| {
                            if span_touches(p.span, self.offset) {
                                if let Some(resolve::Res::Param(param_id)) =
                                    self.compilation.get_res(p.id)
                                {
                                    self.inner.at_type_param_def(&self.context, p, *param_id);
                                }
                            }
                        });
                        self.context.in_params = true;
                        self.visit_pat(&decl.input);
                        self.context.in_params = false;
                        self.visit_ty(&decl.output);
                        match &*decl.body {
                            ast::CallableBody::Block(block) => self.visit_block(block),
                            ast::CallableBody::Specs(specs) => {
                                specs.iter().for_each(|s| self.visit_spec_decl(s));
                            }
                        }

                        self.context.current_callable = context;
                    }
                    // Note: the `item.span` can cover things like doc
                    // comment, attributes, and visibility keywords, which aren't
                    // things we want to have logic for, while the `decl.span` is
                    // specific to the contents of the callable decl, which we do want
                    // logic for. If the `if` or `else if` above is not met, then
                    // the cursor is at one of these non-decl parts of the item,
                    // and we want to do nothing.
                }
                ast::ItemKind::Ty(ident, def) => {
                    if let Some(resolve::Res::Item(item_id, _)) = self.compilation.get_res(ident.id)
                    {
                        let context = self.context.current_udt_id;
                        self.context.current_udt_id = Some(item_id);

                        if span_touches(ident.span, self.offset) {
                            self.inner.at_new_type_def(ident, def);
                        } else {
                            self.visit_ty_def(def);
                        }

                        self.context.current_udt_id = context;
                    }
                }
                _ => {}
            }
            self.context.current_item_doc = context;
        }
    }

    fn visit_spec_decl(&mut self, decl: &'package ast::SpecDecl) {
        // Walk Spec Decl
        match &decl.body {
            ast::SpecBody::Gen(_) => {}
            ast::SpecBody::Impl(pat, block) => {
                self.context.in_params = true;
                self.visit_pat(pat);
                self.context.in_params = false;
                self.visit_block(block);
            }
        }
    }

    // Handles UDT field definitions
    fn visit_ty_def(&mut self, def: &'package ast::TyDef) {
        if span_contains(def.span, self.offset) {
            if let ast::TyDefKind::Field(ident, ty) = &*def.kind {
                if let Some(ident) = ident {
                    if span_touches(ident.span, self.offset) {
                        self.inner.at_field_def(&self.context, ident, ty);
                    } else {
                        self.visit_ty(ty);
                    }
                } else {
                    self.visit_ty(ty);
                }
            } else {
                walk_ty_def(self, def);
            }
        }
    }

    // Handles type param references
    fn visit_ty(&mut self, ty: &'package ast::Ty) {
        if span_touches(ty.span, self.offset) {
            if let ast::TyKind::Param(param) = &*ty.kind {
                if let Some(resolve::Res::Param(param_id)) = self.compilation.get_res(param.id) {
                    if let Some(curr) = self.context.current_callable {
                        if let Some(def_name) = curr.generics.get(usize::from(*param_id)) {
                            self.inner
                                .at_type_param_ref(&self.context, param, *param_id, def_name);
                        }
                    }
                }
            } else {
                walk_ty(self, ty);
            }
        }
    }

    // Handles local variable definitions
    fn visit_pat(&mut self, pat: &'package ast::Pat) {
        if span_touches(pat.span, self.offset) {
            match &*pat.kind {
                ast::PatKind::Bind(ident, anno) => {
                    if span_touches(ident.span, self.offset) {
                        self.inner.at_local_def(&self.context, ident, pat);
                    } else if let Some(ty) = anno {
                        self.visit_ty(ty);
                    }
                }
                _ => walk_pat(self, pat),
            }
        }
    }

    // Handles UDT field references
    fn visit_expr(&mut self, expr: &'package ast::Expr) {
        if span_touches(expr.span, self.offset) {
            match &*expr.kind {
                ast::ExprKind::Field(udt, field_ref)
                    if span_touches(field_ref.span, self.offset) =>
                {
                    if let Some(hir::ty::Ty::Udt(res)) = &self.compilation.get_ty(udt.id) {
                        let (item, resolved_item_id) = self
                            .compilation
                            .resolve_item_res(self.compilation.user_package_id, res);
                        match &item.kind {
                            hir::ItemKind::Ty(_, udt) => {
                                if let Some(field_def) = udt.find_field_by_name(&field_ref.name) {
                                    self.inner.at_field_ref(
                                        field_ref,
                                        &expr.id,
                                        &resolved_item_id,
                                        field_def,
                                    );
                                }
                            }
                            _ => panic!("UDT has invalid resolution."),
                        }
                    }
                }
                ast::ExprKind::Lambda(_, pat, expr) => {
                    self.context.in_lambda_params = true;
                    self.visit_pat(pat);
                    self.context.in_lambda_params = false;
                    self.context.lambda_params.push(pat);
                    self.visit_expr(expr);
                    self.context.lambda_params.pop();
                }
                _ => walk_expr(self, expr),
            }
        }
    }

    // Handles local variable, UDT, and callable references
    fn visit_path(&mut self, path: &'package ast::Path) {
        if span_touches(path.span, self.offset) {
            let res = self.compilation.get_res(path.id);
            if let Some(res) = res {
                match &res {
                    resolve::Res::Item(item_id, _) => {
                        let (item, package, resolved_item_id) = self
                            .compilation
                            .resolve_item_relative_to_user_package(item_id);
                        match &item.kind {
                            hir::ItemKind::Callable(decl) => {
                                self.inner.at_callable_ref(
                                    path,
                                    &resolved_item_id,
                                    item,
                                    package,
                                    decl,
                                );
                            }
                            hir::ItemKind::Ty(type_name, udt) => {
                                self.inner.at_new_type_ref(
                                    path,
                                    &resolved_item_id,
                                    package,
                                    type_name,
                                    udt,
                                );
                            }
                            hir::ItemKind::Namespace(_, _) => {
                                panic!(
                                    "Reference node should not refer to a namespace: {}",
                                    path.id
                                )
                            }
                        }
                    }
                    resolve::Res::Local(node_id) => {
                        if let Some(curr) = self.context.current_callable {
                            {
                                if let Some(definition) = find_ident(node_id, curr) {
                                    self.inner.at_local_ref(
                                        &self.context,
                                        path,
                                        node_id,
                                        definition,
                                    );
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
