// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::fmt::Display;

use crate::qsc_utils::{find_item, map_offset, span_contains, Compilation};
use qsc::ast::visit::{walk_callable_decl, walk_expr, walk_pat, walk_ty_def, Visitor};
use qsc::ast::{self, CallableDecl, CallableKind, Expr, ExprKind, NodeId, Pat, PatKind, Path};
use qsc::{hir, resolve};

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

struct HoverVisitor<'a> {
    compilation: &'a Compilation,
    offset: u32,
    header: Option<String>,
    start: u32,
    end: u32,
}

impl Visitor<'_> for HoverVisitor<'_> {
    fn visit_item(&mut self, item: &'_ ast::Item) {
        if span_contains(item.span, self.offset) {
            match &*item.kind {
                ast::ItemKind::Callable(decl) => self.visit_callable_decl(decl),
                ast::ItemKind::Ty(ident, def) => {
                    // ToDo: UDTs should show their description
                    if span_contains(ident.span, self.offset) {
                        self.header = Some(header_from(&ident.name.to_string()));
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
                        self.header = Some(header_from_name(
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

    fn visit_callable_decl(&mut self, decl: &'_ CallableDecl) {
        if span_contains(decl.name.span, self.offset) {
            self.header = Some(self.header_from_ast_call_decl(decl));
            self.start = decl.name.span.lo;
            self.end = decl.name.span.hi;
        } else if span_contains(decl.span, self.offset) {
            walk_callable_decl(self, decl);
        }
    }

    fn visit_pat(&mut self, pat: &'_ Pat) {
        if span_contains(pat.span, self.offset) {
            match &*pat.kind {
                PatKind::Bind(ident, anno) => {
                    if span_contains(ident.span, self.offset) {
                        self.header =
                            Some(header_from_name(&ident.name, &self.get_type_name(pat.id)));
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

    fn visit_expr(&mut self, expr: &'_ Expr) {
        if span_contains(expr.span, self.offset) {
            match &*expr.kind {
                ExprKind::Field(_, field) if span_contains(field.span, self.offset) => {
                    self.header = Some(header_from_name(&field.name, &self.get_type_name(expr.id)));
                    self.start = field.span.lo;
                    self.end = field.span.hi;
                }
                _ => walk_expr(self, expr),
            }
        }
    }

    fn visit_path(&mut self, path: &'_ Path) {
        if span_contains(path.span, self.offset) {
            let res = self
                .compilation
                .unit
                .ast
                .names
                .get(path.id)
                .unwrap_or_else(|| panic!("Can't find definition for reference node: {}", path.id));
            match &res {
                resolve::Res::Item(item_id) => {
                    let item = find_item(self.compilation, item_id).unwrap_or_else(|| {
                        panic!("Can't find definition for reference node: {}", path.id)
                    });
                    self.header = match &item.kind {
                        hir::ItemKind::Callable(decl) => Some(self.header_from_hir_call_decl(decl)),
                        hir::ItemKind::Namespace(_, _) => {
                            panic!(
                                "Reference node should not refer to a namespace: {}",
                                path.id
                            )
                        }
                        hir::ItemKind::Ty(ident, udt) => Some(header_from_hir_udt(ident, udt)),
                    };
                    self.start = path.span.lo;
                    self.end = path.span.hi;
                }
                resolve::Res::Local(node_id) => {
                    self.header = Some(header_from_name(
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

impl HoverVisitor<'_> {
    fn header_from_ast_call_decl(&mut self, decl: &ast::CallableDecl) -> String {
        let (kind, arrow) = match decl.kind {
            CallableKind::Function => ("function", "->"),
            CallableKind::Operation => ("operation", "=>"),
        };

        // ToDo: Functors
        // let functors = if let FunctorSetValue::Empty = decl.functors {
        //     String::new()
        // } else {
        //     format!(" is {}", decl.functors)
        // };

        // Doc comments would be formatted as markdown into this
        // string once we're able to parse them out.
        format!(
            "```qsharp
{} {} {} {} {}
```
",
            kind,
            decl.name.name,
            self.get_type_name(decl.input.id),
            arrow,
            get_type_name_from_ast_ty(&decl.output),
        )
    }

    fn header_from_hir_call_decl(&self, decl: &hir::CallableDecl) -> String {
        let (kind, arrow) = match decl.kind {
            hir::CallableKind::Function => ("function", "->"),
            hir::CallableKind::Operation => ("operation", "=>"),
        };

        // ToDo: Functors
        // let functors = if let FunctorSetValue::Empty = decl.functors {
        //     String::new()
        // } else {
        //     format!(" is {}", decl.functors)
        // };

        // Doc comments would be formatted as markdown into this
        // string once we're able to parse them out.
        format!(
            "```qsharp
{} {} {} {} {}
```
",
            kind,
            decl.name.name,
            self.get_type_name_from_hir_ty(&decl.input.ty),
            arrow,
            self.get_type_name_from_hir_ty(&decl.output),
        )
    }

    fn get_type_name(&self, node_id: NodeId) -> String {
        let ty = self
            .compilation
            .unit
            .ast
            .tys
            .terms
            .get(node_id)
            .unwrap_or_else(|| panic!("Can't find type for node: {node_id}"));
        self.get_type_name_from_hir_ty(ty)
    }

    // This is very similar to the Display impl for Ty, except that UDTs are resolved to their names.
    fn get_type_name_from_hir_ty(&self, ty: &hir::ty::Ty) -> String {
        match ty {
            hir::ty::Ty::Array(item) => format!("{}[]", self.get_type_name_from_hir_ty(item)),
            hir::ty::Ty::Arrow(arrow) => {
                format!(
                    "({} {} {})",
                    self.get_type_name_from_hir_ty(&arrow.input),
                    match arrow.kind {
                        hir::CallableKind::Function => "->",
                        hir::CallableKind::Operation => "=>",
                    },
                    self.get_type_name_from_hir_ty(&arrow.output)
                )
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
                    let item = find_item(self.compilation, item_id)
                        .unwrap_or_else(|| panic!("Can't find type with item id: {item_id}"));
                    match &item.kind {
                        hir::ItemKind::Ty(ident, _) => ident.name.to_string(),
                        _ => panic!("UDT has invalid resolution."),
                    }
                }
                _ => panic!("UDT has invalid resolution."),
            },
            _ => ty.to_string(),
        }
    }
}

// ToDo: display more info for UDTs
fn header_from_hir_udt(name: &hir::Ident, _: &hir::ty::Udt) -> String {
    format!(
        "```qsharp
{}
```
",
        name.name
    )
}

fn header_from_name(name: &impl Display, ty_name: &String) -> String {
    format!(
        "```qsharp
{name}: {ty_name}
```
"
    )
}

fn header_from(display: &impl Display) -> String {
    format!(
        "```qsharp
{display}
```
"
    )
}

fn get_type_name_from_ast_ty(ty: &ast::Ty) -> String {
    match &*ty.kind {
        qsc::ast::TyKind::Array(ty) => format!("{}[]", get_type_name_from_ast_ty(ty)),
        qsc::ast::TyKind::Arrow(kind, input, output, _) => {
            let input = get_type_name_from_ast_ty(input);
            let output = get_type_name_from_ast_ty(output);
            let arrow = match kind {
                CallableKind::Function => "->",
                CallableKind::Operation => "=>",
            };
            format!("({input} {arrow} {output})")
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

fn print_path(path: &Path) -> String {
    match &path.namespace {
        Some(ns) => format!("{ns}.{}", path.name.name),
        None => format!("{}", path.name.name),
    }
}
