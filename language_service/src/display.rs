// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::qsc_utils::{find_item, Compilation};
use qsc::{
    ast,
    hir::{self},
};
use std::fmt::{Display, Formatter, Result};

pub(crate) struct CodeDisplay<'a> {
    pub(crate) compilation: &'a Compilation,
}

#[allow(clippy::unused_self)]
impl<'a> CodeDisplay<'a> {
    pub(crate) fn hir_callable_decl(&self, decl: &'a hir::CallableDecl) -> HirCallableDecl {
        HirCallableDecl {
            compilation: self.compilation,
            decl,
        }
    }

    pub(crate) fn ast_callable_decl(&self, decl: &'a ast::CallableDecl) -> AstCallableDecl {
        AstCallableDecl {
            compilation: self.compilation,
            decl,
        }
    }

    pub(crate) fn name_with_ty_id(
        &self,
        name: &'a dyn Display,
        ty_id: ast::NodeId,
    ) -> NameWithTyId {
        NameWithTyId {
            compilation: self.compilation,
            name,
            ty_id,
        }
    }

    pub(crate) fn path_with_ty_id(&self, path: &'a ast::Path, ty_id: ast::NodeId) -> PathWithTyId {
        PathWithTyId {
            compilation: self.compilation,
            path,
            ty_id,
        }
    }

    pub(crate) fn name_with_ty(&self, name: &'a dyn Display, ty: &'a ast::Ty) -> NameWithTy {
        NameWithTy { name, ty }
    }

    pub(crate) fn ident_ty_def(&self, ident: &'a ast::Ident, def: &'a ast::TyDef) -> IdentTyDef {
        IdentTyDef { ident, def }
    }

    pub(crate) fn hir_udt(&self, ident: &'a hir::Ident, udt: &'a hir::ty::Udt) -> HirUdt {
        HirUdt { ident, _udt: udt }
    }

    // The rest of the display implementations are not made public b/c they're not used,
    // but there's no reason they couldn't be
}

// Display impls for each syntax/hir element we may encounter

pub(crate) struct NameWithTy<'a> {
    name: &'a dyn Display,
    ty: &'a ast::Ty,
}

impl<'a> Display for NameWithTy<'a> {
    /// formerly `contents_from_name`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}: {}", self.name, Ty { ty: self.ty },)
    }
}

pub(crate) struct NameWithTyId<'a> {
    compilation: &'a Compilation,
    name: &'a dyn Display,
    ty_id: ast::NodeId,
}

impl<'a> Display for NameWithTyId<'a> {
    /// formerly `contents_from_name`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}: {}",
            self.name,
            TyId {
                ty_id: self.ty_id,
                compilation: self.compilation
            },
        )
    }
}

pub(crate) struct PathWithTyId<'a> {
    compilation: &'a Compilation,
    path: &'a ast::Path,
    ty_id: ast::NodeId,
}

impl<'a> Display for PathWithTyId<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            NameWithTyId {
                compilation: self.compilation,
                name: &Path { path: self.path },
                ty_id: self.ty_id
            }
        )
    }
}

pub(crate) struct HirCallableDecl<'a, 'b> {
    compilation: &'a Compilation,
    decl: &'b hir::CallableDecl,
}

impl Display for HirCallableDecl<'_, '_> {
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
            HirTy {
                ty: &self.decl.input.ty,
                compilation: self.compilation
            },
            arrow,
            HirTy {
                ty: &self.decl.output,
                compilation: self.compilation
            },
            FunctorSetValue {
                functors: self.decl.functors,
            },
        )
    }
}

pub(crate) struct AstCallableDecl<'a> {
    compilation: &'a Compilation,
    decl: &'a ast::CallableDecl,
}

impl<'a> Display for AstCallableDecl<'a> {
    /// formerly `contents_from_ast_call_decl`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let (kind, arrow) = match self.decl.kind {
            ast::CallableKind::Function => ("function", "->"),
            ast::CallableKind::Operation => ("operation", "=>"),
        };

        let functors = ast_callable_functors(self.decl);
        let functors = FunctorSetValue { functors };

        write!(
            f,
            "{} {} {} {} {}{}",
            kind,
            self.decl.name.name,
            TyId {
                ty_id: self.decl.input.id,
                compilation: self.compilation
            },
            arrow,
            Ty {
                ty: &self.decl.output
            },
            functors,
        )
    }
}

pub(crate) struct IdentTyDef<'a> {
    ident: &'a ast::Ident,
    def: &'a ast::TyDef,
}

impl<'a> Display for IdentTyDef<'a> {
    /// formerly `contents_from_ast_udt`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}: {}", self.ident.name, TyDef { def: self.def })
    }
}

pub(crate) struct HirUdt<'a> {
    ident: &'a hir::Ident,
    _udt: &'a hir::ty::Udt,
}

