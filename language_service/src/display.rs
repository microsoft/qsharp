// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::compilation::{Compilation, Lookup};
use qsc::{
    ast,
    hir::{self, ty::GenericParam},
};
use regex_lite::Regex;
use std::{
    fmt::{Display, Formatter, Result},
    rc::Rc,
};

pub(crate) struct CodeDisplay<'a> {
    pub(crate) compilation: &'a Compilation,
}

#[allow(clippy::unused_self)]
impl<'a> CodeDisplay<'a> {
    pub(crate) fn hir_callable_decl(&self, decl: &'a hir::CallableDecl) -> impl Display + '_ {
        HirCallableDecl { decl }
    }

    pub(crate) fn ast_callable_decl(&self, decl: &'a ast::CallableDecl) -> impl Display + '_ {
        AstCallableDecl {
            lookup: self.compilation,
            decl,
        }
    }

    pub(crate) fn name_ty_id(&self, name: &'a str, ty_id: ast::NodeId) -> impl Display + '_ {
        NameTyId {
            lookup: self.compilation,
            name,
            ty_id,
        }
    }

    pub(crate) fn ident_ty(&self, ident: &'a ast::Ident, ty: &'a ast::Ty) -> impl Display + '_ {
        IdentTy { ident, ty }
    }

    pub(crate) fn ident_ty_def(
        &self,
        ident: &'a ast::Ident,
        def: &'a ast::TyDef,
    ) -> impl Display + 'a {
        IdentTyDef { ident, def }
    }

    pub(crate) fn hir_udt(&self, udt: &'a hir::ty::Udt) -> impl Display + '_ {
        HirUdt { udt }
    }

    pub(crate) fn hir_pat(&self, pat: &'a hir::Pat) -> impl Display + '_ {
        HirPat { pat }
    }

    pub(crate) fn get_param_offset(&self, decl: &hir::CallableDecl) -> u32 {
        HirCallableDecl { decl }.get_param_offset()
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
        write!(f, "{} : {}", self.ident.name, AstTy { ty: self.ty },)
    }
}

struct NameTyId<'a> {
    lookup: &'a Compilation,
    name: &'a str,
    ty_id: ast::NodeId,
}

impl<'a> Display for NameTyId<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{} : {}",
            self.name,
            TyId {
                lookup: self.lookup,
                ty_id: self.ty_id,
            },
        )
    }
}

struct HirCallableDecl<'a> {
    decl: &'a hir::CallableDecl,
}

impl HirCallableDecl<'_> {
    fn get_param_offset(&self) -> u32 {
        let offset = match self.decl.kind {
            hir::CallableKind::Function => "function".len(),
            hir::CallableKind::Operation => "operation".len(),
        } + 1 // this is for the space between keyword and name
        + self.decl.name.name.len()
        + display_type_params(&self.decl.generics).len();

        u32::try_from(offset)
            .expect("failed to cast usize to u32 while calculating parameter offset")
    }
}

impl Display for HirCallableDecl<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let kind = match self.decl.kind {
            hir::CallableKind::Function => "function",
            hir::CallableKind::Operation => "operation",
        };

        write!(f, "{} {}", kind, self.decl.name.name)?;
        let type_params = display_type_params(&self.decl.generics);
        write!(f, "{type_params}")?;
        let input = HirPat {
            pat: &self.decl.input,
        };
        if matches!(self.decl.input.kind, hir::PatKind::Tuple(_)) {
            write!(f, "{input}")?;
        } else {
            write!(f, "({input})")?;
        }
        write!(
            f,
            " : {}{}",
            self.decl.output.display(),
            FunctorSetValue {
                functors: self.decl.functors,
            },
        )
    }
}

struct AstCallableDecl<'a> {
    lookup: &'a Compilation,
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
        if !self.decl.generics.is_empty() {
            let type_params = self
                .decl
                .generics
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            write!(f, "<{type_params}>")?;
        }
        let input = AstPat {
            pat: &self.decl.input,
            lookup: self.lookup,
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
}

impl<'a> Display for HirPat<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &self.pat.kind {
            hir::PatKind::Bind(name) => write!(f, "{} : {}", name.name, self.pat.ty.display()),
            hir::PatKind::Discard => write!(f, "_ : {}", self.pat.ty.display()),
            hir::PatKind::Tuple(items) => {
                let mut elements = items.iter();
                if let Some(elem) = elements.next() {
                    write!(f, "({}", HirPat { pat: elem })?;
                    for elem in elements {
                        write!(f, ", {}", HirPat { pat: elem })?;
                    }
                    write!(f, ")")
                } else {
                    write!(f, "()")
                }
            }
            hir::PatKind::Err => write!(f, "?"),
        }
    }
}

struct AstPat<'a> {
    lookup: &'a Compilation,
    pat: &'a ast::Pat,
}

