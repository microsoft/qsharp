// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::qsc_utils::{find_item, map_offset, span_contains, Compilation};
use qsc::ast::visit::{walk_callable_decl, walk_expr, walk_pat, walk_ty_def, Visitor};
use qsc::{ast, hir, resolve};
use std::fmt::Display;

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
    let offset = map_offset(&compilation.unit.sources, source_name, offset);
    let package = &compilation.unit.ast.package;

    let mut hover_visitor = HoverVisitor {
        compilation,
        offset,
        contents: None,
        start: 0,
        end: 0,
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
}

impl Visitor<'_> for HoverVisitor<'_> {
    fn visit_item(&mut self, item: &'_ ast::Item) {
        if span_contains(item.span, self.offset) {
            match &*item.kind {
                ast::ItemKind::Callable(decl) => {
                    if span_contains(decl.name.span, self.offset) {
                        self.contents = Some(if item.doc.is_empty() {
                            self.contents_from_ast_call_decl(decl)
                        } else {
                            format!("{}\n{}", item.doc, self.contents_from_ast_call_decl(decl))
                        });
                        self.start = decl.name.span.lo;
                        self.end = decl.name.span.hi;
                    } else if span_contains(decl.span, self.offset) {
                        walk_callable_decl(self, decl);
                    }
                }
                ast::ItemKind::Ty(ident, def) => {
                    if span_contains(ident.span, self.offset) {
                        self.contents = Some(contents_from_ast_udt(ident, def));
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
                        self.contents = Some(contents_from_name(
                            &ident.name,
                            &get_type_name_from_ast_ty(ty),
                        ));
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
                        self.contents =
                            Some(contents_from_name(&ident.name, &self.get_type_name(pat.id)));
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
                    self.contents = Some(contents_from_name(
                        &field.name,
                        &self.get_type_name(expr.id),
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
                        if let Some(item) = find_item(self.compilation, item_id) {
                            self.contents = match &item.kind {
                                hir::ItemKind::Callable(decl) => Some(if item.doc.is_empty() {
                                    self.contents_from_hir_call_decl(decl)
                                } else {
                                    format!(
                                        "{}\n{}",
                                        item.doc,
                                        self.contents_from_hir_call_decl(decl)
                                    )
                                }),
                                hir::ItemKind::Namespace(_, _) => {
                                    panic!(
                                        "Reference node should not refer to a namespace: {}",
                                        path.id
                                    )
                                }
                                hir::ItemKind::Ty(ident, udt) => {
                                    Some(contents_from_hir_udt(ident, udt))
                                }
                            };
                            self.start = path.span.lo;
                            self.end = path.span.hi;
                        }
                    }
                    resolve::Res::Local(node_id) => {
                        self.contents = Some(contents_from_name(
                            &print_path(path),
                            &self.get_type_name(*node_id),
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

impl HoverVisitor<'_> {
    fn contents_from_ast_call_decl(&mut self, decl: &ast::CallableDecl) -> String {
        let (kind, arrow) = match decl.kind {
            ast::CallableKind::Function => ("function", "->"),
            ast::CallableKind::Operation => ("operation", "=>"),
        };

        let functors = ast_callable_functors(decl);
        let functors = if let hir::ty::FunctorSetValue::Empty = functors {
            String::new()
        } else {
            format!(" is {functors}")
        };

        let inner = format!(
            "{} {} {} {} {}{}",
            kind,
            decl.name.name,
            self.get_type_name(decl.input.id),
            arrow,
            get_type_name_from_ast_ty(&decl.output),
            functors,
        );
        markdown_wrapper(&inner)
    }

    fn contents_from_hir_call_decl(&self, decl: &hir::CallableDecl) -> String {
        let (kind, arrow) = match decl.kind {
            hir::CallableKind::Function => ("function", "->"),
            hir::CallableKind::Operation => ("operation", "=>"),
        };

        let functors = if let hir::ty::FunctorSetValue::Empty = decl.functors {
            String::new()
        } else {
            format!(" is {}", decl.functors)
        };

        let inner = format!(
            "{} {} {} {} {}{}",
            kind,
            decl.name.name,
            self.get_type_name_from_hir_ty(&decl.input.ty),
            arrow,
            self.get_type_name_from_hir_ty(&decl.output),
            functors,
        );
        markdown_wrapper(&inner)
    }

    fn get_type_name(&self, node_id: ast::NodeId) -> String {
        if let Some(ty) = self.compilation.unit.ast.tys.terms.get(node_id) {
            self.get_type_name_from_hir_ty(ty)
        } else {
            "?".to_string()
        }
    }

    // This is very similar to the Display impl for Ty, except that UDTs are resolved to their names.
    fn get_type_name_from_hir_ty(&self, ty: &hir::ty::Ty) -> String {
        match ty {
            hir::ty::Ty::Array(item) => format!("{}[]", self.get_type_name_from_hir_ty(item)),
            hir::ty::Ty::Arrow(arrow) => {
                let input = self.get_type_name_from_hir_ty(&arrow.input);
                let output = self.get_type_name_from_hir_ty(&arrow.output);
                let functors = if arrow.functors
                    == hir::ty::FunctorSet::Value(hir::ty::FunctorSetValue::Empty)
                {
                    String::new()
                } else {
                    format!(" is {}", arrow.functors)
                };
                let arrow = match arrow.kind {
                    hir::CallableKind::Function => "->",
                    hir::CallableKind::Operation => "=>",
                };
                format!("({input} {arrow} {output}{functors})",)
            }
            hir::ty::Ty::Tuple(tys) => {
                if tys.is_empty() {
                    "Unit".to_owned()
                } else {
                    let elements = tys
                        .iter()
                        .map(|e| self.get_type_name_from_hir_ty(e))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("({elements})")
                }
            }
            hir::ty::Ty::Udt(res) => match res {
                hir::Res::Item(item_id) => {
                    if let Some(item) = find_item(self.compilation, item_id) {
                        match &item.kind {
                            hir::ItemKind::Ty(ident, _) => ident.name.to_string(),
                            _ => panic!("UDT has invalid resolution."),
                        }
                    } else {
                        "?".to_string()
                    }
                }
                _ => panic!("UDT has invalid resolution."),
            },
            _ => ty.to_string(),
        }
    }
}

fn contents_from_hir_udt(name: &hir::Ident, _: &hir::ty::Udt) -> String {
    let name = &name.name;
    markdown_wrapper(name)
}

fn contents_from_ast_udt(name: &ast::Ident, def: &ast::TyDef) -> String {
    let name = &name.name;
    let def = ty_def_to_string(def);
    let inner = format!("{name}: {def}");
    markdown_wrapper(&inner)
}

fn ty_def_to_string(def: &ast::TyDef) -> String {
    match &*def.kind {
        ast::TyDefKind::Field(name, ty) => {
            let ty = get_type_name_from_ast_ty(ty);
            match name {
                Some(name) => format!("{}: {ty}", name.name),
                None => ty,
            }
        }
        ast::TyDefKind::Paren(def) => ty_def_to_string(def),
        ast::TyDefKind::Tuple(tys) => {
            if tys.is_empty() {
                "Unit".to_owned()
            } else {
                let elements = tys
                    .iter()
                    .map(|def| ty_def_to_string(def))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({elements})")
            }
        }
    }
}

fn contents_from_name(name: &impl Display, ty_name: &String) -> String {
    markdown_wrapper(&format!("{name}: {ty_name}"))
}

fn markdown_wrapper(contents: &impl Display) -> String {
    format!(
        "```qsharp
{contents}
```
"
    )
}

fn get_type_name_from_ast_ty(ty: &ast::Ty) -> String {
    match &*ty.kind {
        qsc::ast::TyKind::Array(ty) => format!("{}[]", get_type_name_from_ast_ty(ty)),
        qsc::ast::TyKind::Arrow(kind, input, output, functors) => {
            let input = get_type_name_from_ast_ty(input);
            let output = get_type_name_from_ast_ty(output);
            let arrow = match kind {
                ast::CallableKind::Function => "->",
                ast::CallableKind::Operation => "=>",
            };
            let functors = match functors {
                Some(functors) => {
                    let functors = eval_functor_expr(functors);
                    if let hir::ty::FunctorSetValue::Empty = functors {
                        String::new()
                    } else {
                        format!(" is {functors}")
                    }
                }
                None => String::new(),
            };
            format!("({input} {arrow} {output}{functors})")
        }
        qsc::ast::TyKind::Hole => "_".to_owned(),
        qsc::ast::TyKind::Paren(ty) => get_type_name_from_ast_ty(ty),
        qsc::ast::TyKind::Path(path) => print_path(path),
        qsc::ast::TyKind::Param(id) => id.name.to_string(),
        qsc::ast::TyKind::Tuple(tys) => {
            if tys.is_empty() {
                "Unit".to_owned()
            } else {
                let elements = tys
                    .iter()
                    .map(get_type_name_from_ast_ty)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({elements})")
            }
        }
    }
}

fn ast_callable_functors(callable: &ast::CallableDecl) -> hir::ty::FunctorSetValue {
    let mut functors = callable
        .functors
        .as_ref()
        .map_or(hir::ty::FunctorSetValue::Empty, |f| {
            eval_functor_expr(f.as_ref())
        });

    if let ast::CallableBody::Specs(specs) = callable.body.as_ref() {
        for spec in specs.iter() {
            let spec_functors = match spec.spec {
                ast::Spec::Body => hir::ty::FunctorSetValue::Empty,
                ast::Spec::Adj => hir::ty::FunctorSetValue::Adj,
                ast::Spec::Ctl => hir::ty::FunctorSetValue::Ctl,
                ast::Spec::CtlAdj => hir::ty::FunctorSetValue::CtlAdj,
            };
            functors = functors.union(&spec_functors);
        }
    }

    functors
}

fn eval_functor_expr(expr: &ast::FunctorExpr) -> hir::ty::FunctorSetValue {
    match expr.kind.as_ref() {
        ast::FunctorExprKind::BinOp(op, lhs, rhs) => {
            let lhs_functors = eval_functor_expr(lhs);
            let rhs_functors = eval_functor_expr(rhs);
            match op {
                ast::SetOp::Union => lhs_functors.union(&rhs_functors),
                ast::SetOp::Intersect => lhs_functors.intersect(&rhs_functors),
            }
        }
        ast::FunctorExprKind::Lit(ast::Functor::Adj) => hir::ty::FunctorSetValue::Adj,
        ast::FunctorExprKind::Lit(ast::Functor::Ctl) => hir::ty::FunctorSetValue::Ctl,
        ast::FunctorExprKind::Paren(inner) => eval_functor_expr(inner),
    }
}

fn print_path(path: &ast::Path) -> String {
    match &path.namespace {
        Some(ns) => format!("{ns}.{}", path.name.name),
        None => format!("{}", path.name.name),
    }
}