impl<'a> Display for HirUdt<'a> {
    /// formerly `contents_from_hir_udt`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.ident.name)
    }
}

struct FunctorSet<'a> {
    functor_set: &'a hir::ty::FunctorSet,
}

impl<'a> Display for FunctorSet<'a> {
    /// extracted from `contents_from_ast_call_decl`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if *self.functor_set == hir::ty::FunctorSet::Value(hir::ty::FunctorSetValue::Empty) {
            Ok(())
        } else {
            write!(f, " is {}", self.functor_set)
        }
    }
}

struct FunctorSetValue {
    functors: hir::ty::FunctorSetValue,
}

impl Display for FunctorSetValue {
    /// extracted from a few different places
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if let hir::ty::FunctorSetValue::Empty = self.functors {
            Ok(())
        } else {
            write!(f, " is {}", self.functors)
        }
    }
}

struct HirTy<'a> {
    ty: &'a hir::ty::Ty,
    compilation: &'a Compilation,
}

impl<'a> Display for HirTy<'a> {
    /// formerly `get_type_name_from_hir_ty`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        // This is very similar to the Display impl for Ty, except that UDTs are resolved to their names.
        match self.ty {
            hir::ty::Ty::Array(item) => {
                write!(
                    f,
                    "{}[]",
                    HirTy {
                        compilation: self.compilation,
                        ty: item,
                    }
                )
            }
            hir::ty::Ty::Arrow(arrow) => {
                let input = HirTy {
                    compilation: self.compilation,
                    ty: &arrow.input,
                };
                let output = HirTy {
                    compilation: self.compilation,
                    ty: &arrow.output,
                };
                let functors = FunctorSet {
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
                            HirTy {
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

struct TyId<'a> {
    ty_id: ast::NodeId,
    compilation: &'a Compilation,
}

impl<'a> Display for TyId<'a> {
    /// formerly `get_type_name`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if let Some(ty) = self.compilation.unit.ast.tys.terms.get(self.ty_id) {
            write!(
                f,
                "{}",
                HirTy {
                    compilation: self.compilation,
                    ty
                }
            )
        } else {
            write!(f, "?")
        }
    }
}

struct Ty<'a> {
    ty: &'a ast::Ty,
}

impl<'a> Display for Ty<'a> {
    /// formerly `get_type_name_from_ast_ty`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.ty.kind.as_ref() {
            ast::TyKind::Array(ty) => write!(f, "{}[]", Ty { ty }),
            ast::TyKind::Arrow(kind, input, output, functors) => {
                let arrow = match kind {
                    ast::CallableKind::Function => "->",
                    ast::CallableKind::Operation => "=>",
                };
                write!(
                    f,
                    "({} {} {}{})",
                    Ty { ty: input },
                    arrow,
                    Ty { ty: output },
                    FunctorExpr { functors }
                )
            }
            ast::TyKind::Hole => write!(f, "_"),
            ast::TyKind::Paren(ty) => write!(f, "{}", Ty { ty }),
            ast::TyKind::Path(path) => write!(f, "{}", Path { path }),
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
                        write!(f, "{}", Ty { ty: def })?;
                    }
                    write!(f, ")")
                }
            }
        }
    }
}

struct FunctorExpr<'a> {
    functors: &'a Option<Box<ast::FunctorExpr>>,
}

impl<'a> Display for FunctorExpr<'a> {
    /// extracted from `get_type_name_from_ast_ty`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.functors {
            Some(functors) => {
                let functors = eval_functor_expr(functors);
                write!(f, "{}", FunctorSetValue { functors })
            }
            None => Ok(()),
        }
    }
}

struct Path<'a> {
    path: &'a ast::Path,
}

impl<'a> Display for Path<'a> {
    /// formerly `print_path`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.path.namespace.as_ref() {
            Some(ns) => write!(f, "{ns}.{}", self.path.name.name),
            None => write!(f, "{}", self.path.name.name),
        }
    }
}

struct TyDef<'a> {
    def: &'a ast::TyDef,
}

impl<'a> Display for TyDef<'a> {
    /// formerly `ty_def_to_string`
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.def.kind.as_ref() {
            ast::TyDefKind::Field(name, ty) => match name {
                Some(name) => write!(f, "{}: {}", name.name, Ty { ty }),
                None => write!(f, "{}", Ty { ty }),
            },
            ast::TyDefKind::Paren(def) => write!(f, "{}", TyDef { def }),
            ast::TyDefKind::Tuple(tys) => {
                if tys.is_empty() {
                    write!(f, "Unit")
                } else {
                    write!(f, "(")?;
                    for (count, def) in tys.iter().enumerate() {
                        if count != 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", TyDef { def })?;
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
