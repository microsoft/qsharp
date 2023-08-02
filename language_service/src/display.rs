// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::qsc_utils::{find_item, Compilation};
use qsc::{
    ast,
    hir::{self},
};
use std::{
    fmt::{Display, Formatter, Result},
    rc::Rc,
};

pub(crate) struct CodeDisplay<'a> {
    pub(crate) compilation: &'a Compilation,
}

#[allow(clippy::unused_self)]
impl<'a> CodeDisplay<'a> {
    pub(crate) fn hir_callable_decl(&self, decl: &'a hir::CallableDecl) -> impl Display + 'a {
        HirCallableDecl {
            compilation: self.compilation,
            decl,
        }
    }

    pub(crate) fn ast_callable_decl(&self, decl: &'a ast::CallableDecl) -> impl Display + 'a {
        AstCallableDecl {
            compilation: self.compilation,
            decl,
        }
    }

    pub(crate) fn ident_ty_id(
        &self,
        ident: &'a ast::Ident,
        ty_id: ast::NodeId,
    ) -> impl Display + 'a {
        IdentTyId {
            compilation: self.compilation,
            ident,
            ty_id,
        }
    }

    pub(crate) fn path_ty_id(&self, path: &'a ast::Path, ty_id: ast::NodeId) -> impl Display + 'a {
        PathTyId {
            compilation: self.compilation,
            path,
            ty_id,
        }
    }

    pub(crate) fn ident_ty(&self, ident: &'a ast::Ident, ty: &'a ast::Ty) -> impl Display + 'a {
        IdentTy { ident, ty }
    }

    pub(crate) fn ident_ty_def(
        &self,
        ident: &'a ast::Ident,
        def: &'a ast::TyDef,
    ) -> impl Display + 'a {
        IdentTyDef { ident, def }
    }

    pub(crate) fn hir_udt(&self, udt: &'a hir::ty::Udt) -> impl Display + 'a {
        HirUdt {
            compilation: self.compilation,
            udt,
        }
    }

    // The rest of the display implementations are not made public b/c they're not used,
    // but there's no reason they couldn't be
}

// Display impls for each syntax/hir element we may encounter

struct IdentTy<'a> {
    ident: &'a ast::Ident,
    ty: &'a ast::Ty,
}

impl<'a> Display for IdentTy<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}: {}", self.ident.name, AstTy { ty: self.ty },)
    }
}

struct IdentTyId<'a> {
    compilation: &'a Compilation,
    ident: &'a ast::Ident,
    ty_id: ast::NodeId,
}

impl<'a> Display for IdentTyId<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}: {}",
            self.ident.name,
            TyId {
                ty_id: self.ty_id,
                compilation: self.compilation
            },
        )
    }
}

struct PathTyId<'a> {
    compilation: &'a Compilation,
    path: &'a ast::Path,
    ty_id: ast::NodeId,
}

impl<'a> Display for PathTyId<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}: {}",
            &Path { path: self.path },
            TyId {
                ty_id: self.ty_id,
                compilation: self.compilation
            },
        )
    }
}

struct HirCallableDecl<'a, 'b> {
    compilation: &'a Compilation,
    decl: &'b hir::CallableDecl,
}

impl Display for HirCallableDecl<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let kind = match self.decl.kind {
            hir::CallableKind::Function => "function",
            hir::CallableKind::Operation => "operation",
        };

        write!(f, "{} {}", kind, self.decl.name.name)?;
        let input = HirPat {
            pat: &self.decl.input,
            compilation: self.compilation,
        };
        if matches!(self.decl.input.kind, hir::PatKind::Tuple(_)) {
            write!(f, "{input}")?;
        } else {
            write!(f, "({input})")?;
        }
        write!(
            f,
            " : {}{}",
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

struct AstCallableDecl<'a> {
    compilation: &'a Compilation,
    decl: &'a ast::CallableDecl,
}

impl<'a> Display for AstCallableDecl<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let kind = match self.decl.kind {
            ast::CallableKind::Function => "function",
            ast::CallableKind::Operation => "operation",
        };

        let functors = ast_callable_functors(self.decl);
        let functors = FunctorSetValue { functors };

        write!(f, "{} {}", kind, self.decl.name.name)?;
        let input = AstPat {
            pat: &self.decl.input,
            compilation: self.compilation,
        };
        if matches!(*self.decl.input.kind, ast::PatKind::Tuple(_)) {
            write!(f, "{input}")?;
        } else {
            write!(f, "({input})")?;
        }
        write!(
            f,
            " : {}{}",
            AstTy {
                ty: &self.decl.output
            },
            functors,
        )
    }
}

struct HirPat<'a> {
    pat: &'a hir::Pat,
    compilation: &'a Compilation,
}

impl<'a> Display for HirPat<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let ty = HirTy {
            ty: &self.pat.ty,
            compilation: self.compilation,
        };
        match &self.pat.kind {
            hir::PatKind::Bind(name) => write!(f, "{}: {ty}", name.name),
            hir::PatKind::Discard => write!(f, "_: {ty}"),
            hir::PatKind::Tuple(items) => {
                let mut elements = items.iter();
                if let Some(elem) = elements.next() {
                    write!(
                        f,
                        "({}",
                        HirPat {
                            pat: elem,
                            compilation: self.compilation
                        }
                    )?;
                    for elem in elements {
                        write!(
                            f,
                            ", {}",
                            HirPat {
                                pat: elem,
                                compilation: self.compilation
                            }
                        )?;
                    }
                    write!(f, ")")
                } else {
                    write!(f, "()")
                }
            }
        }
    }
}

