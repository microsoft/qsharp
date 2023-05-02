// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_frontend::compile::CompileUnit;
use qsc_hir::{
    hir::{CallableBody, CallableDecl, CallableKind, Expr, ExprKind, Spec, Stmt, StmtKind, Ty},
    visit::{walk_callable_decl, walk_expr, walk_stmt, Visitor},
};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
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

#[allow(clippy::module_name_repetitions)]
#[must_use]
pub fn validate_semantics(unit: &CompileUnit) -> Vec<Error> {
    let mut pass = Validate {
        errors: Vec::new(),
        in_func: false,
        in_op: false,
    };
    pass.visit_package(&unit.package);
    pass.errors
}

struct Validate {
    errors: Vec<Error>,
    in_func: bool,
    in_op: bool,
}

impl<'a> Visitor<'a> for Validate {
    fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
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

        walk_callable_decl(self, decl);
        self.in_func = false;
        self.in_op = false;
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        if let StmtKind::Qubit(..) = &stmt.kind {
            if self.in_func {
                self.errors.push(Error::QubitAllocInFunc(stmt.span));
            }
        }
        walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        match &expr.kind {
            ExprKind::Call(callee, _)
                if matches!(callee.ty, Ty::Arrow(CallableKind::Operation, _, _, _))
                    && self.in_func =>
            {
                self.errors.push(Error::OpCallInFunc(expr.span));
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
        walk_expr(self, expr);
    }
}
