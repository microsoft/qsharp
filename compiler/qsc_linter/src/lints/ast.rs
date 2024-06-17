// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::lint;
use crate::linter::ast::declare_ast_lints;
use qsc_ast::ast::{BinOp, Block, Expr, ExprKind, Item, ItemKind, Lit, Stmt, StmtKind};
use qsc_data_structures::span::Span;
use qsc_frontend::resolve::Res;
use qsc_hir::hir;

declare_ast_lints! {
    (DivisionByZero, LintLevel::Error, "attempt to divide by zero", "division by zero will fail at runtime"),
    (NeedlessParens, LintLevel::Allow, "unnecessary parentheses", "remove the extra parentheses for clarity"),
    (RedundantSemicolons, LintLevel::Warn, "redundant semicolons", "remove the redundant semicolons"),
    (DeprecatedNewtype, LintLevel::Warn, "deprecated `newtype` declarations", "`newtype` declarations are deprecated, use `struct` instead"),
    (DeprecatedFunctionConstructor, LintLevel::Warn, "deprecated function constructors", "function constructors for struct types are deprecated, use `new` instead"),
}

impl AstLintPass for DivisionByZero {
    fn check_expr(&self, expr: &Expr, buffer: &mut Vec<Lint>) {
        if let ExprKind::BinOp(BinOp::Div, _, ref rhs) = *expr.kind {
            if let ExprKind::Lit(ref lit) = *rhs.kind {
                if let Lit::Int(0) = **lit {
                    buffer.push(lint!(self, expr.span));
                }
            }
        }
    }
}

impl NeedlessParens {
    /// The idea is that if we find a expr of the form:
    /// a + (expr)
    /// and `expr` has higher precedence than `+`, then the
    /// parentheses are needless. Parentheses around a literal
    /// are also needless.
    fn push(&self, parent: &Expr, child: &Expr, buffer: &mut Vec<Lint>) {
        if let ExprKind::Paren(expr) = &*child.kind {
            if precedence(parent) < precedence(expr) {
                buffer.push(lint!(self, child.span));
            }
        }
    }
}

impl AstLintPass for NeedlessParens {
    fn check_expr(&self, expr: &Expr, buffer: &mut Vec<Lint>) {
        match &*expr.kind {
            ExprKind::BinOp(_, left, right) => {
                self.push(expr, left, buffer);
                self.push(expr, right, buffer);
            }
            ExprKind::Assign(_, right) | ExprKind::AssignOp(_, _, right) => {
                self.push(expr, right, buffer);
            }
            _ => (),
        }
    }

    /// Checks the assignment statements.
    fn check_stmt(&self, stmt: &Stmt, buffer: &mut Vec<Lint>) {
        if let StmtKind::Local(_, _, right) = &*stmt.kind {
            if let ExprKind::Paren(_) = &*right.kind {
                buffer.push(lint!(self, right.span));
            }
        }
    }
}

impl RedundantSemicolons {
    /// Helper function that pushes a lint to the buffer if we have
    /// found two or more semicolons.
    fn maybe_push(&self, seq: &mut Option<Span>, buffer: &mut Vec<Lint>) {
        if let Some(span) = seq.take() {
            buffer.push(lint!(self, span));
        }
    }
}

impl AstLintPass for RedundantSemicolons {
    /// Checks if there are redundant semicolons. The idea is that a redundant
    /// semicolon is parsed as an Empty statement. If we have multiple empty
    /// statements in a row, we group them as single lint, that spans from
    /// the first redundant semicolon to the last redundant semicolon.
    fn check_block(&self, block: &Block, buffer: &mut Vec<Lint>) {
        // a finite state machine that keeps track of the span of the redundant semicolons
        // None: no redundant semicolons
        // Some(_): one or more redundant semicolons
        let mut seq: Option<Span> = None;

        for stmt in block.stmts.iter() {
            match (&*stmt.kind, &mut seq) {
                (StmtKind::Empty, None) => seq = Some(stmt.span),
                (StmtKind::Empty, Some(span)) => span.hi = stmt.span.hi,
                (_, seq) => self.maybe_push(seq, buffer),
            }
        }

        self.maybe_push(&mut seq, buffer);
    }
}

fn precedence(expr: &Expr) -> u8 {
    match &*expr.kind {
        ExprKind::Lit(_) => 15,
        ExprKind::Paren(_) => 14,
        ExprKind::UnOp(_, _) => 13,
        ExprKind::BinOp(op, _, _) => match op {
            BinOp::Exp => 12,
            BinOp::Div | BinOp::Mod | BinOp::Mul => 10,
            BinOp::Add | BinOp::Sub => 9,
            BinOp::Shl | BinOp::Shr => 8,
            BinOp::AndB => 7,
            BinOp::XorB => 6,
            BinOp::OrB => 5,
            BinOp::Gt | BinOp::Gte | BinOp::Lt | BinOp::Lte | BinOp::Eq | BinOp::Neq => 4,
            BinOp::AndL => 3,
            BinOp::OrL => 2,
        },
        ExprKind::Assign(_, _) | ExprKind::AssignOp(_, _, _) => 1,
        _ => 0,
    }
}

/// Crates a lint for deprecated user-defined types declarations using `newtype`.
impl AstLintPass for DeprecatedNewtype {
    fn check_item(&self, item: &Item, buffer: &mut Vec<Lint>) {
        if let ItemKind::Ty(_, _) = item.kind.as_ref() {
            buffer.push(lint!(self, item.span));
        }
    }
}

/// Crates a lint for deprecated function constructors of structs.
impl AstLintPass for DeprecatedFunctionConstructor {
    fn check_expr(&self, expr: &Expr, buffer: &mut Vec<Lint>) {
        if let ExprKind::Path(path) = expr.kind.as_ref() {
            // ToDo: check if the path id maps to an item that is a struct:
            // if let Some(&Res::Item(item_id, _)) = self.compilation.names.get(path.id) {
            //     let (item, _, _) = self
            //         .compilation
            //         .resolve_item_relative_to_user_package(item_id);
            //     if let hir::ItemKind::Ty(_, udt) = item.kind {
            //         if udt.is_struct() {
            //             buffer.push(lint!(self, expr.span));
            //         }
            //     }
            // }
        }
    }
}
