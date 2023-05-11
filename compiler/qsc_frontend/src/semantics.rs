// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::typeck::Tys;
use miette::Diagnostic;
use qsc_ast::{
    ast::{
        CallableBody, CallableDecl, CallableKind, Expr, ExprKind, Package, Spec, Stmt, StmtKind,
    },
    visit::{self, Visitor},
};
use qsc_data_structures::span::Span;
use qsc_hir::hir;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub(super) enum Error {
    #[error("functions cannot use conjugate expressions")]
    ConjInFunc(#[label] Span),

    #[error("functions cannot have functor expressions")]
    FunctorInFunc(#[label] Span),

    #[error("functions cannot call operations")]
    OpCallInFunc(#[label] Span),

    #[error("functions cannot allocate qubits")]
    QubitAllocInFunc(#[label] Span),

    #[error("functions cannot use repeat-loop expressions")]
    RepeatInFunc(#[label] Span),

    #[error("functions cannot have specializations")]
    SpecInFunc(#[label] Span),

    #[error("operations cannot use while-loop expressions")]
    WhileInOp(#[label] Span),
}

pub(super) fn validate(tys: &Tys, package: &Package) -> Vec<Error> {
    let mut validator = Validator {
        tys,
        in_func: false,
        in_op: false,
        errors: Vec::new(),
    };
    validator.visit_package(package);
    validator.errors
}

struct Validator<'a> {
    tys: &'a Tys,
    in_func: bool,
    in_op: bool,
    errors: Vec<Error>,
}

impl Visitor<'_> for Validator<'_> {
    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        self.in_func = decl.kind == CallableKind::Function;
        self.in_op = decl.kind == CallableKind::Operation;

        if self.in_func {
            match &decl.body {
                CallableBody::Specs(specs) => {
                    if specs.iter().any(|spec| spec.spec != Spec::Body) {
                        self.errors.push(Error::SpecInFunc(decl.span));
                    }
                }
                CallableBody::Block(_) => {}
            }
            match &decl.functors {
                Some(functors) => self.errors.push(Error::FunctorInFunc(functors.span)),
                None => {}
            }
        }

        visit::walk_callable_decl(self, decl);
        self.in_func = false;
        self.in_op = false;
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::Qubit(..) = &stmt.kind {
            if self.in_func {
                self.errors.push(Error::QubitAllocInFunc(stmt.span));
            }
        }
        visit::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Call(callee, _) if self.in_func => {
                let ty = self.tys.get(callee.id);
                if matches!(ty, Some(hir::Ty::Arrow(hir::CallableKind::Operation, ..))) {
                    self.errors.push(Error::OpCallInFunc(expr.span));
                }
            }
            ExprKind::Conjugate(..) if self.in_func => {
                self.errors.push(Error::ConjInFunc(expr.span));
            }
            ExprKind::Repeat(..) if self.in_func => {
                self.errors.push(Error::RepeatInFunc(expr.span));
            }
            ExprKind::While(..) if self.in_op => {
                self.errors.push(Error::WhileInOp(expr.span));
            }
            _ => {}
        }
        visit::walk_expr(self, expr);
    }
}
