// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::rc::Rc;

use qsc::{
    ast::{
        self,
        visit::{walk_callable_decl, walk_expr, walk_pat, walk_ty, walk_ty_def, Visitor},
    },
    hir::{self, ty::Ty, Res},
    resolve, Span,
};

use crate::{
    protocol,
    qsc_utils::{map_offset, protocol_span, span_contains, span_touches, Compilation},
};

pub(crate) fn prepare_rename(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Option<(protocol::Span, String)> {
    // Map the file offset into a SourceMap offset
    let offset = map_offset(&compilation.unit.sources, source_name, offset);
    let package = &compilation.unit.ast.package;

    let mut prepare_rename = Rename::new(compilation, offset, true);
    prepare_rename.visit_package(package);
    prepare_rename
        .prepare
        .map(|p| (protocol_span(p.0, &compilation.unit.sources), p.1))
}

pub(crate) fn get_rename(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Vec<protocol::Span> {
    // Map the file offset into a SourceMap offset
    let offset = map_offset(&compilation.unit.sources, source_name, offset);
    let package = &compilation.unit.ast.package;

    let mut rename_visitor = Rename::new(compilation, offset, false);
    rename_visitor.visit_package(package);
    rename_visitor
        .locations
        .into_iter()
        .map(|s| protocol_span(s, &compilation.unit.sources))
        .collect::<Vec<_>>()
}

struct Rename<'a> {
    compilation: &'a Compilation,
    offset: u32,
    current_callable: Option<&'a ast::CallableDecl>,
    current_udt_id: Option<&'a hir::ItemId>,
    locations: Vec<Span>,
    is_prepare: bool,
    prepare: Option<(Span, String)>,
}

impl<'a> Rename<'a> {
    fn new(compilation: &'a Compilation, offset: u32, is_prepare: bool) -> Self {
        Self {
            compilation,
            offset,
            current_callable: None,
            current_udt_id: None,
            locations: vec![],
            is_prepare,
            prepare: None,
        }
    }

    fn get_spans_for_item_rename(&mut self, item_id: &hir::ItemId, ast_name: &ast::Ident) {
        // Only rename items that are part of the local package
        if item_id.package.is_none() {
            if let Some(def) = self.compilation.unit.package.items.get(item_id.item) {
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
                    rename.visit_package(&self.compilation.unit.ast.package);
                    rename.locations.push(def_span);
                    self.locations = rename.locations;
                }
            }
        }
    }

    fn get_spans_for_field_rename(&mut self, item_id: &hir::ItemId, ast_name: &ast::Ident) {
        // Only rename items that are part of the local package
        if item_id.package.is_none() {
            if let Some(def) = self.compilation.unit.package.items.get(item_id.item) {
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
                            rename.visit_package(&self.compilation.unit.ast.package);
                            rename.locations.push(def_span);
                            self.locations = rename.locations;
                        }
                    }
                }
            }
        }
    }

    fn get_spans_for_local_rename(&mut self, node_id: ast::NodeId, ast_name: &ast::Ident) {
        if let Some(curr) = self.current_callable {
            if self.is_prepare {
                self.prepare = Some((ast_name.span, ast_name.name.to_string()));
            } else {
                let mut rename = LocalRename {
                    node_id,
                    compilation: self.compilation,
                    locations: vec![],
                };
                rename.visit_callable_decl(curr);
                self.locations = rename.locations;
            }
        }
    }
}

impl<'a> Visitor<'a> for Rename<'a> {
    // Handles callable and UDT definitions
    fn visit_item(&mut self, item: &'a ast::Item) {
        if span_contains(item.span, self.offset) {
            match &*item.kind {
                ast::ItemKind::Callable(decl) => {
                    if span_touches(decl.name.span, self.offset) {
                        if let Some(resolve::Res::Item(item_id)) =
                            self.compilation.unit.ast.names.get(decl.name.id)
                        {
                            self.get_spans_for_item_rename(item_id, &decl.name);
                        }
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
                    if let Some(resolve::Res::Item(item_id)) =
                        self.compilation.unit.ast.names.get(ident.id)
                    {
                        if span_touches(ident.span, self.offset) {
                            self.get_spans_for_item_rename(item_id, ident);
                        } else if span_contains(def.span, self.offset) {
                            let context = self.current_udt_id;
                            self.current_udt_id = Some(item_id);
                            self.visit_ty_def(def);
                            self.current_udt_id = context;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Handles UDT field definitions
    fn visit_ty_def(&mut self, def: &'a ast::TyDef) {
        if let ast::TyDefKind::Field(ident, ty) = &*def.kind {
            if let Some(ident) = ident {
                if span_touches(ident.span, self.offset) {
                    if let Some(item_id) = self.current_udt_id {
                        self.get_spans_for_field_rename(item_id, ident);
                    }
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

    // Handles local variable definitions
    fn visit_pat(&mut self, pat: &'a ast::Pat) {
        if span_touches(pat.span, self.offset) {
            match &*pat.kind {
                ast::PatKind::Bind(ident, anno) => {
                    if span_touches(ident.span, self.offset) {
                        self.get_spans_for_local_rename(ident.id, ident);
                    } else if let Some(ty) = anno {
                        self.visit_ty(ty);
                    }
                }
                _ => walk_pat(self, pat),
            }
        }
    }

    // Handles UDT field references
    fn visit_expr(&mut self, expr: &'a ast::Expr) {
        if span_touches(expr.span, self.offset) {
            match &*expr.kind {
                ast::ExprKind::Field(udt, field) if span_touches(field.span, self.offset) => {
                    if let Some(hir::ty::Ty::Udt(res)) =
                        self.compilation.unit.ast.tys.terms.get(udt.id)
                    {
                        match res {
                            hir::Res::Item(item_id) => {
                                self.get_spans_for_field_rename(item_id, field);
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
                        self.get_spans_for_item_rename(item_id, &path.name);
                    }
                    resolve::Res::Local(node_id) => {
                        self.get_spans_for_local_rename(*node_id, &path.name);
                    }
                    _ => {}
                }
            }
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
        let res = self.compilation.unit.ast.names.get(path.id);
        if let Some(resolve::Res::Item(item_id)) = res {
            if *item_id == *self.item_id {
                self.locations.push(path.name.span);
            }
        }
    }

    fn visit_ty(&mut self, ty: &'_ ast::Ty) {
        if let ast::TyKind::Path(ty_path) = &*ty.kind {
            let res = self.compilation.unit.ast.names.get(ty_path.id);
            if let Some(resolve::Res::Item(item_id)) = res {
                if *item_id == *self.item_id {
                    self.locations.push(ty_path.name.span);
                }
            }
        } else {
            walk_ty(self, ty);
        }
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
                if let Some(Ty::Udt(Res::Item(id))) =
                    self.compilation.unit.ast.tys.terms.get(qualifier.id)
                {
                    if id == self.item_id {
                        self.locations.push(field_name.span);
                    }
                }
            }
        } else {
            walk_expr(self, expr);
        }
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
        let res = self.compilation.unit.ast.names.get(path.id);
        if let Some(resolve::Res::Local(node_id)) = res {
            if *node_id == self.node_id {
                self.locations.push(path.name.span);
            }
        }
    }
}
