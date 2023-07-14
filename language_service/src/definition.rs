// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::qsc_utils::{find_item, map_offset, span_contains, Compilation};
use qsc::ast::visit::{walk_callable_decl, walk_expr, walk_pat, walk_ty_def, Visitor};
use qsc::SourceMap;
use qsc::{ast, hir, resolve};

#[derive(Debug, PartialEq)]
pub struct Definition {
    pub source: String,
    pub offset: u32,
}

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
        source_map: &compilation.unit.sources,
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
    source_map: &'a SourceMap,
    offset: u32,
    definition: Option<(String, u32)>,
    curr_callable: Option<&'a ast::CallableDecl>,
}

impl DefinitionFinder<'_> {
    fn set_definition_from_position(&mut self, lo: u32) {
        self.definition = Some((
            self.source_map
                .find_by_offset(lo)
                .expect("source should exist for offset")
                .name
                .to_string(),
            lo,
        ));
    }
}

impl<'a> Visitor<'a> for DefinitionFinder<'a> {
    // Handles callable and UDT definitions
    fn visit_item(&mut self, item: &'a ast::Item) {
        if span_contains(item.span, self.offset) {
            match &*item.kind {
                ast::ItemKind::Callable(decl) => {
                    if span_contains(decl.name.span, self.offset) {
                        self.set_definition_from_position(decl.name.span.lo);
                    } else if span_contains(decl.span, self.offset) {
                        self.curr_callable = Some(decl);
                        walk_callable_decl(self, decl);
                        self.curr_callable = None;
                    }
                }
                ast::ItemKind::Ty(ident, def) => {
                    if span_contains(ident.span, self.offset) {
                        self.set_definition_from_position(ident.span.lo);
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
                        self.set_definition_from_position(ident.span.lo);
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
                        self.set_definition_from_position(ident.span.lo);
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
                ast::ExprKind::Field(_, field) if span_contains(field.span, self.offset) => {
                    self.set_definition_from_position(field.span.lo);
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
                        if let Some(item) = find_item(self.compilation, item_id) {
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
                            self.set_definition_from_position(lo);
                        };
                    }
                    resolve::Res::Local(node_id) => {
                        if let Some(curr) = self.curr_callable {
                            {
                                let mut finder = AstFindNode {
                                    node_id,
                                    result: None,
                                };
                                finder.visit_callable_decl(curr);
                                if let Some(lo) = finder.result {
                                    self.set_definition_from_position(lo);
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

struct AstFindNode<'a> {
    node_id: &'a ast::NodeId,
    result: Option<u32>,
}

impl<'a> Visitor<'a> for AstFindNode<'_> {
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
