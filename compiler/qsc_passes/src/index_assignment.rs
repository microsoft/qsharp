// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::mem::take;

use qsc_hir::{
    hir::{Expr, ExprKind},
    mut_visit::{walk_expr, MutVisitor},
};

#[derive(Default)]
pub(crate) struct ConvertToWSlash {}

impl MutVisitor for ConvertToWSlash {
    fn visit_expr(&mut self, expr: &mut Expr) {
        match take(&mut expr.kind) {
            ExprKind::Assign(mut lhs, rhs) => match take(&mut lhs.kind) {
                ExprKind::Index(array, index) => {
                    expr.kind = ExprKind::AssignIndex(array, index, rhs);
                }
                other => {
                    lhs.kind = other;
                    expr.kind = ExprKind::Assign(lhs, rhs);
                }
            },
            kind => expr.kind = kind,
        }
        walk_expr(self, expr);
    }
}
