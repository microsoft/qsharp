// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::display::CodeDisplay;
use crate::qsc_utils::{find_item, map_offset, span_contains, Compilation};
use qsc::ast::visit::{
    walk_callable_decl, walk_expr, walk_namespace, walk_pat, walk_ty_def, Visitor,
};
use qsc::{ast, hir, resolve};
use regex_lite::Regex;
use std::fmt::Display;
use std::rc::Rc;

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

struct Documentation {
    summary: String,
}

pub(crate) fn get_hover(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Option<Hover> {
    // Map the file offset into a SourceMap offset
    let offset = map_offset(&compilation.unit.sources, source_name, offset);
    let package = &compilation.unit.ast.package;

    let mut hover_visitor = HoverVisitor {
        compilation,
        offset,
        contents: None,
        start: 0,
        end: 0,
        display: CodeDisplay { compilation },
        current_namespace: None,
    };

    hover_visitor.visit_package(package);

    hover_visitor.contents.map(|contents| Hover {
        contents,
        span: Span {
            start: hover_visitor.start,
            end: hover_visitor.end,
        },
    })
}

struct HoverVisitor<'a> {
    compilation: &'a Compilation,
    offset: u32,
    contents: Option<String>,
    start: u32,
    end: u32,
    display: CodeDisplay<'a>,
    current_namespace: Option<Rc<str>>,
}

impl Visitor<'_> for HoverVisitor<'_> {
    fn visit_namespace(&mut self, namespace: &'_ ast::Namespace) {
        if span_contains(namespace.span, self.offset) {
            self.current_namespace = Some(namespace.name.name.clone());
            walk_namespace(self, namespace);
        }
    }

    fn visit_item(&mut self, item: &'_ ast::Item) {
        if span_contains(item.span, self.offset) {
            match &*item.kind {
                ast::ItemKind::Callable(decl) => {
                    if span_contains(decl.name.span, self.offset) {
                        self.contents = Some(markdown_with_doc(
                            &item.doc,
                            self.current_namespace.clone(),
                            self.display.ast_callable_decl(decl),
                        ));
                        self.start = decl.name.span.lo;
                        self.end = decl.name.span.hi;
                    } else if span_contains(decl.span, self.offset) {
                        walk_callable_decl(self, decl);
                    }
                }
                ast::ItemKind::Ty(ident, def) => {
                    if span_contains(ident.span, self.offset) {
                        self.contents =
                            Some(markdown_fenced_block(self.display.ident_ty_def(ident, def)));
                        self.start = ident.span.lo;
                        self.end = ident.span.hi;
                    } else {
                        self.visit_ty_def(def);
                    }
                }
                _ => {}
            }
        }
    }

    fn visit_ty_def(&mut self, def: &'_ ast::TyDef) {
        if span_contains(def.span, self.offset) {
            if let ast::TyDefKind::Field(ident, ty) = &*def.kind {
                if let Some(ident) = ident {
                    if span_contains(ident.span, self.offset) {
                        self.contents =
                            Some(markdown_fenced_block(self.display.ident_ty(ident, ty)));
                        self.start = ident.span.lo;
                        self.end = ident.span.hi;
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

    fn visit_pat(&mut self, pat: &'_ ast::Pat) {
        if span_contains(pat.span, self.offset) {
            match &*pat.kind {
                ast::PatKind::Bind(ident, anno) => {
                    if span_contains(ident.span, self.offset) {
                        self.contents = Some(markdown_fenced_block(
                            self.display.ident_ty_id(ident, pat.id),
                        ));
                        self.start = ident.span.lo;
                        self.end = ident.span.hi;
                    } else if let Some(ty) = anno {
                        self.visit_ty(ty);
                    }
                }
                _ => walk_pat(self, pat),
            }
        }
    }

    fn visit_expr(&mut self, expr: &'_ ast::Expr) {
        if span_contains(expr.span, self.offset) {
            match &*expr.kind {
                ast::ExprKind::Field(_, field) if span_contains(field.span, self.offset) => {
                    self.contents = Some(markdown_fenced_block(
                        self.display.ident_ty_id(field, expr.id),
                    ));
                    self.start = field.span.lo;
                    self.end = field.span.hi;
                }
                _ => walk_expr(self, expr),
            }
        }
    }

    fn visit_path(&mut self, path: &'_ ast::Path) {
        if span_contains(path.span, self.offset) {
            let res = self.compilation.unit.ast.names.get(path.id);
            if let Some(res) = res {
                match &res {
                    resolve::Res::Item(item_id) => {
                        if let (Some(item), Some(package)) = find_item(self.compilation, item_id) {
                            let ns = item
                                .parent
                                .and_then(|parent_id| package.items.get(parent_id))
                                .and_then(|parent| match &parent.kind {
                                    qsc::hir::ItemKind::Namespace(namespace, _) => {
                                        Some(namespace.name.clone())
                                    }
                                    _ => None,
                                });

                            self.contents = match &item.kind {
                                hir::ItemKind::Callable(decl) => Some(markdown_with_doc(
                                    &item.doc,
                                    ns,
                                    self.display.hir_callable_decl(decl),
                                )),
                                hir::ItemKind::Namespace(_, _) => {
                                    panic!(
                                        "Reference node should not refer to a namespace: {}",
                                        path.id
                                    )
                                }
                                hir::ItemKind::Ty(_, udt) => {
                                    Some(markdown_fenced_block(self.display.hir_udt(udt)))
                                }
                            };
                            self.start = path.span.lo;
                            self.end = path.span.hi;
                        }
                    }
                    resolve::Res::Local(node_id) => {
                        self.contents = Some(markdown_fenced_block(
                            self.display.path_ty_id(path, *node_id),
                        ));
                        self.start = path.span.lo;
                        self.end = path.span.hi;
                    }
                    _ => {}
                };
            }
        }
    }
}

fn markdown_with_doc(doc: &str, namespace: Option<Rc<str>>, code: impl Display) -> String {
    let parsed_doc = parse_doc(doc);

    let code = match namespace {
        Some(namespace) if !namespace.is_empty() => {
            markdown_fenced_block(format!("{namespace}\n{code}"))
        }
        _ => markdown_fenced_block(code),
    };

    if parsed_doc.summary.is_empty() {
        code
    } else {
        format!("{}---\n{}\n", code, parsed_doc.summary,)
    }
}

fn parse_doc(doc: &str) -> Documentation {
    let re = Regex::new(r"(?mi)(?:^# Summary$)([\s\S]*?)(?:(^# .*)|\z)").expect("Invalid regex");
    let summary = match re.captures(doc) {
        Some(captures) => {
            let capture = captures
                .get(1)
                .expect("Didn't find the capture for the given regex");
            capture.as_str()
        }
        None => doc,
    }
    .trim()
    .to_string();

    Documentation { summary }
}

fn markdown_fenced_block(code: impl Display) -> String {
    format!(
        "```qsharp
{code}
```
"
    )
}
