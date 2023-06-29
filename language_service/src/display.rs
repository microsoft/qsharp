// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::qsc_utils::{find_item, Compilation};
use qsc::{
    ast,
    hir::{self},
};
use std::fmt::{Display, Formatter, Result};

pub(crate) struct Formatted<'a> {
    pub(crate) compilation: &'a Compilation,
}

impl<'a> Formatted<'a> {}

pub(crate) struct FormattedNameWithTy<'a> {
    pub(crate) name: &'a dyn Display,
    pub(crate) ty: &'a ast::Ty,
}

impl<'a> Display for FormattedNameWithTy<'a> {
    /// formerly `contents_from_name`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}: {}", self.name, FormattedTy { ty: self.ty },)
    }
}

pub(crate) struct FormattedNameWithTyId<'a> {
    pub(crate) compilation: &'a Compilation,
    pub(crate) name: &'a dyn Display,
    pub(crate) ty_id: ast::NodeId,
}

impl<'a> Display for FormattedNameWithTyId<'a> {
    /// formerly `contents_from_name`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}: {}",
            self.name,
            FormattedTyId {
                ty_id: self.ty_id,
                compilation: self.compilation
            },
        )
    }
}

pub(crate) struct FormattedHirCallableDecl<'a> {
    pub(crate) compilation: &'a Compilation,
    pub(crate) decl: &'a hir::CallableDecl,
}

impl<'a> Display for FormattedHirCallableDecl<'a> {
    /// formerly `contents_from_hir_call_decl`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let (kind, arrow) = match self.decl.kind {
            hir::CallableKind::Function => ("function", "->"),
            hir::CallableKind::Operation => ("operation", "=>"),
        };

        write!(
            f,
            "{} {} {} {} {}{}",
            kind,
            self.decl.name.name,
            FormattedHirTy {
                ty: &self.decl.input.ty,
                compilation: self.compilation
            },
            arrow,
            FormattedHirTy {
                ty: &self.decl.output,
                compilation: self.compilation
            },
            FormattedFunctorSetValue {
                functors: self.decl.functors,
            },
        )
    }
}

pub(crate) struct FormattedAstCallableDecl<'a> {
    pub(crate) compilation: &'a Compilation,
    pub(crate) decl: &'a ast::CallableDecl,
}

impl<'a> Display for FormattedAstCallableDecl<'a> {
    /// formerly `contents_from_ast_call_decl`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let (kind, arrow) = match self.decl.kind {
            ast::CallableKind::Function => ("function", "->"),
            ast::CallableKind::Operation => ("operation", "=>"),
        };

        let functors = ast_callable_functors(self.decl);
        let functors = FormattedFunctorSetValue { functors };

        write!(
            f,
            "{} {} {} {} {}{}",
            kind,
            self.decl.name.name,
            FormattedTyId {
                ty_id: self.decl.input.id,
                compilation: self.compilation
            },
            arrow,
            FormattedTy {
                ty: &self.decl.output
            },
            functors,
        )
    }
}

pub(crate) struct FormattedFunctorSet<'a> {
    pub(crate) functor_set: &'a hir::ty::FunctorSet,
}

impl<'a> Display for FormattedFunctorSet<'a> {
    /// extracted from `contents_from_ast_call_decl`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if *self.functor_set == hir::ty::FunctorSet::Value(hir::ty::FunctorSetValue::Empty) {
            Ok(())
        } else {
            write!(f, " is {}", self.functor_set)
        }
    }
}

pub(crate) struct FormattedFunctorSetValue {
    pub(crate) functors: hir::ty::FunctorSetValue,
}

impl Display for FormattedFunctorSetValue {
    /// extracted from a few different places
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if let hir::ty::FunctorSetValue::Empty = self.functors {
            Ok(())
        } else {
            write!(f, " is {}", self.functors)
        }
    }
}

pub(crate) struct FormattedHirTy<'a> {
    pub(crate) ty: &'a hir::ty::Ty,
    pub(crate) compilation: &'a Compilation,
}

impl<'a> Display for FormattedHirTy<'a> {
    /// formerly `get_type_name_from_hir_ty`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        // This is very similar to the Display impl for Ty, except that UDTs are resolved to their names.
        match self.ty {
            hir::ty::Ty::Array(item) => {
                write!(
                    f,
                    "{}[]",
                    FormattedHirTy {
                        compilation: self.compilation,
                        ty: item,
                    }
                )
            }
            hir::ty::Ty::Arrow(arrow) => {
                let input = FormattedHirTy {
                    compilation: self.compilation,
                    ty: &arrow.input,
                };
                let output = FormattedHirTy {
                    compilation: self.compilation,
                    ty: &arrow.output,
                };
                let functors = FormattedFunctorSet {
                    functor_set: &arrow.functors,
                };
                let arrow = match arrow.kind {
                    hir::CallableKind::Function => "->",
                    hir::CallableKind::Operation => "=>",
                };
                write!(f, "({input} {arrow} {output}{functors})",)
            }
            hir::ty::Ty::Tuple(tys) => {
                if tys.is_empty() {
                    write!(f, "Unit")
                } else {
                    write!(f, "(")?;
                    for (count, ty) in tys.iter().enumerate() {
                        if count != 0 {
                            write!(f, ", ")?;
                        }
                        write!(
                            f,
                            "{}",
                            FormattedHirTy {
                                compilation: self.compilation,
                                ty
                            }
                        )?;
                    }
                    write!(f, ")")
                }
            }
            hir::ty::Ty::Udt(res) => match res {
                hir::Res::Item(item_id) => {
                    if let Some(item) = find_item(self.compilation, item_id) {
                        match &item.kind {
                            hir::ItemKind::Ty(ident, _) => write!(f, "{}", ident.name),
                            _ => panic!("UDT has invalid resolution."),
                        }
                    } else {
                        write!(f, "?")
                    }
                }
                _ => panic!("UDT has invalid resolution."),
            },
            _ => write!(f, "{}", self.ty),
        }
    }
}