impl<'a> Display for AstPat<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &*self.pat.kind {
            ast::PatKind::Bind(ident, anno) => match anno {
                Some(ty) => write!(f, "{}", IdentTy { ident, ty }),
                None => write!(
                    f,
                    "{}",
                    NameTyId {
                        lookup: self.lookup,
                        name: &ident.name,
                        ty_id: self.pat.id
                    }
                ),
            },
            ast::PatKind::Discard(anno) => match anno {
                Some(ty) => write!(f, "{}", AstTy { ty }),
                None => write!(
                    f,
                    "_ : {}",
                    TyId {
                        lookup: self.lookup,
                        ty_id: self.pat.id,
                    }
                ),
            },
            ast::PatKind::Elided => write!(f, "..."),
            ast::PatKind::Paren(item) => write!(
                f,
                "{}",
                AstPat {
                    lookup: self.lookup,
                    pat: item,
                }
            ),
            ast::PatKind::Tuple(items) => {
                let mut elements = items.iter();
                if let Some(elem) = elements.next() {
                    write!(
                        f,
                        "({}",
                        AstPat {
                            lookup: self.lookup,
                            pat: elem,
                        }
                    )?;
                    for elem in elements {
                        write!(
                            f,
                            ", {}",
                            AstPat {
                                lookup: self.lookup,
                                pat: elem,
                            }
                        )?;
                    }
                    write!(f, ")")
                } else {
                    write!(f, "()")
                }
            }
            ast::PatKind::Err => write!(f, "?"),
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
    udt: &'a hir::ty::Udt,
}

impl<'a> Display for HirUdt<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let udt_def = UdtDef::new(&self.udt.definition);
        write!(f, "newtype {} = {}", self.udt.name, udt_def)
    }
}

struct UdtDef<'a> {
    name: Option<Rc<str>>,
    kind: UdtDefKind<'a>,
}

enum UdtDefKind<'a> {
    SingleTy(&'a hir::ty::Ty),
    TupleTy(Vec<UdtDef<'a>>),
}

impl<'a> UdtDef<'a> {
    pub fn new(def: &'a hir::ty::UdtDef) -> Self {
        match &def.kind {
            hir::ty::UdtDefKind::Field(field) => UdtDef {
                name: field.name.as_ref().cloned(),
                kind: UdtDefKind::SingleTy(&field.ty),
            },
            hir::ty::UdtDefKind::Tuple(defs) => UdtDef {
                name: None,
                kind: UdtDefKind::TupleTy(defs.iter().map(UdtDef::new).collect()),
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
                write!(f, "{}", ty.display())
            }
            UdtDefKind::TupleTy(defs) => fmt_tuple(f, defs, |def| def),
        }
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

struct TyId<'a> {
    lookup: &'a Compilation,
    ty_id: ast::NodeId,
}

impl<'a> Display for TyId<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if let Some(ty) = self.lookup.get_ty(self.ty_id) {
            write!(f, "{}", ty.display())
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
            ast::TyKind::Path(path) => write!(f, "{}", AstPath { path }),
            ast::TyKind::Param(id) => write!(f, "{}", id.name),
            ast::TyKind::Tuple(tys) => fmt_tuple(f, tys, |ty| AstTy { ty }),
            ast::TyKind::Err => write!(f, "?"),
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

struct AstPath<'a> {
    path: &'a ast::Path,
}

impl<'a> Display for AstPath<'a> {
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
                Some(name) => write!(f, "{} : {}", name.name, AstTy { ty }),
                None => write!(f, "{}", AstTy { ty }),
            },
            ast::TyDefKind::Paren(def) => write!(f, "{}", TyDef { def }),
            ast::TyDefKind::Tuple(tys) => fmt_tuple(f, tys, |def| TyDef { def }),
            ast::TyDefKind::Err => write!(f, "?"),
        }
    }
}

fn fmt_tuple<'a, 'b, D, I>(
    formatter: &'a mut Formatter,
    elements: &'b [I],
    map: impl Fn(&'b I) -> D,
) -> Result
where
    D: Display,
{
    let mut elements = elements.iter();
    if let Some(elem) = elements.next() {
        write!(formatter, "({}", map(elem))?;
        if elements.len() == 0 {
            write!(formatter, ",)")?;
        } else {
            for elem in elements {
                write!(formatter, ", {}", map(elem))?;
            }
            write!(formatter, ")")?;
        }
    } else {
        write!(formatter, "Unit")?;
    }
    Ok(())
}

fn display_type_params(generics: &[GenericParam]) -> String {
    let type_params = generics
        .iter()
        .filter_map(|generic| match generic {
            GenericParam::Ty(name) => Some(name.name.clone()),
            GenericParam::Functor(_) => None,
        })
        .collect::<Vec<_>>()
        .join(", ");
    if type_params.is_empty() {
        type_params
    } else {
        format!("<{type_params}>")
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

//
// parsing functions for working with doc comments
//

#[cfg(test)]
pub fn increase_header_level(doc: &str) -> String {
    let re = Regex::new(r"(?mi)^(#+)( [\s\S]+?)$").expect("Invalid regex");
    re.replace_all(doc, "$1#$2").to_string()
}

pub fn parse_doc_for_summary(doc: &str) -> String {
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

pub fn parse_doc_for_param(doc: &str, param: &str) -> String {
    let re = Regex::new(r"(?mi)(?:^# Input$)([\s\S]*?)(?:(^# .*)|\z)").expect("Invalid regex");
    let input = match re.captures(doc) {
        Some(captures) => {
            let capture = captures
                .get(1)
                .expect("Didn't find the capture for the given regex");
            capture.as_str()
        }
        None => return String::new(),
    }
    .trim();

    let re = Regex::new(format!(r"(?mi)(?:^## {param}$)([\s\S]*?)(?:(^(#|##) .*)|\z)").as_str())
        .expect("Invalid regex");
    match re.captures(input) {
        Some(captures) => {
            let capture = captures
                .get(1)
                .expect("Didn't find the capture for the given regex");
            capture.as_str()
        }
        None => return String::new(),
    }
    .trim()
    .to_string()
}
