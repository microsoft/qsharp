// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// #[cfg(test)]
// mod tests;

use crate::qsc_utils::{find_ident, find_item, span_contains, span_touches, Compilation};
use qsc::ast::visit::{walk_callable_decl, walk_expr, walk_pat, walk_ty_def, Visitor};
use qsc::{ast, hir, resolve};

pub(crate) trait CursorLocatorAPI<'a> {
    fn at_callable_def(&mut self, decl: &'a ast::CallableDecl) {}

    fn at_callable_ref(&mut self, decl: &'a hir::CallableDecl, item_id: &'a hir::ItemId) {}

    fn at_new_type_def(&mut self, type_name: &'a ast::Ident) {}

    fn at_new_type_ref(&mut self, type_name: &'a hir::Ident, item_id: &'a hir::ItemId) {}

    fn at_field_def(&mut self, field_name: &'a ast::Ident) {}

    fn at_field_ref(&mut self, field: &'a hir::ty::UdtField, item_id: &'a hir::ItemId) {}

    fn at_local_def(&mut self, ident: &'a ast::Ident) {}

    fn at_local_ref(&mut self, ident: &'a ast::Ident) {}
}

pub(crate) struct CursorLocator<'a, 'b, T> {
    inner: &'a mut T,
    offset: u32,
    compilation: &'b Compilation,
    current_callable: Option<&'b ast::CallableDecl>,
}

impl<'a, 'b, T> CursorLocator<'a, 'b, T> {
    pub(crate) fn new(inner: &'a mut T, offset: u32, compilation: &'b Compilation) -> Self {
        Self {
            inner,
            offset,
            compilation,
            current_callable: None,
        }
    }
}

impl<'a, 'b, T: CursorLocatorAPI<'b>> Visitor<'b> for CursorLocator<'a, 'b, T> {
    // Handles callable and UDT definitions
    fn visit_item(&mut self, item: &'b ast::Item) {
        if span_contains(item.span, self.offset) {
            match &*item.kind {
                ast::ItemKind::Callable(decl) => {
                    if span_touches(decl.name.span, self.offset) {
                        self.inner.at_callable_def(decl);
                    } else if span_contains(decl.span, self.offset) {
                        let context = self.current_callable;
                        self.current_callable = Some(decl);
                        walk_callable_decl(self, decl);
                        self.current_callable = context;
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
                    if span_touches(ident.span, self.offset) {
                        self.inner.at_new_type_def(ident);
                    } else {
                        self.visit_ty_def(def);
                    }
                }
                _ => {}
            }
        }
    }

    // Handles UDT field definitions
    fn visit_ty_def(&mut self, def: &'b ast::TyDef) {
        if span_contains(def.span, self.offset) {
            if let ast::TyDefKind::Field(ident, ty) = &*def.kind {
                if let Some(ident) = ident {
                    if span_touches(ident.span, self.offset) {
                        self.inner.at_field_def(ident);
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

    // Handles local variable definitions
    fn visit_pat(&mut self, pat: &'b ast::Pat) {
        if span_touches(pat.span, self.offset) {
            match &*pat.kind {
                ast::PatKind::Bind(ident, anno) => {
                    if span_touches(ident.span, self.offset) {
                        self.inner.at_local_def(ident);
                    } else if let Some(ty) = anno {
                        self.visit_ty(ty);
                    }
                }
                _ => walk_pat(self, pat),
            }
        }
    }

    // Handles UDT field references
    fn visit_expr(&mut self, expr: &'b ast::Expr) {
        if span_touches(expr.span, self.offset) {
            match &*expr.kind {
                ast::ExprKind::Field(udt, field) if span_touches(field.span, self.offset) => {
                    if let Some(hir::ty::Ty::Udt(res)) =
                        self.compilation.unit.ast.tys.terms.get(udt.id)
                    {
                        match res {
                            hir::Res::Item(item_id) => {
                                if let (Some(item), _) = find_item(self.compilation, item_id) {
                                    match &item.kind {
                                        hir::ItemKind::Ty(_, udt) => {
                                            if let Some(field) = udt.find_field_by_name(&field.name)
                                            {
                                                self.inner.at_field_ref(field, item_id);
                                            }
                                        }
                                        _ => panic!("UDT has invalid resolution."),
                                    }
                                }
                            }
                            _ => panic!("UDT has invalid resolution."),
                        }
                    }
                }
                _ => walk_expr(self, expr),
            }
        }
    }

    // Handles local variable, UDT, and callable references
    fn visit_path(&mut self, path: &'_ ast::Path) {
        if span_touches(path.span, self.offset) {
            let res = self.compilation.unit.ast.names.get(path.id);
            if let Some(res) = res {
                match &res {
                    resolve::Res::Item(item_id) => {
                        if let (Some(item), _) = find_item(self.compilation, item_id) {
                            match &item.kind {
                                hir::ItemKind::Callable(decl) => {
                                    self.inner.at_callable_ref(decl, item_id);
                                }
                                hir::ItemKind::Ty(ident, _) => {
                                    self.inner.at_new_type_ref(ident, item_id);
                                }
                                hir::ItemKind::Namespace(_, _) => {
                                    panic!(
                                        "Reference node should not refer to a namespace: {}",
                                        path.id
                                    )
                                }
                            }
                        };
                    }
                    resolve::Res::Local(node_id) => {
                        if let Some(curr) = self.current_callable {
                            {
                                if let Some(ident) = find_ident(node_id, curr) {
                                    self.inner.at_local_ref(ident);
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
