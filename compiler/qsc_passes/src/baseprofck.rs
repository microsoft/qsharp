// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{
        BinOp, CallableDecl, CallableKind, Expr, ExprKind, Item, ItemKind, Lit, Package, SpecBody,
        SpecGen, Stmt, StmtKind,
    },
    ty::{Prim, Ty},
    visit::{walk_expr, walk_item, Visitor},
};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("cannot compare measurement results")]
    #[diagnostic(help(
        "comparing measurement results is not supported when performing base profile QIR generation"
    ))]
    #[diagnostic(code("Qsc.BaseProfCk.ResultComparison"))]
    ResultComparison(#[label] Span),

    #[error("result literals are not supported")]
    #[diagnostic(help(
        "result literals `One` and `Zero` are not supported when performing base profile QIR generation"
    ))]
    #[diagnostic(code("Qsc.BaseProfCk.ResultLiteral"))]
    ResultLiteral(#[label] Span),

    #[error("non-Result return type in entry expression")]
    #[diagnostic(help(
        "returning types other than Result from the entry expression is not supported when performing base profile QIR generation"
    ))]
    #[diagnostic(code("Qsc.BaseProfCk.ReturnNonResult"))]
    ReturnNonResult(#[label] Span),

    #[error("intrinsic operations that return types other than Result or Unit are not supported")]
    #[diagnostic(help(
        "intrinsic operations that return types other than Result or Unit are not supported when performing base profile QIR generation"
    ))]
    #[diagnostic(code("Qsc.BaseProfCk.UnsupportedIntrinsic"))]
    UnsupportedIntrinsic(#[label] Span),
}

#[must_use]
pub fn check_base_profile_compliance(package: &Package) -> Vec<Error> {
    let mut checker = Checker { errors: Vec::new() };
    if let Some(entry) = &package.entry {
        if any_non_result_ty(&entry.ty) {
            checker.errors.push(Error::ReturnNonResult(entry.span));
        }
    }
    checker.visit_package(package);

    checker.errors
}

#[must_use]
pub fn check_base_profile_compliance_for_stmt(stmt: &Stmt) -> Vec<Error> {
    let mut checker = Checker { errors: Vec::new() };
    checker.visit_stmt(stmt);
    if let StmtKind::Expr(expr) = &stmt.kind {
        if any_non_result_ty(&expr.ty) {
            checker.errors.push(Error::ReturnNonResult(stmt.span));
        }
    }

    checker.errors
}

#[must_use]
pub fn check_base_profile_compliance_for_callable(decl: &CallableDecl) -> Vec<Error> {
    let mut checker = Checker { errors: Vec::new() };
    checker.visit_callable_decl(decl);

    checker.errors
}

struct Checker {
    errors: Vec<Error>,
}

impl<'a> Visitor<'a> for Checker {
    fn visit_item(&mut self, item: &'a Item) {
        match &item.kind {
            ItemKind::Callable(callable)
                if callable.kind == CallableKind::Operation
                    && callable.body.body == SpecBody::Gen(SpecGen::Intrinsic)
                    && callable.output != Ty::Prim(Prim::Result)
                    && callable.output != Ty::Prim(Prim::Qubit)
                    && callable.output != Ty::UNIT =>
            {
                self.errors
                    .push(Error::UnsupportedIntrinsic(callable.name.span));
            }
            _ => {}
        }

        walk_item(self, item);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        match &expr.kind {
            ExprKind::BinOp(BinOp::Eq | BinOp::Neq, lhs, _) if any_result_ty(&lhs.ty) => {
                self.errors.push(Error::ResultComparison(expr.span));
            }
            ExprKind::Lit(Lit::Result(_)) => {
                self.errors.push(Error::ResultLiteral(expr.span));
            }
            _ => {}
        }
        walk_expr(self, expr);
    }
}

fn any_result_ty(ty: &Ty) -> bool {
    match ty {
        Ty::Array(ty) => any_result_ty(ty),
        Ty::Prim(Prim::Result) => true,
        Ty::Tuple(tys) => tys.iter().any(any_result_ty),
        _ => false,
    }
}

fn any_non_result_ty(ty: &Ty) -> bool {
    match ty {
        Ty::Array(ty) => any_non_result_ty(ty),
        Ty::Prim(Prim::Result) => false,
        Ty::Tuple(tys) => tys.iter().any(any_non_result_ty),
        _ => true,
    }
}
