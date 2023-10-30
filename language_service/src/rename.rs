// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::compilation::Compilation;
use crate::name_locator::{Handler, Locator, LocatorContext};
use crate::protocol;
use crate::qsc_utils::{protocol_span, resolve_offset};
use qsc::ast::visit::{walk_expr, walk_ty, Visitor};
use qsc::hir::{ty::Ty, Res};
use qsc::{ast, hir, resolve, Span};
use std::rc::Rc;

pub(crate) fn prepare_rename(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Option<(protocol::Span, String)> {
    let (ast, offset) = resolve_offset(compilation, source_name, offset);

    let mut prepare_rename = Rename::new(compilation, true);
    let mut locator = Locator::new(&mut prepare_rename, offset, compilation);
    locator.visit_package(&ast.package);
    prepare_rename
        .prepare
        .map(|p| (protocol_span(p.0, &compilation.current_unit().sources), p.1))
}

pub(crate) fn get_rename(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Vec<protocol::Span> {
    let (ast, offset) = resolve_offset(compilation, source_name, offset);

    let mut rename = Rename::new(compilation, false);
    let mut locator = Locator::new(&mut rename, offset, compilation);
    locator.visit_package(&ast.package);
    rename
        .locations
        .into_iter()
        .map(|s| protocol_span(s, &compilation.current_unit().sources))
        .collect::<Vec<_>>()
}

struct Rename<'a> {
    compilation: &'a Compilation,
    locations: Vec<Span>,
    is_prepare: bool,
    prepare: Option<(Span, String)>,
}

impl<'a> Rename<'a> {
    fn new(compilation: &'a Compilation, is_prepare: bool) -> Self {
        Self {
            compilation,
            locations: vec![],
            is_prepare,
            prepare: None,
        }
    }

    fn get_spans_for_item_rename(&mut self, item_id: &hir::ItemId, ast_name: &ast::Ident) {
        let package_id = item_id.package.expect("package id should be resolved");
        // Only rename items that are part of the user package
        if package_id == self.compilation.current {
            let user_unit = self.compilation.current_unit();
            if let Some(def) = user_unit.package.items.get(item_id.item) {
                if self.is_prepare {
                    self.prepare = Some((ast_name.span, ast_name.name.to_string()));
                } else {
                    let def_span = match &def.kind {
                        hir::ItemKind::Callable(decl) => decl.name.span,
                        hir::ItemKind::Namespace(name, _) | hir::ItemKind::Ty(name, _) => name.span,
                    };
                    let mut rename = ItemRename {
                        item_id,
                        compilation: self.compilation,
                        locations: vec![],
                    };
                    rename.visit_package(&user_unit.ast.package);
                    rename.locations.push(def_span);
                    self.locations = rename.locations;
                }
            }
        }
    }

    fn get_spans_for_field_rename(&mut self, item_id: &hir::ItemId, ast_name: &ast::Ident) {
        let package_id = item_id.package.expect("package id should be resolved");
        // Only rename items that are part of the user package
        if package_id == self.compilation.current {
            let user_unit = self.compilation.current_unit();
            if let Some(def) = user_unit.package.items.get(item_id.item) {
                if let hir::ItemKind::Ty(_, udt) = &def.kind {
                    if let Some(ty_field) = udt.find_field_by_name(&ast_name.name) {
                        if self.is_prepare {
                            self.prepare = Some((ast_name.span, ast_name.name.to_string()));
                        } else {
                            let def_span = ty_field
                                .name_span
                                .expect("field found via name should have a name");
                            let mut rename = FieldRename {
                                item_id,
                                field_name: ast_name.name.clone(),
                                compilation: self.compilation,
                                locations: vec![],
                            };
                            rename.visit_package(&user_unit.ast.package);
                            rename.locations.push(def_span);
                            self.locations = rename.locations;
                        }
                    }
                }
            }
        }
    }

    fn get_spans_for_local_rename(
        &mut self,
        node_id: ast::NodeId,
        ast_name: &ast::Ident,
        current_callable: &ast::CallableDecl,
    ) {
        if self.is_prepare {
            self.prepare = Some((ast_name.span, ast_name.name.to_string()));
        } else {
            let mut rename = LocalRename {
                node_id,
                compilation: self.compilation,
                locations: vec![],
            };
            rename.visit_callable_decl(current_callable);
            self.locations = rename.locations;
        }
    }
}

impl<'a> Handler<'a> for Rename<'a> {
    fn at_callable_def(
        &mut self,
        _: &LocatorContext<'a>,
        name: &'a ast::Ident,
        _: &'a ast::CallableDecl,
    ) {
        if let Some(resolve::Res::Item(item_id)) =
            self.compilation.current_unit().ast.names.get(name.id)
        {
            self.get_spans_for_item_rename(
                &resolve_package(self.compilation.current, item_id),
                name,
            );
        }
    }

    fn at_callable_ref(
        &mut self,
        path: &'a ast::Path,
        item_id: &'_ hir::ItemId,
        _: &'a hir::Item,
        _: &'a hir::Package,
        _: &'a hir::CallableDecl,
    ) {
        self.get_spans_for_item_rename(item_id, &path.name);
    }

    fn at_new_type_def(&mut self, type_name: &'a ast::Ident, _: &'a ast::TyDef) {
        if let Some(resolve::Res::Item(item_id)) =
            self.compilation.current_unit().ast.names.get(type_name.id)
        {
            self.get_spans_for_item_rename(
                &resolve_package(self.compilation.current, item_id),
                type_name,
            );
        }
    }

    fn at_new_type_ref(
        &mut self,
        path: &'a ast::Path,
        item_id: &'_ hir::ItemId,
        _: &'a hir::Package,
        _: &'a hir::Ident,
        _: &'a hir::ty::Udt,
    ) {
        self.get_spans_for_item_rename(item_id, &path.name);
    }

    fn at_field_def(
        &mut self,
        context: &LocatorContext<'a>,
        field_name: &'a ast::Ident,
        _: &'a ast::Ty,
    ) {
        if let Some(item_id) = context.current_udt_id {
            self.get_spans_for_field_rename(
                &resolve_package(self.compilation.current, item_id),
                field_name,
            );
        }
    }

    fn at_field_ref(
        &mut self,
        field_ref: &'a ast::Ident,
        _: &'a ast::NodeId,
        item_id: &'_ hir::ItemId,
        _: &'a hir::ty::UdtField,
    ) {
        self.get_spans_for_field_rename(item_id, field_ref);
    }

    fn at_local_def(
        &mut self,
        context: &LocatorContext<'a>,
        ident: &'a ast::Ident,
        _: &'a ast::Pat,
    ) {
        if let Some(curr) = context.current_callable {
            self.get_spans_for_local_rename(ident.id, ident, curr);
        }
    }

    fn at_local_ref(
        &mut self,
        context: &LocatorContext<'a>,
        path: &'a ast::Path,
        node_id: &'a ast::NodeId,
        _: &'a ast::Ident,
    ) {
        if let Some(curr) = context.current_callable {
            self.get_spans_for_local_rename(*node_id, &path.name, curr);
        }
    }
}

