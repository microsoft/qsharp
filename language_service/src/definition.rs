// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::protocol::Definition;
use crate::qsc_utils::{
    find_item, map_offset, span_contains, Compilation, QSHARP_LIBRARY_URI_SCHEME,
};
use qsc::ast::visit::{walk_callable_decl, walk_expr, walk_pat, walk_ty_def, Visitor};
use qsc::hir::PackageId;
use qsc::{ast, hir, resolve};

pub(crate) fn get_definition(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Option<Definition> {
    // Map the file offset into a SourceMap offset
    let offset = map_offset(&compilation.unit.sources, source_name, offset);
    let ast_package = &compilation.unit.ast;

    let mut definition_finder = DefinitionFinder {
        compilation,
        offset,
        definition: None,
        curr_callable: None,
    };
    definition_finder.visit_package(&ast_package.package);

    definition_finder
        .definition
        .map(|(name, offset)| Definition {
            source: name,
            offset,
        })
}

struct DefinitionFinder<'a> {
    compilation: &'a Compilation,
    offset: u32,
    definition: Option<(String, u32)>,
    curr_callable: Option<&'a ast::CallableDecl>,
}

impl DefinitionFinder<'_> {
    fn set_definition_from_position(&mut self, lo: u32, package_id: Option<PackageId>) {
        let source_map = match package_id {
            Some(id) => {
                &self
                    .compilation
                    .package_store
                    .get(id)
                    .unwrap_or_else(|| panic!("package should exist for id {id}"))
                    .sources
            }
            None => &self.compilation.unit.sources,
        };
        let source = source_map
            .find_by_offset(lo)
            .expect("source should exist for offset");
        // Note: Having a package_id means the position references a foreign package.
        // Currently the only supported foreign packages are our library packages,
        // URI's to which need to include our custom library scheme.
        let source_name = match package_id {
            Some(_) => format!("{}:{}", QSHARP_LIBRARY_URI_SCHEME, source.name),
            None => source.name.to_string(),
        };

        self.definition = Some((source_name, lo - source.offset));
    }
}

impl<'a> Visitor<'a> for DefinitionFinder<'a> {
    // Handles callable and UDT definitions
    fn visit_item(&mut self, item: &'a ast::Item) {
        if span_contains(item.span, self.offset) {
            match &*item.kind {
                ast::ItemKind::Callable(decl) => {
                    if span_contains(decl.name.span, self.offset) {
                        self.set_definition_from_position(decl.name.span.lo, None);
                    } else if span_contains(decl.span, self.offset) {
                        self.curr_callable = Some(decl);
                        walk_callable_decl(self, decl);
                        self.curr_callable = None;
                    }
                    // Note: the `item.span` can cover things like doc
                    // comment, attributes, and visibility keywords, which aren't
                    // things we want to have hover logic for, while the `decl.span` is
                    // specific to the contents of the callable decl, which we do want
                    // hover logic for. If the `if` or `else if` above is not met, then
                    // the user is hovering over one of these non-decl parts of the item,
                    // and we want to do nothing.
                }
                ast::ItemKind::Ty(ident, def) => {
                    if span_contains(ident.span, self.offset) {
                        self.set_definition_from_position(ident.span.lo, None);
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
                        self.set_definition_from_position(ident.span.lo, None);
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
                        self.set_definition_from_position(ident.span.lo, None);
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
                                                self.set_definition_from_position(
                                                    span.lo,
                                                    item_id.package,
                                                );
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
                        if let (Some(item), _) = find_item(self.compilation, item_id) {
                            let lo = match &item.kind {
                                hir::ItemKind::Callable(decl) => decl.name.span.lo,
                                hir::ItemKind::Namespace(_, _) => {
                                    panic!(
                                        "Reference node should not refer to a namespace: {}",
                                        path.id
                                    )
                                }
                                hir::ItemKind::Ty(ident, _) => ident.span.lo,
                            };
                            self.set_definition_from_position(lo, item_id.package);
                        };
                    }
                    resolve::Res::Local(node_id) => {
                        if let Some(curr) = self.curr_callable {
                            {
                                let mut finder = AstPatFinder {
                                    node_id,
                                    result: None,
                                };
                                finder.visit_callable_decl(curr);
                                if let Some(lo) = finder.result {
                                    self.set_definition_from_position(lo, None);
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

struct AstPatFinder<'a> {
    node_id: &'a ast::NodeId,
    result: Option<u32>,
}

impl<'a> Visitor<'a> for AstPatFinder<'_> {
    fn visit_pat(&mut self, pat: &'a ast::Pat) {
        match &*pat.kind {
            ast::PatKind::Bind(ident, _) => {
                if ident.id == *self.node_id {
                    self.result = Some(ident.span.lo);
                }
            }
            _ => walk_pat(self, pat),
        }
    }

    fn visit_expr(&mut self, expr: &'a ast::Expr) {
        if self.result.is_none() {
            walk_expr(self, expr);
        }
    }
}