struct AstPat<'a> {
    pat: &'a ast::Pat,
    compilation: &'a Compilation,
}

impl<'a> Display for AstPat<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &*self.pat.kind {
            ast::PatKind::Bind(ident, anno) => match anno {
                Some(ty) => write!(f, "{}", IdentTy { ident, ty }),
                None => write!(
                    f,
                    "{}",
                    IdentTyId {
                        compilation: self.compilation,
                        ident,
                        ty_id: self.pat.id
                    }
                ),
            },
            ast::PatKind::Discard(anno) => match anno {
                Some(ty) => write!(f, "{}", AstTy { ty }),
                None => write!(
                    f,
                    "_: {}",
                    TyId {
                        ty_id: self.pat.id,
                        compilation: self.compilation
                    }
                ),
            },
            ast::PatKind::Elided => write!(f, "..."),
            ast::PatKind::Paren(item) => write!(
                f,
                "{}",
                AstPat {
                    pat: item,
                    compilation: self.compilation
                }
            ),
            ast::PatKind::Tuple(items) => {
                let mut elements = items.iter();
                if let Some(elem) = elements.next() {
                    write!(
                        f,
                        "({}",
                        AstPat {
                            pat: elem,
                            compilation: self.compilation
                        }
                    )?;
                    for elem in elements {
                        write!(
                            f,
                            ", {}",
                            AstPat {
                                pat: elem,
                                compilation: self.compilation
                            }
                        )?;
                    }
                    write!(f, ")")
                } else {
                    write!(f, "()")
                }
            }
        }
    }
}

struct IdentTyDef<'a> {
    ident: &'a ast::Ident,
    def: &'a ast::TyDef,
}

impl<'a> Display for IdentTyDef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "newtype {} = {}",
            self.ident.name,
            TyDef { def: self.def }
        )
    }
}

struct HirUdt<'a> {
    compilation: &'a Compilation,
    udt: &'a hir::ty::Udt,
}

impl<'a> Display for HirUdt<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let udt_def = UdtDef::new(self.compilation, &self.udt.definition);
        write!(f, "newtype {} = {}", self.udt.name, udt_def)
    }
}

struct UdtDef<'a> {
    compilation: &'a Compilation,
    name: Option<Rc<str>>,
    kind: UdtDefKind<'a>,
}

enum UdtDefKind<'a> {
    SingleTy(&'a hir::ty::Ty),
    TupleTy(Vec<UdtDef<'a>>),
}

impl<'a> UdtDef<'a> {
    pub fn new(compilation: &'a Compilation, def: &'a hir::ty::UdtDef) -> Self {
        match &def.kind {
            hir::ty::UdtDefKind::Field(field) => UdtDef {
                compilation,
                name: field.name.as_ref().cloned(),
                kind: UdtDefKind::SingleTy(&field.ty),
            },
            hir::ty::UdtDefKind::Tuple(defs) => UdtDef {
                compilation,
                name: None,
                kind: UdtDefKind::TupleTy(
                    defs.iter().map(|f| UdtDef::new(compilation, f)).collect(),
                ),
            },
        }
    }
}

impl Display for UdtDef<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if let Some(name) = &self.name {
            write!(f, "{name}: ")?;
        }

        match &self.kind {
            UdtDefKind::SingleTy(ty) => {
                write!(
                    f,
                    "{}",
                    HirTy {
                        ty,
                        compilation: self.compilation
                    }
                )?;
            }
            UdtDefKind::TupleTy(defs) => {
                let mut elements = defs.iter();
                if let Some(elem) = elements.next() {
                    write!(f, "({elem}")?;
                    for elem in elements {
                        write!(f, ", {elem}")?;
                    }
                    write!(f, ")")?;
                } else {
                    write!(f, "Unit")?;
                }
            }
        }
        Ok(())
    }
}

struct FunctorSet<'a> {
    functor_set: &'a hir::ty::FunctorSet,
}

impl<'a> Display for FunctorSet<'a> {
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
                    if let (Some(item), _) = find_item(self.compilation, item_id) {
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

struct AstTy<'a> {
    ty: &'a ast::Ty,
}

impl<'a> Display for AstTy<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.ty.kind.as_ref() {
            ast::TyKind::Array(ty) => write!(f, "{}[]", AstTy { ty }),
            ast::TyKind::Arrow(kind, input, output, functors) => {
                let arrow = match kind {
                    ast::CallableKind::Function => "->",
                    ast::CallableKind::Operation => "=>",
                };
                write!(
                    f,
                    "({} {} {}{})",
                    AstTy { ty: input },
                    arrow,
                    AstTy { ty: output },
                    FunctorExpr { functors }
                )
            }
            ast::TyKind::Hole => write!(f, "_"),
            ast::TyKind::Paren(ty) => write!(f, "{}", AstTy { ty }),
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
                        write!(f, "{}", AstTy { ty: def })?;
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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.def.kind.as_ref() {
            ast::TyDefKind::Field(name, ty) => match name {
                Some(name) => write!(f, "{}: {}", name.name, AstTy { ty }),
                None => write!(f, "{}", AstTy { ty }),
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
