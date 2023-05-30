// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{CallableBody, CallableDecl, CallableKind, Expr, ExprKind, Spec, Stmt, StmtKind, Ty},
    visit::{self, Visitor},
};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("functions cannot use conjugate expressions")]
    Conj(#[label] Span),

    #[error("functions cannot have functor expressions")]
    Functor(#[label] Span),

    #[error("functions cannot call operations")]
    OpCall(#[label] Span),

    #[error("functions cannot allocate qubits")]
    QubitAlloc(#[label] Span),

    #[error("functions cannot use repeat-loop expressions")]
    Repeat(#[label] Span),

    #[error("functions cannot have specializations")]
    Spec(#[label] Span),
}

#[derive(Default)]
pub(super) struct CallableLimits {
    pub(super) errors: Vec<Error>,
}

impl Visitor<'_> for CallableLimits {
    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        if decl.kind == CallableKind::Function {
            match &decl.body {
                CallableBody::Specs(specs) => {
                    if specs.iter().any(|spec| spec.spec != Spec::Body) {
                        self.errors.push(Error::Spec(decl.span));
                    }
                }
                CallableBody::Block(_) => {}
            }
            if decl.functors.is_empty() != Some(true) {
                self.errors.push(Error::Functor(decl.name.span));
            }
            visit::walk_callable_decl(self, decl);
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::Qubit(..) = &stmt.kind {
            self.errors.push(Error::QubitAlloc(stmt.span));
        }
        visit::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Call(callee, _) => {
                if matches!(callee.ty, Ty::Arrow(CallableKind::Operation, ..)) {
                    self.errors.push(Error::OpCall(expr.span));
                }
            }
            ExprKind::Conjugate(..) => {
                self.errors.push(Error::Conj(expr.span));
            }
            ExprKind::Repeat(..) => {
                self.errors.push(Error::Repeat(expr.span));
            }
            _ => {}
        }
        visit::walk_expr(self, expr);
    }
}
