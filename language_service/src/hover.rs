// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::display::{parse_doc_for_param, parse_doc_for_summary, CodeDisplay};
use crate::protocol::{self, Hover};
use crate::qsc_utils::{find_ident, find_item, map_offset, span_contains, Compilation};
use qsc::ast::visit::{walk_expr, walk_namespace, walk_pat, walk_ty_def, Visitor};
use qsc::{ast, hir, resolve, SourceMap};
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

    let mut hover_visitor = HoverVisitor::new(compilation, offset);

    hover_visitor.visit_package(package);

    hover_visitor.hover
}

enum LocalKind {
    Param,
    LambdaParam,
    Local,
}

struct HoverVisitor<'a> {
    // Input
    compilation: &'a Compilation,
    offset: u32,

    // Output
    hover: Option<Hover>,

    // State
    display: CodeDisplay<'a>,
    current_namespace: Rc<str>,
    current_callable: Option<&'a ast::CallableDecl>,
    in_params: bool,
    lambda_params: Vec<&'a ast::Pat>,
    in_lambda_params: bool,
    current_item_doc: Rc<str>,
}

impl<'a> HoverVisitor<'a> {
    fn new(compilation: &'a Compilation, offset: u32) -> Self {
        Self {
            compilation,
            offset,
            hover: None,
            display: CodeDisplay { compilation },
            current_namespace: Rc::from(""),
            current_callable: None,
            in_params: false,
            lambda_params: vec![],
            in_lambda_params: false,
            current_item_doc: Rc::from(""),
        }
    }
}

impl<'a> Visitor<'a> for HoverVisitor<'a> {
    fn visit_namespace(&mut self, namespace: &'a ast::Namespace) {
        if span_contains(namespace.span, self.offset) {
            self.current_namespace = namespace.name.name.clone();
            walk_namespace(self, namespace);
        }
    }

    fn visit_item(&mut self, item: &'a ast::Item) {
        if span_contains(item.span, self.offset) {
            let context = replace(&mut self.current_item_doc, item.doc.clone());
            match &*item.kind {
                ast::ItemKind::Callable(decl) => {
                    if span_contains(decl.name.span, self.offset) {
                        let contents = display_callable(
                            &item.doc,
                            &self.current_namespace,
                            self.display.ast_callable_decl(decl),
                        );
                        self.hover = Some(Hover {
                            contents,
                            span: protocol_span(decl.name.span, &self.compilation.unit.sources),
                        });
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
                        let contents = markdown_fenced_block(self.display.ident_ty_def(ident, def));
                        self.hover = Some(Hover {
                            contents,
                            span: protocol_span(ident.span, &self.compilation.unit.sources),
                        });
                    } else {
                        self.visit_ty_def(def);
                    }
                }
                _ => {}
            }
            self.current_item_doc = context;
        }
    }

    fn visit_spec_decl(&mut self, decl: &'a ast::SpecDecl) {
        // Walk Spec Decl
        match &decl.body {
            ast::SpecBody::Gen(_) => {}
            ast::SpecBody::Impl(pat, block) => {
                self.in_params = true;
                self.visit_pat(pat);
                self.in_params = false;
                self.visit_block(block);
            }
        }
    }

    fn visit_ty_def(&mut self, def: &'a ast::TyDef) {
        if span_contains(def.span, self.offset) {
            if let ast::TyDefKind::Field(ident, ty) = &*def.kind {
                if let Some(ident) = ident {
                    if span_contains(ident.span, self.offset) {
                        let contents = markdown_fenced_block(self.display.ident_ty(ident, ty));
                        self.hover = Some(Hover {
                            contents,
                            span: protocol_span(ident.span, &self.compilation.unit.sources),
                        });
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
                        let kind = if self.in_params {
                            LocalKind::Param
                        } else if self.in_lambda_params {
                            LocalKind::LambdaParam
                        } else {
                            LocalKind::Local
                        };
                        let mut callable_name = Rc::from("");
                        if let Some(decl) = self.current_callable {
                            callable_name = decl.name.name.clone();
                        }
                        let contents = display_local(
                            &kind,
                            &code,
                            &ident.name,
                            &callable_name,
                            &self.current_item_doc,
                        );
                        self.hover = Some(Hover {
                            contents,
                            span: protocol_span(ident.span, &self.compilation.unit.sources),
                        });
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
                    let contents = markdown_fenced_block(self.display.ident_ty_id(field, expr.id));
                    self.hover = Some(Hover {
                        contents,
                        span: protocol_span(field.span, &self.compilation.unit.sources),
                    });
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
                                .map_or_else(
                                    || Rc::from(""),
                                    |parent| match &parent.kind {
                                        qsc::hir::ItemKind::Namespace(namespace, _) => {
                                            namespace.name.clone()
                                        }
                                        _ => Rc::from(""),
                                    },
                                );

                            let contents = match &item.kind {
                                hir::ItemKind::Callable(decl) => display_callable(
                                    &item.doc,
                                    &ns,
                                    self.display.hir_callable_decl(decl),
                                ),
                                hir::ItemKind::Namespace(_, _) => {
                                    panic!(
                                        "Reference node should not refer to a namespace: {}",
                                        path.id
                                    )
                                }
                                hir::ItemKind::Ty(_, udt) => {
                                    markdown_fenced_block(self.display.hir_udt(udt))
                                }
                            };
                            self.hover = Some(Hover {
                                contents,
                                span: protocol_span(path.span, &self.compilation.unit.sources),
                            });
                        }
                    }
                    resolve::Res::Local(node_id) => {
                        let mut local_name = Rc::from("");
                        let mut callable_name = Rc::from("");
                        if let Some(curr) = self.current_callable {
                            callable_name = curr.name.name.clone();
                            if let Some(ident) = find_ident(node_id, curr) {
                                local_name = ident.name.clone();
                            }
                        }

                        let code = markdown_fenced_block(self.display.path_ty_id(path, *node_id));
                        let kind = if is_param(
                            &curr_callable_to_params(self.current_callable),
                            *node_id,
                        ) {
                            LocalKind::Param
                        } else if is_param(&self.lambda_params, *node_id) {
                            LocalKind::LambdaParam
                        } else {
                            LocalKind::Local
                        };
                        let contents = display_local(
                            &kind,
                            &code,
                            &local_name,
                            &callable_name,
                            &self.current_item_doc,
                        );
                        self.hover = Some(Hover {
                            contents,
                            span: protocol_span(path.span, &self.compilation.unit.sources),
                        });
                    }
                    _ => {}
                };
            }
        }
    }
}

