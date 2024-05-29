// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// use miette::Diagnostic;
use qsc_hir::{
    hir::{CallableDecl, CallableKind, Expr, ExprKind, Stmt, StmtKind},
    ty::Ty,
    visit::{self, Visitor},
};
// use thiserror::Error;

use crate::linter::hir::declare_hir_lints;

use super::lint;

declare_hir_lints! {
    (NeedlessOperation, LintLevel::Warn, "unnecessary operation declaration", "convert to function")
}

#[derive(Default)]
pub(super) struct OperationLimits {
    // Operation Characteristics
    pub(super) op_char: bool,
}

impl Visitor<'_> for OperationLimits {
    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        if decl.kind == CallableKind::Operation {
            // Checking for special Ctl and Adj implementations
            // if decl.adj.is_some() || decl.ctl.is_some() || decl.ctl_adj.is_some() {
            //     self.op_char.push(decl.span);
            // }
            // if decl.functors != FunctorSetValue::Empty {
            //     self.op_char.push(decl.name.span);
            // }
            visit::walk_callable_decl(self, decl);
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        if !self.op_char {
            if let StmtKind::Qubit(..) = &stmt.kind {
                self.op_char = true;
            } else {
                visit::walk_stmt(self, stmt);
            }
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        if !self.op_char {
            match &expr.kind {
                ExprKind::Call(callee, _) => {
                    if matches!(&callee.ty, Ty::Arrow(arrow) if arrow.kind == CallableKind::Operation)
                    {
                        self.op_char = true;
                    }
                }
                ExprKind::Conjugate(..) | ExprKind::Repeat(..) => {
                    self.op_char = true;
                }
                _ => {
                    visit::walk_expr(self, expr);
                }
            }
        }
    }
}

impl HirLintPass for NeedlessOperation {
    fn check_callable_decl(&self, decl: &CallableDecl, buffer: &mut Vec<Lint>) {
        if decl.kind == CallableKind::Operation {
            let mut op_limits = OperationLimits::default();

            op_limits.visit_callable_decl(decl);

            if !op_limits.op_char {
                buffer.push(lint!(self, decl.span));
            }
        }
    }
}
