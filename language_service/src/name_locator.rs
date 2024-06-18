// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::mem::replace;
use std::rc::Rc;

use crate::compilation::Compilation;
use crate::qsc_utils::find_ident;
use qsc::ast::visit::{walk_expr, walk_namespace, walk_pat, walk_ty, walk_ty_def, Visitor};
use qsc::display::Lookup;
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
        reference: &'package ast::Ident,
        param_id: hir::ty::ParamId,
        definition: &'package ast::Ident,
    );

    fn at_new_type_def(
        &mut self,
        context: &LocatorContext<'package>,
        type_name: &'package ast::Ident,
        def: &'package ast::TyDef,
    );

    fn at_struct_def(
        &mut self,
        context: &LocatorContext<'package>,
        type_name: &'package ast::Ident,
        def: &'package ast::StructDecl,
    );

    fn at_new_type_ref(
        &mut self,
        path: &'package ast::Path,
        item_id: &'_ hir::ItemId,
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
    pub(crate) current_item_name: Rc<str>,
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

impl<'inner, 'package, T: Handler<'package>> Locator<'inner, 'package, T> {
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
                current_item_name: Rc::from(""),
                current_udt_id: None,
            },
        }
    }

    fn get_field_def(
        &mut self,
        udt_res: &'package hir::Res,
        field_ref: &'package ast::Ident,
    ) -> Option<(hir::ItemId, &'package hir::ty::UdtField)> {
        let (item, resolved_item_id) = self
            .compilation
            .resolve_item_res(self.compilation.user_package_id, udt_res);
        if let hir::ItemKind::Ty(_, udt) = &item.kind {
            if let Some(field_def) = udt.find_field_by_name(&field_ref.name) {
                return Some((resolved_item_id, field_def));
            }
        }
        None
    }
}

impl<'inner, 'package, T: Handler<'package>> Visitor<'package> for Locator<'inner, 'package, T> {
    fn visit_namespace(&mut self, namespace: &'package ast::Namespace) {
        if namespace.span.contains(self.offset) {
            self.context.current_namespace = namespace.name.name();
            walk_namespace(self, namespace);
        }
    }

    // Handles callable, UDT, struct, and type param definitions
    fn visit_item(&mut self, item: &'package ast::Item) {
        if item.span.contains(self.offset) {
            let context = replace(&mut self.context.current_item_doc, item.doc.clone());
            match &*item.kind {
                ast::ItemKind::Callable(decl) => {
                    if decl.name.span.touches(self.offset) {
                        self.inner.at_callable_def(&self.context, &decl.name, decl);
                    } else if decl.span.contains(self.offset) {
                        let context_curr_item_name =
                            replace(&mut self.context.current_item_name, decl.name.name.clone());
                        let context_curr_callable = self.context.current_callable;
                        self.context.current_callable = Some(decl);

                        // walk callable decl
                        decl.generics.iter().for_each(|p| {
                            if p.span.touches(self.offset) {
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
                        self.context.current_callable = context_curr_callable;
                        self.context.current_item_name = context_curr_item_name;
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
                        let context_curr_item_name =
                            replace(&mut self.context.current_item_name, ident.name.clone());
                        let context = self.context.current_udt_id;
                        self.context.current_udt_id = Some(item_id);

                        if ident.span.touches(self.offset) {
                            self.inner.at_new_type_def(&self.context, ident, def);
                        } else {
                            self.visit_ty_def(def);
                        }

                        self.context.current_udt_id = context;
                        self.context.current_item_name = context_curr_item_name;
                    }
                }
                ast::ItemKind::Struct(def) => {
                    if let Some(resolve::Res::Item(item_id, _)) =
                        self.compilation.get_res(def.name.id)
                    {
                        let context_curr_item_name =
                            replace(&mut self.context.current_item_name, def.name.name.clone());
                        let context = self.context.current_udt_id;
                        self.context.current_udt_id = Some(item_id);

                        if def.name.span.touches(self.offset) {
                            self.inner.at_struct_def(&self.context, &def.name, def);
                        } else {
                            self.visit_struct_decl(def);
                        }

                        self.context.current_udt_id = context;
                        self.context.current_item_name = context_curr_item_name;
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
        if def.span.contains(self.offset) {
            if let ast::TyDefKind::Field(ident, ty) = &*def.kind {
                if let Some(ident) = ident {
                    if ident.span.touches(self.offset) {
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

    // Handles struct field definitions
    fn visit_field_def(&mut self, def: &'package ast::FieldDef) {
        if def.span.contains(self.offset) {
            if def.name.span.touches(self.offset) {
                self.inner.at_field_def(&self.context, &def.name, &def.ty);
            } else {
                self.visit_ty(&def.ty);
            }
        }
    }

    // Handles type param references
    fn visit_ty(&mut self, ty: &'package ast::Ty) {
        if ty.span.touches(self.offset) {
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
        if pat.span.touches(self.offset) {
            match &*pat.kind {
                ast::PatKind::Bind(ident, anno) => {
                    if ident.span.touches(self.offset) {
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
        if expr.span.touches(self.offset) {
            match &*expr.kind {
                ast::ExprKind::Field(udt, field_ref) if field_ref.span.touches(self.offset) => {
                    if let Some(hir::ty::Ty::Udt(_, res)) = &self.compilation.get_ty(udt.id) {
                        if let Some((item_id, field_def)) = self.get_field_def(res, field_ref) {
                            self.inner.at_field_ref(field_ref, &item_id, field_def);
                        }
                    }
                }
                ast::ExprKind::Struct(ty_name, copy, fields) => {
                    if ty_name.span.touches(self.offset) {
                        self.visit_path(ty_name);
                        return;
                    }

                    if let Some(copy) = &copy {
                        if copy.span.touches(self.offset) {
                            self.visit_expr(copy);
                            return;
                        }
                    }

                    for field in fields.iter() {
                        if field.span.touches(self.offset) {
                            if field.field.span.touches(self.offset) {
                                if let Some(hir::ty::Ty::Udt(_, res)) =
                                    &self.compilation.get_ty(expr.id)
                                {
                                    if let Some((item_id, field_def)) =
                                        self.get_field_def(res, &field.field)
                                    {
                                        self.inner.at_field_ref(&field.field, &item_id, field_def);
                                    }
                                }
                            } else if field.value.span.touches(self.offset) {
                                self.visit_expr(&field.value);
                            }
                            return;
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
        if path.span.touches(self.offset) {
            let res = self.compilation.get_res(path.id);
            if let Some(res) = res {
                match &res {
                    resolve::Res::Item(item_id, _) => {
                        let (item, _, resolved_item_id) = self
                            .compilation
                            .resolve_item_relative_to_user_package(item_id);
                        match &item.kind {
                            hir::ItemKind::Callable(decl) => {
                                self.inner.at_callable_ref(path, &resolved_item_id, decl);
                            }
                            hir::ItemKind::Ty(type_name, udt) => {
                                self.inner
                                    .at_new_type_ref(path, &resolved_item_id, type_name, udt);
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