struct ItemRename<'a> {
    item_id: &'a hir::ItemId,
    compilation: &'a Compilation,
    locations: Vec<Span>,
}

impl<'a> Visitor<'_> for ItemRename<'a> {
    fn visit_path(&mut self, path: &'_ ast::Path) {
        let res = self.compilation.current_unit().ast.names.get(path.id);
        if let Some(resolve::Res::Item(item_id)) = res {
            if self.eq(item_id) {
                self.locations.push(path.name.span);
            }
        }
    }

    fn visit_ty(&mut self, ty: &'_ ast::Ty) {
        if let ast::TyKind::Path(ty_path) = &*ty.kind {
            let res = self.compilation.current_unit().ast.names.get(ty_path.id);
            if let Some(resolve::Res::Item(item_id)) = res {
                if self.eq(item_id) {
                    self.locations.push(ty_path.name.span);
                }
            }
        } else {
            walk_ty(self, ty);
        }
    }
}

impl<'a> ItemRename<'a> {
    fn eq(&mut self, item_id: &hir::ItemId) -> bool {
        item_id.item == self.item_id.item
            && item_id.package.unwrap_or(self.compilation.current)
                == self.item_id.package.expect("package id should be resolved")
    }
}

struct FieldRename<'a> {
    item_id: &'a hir::ItemId,
    field_name: Rc<str>,
    compilation: &'a Compilation,
    locations: Vec<Span>,
}

impl<'a> Visitor<'_> for FieldRename<'a> {
    fn visit_expr(&mut self, expr: &'_ ast::Expr) {
        if let ast::ExprKind::Field(qualifier, field_name) = &*expr.kind {
            self.visit_expr(qualifier);
            if field_name.name == self.field_name {
                if let Some(Ty::Udt(Res::Item(id))) = self
                    .compilation
                    .current_unit()
                    .ast
                    .tys
                    .terms
                    .get(qualifier.id)
                {
                    if self.eq(id) {
                        self.locations.push(field_name.span);
                    }
                }
            }
        } else {
            walk_expr(self, expr);
        }
    }
}

impl<'a> FieldRename<'a> {
    fn eq(&mut self, item_id: &hir::ItemId) -> bool {
        item_id.item == self.item_id.item
            && item_id.package.unwrap_or(self.compilation.current)
                == self.item_id.package.expect("package id should be resolved")
    }
}

struct LocalRename<'a> {
    node_id: ast::NodeId,
    compilation: &'a Compilation,
    locations: Vec<Span>,
}

impl<'a> Visitor<'_> for LocalRename<'a> {
    fn visit_pat(&mut self, pat: &'_ ast::Pat) {
        match &*pat.kind {
            ast::PatKind::Bind(ident, _) => {
                if ident.id == self.node_id {
                    self.locations.push(ident.span);
                }
            }
            _ => ast::visit::walk_pat(self, pat),
        }
    }

    fn visit_path(&mut self, path: &'_ ast::Path) {
        let res = self.compilation.current_unit().ast.names.get(path.id);
        if let Some(resolve::Res::Local(node_id)) = res {
            if *node_id == self.node_id {
                self.locations.push(path.name.span);
            }
        }
    }
}

fn resolve_package(local_package_id: hir::PackageId, item_id: &hir::ItemId) -> hir::ItemId {
    hir::ItemId {
        item: item_id.item,
        package: item_id.package.or(Some(local_package_id)),
    }
}
