// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::qsc_utils::{map_offset, span_contains, Compilation};
use qsc::hir::{
    ty::{FunctorSetValue, Ty},
    visit::{walk_callable_decl, walk_expr, walk_item, walk_pat, Visitor},
    CallableDecl, CallableKind, Expr, ExprKind, Ident, Item, LocalItemId, NodeId, Pat, PatKind,
    Res,
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

struct HoverFromItemId {
    item_id: LocalItemId,
    header: Option<String>,
}

impl Visitor<'_> for HoverFromItemId {
    fn visit_item(&mut self, item: &'_ Item) {
        if item.id == self.item_id {
            walk_item(self, item);
        }
    }

    fn visit_callable_decl(&mut self, decl: &'_ CallableDecl) {
        self.header = Some(header_from_call_decl(decl));
    }
}

struct HoverFromNodeId {
    node_id: NodeId,
    header: Option<String>,
}

impl Visitor<'_> for HoverFromNodeId {
    fn visit_pat(&mut self, pat: &'_ Pat) {
        match &pat.kind {
            PatKind::Bind(ident) => {
                if ident.id == self.node_id {
                    self.header = Some(header_from_ident(ident, &pat.ty));
                }
            }
            _ => walk_pat(self, pat),
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
            self.header = Some(header_from_call_decl(decl));
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
                    self.header = Some(header_from_ident(ident, &pat.ty));
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
                        let mut finder_pass = HoverFromItemId {
                            item_id: item_id.item,
                            header: None,
                        };
                        if let Some(package_id) = item_id.package {
                            let foreign_package = &self
                                .compilation
                                .package_store
                                .get(package_id)
                                .unwrap_or_else(|| panic!("bad package id: {package_id}"))
                                .package;
                            finder_pass.visit_package(foreign_package);
                            finder_pass.header
                        } else {
                            finder_pass.visit_package(&self.compilation.package);
                            finder_pass.header
                        }
                    }
                    Res::Local(node_id) => {
                        let mut finder_pass = HoverFromNodeId {
                            node_id: *node_id,
                            header: None,
                        };
                        finder_pass.visit_package(&self.compilation.package);
                        finder_pass.header
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

fn header_from_call_decl(decl: &CallableDecl) -> String {
    let (kind, arrow) = match decl.kind {
        CallableKind::Function => ("function", "->"),
        CallableKind::Operation => ("operation", "=>"),
    };

    let functors = if let FunctorSetValue::Empty = decl.functors {
        String::new()
    } else {
        format!(" is {}", decl.functors)
    };

    let temp = decl.input.ty.to_string();

    // Doc comments would be formatted as markdown into this
    // string once we're able to parse them out.
    format!(
        "```qsharp
{} {} {} {} {}{}
```
",
        kind, decl.name.name, temp, arrow, decl.output, functors
    )
}

fn header_from_ident(ident: &Ident, ty: &Ty) -> String {
    format!(
        "```qsharp
{} {}
```
",
        ident.name, ty,
    )
}
