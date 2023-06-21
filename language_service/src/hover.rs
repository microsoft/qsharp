// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::rc::Rc;

use crate::qsc_utils::{map_offset, span_contains, Compilation};
use qsc::hir::{
    ty::{FunctorSetValue, Ty},
    visit::{walk_callable_decl, walk_expr, walk_item, walk_pat, Visitor},
    CallableDecl, CallableKind, Expr, ExprKind, Ident, Item, ItemKind, LocalItemId, NodeId, Pat,
    PatKind, Res,
};

#[derive(Debug, PartialEq)]
pub struct Hover {
    pub contents: String,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

pub(crate) fn get_hover(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Option<Hover> {
    // Map the file offset into a SourceMap offset
    let offset = map_offset(&compilation.source_map, source_name, offset);
    let package = &compilation.package;

    let mut hover_visitor = HoverVisitor {
        compilation,
        offset,
        header: None,
        start: 0,
        end: 0,
    };
    hover_visitor.visit_package(package);

    hover_visitor.header.map(|header| Hover {
        contents: header,
        span: Span {
            start: hover_visitor.start,
            end: hover_visitor.end,
        },
    })
}

struct FindDeclFromItemId<'a> {
    item_id: LocalItemId,
    decl: Option<&'a CallableDecl>,
}

impl<'a> Visitor<'a> for FindDeclFromItemId<'a> {
    fn visit_item(&mut self, item: &'a Item) {
        if item.id == self.item_id {
            walk_item(self, item);
        }
    }

    fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
        self.decl = Some(decl);
    }
}

struct FindIdentFromNodeId<'a> {
    node_id: NodeId,
    ident: Option<(&'a Ident, &'a Ty)>,
}

impl<'a> Visitor<'a> for FindIdentFromNodeId<'a> {
    fn visit_pat(&mut self, pat: &'a Pat) {
        match &pat.kind {
            PatKind::Bind(ident) => {
                if ident.id == self.node_id {
                    self.ident = Some((ident, &pat.ty));
                }
            }
            _ => walk_pat(self, pat),
        }
    }
}

struct FindTypeNameFromItemId {
    item_id: LocalItemId,
    ty_name: Option<Rc<str>>,
}

impl Visitor<'_> for FindTypeNameFromItemId {
    fn visit_item(&mut self, item: &'_ Item) {
        if item.id == self.item_id {
            if let ItemKind::Ty(name, _) = &item.kind {
                self.ty_name = Some(name.name.clone());
            }
        }
    }
}

struct HoverVisitor<'a> {
    compilation: &'a Compilation,
    offset: u32,
    header: Option<String>,
    start: u32,
    end: u32,
}

impl Visitor<'_> for HoverVisitor<'_> {
    fn visit_callable_decl(&mut self, decl: &'_ CallableDecl) {
        if span_contains(decl.name.span, self.offset) {
            self.header = Some(self.header_from_call_decl(decl));
            self.start = decl.name.span.lo;
            self.end = decl.name.span.hi;
        } else if span_contains(decl.span, self.offset) {
            walk_callable_decl(self, decl);
        }
    }

    fn visit_pat(&mut self, pat: &'_ Pat) {
        if span_contains(pat.span, self.offset) {
            match &pat.kind {
                PatKind::Bind(ident) => {
                    self.header = Some(self.header_from_ident(ident, &pat.ty));
                    self.start = pat.span.lo;
                    self.end = pat.span.hi;
                }
                _ => walk_pat(self, pat),
            }
        }
    }

    fn visit_expr(&mut self, expr: &'_ Expr) {
        if span_contains(expr.span, self.offset) {
            if let ExprKind::Var(r, _) = &expr.kind {
                self.header = match r {
                    Res::Err => None,
                    Res::Item(item_id) => {
                        let mut finder_pass = FindDeclFromItemId {
                            item_id: item_id.item,
                            decl: None,
                        };
                        let decl = if let Some(package_id) = item_id.package {
                            let foreign_package = &self
                                .compilation
                                .package_store
                                .get(package_id)
                                .unwrap_or_else(|| panic!("bad package id: {package_id}"))
                                .package;
                            finder_pass.visit_package(foreign_package);
                            finder_pass.decl
                        } else {
                            finder_pass.visit_package(&self.compilation.package);
                            finder_pass.decl
                        };
                        decl.map(|decl| self.header_from_call_decl(decl))
                    }
                    Res::Local(node_id) => {
                        let mut finder_pass = FindIdentFromNodeId {
                            node_id: *node_id,
                            ident: None,
                        };
                        finder_pass.visit_package(&self.compilation.package);
                        finder_pass
                            .ident
                            .map(|(ident, ty)| self.header_from_ident(ident, ty))
                    }
                };
                self.start = expr.span.lo;
                self.end = expr.span.hi;
            } else {
                walk_expr(self, expr);
            }
        }
    }
}

impl HoverVisitor<'_> {
    fn header_from_call_decl(&mut self, decl: &CallableDecl) -> String {
        let (kind, arrow) = match decl.kind {
            CallableKind::Function => ("function", "->"),
            CallableKind::Operation => ("operation", "=>"),
        };

        let functors = if let FunctorSetValue::Empty = decl.functors {
            String::new()
        } else {
            format!(" is {}", decl.functors)
        };

        // Doc comments would be formatted as markdown into this
        // string once we're able to parse them out.
        format!(
            "```qsharp
{} {} {} {} {}{}
```
",
            kind,
            decl.name.name,
            self.get_type_name(&decl.input.ty),
            arrow,
            self.get_type_name(&decl.output),
            functors
        )
    }

    fn header_from_ident(&mut self, ident: &Ident, ty: &Ty) -> String {
        format!(
            "```qsharp
{} {}
```
",
            ident.name,
            self.get_type_name(ty),
        )
    }

    fn get_type_name(&mut self, ty: &Ty) -> String {
        match ty {
            Ty::Udt(res) => match res {
                Res::Item(item_id) => {
                    let mut finder_pass = FindTypeNameFromItemId {
                        item_id: item_id.item,
                        ty_name: None,
                    };
                    let ty_name = if let Some(package_id) = item_id.package {
                        let foreign_package = &self
                            .compilation
                            .package_store
                            .get(package_id)
                            .unwrap_or_else(|| panic!("bad package id: {package_id}"))
                            .package;
                        finder_pass.visit_package(foreign_package);
                        finder_pass.ty_name
                    } else {
                        finder_pass.visit_package(&self.compilation.package);
                        finder_pass.ty_name
                    };
                    match ty_name {
                        Some(rc) => rc.to_string(),
                        None => ty.to_string(),
                    }
                }
                _ => panic!("UDT has invalid item id."),
            },
            _ => ty.to_string(),
        }
    }
}
