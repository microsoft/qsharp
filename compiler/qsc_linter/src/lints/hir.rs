// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_hir::{
    hir::{CallableDecl, CallableKind, Expr, ExprKind, SpecBody, SpecDecl, Stmt, StmtKind},
    ty::Ty,
    visit::{self, Visitor},
};

use crate::linter::hir::declare_hir_lints;

use super::lint;

declare_hir_lints! {
    (NeedlessOperation, LintLevel::Allow, "operation does not contain any quantum operations", "this callable can be declared as a function instead")
}

/// Helper to check if an operation has desired operation characteristics
///
/// empty operations: no lint, special case of `I` operation used for Delay
/// operations with errors (e.g. partially typed code): no lint because linter does not run
/// non-empty operations, with specializations, and no quantum operations: show lint, but don't offer quickfix (to avoid deleting user code in any explicit specializations)
/// non-empty operations with no specializations, and no quantum operations: show lint, offer quickfix to convert to function
#[derive(Default)]
struct IsQuantumOperation {
    /// This field is set to `true` after calling `Visitor::visit_callable_decl(...)`
    /// if the operation satisfies the characteristics described above.
    is_op: bool,
}

impl IsQuantumOperation {
    /// Returns `true` if the declaration is empty.
    fn is_empty_decl(spec_decl: Option<&SpecDecl>) -> bool {
        match spec_decl {
            None => true,
            Some(decl) => match &decl.body {
                SpecBody::Gen(_) => true,
                SpecBody::Impl(_, block) => block.stmts.is_empty(),
            },
        }
    }

    /// Returns `true` if the operation is empty.
    /// An operation is empty if there is no code for its body and specializations.
    fn is_empty_op(call_decl: &CallableDecl) -> bool {
        Self::is_empty_decl(Some(&call_decl.body))
            && Self::is_empty_decl(call_decl.adj.as_ref())
            && Self::is_empty_decl(call_decl.ctl.as_ref())
            && Self::is_empty_decl(call_decl.ctl_adj.as_ref())
    }
}

impl Visitor<'_> for IsQuantumOperation {
    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        if Self::is_empty_op(decl) {
            self.is_op = true;
        } else {
            visit::walk_callable_decl(self, decl);
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        if !self.is_op {
            if let StmtKind::Qubit(..) = &stmt.kind {
                self.is_op = true;
            } else {
                visit::walk_stmt(self, stmt);
            }
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        if !self.is_op {
            match &expr.kind {
                ExprKind::Call(callee, _) => {
                    if matches!(&callee.ty, Ty::Arrow(arrow) if arrow.kind == CallableKind::Operation)
                    {
                        self.is_op = true;
                    }
                }
                ExprKind::Conjugate(..) | ExprKind::Repeat(..) => {
                    self.is_op = true;
                }
                _ => {
                    visit::walk_expr(self, expr);
                }
            }
        }
    }
}

/// HIR Lint for [`NeedlessOperation`], suggesting to use function
/// We use [`IsQuantumOperation`] helper to check if a operation has desired operation characteristics
impl HirLintPass for NeedlessOperation {
    fn check_callable_decl(&self, decl: &CallableDecl, buffer: &mut Vec<Lint>) {
        if decl.kind == CallableKind::Operation {
            let mut op_limits = IsQuantumOperation::default();

            op_limits.visit_callable_decl(decl);

            if !op_limits.is_op {
                buffer.push(lint!(self, decl.name.span));
            }
        }
    }
}
