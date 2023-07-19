// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_frontend::compile::PackageStore;
use qsc_hir::{
    hir::{
        BinOp, CallableKind, Expr, ExprKind, Item, ItemId, ItemKind, Lit, Package, PackageId, Res,
        SpecBody, SpecGen,
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
        "intrinsic operations that return values other than Result or Unit are not supported when performing base profile QIR generation"
    ))]
    #[diagnostic(code("Qsc.BaseProfCk.UnsupportedIntrinsic"))]
    UnsupportedIntrinsic(#[label] Span),
}

#[must_use]
pub fn check_base_profile_compliance(store: &PackageStore, package: &Package) -> Vec<Error> {
    let mut checker = Checker {
        current_package: None,
        pending: Vec::new(),
        processed: HashSet::new(),
        errors: Vec::new(),
    };
    if let Some(entry) = &package.entry {
        if any_non_result_ty(&entry.ty) {
            checker.errors.push(Error::ReturnNonResult(entry.span));
        }
    }
    checker.visit_package(package);

    while let Some(item_id) = checker.pending.pop() {
        if !checker.processed.insert(item_id) {
            continue;
        }

        let item = store
            .get(item_id.package.expect("package id should be set"))
            .expect("package should be present in store")
            .package
            .items
            .get(item_id.item)
            .expect("item should be present in package");
        checker.current_package = item_id.package;
        checker.visit_item(item);
    }

    checker.errors
}

struct Checker {
    current_package: Option<PackageId>,
    pending: Vec<ItemId>,
    processed: HashSet<ItemId>,
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
            ExprKind::Var(Res::Item(item_id), _) => {
                if item_id.package.is_some() && !self.processed.contains(item_id) {
                    self.pending.push(*item_id);
                } else if self.current_package.is_some() {
                    self.pending.push(ItemId {
                        package: self.current_package,
                        item: item_id.item,
                    });
                }
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
