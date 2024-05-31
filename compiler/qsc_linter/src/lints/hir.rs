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
    (NeedlessOperation, LintLevel::Warn, "unnecessary operation declaration", "convert to function")
}

// Helper to check if a operation has desired operation characteristics
//
// empty operations: no lint, special case of `I` operation used for Delay
// operations with errors (e.g. partially typed code): no lint because linter does not run
// non-empty operations, with specializations, and no quantum operations: show lint, but don't offer quickfix (to avoid deleting user code in any explicit specializations)
// non-empty operations with no specializations, and no quantum operations: show lint, offer quickfix to convert to function
#[derive(Default)]
struct OperationLimits {
    // Operation Characteristics
    op_char: bool,
}

impl OperationLimits {
    // Checks for empty declarations
    fn is_empty_optional_decl(spec_decl: &Option<SpecDecl>) -> bool {
        match spec_decl {
            None => true,
            Some(decl) => Self::is_empty_decl(decl),
        }
    }

    fn is_empty_decl(spec_decl: &SpecDecl) -> bool {
        match &spec_decl.body {
            SpecBody::Gen(_) => true,
            SpecBody::Impl(_, block) => block.stmts.is_empty(),
        }
    }

    // Empty operation means no code for body or specializations(implicit or explicit)
    fn is_empty_op(call_decl: &CallableDecl) -> bool {
        Self::is_empty_decl(&call_decl.body)
            && Self::is_empty_optional_decl(&call_decl.adj)
            && Self::is_empty_optional_decl(&call_decl.ctl)
            && Self::is_empty_optional_decl(&call_decl.ctl_adj)
    }
}

impl Visitor<'_> for OperationLimits {
    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        if Self::is_empty_op(decl) {
            self.op_char = true;
        } else {
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

// HIR Lint for `NeedlessOperation`, suggesting to use function
// We use `OperationLimits` helper to check if a operation has desired operation characteristics
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