pub(crate) struct FormattedTyId<'a> {
    pub(crate) ty_id: ast::NodeId,
    pub(crate) compilation: &'a Compilation,
}

impl<'a> Display for FormattedTyId<'a> {
    /// formerly `get_type_name`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if let Some(ty) = self.compilation.unit.ast.tys.terms.get(self.ty_id) {
            write!(
                f,
                "{}",
                FormattedHirTy {
                    compilation: self.compilation,
                    ty
                }
            )
        } else {
            write!(f, "?")
        }
    }
}

pub(crate) struct FormattedHirUdt<'a> {
    pub(crate) ident: &'a hir::Ident,
    pub(crate) _udt: &'a hir::ty::Udt,
}

impl<'a> Display for FormattedHirUdt<'a> {
    /// formerly `contents_from_hir_udt`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.ident.name)
    }
}

pub(crate) struct FormattedTy<'a> {
    pub(crate) ty: &'a ast::Ty,
}

impl<'a> Display for FormattedTy<'a> {
    /// formerly `get_type_name_from_ast_ty`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.ty.kind.as_ref() {
            ast::TyKind::Array(ty) => write!(f, "{}[]", FormattedTy { ty }),
            ast::TyKind::Arrow(kind, input, output, functors) => {
                let arrow = match kind {
                    ast::CallableKind::Function => "->",
                    ast::CallableKind::Operation => "=>",
                };
                write!(
                    f,
                    "({} {} {}{})",
                    FormattedTy { ty: input },
                    arrow,
                    FormattedTy { ty: output },
                    FormattedFunctorExpr { functors }
                )
            }
            ast::TyKind::Hole => write!(f, "_"),
            ast::TyKind::Paren(ty) => write!(f, "{}", FormattedTy { ty }),
            ast::TyKind::Path(path) => write!(f, "{}", FormattedPath { path }),
            ast::TyKind::Param(id) => write!(f, "{}", id.name),
            ast::TyKind::Tuple(tys) => {
                if tys.is_empty() {
                    write!(f, "Unit")
                } else {
                    write!(f, "(")?;
                    for (count, def) in tys.iter().enumerate() {
                        if count != 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", FormattedTy { ty: def })?;
                    }
                    write!(f, ")")
                }
            }
        }
    }
}

struct FormattedFunctorExpr<'a> {
    functors: &'a Option<Box<ast::FunctorExpr>>,
}

impl<'a> Display for FormattedFunctorExpr<'a> {
    /// extracted from `get_type_name_from_ast_ty`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.functors {
            Some(functors) => {
                let functors = eval_functor_expr(functors);
                write!(f, "{}", FormattedFunctorSetValue { functors })
            }
            None => Ok(()),
        }
    }
}

pub(crate) struct FormattedPath<'a> {
    pub(crate) path: &'a ast::Path,
}

impl<'a> Display for FormattedPath<'a> {
    /// formerly `print_path`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.path.namespace.as_ref() {
            Some(ns) => write!(f, "{ns}.{}", self.path.name.name),
            None => write!(f, "{}", self.path.name.name),
        }
    }
}

pub(crate) struct FormattedIdentTyDef<'a> {
    pub(crate) ident: &'a ast::Ident,
    pub(crate) def: &'a ast::TyDef,
}

impl<'a> Display for FormattedIdentTyDef<'a> {
    /// formerly `contents_from_ast_udt`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}: {}",
            self.ident.name,
            FormattedTyDef { def: self.def }
        )
    }
}

struct FormattedTyDef<'a> {
    def: &'a ast::TyDef,
}

impl<'a> Display for FormattedTyDef<'a> {
    /// formerly `ty_def_to_string`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.def.kind.as_ref() {
            ast::TyDefKind::Field(name, ty) => match name {
                Some(name) => write!(f, "{}: {}", name.name, FormattedTy { ty }),
                None => write!(f, "{}", FormattedTy { ty }),
            },
            ast::TyDefKind::Paren(def) => write!(f, "{}", FormattedTyDef { def }),
            ast::TyDefKind::Tuple(tys) => {
                if tys.is_empty() {
                    write!(f, "Unit")
                } else {
                    write!(f, "(")?;
                    for (count, def) in tys.iter().enumerate() {
                        if count != 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", FormattedTyDef { def })?;
                    }
                    write!(f, ")")
                }
            }
        }
    }
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
