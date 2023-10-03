// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use qsc::{
    ast::{
        self,
        visit::{walk_callable_decl, walk_expr, walk_pat, walk_ty, walk_ty_def, Visitor},
    },
    hir, resolve, Span,
};

use crate::{
    protocol,
    qsc_utils::{find_item, map_offset, Compilation},
};

pub(crate) fn get_rename(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Vec<protocol::Span> {
    // Map the file offset into a SourceMap offset
    let offset = map_offset(&compilation.unit.sources, source_name, offset);
    let package = &compilation.unit.ast.package;

    let mut rename_visitor = Rename::new(compilation, offset);
    rename_visitor.visit_package(package);
    rename_visitor
        .locations
        .into_iter()
        .map(|i| protocol::Span {
            start: i.lo,
            end: i.hi,
        })
        .collect::<Vec<_>>()
}

struct Rename<'a> {
    compilation: &'a Compilation,
    offset: u32,
    current_callable: Option<&'a ast::CallableDecl>,
    locations: Vec<Span>,
}

impl<'a> Rename<'a> {
    fn new(compilation: &'a Compilation, offset: u32) -> Self {
        Self {
            compilation,
            offset,
            current_callable: None,
            locations: vec![],
        }
    }
}

fn span_contains(span: Span, offset: u32) -> bool {
    offset >= span.lo && offset <= span.hi
}

fn get_spans_for_item_rename(item_id: &hir::ItemId, compilation: &Compilation) -> Vec<Span> {
    // Only rename items that are part of the local package
    if item_id.package.is_none() {
        if let Some(def) = compilation.unit.package.items.get(item_id.item) {
            let def_span = match &def.kind {
                hir::ItemKind::Callable(decl) => decl.name.span,
                hir::ItemKind::Namespace(name, _) | hir::ItemKind::Ty(name, _) => name.span,
            };
            let mut rename = ItemRename {
                item_id,
                compilation,
                locations: vec![],
            };
            rename.visit_package(&compilation.unit.ast.package);
            rename.locations.push(def_span);
            rename.locations
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

struct ItemRename<'a> {
    item_id: &'a hir::ItemId,
    compilation: &'a Compilation,
    locations: Vec<Span>,
}

impl<'a> Visitor<'_> for ItemRename<'a> {
    fn visit_path(&mut self, path: &'_ ast::Path) {
        // ToDo: Handle Namespace Renames
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

fn ast_item_id_to_hir_item_id(
    item_id: ast::NodeId,
    compilation: &Compilation,
) -> Option<&hir::ItemId> {
    if let Some(resolve::Res::Item(item_id)) = compilation.unit.ast.names.get(item_id) {
        Some(item_id)
    } else {
        None
    }
}

impl<'a> Visitor<'a> for Rename<'a> {
    // Handles callable and UDT definitions
    fn visit_item(&mut self, item: &'a ast::Item) {
        if span_contains(item.span, self.offset) {
            match &*item.kind {
                ast::ItemKind::Callable(decl) => {
                    if span_contains(decl.name.span, self.offset) {
                        if let Some(item_id) =
                            ast_item_id_to_hir_item_id(decl.name.id, self.compilation)
                        {
                            self.locations = get_spans_for_item_rename(item_id, self.compilation);
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
                    if span_contains(ident.span, self.offset) {
                        if let Some(item_id) =
                            ast_item_id_to_hir_item_id(ident.id, self.compilation)
                        {
                            self.locations = get_spans_for_item_rename(item_id, self.compilation);
                        }
                    } else {
                        self.visit_ty_def(def);
                    }
                }
                _ => {}
            }
        }
    }

    // Handles UDT field definitions
    fn visit_ty_def(&mut self, def: &'a ast::TyDef) {
        if span_contains(def.span, self.offset) {
            if let ast::TyDefKind::Field(ident, ty) = &*def.kind {
                if let Some(ident) = ident {
                    if span_contains(ident.span, self.offset) {
                        // ToDo: Handle UDT Field Name
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
    fn visit_pat(&mut self, pat: &'a ast::Pat) {
        if span_contains(pat.span, self.offset) {
            match &*pat.kind {
                ast::PatKind::Bind(ident, anno) => {
                    if span_contains(ident.span, self.offset) {
                        if let Some(curr) = self.current_callable {
                            self.locations =
                                get_spans_for_local_rename(ident.id, curr, self.compilation);
                        }
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
        if span_contains(expr.span, self.offset) {
            match &*expr.kind {
                ast::ExprKind::Field(udt, field) if span_contains(field.span, self.offset) => {
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
                                                let span = field.name_span.expect(
                                                    "field found via name should have a name",
                                                );

                                                // ToDo: Handle UDT Field Refs
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
        if span_contains(path.span, self.offset) {
            let res = self.compilation.unit.ast.names.get(path.id);
            if let Some(res) = res {
                match &res {
                    resolve::Res::Item(item_id) => {
                        self.locations = get_spans_for_item_rename(item_id, self.compilation);
                    }
                    resolve::Res::Local(node_id) => {
                        if let Some(curr) = self.current_callable {
                            self.locations =
                                get_spans_for_local_rename(*node_id, curr, self.compilation);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn get_spans_for_local_rename(
    node_id: ast::NodeId,
    decl: &ast::CallableDecl,
    compilation: &Compilation,
) -> Vec<Span> {
    let mut rename = LocalRename {
        node_id,
        compilation,
        locations: vec![],
    };

    rename.visit_callable_decl(decl);

    rename.locations
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