fn protocol_span(span: qsc::Span, source_map: &SourceMap) -> protocol::Span {
    // Note that lo and hi offsets will usually be the same as
    // the span will usually come from a single source.
    let lo_offset = source_map
        .find_by_offset(span.lo)
        .expect("source should exist for offset")
        .offset;
    let hi_offset = source_map
        .find_by_offset(span.hi)
        .expect("source should exist for offset")
        .offset;
    protocol::Span {
        start: span.lo - lo_offset,
        end: span.hi - hi_offset,
    }
}

fn curr_callable_to_params(curr_callable: Option<&ast::CallableDecl>) -> Vec<&ast::Pat> {
    match curr_callable {
        Some(decl) => match &*decl.body {
            ast::CallableBody::Block(_) => vec![decl.input.as_ref()],
            ast::CallableBody::Specs(spec_decls) => {
                let mut pats = spec_decls
                    .iter()
                    .filter_map(|spec| match &spec.body {
                        ast::SpecBody::Gen(_) => None,
                        ast::SpecBody::Impl(input, _) => Some(input.as_ref()),
                    })
                    .collect::<Vec<&ast::Pat>>();
                pats.push(decl.input.as_ref());
                pats
            }
        },
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

fn display_local(
    param_kind: &LocalKind,
    markdown: &String,
    local_name: &str,
    callable_name: &str,
    callable_doc: &str,
) -> String {
    match param_kind {
        LocalKind::Param => {
            let param_doc = parse_doc_for_param(callable_doc, local_name);
            with_doc(
                &param_doc,
                format!("parameter of `{callable_name}`\n{markdown}",),
            )
        }
        LocalKind::LambdaParam => format!("lambda parameter\n{markdown}"),
        LocalKind::Local => format!("local\n{markdown}"),
    }
}

fn display_callable(doc: &str, namespace: &str, code: impl Display) -> String {
    let summary = parse_doc_for_summary(doc);

    let mut code = if namespace.is_empty() {
        code.to_string()
    } else {
        format!("{namespace}\n{code}")
    };
    code = markdown_fenced_block(code);
    with_doc(&summary, code)
}

fn with_doc(doc: &String, code: impl Display) -> String {
    if doc.is_empty() {
        code.to_string()
    } else {
        format!("{code}---\n{doc}\n")
    }
}

fn markdown_fenced_block(code: impl Display) -> String {
    format!(
        "```qsharp
{code}
```
"
    )
}
