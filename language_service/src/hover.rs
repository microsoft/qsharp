// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::display::CodeDisplay;
use crate::protocol::{self, Hover};
use crate::qsc_utils::{find_item, map_offset, span_contains, Compilation};
use qsc::ast::visit::{walk_expr, walk_namespace, walk_pat, walk_ty_def, Visitor};
use qsc::{ast, hir, resolve};
use regex_lite::Regex;
use std::fmt::Display;
use std::mem::replace;
use std::rc::Rc;

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
        current_callable: None,
        in_params: false,
        lambda_params: vec![],
        in_lambda_params: false,
        current_item_doc: Rc::from(""),
    };

    hover_visitor.visit_package(package);

    hover_visitor.contents.map(|contents| Hover {
        contents,
        span: protocol::Span {
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
    current_callable: Option<&'a ast::CallableDecl>,
    in_params: bool,
    lambda_params: Vec<&'a ast::Pat>,
    in_lambda_params: bool,
    current_item_doc: Rc<str>,
}

impl<'a> Visitor<'a> for HoverVisitor<'a> {
    fn visit_namespace(&mut self, namespace: &'a ast::Namespace) {
        if span_contains(namespace.span, self.offset) {
            self.current_namespace = Some(namespace.name.name.clone());
            walk_namespace(self, namespace);
        }
    }

    fn visit_item(&mut self, item: &'a ast::Item) {
        if span_contains(item.span, self.offset) {
            let context = replace(&mut self.current_item_doc, item.doc.clone());
            match &*item.kind {
                ast::ItemKind::Callable(decl) => {
                    if span_contains(decl.name.span, self.offset) {
                        self.contents = Some(display_callable(
                            &item.doc,
                            self.current_namespace.clone(),
                            self.display.ast_callable_decl(decl),
                        ));
                        self.start = decl.name.span.lo;
                        self.end = decl.name.span.hi;
                    } else if span_contains(decl.span, self.offset) {
                        let context = self.current_callable;
                        self.current_callable = Some(decl);

                        // walk callable decl
                        decl.generics.iter().for_each(|p| self.visit_ident(p));
                        self.in_params = true;
                        self.visit_pat(&decl.input);
                        self.in_params = false;
                        self.visit_ty(&decl.output);
                        match &*decl.body {
                            ast::CallableBody::Block(block) => self.visit_block(block),
                            ast::CallableBody::Specs(specs) => {
                                specs.iter().for_each(|s| self.visit_spec_decl(s));
                            }
                        }

                        self.current_callable = context;
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
            self.current_item_doc = context;
        }
    }

    fn visit_ty_def(&mut self, def: &'a ast::TyDef) {
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

    fn visit_pat(&mut self, pat: &'a ast::Pat) {
        if span_contains(pat.span, self.offset) {
            match &*pat.kind {
                ast::PatKind::Bind(ident, anno) => {
                    if span_contains(ident.span, self.offset) {
                        let code = markdown_fenced_block(self.display.ident_ty_id(ident, pat.id));
                        if self.in_params {
                            match self.current_callable {
                                Some(decl) => {
                                    self.contents =
                                        Some(format!("param of `{}`\n{code}", decl.name.name));
                                }
                                None => self.contents = Some(format!("param\n{code}")),
                            }
                        } else if self.in_lambda_params {
                            self.contents = Some(format!("lambda param\n{code}"));
                        } else {
                            self.contents = Some(format!("local\n{code}"));
                        }
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

    fn visit_expr(&mut self, expr: &'a ast::Expr) {
        if span_contains(expr.span, self.offset) {
            match &*expr.kind {
                ast::ExprKind::Field(_, field) if span_contains(field.span, self.offset) => {
                    self.contents = Some(markdown_fenced_block(
                        self.display.ident_ty_id(field, expr.id),
                    ));
                    self.start = field.span.lo;
                    self.end = field.span.hi;
                }
                ast::ExprKind::Lambda(_, pat, expr) => {
                    self.in_lambda_params = true;
                    self.visit_pat(pat);
                    self.in_lambda_params = false;
                    self.lambda_params.push(pat);
                    self.visit_expr(expr);
                    self.lambda_params.pop();
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
                                hir::ItemKind::Callable(decl) => Some(display_callable(
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
                        let code = markdown_fenced_block(self.display.path_ty_id(path, *node_id));
                        if is_param(&curr_callable_to_params(self.current_callable), *node_id) {
                            match self.current_callable {
                                Some(decl) => {
                                    self.contents = Some(display_param(
                                        self.current_item_doc.clone(),
                                        "param_name",
                                        self.display.path_ty_id(path, *node_id),
                                    ));
                                    //Some(format!("param of `{}`\n{code}", decl.name.name));
                                }
                                None => self.contents = Some(format!("param\n{code}")),
                            }
                        } else if is_param(&self.lambda_params, *node_id) {
                            self.contents = Some(format!("lambda param\n{code}"));
                        } else {
                            self.contents = Some(format!("local\n{code}"));
                        }
                        self.start = path.span.lo;
                        self.end = path.span.hi;
                    }
                    _ => {}
                };
            }
        }
    }
}

fn curr_callable_to_params(curr_callable: Option<&ast::CallableDecl>) -> Vec<&ast::Pat> {
    match curr_callable {
        Some(decl) => vec![decl.input.as_ref()],
        None => vec![],
    }
}

fn is_param(param_pats: &[&ast::Pat], node_id: ast::NodeId) -> bool {
    fn find_in_pat(pat: &ast::Pat, node_id: ast::NodeId) -> bool {
        match &*pat.kind {
            ast::PatKind::Bind(ident, _) => node_id == ident.id,
            ast::PatKind::Discard(_) | ast::PatKind::Elided => false,
            ast::PatKind::Paren(inner) => find_in_pat(inner, node_id),
            ast::PatKind::Tuple(inner) => inner.iter().any(|x| find_in_pat(x, node_id)),
        }
    }

    param_pats.iter().any(|pat| find_in_pat(pat, node_id))
}

fn display_callable(doc: &str, namespace: Option<Rc<str>>, code: impl Display) -> String {
    let summary = parse_doc_for_summary(doc);

    let code = match namespace {
        Some(namespace) if !namespace.is_empty() => {
            format!("{namespace}\n{code}")
        }
        _ => code.to_string(),
    };

    markdown_with_doc(&summary, code)
}

fn display_param(doc: Rc<str>, param_name: &str, code: impl Display) -> String {
    let param = parse_doc_for_param(doc, param_name);

    markdown_with_doc(&param, code)
}

fn markdown_with_doc(doc: &String, code: impl Display) -> String {
    let code = markdown_fenced_block(code);
    if doc.is_empty() {
        code
    } else {
        format!("{code}---\n{doc}\n")
    }
}

fn parse_doc_for_summary(doc: &str) -> String {
    let re = Regex::new(r"(?mi)(?:^# Summary$)([\s\S]*?)(?:(^# .*)|\z)").expect("Invalid regex");
    match re.captures(doc) {
        Some(captures) => {
            let capture = captures
                .get(1)
                .expect("Didn't find the capture for the given regex");
            capture.as_str()
        }
        None => doc,
    }
    .trim()
    .to_string()
}

fn parse_doc_for_param(doc: Rc<str>, param: &str) -> String {
    "this is a param doc".to_string()
}

fn markdown_fenced_block(code: impl Display) -> String {
    format!(
        "```qsharp
{code}
```
"
    )
}
