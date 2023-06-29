// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::qsc_utils::{find_item, Compilation};
use qsc::{ast, hir};
use std::fmt::Display;

pub(crate) struct Formatter<'a> {
    pub compilation: &'a Compilation,
}

impl<'a> Formatter<'a> {
    /// formerly `get_type_name_from_hir_ty`
    pub fn format_hir_ty(&self, ty: &hir::ty::Ty) -> String {
        // This is very similar to the Display impl for Ty, except that UDTs are resolved to their names.
        match ty {
            hir::ty::Ty::Array(item) => {
                format!("{}[]", self.format_hir_ty(item))
            }
            hir::ty::Ty::Arrow(arrow) => {
                let input = self.format_hir_ty(&arrow.input);
                let output = self.format_hir_ty(&arrow.output);
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
                        .map(|e| self.format_hir_ty(e))
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

    /// formerly `contents_from_hir_call_decl`
    pub fn format_hir_callable_decl(&self, decl: &hir::CallableDecl) -> String {
        let (kind, arrow) = match decl.kind {
            hir::CallableKind::Function => ("function", "->"),
            hir::CallableKind::Operation => ("operation", "=>"),
        };

        let functors = if let hir::ty::FunctorSetValue::Empty = decl.functors {
            String::new()
        } else {
            format!(" is {}", decl.functors)
        };

        format!(
            "{} {} {} {} {}{}",
            kind,
            decl.name.name,
            self.format_hir_ty(&decl.input.ty),
            arrow,
            self.format_hir_ty(&decl.output),
            functors,
        )
    }

    /// formerly `get_type_name`
    pub fn format_ast_ty(&self, node_id: ast::NodeId) -> String {
        if let Some(ty) = self.compilation.unit.ast.tys.terms.get(node_id) {
            self.format_hir_ty(ty)
        } else {
            "?".to_string()
        }
    }

    /// formerly `contents_from_ast_call_decl`
    pub fn format_ast_callable_decl(&self, decl: &ast::CallableDecl) -> String {
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

        format!(
            "{} {} {} {} {}{}",
            kind,
            decl.name.name,
            self.format_ast_ty(decl.input.id),
            arrow,
            format_ty(&decl.output),
            functors,
        )
    }
}

/// formerly `get_type_name_from_ast_ty`
pub fn format_ty(ty: &ast::Ty) -> String {
    match &*ty.kind {
        qsc::ast::TyKind::Array(ty) => format!("{}[]", format_ty(ty)),
        qsc::ast::TyKind::Arrow(kind, input, output, functors) => {
            let input = format_ty(input);
            let output = format_ty(output);
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
        qsc::ast::TyKind::Paren(ty) => format_ty(ty),
        qsc::ast::TyKind::Path(path) => format_path(path),
        qsc::ast::TyKind::Param(id) => id.name.to_string(),
        qsc::ast::TyKind::Tuple(tys) => {
            if tys.is_empty() {
                "Unit".to_owned()
            } else {
                let elements = tys.iter().map(format_ty).collect::<Vec<_>>().join(", ");
                format!("({elements})")
            }
        }
    }
}

/// formerly `print_path`
pub fn format_path(path: &ast::Path) -> String {
    match &path.namespace {
        Some(ns) => format!("{ns}.{}", path.name.name),
        None => format!("{}", path.name.name),
    }
}

/// formerly `contents_from_hir_udt`
pub fn format_hir_udt(name: &hir::Ident, _: &hir::ty::Udt) -> String {
    name.name.to_string()
}

/// formerly `contents_from_ast_udt`
pub fn format_udt(name: &ast::Ident, def: &ast::TyDef) -> String {
    let name = &name.name;
    let def = format_ty_def(def);
    format!("{name}: {def}")
}

/// formerly `ty_def_to_string`
fn format_ty_def(def: &ast::TyDef) -> String {
    match &*def.kind {
        ast::TyDefKind::Field(name, ty) => {
            let ty = format_ty(ty);
            match name {
                Some(name) => format!("{}: {ty}", name.name),
                None => ty,
            }
        }
        ast::TyDefKind::Paren(def) => format_ty_def(def),
        ast::TyDefKind::Tuple(tys) => {
            if tys.is_empty() {
                "Unit".to_owned()
            } else {
                let elements = tys
                    .iter()
                    .map(|def| format_ty_def(def))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({elements})")
            }
        }
    }
}

/// formerly `contents_from_name`
pub fn format_name(name: &impl Display, ty_name: &String) -> String {
    format!("{name}: {ty_name}")
}

//
// helpers that don't manipulate any strings
//

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
