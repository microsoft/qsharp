// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_hir::{
    hir::{
        CallableDecl, CallableKind, Expr, ExprKind, Item, ItemId, ItemKind, Package, Res, SpecBody,
        SpecDecl, Stmt, StmtKind,
    },
    ty::Ty,
    visit::{self, Visitor},
};

use crate::linter::hir::declare_hir_lints;

use super::lint;

// Read Me:
//  To add a new lint add a new tuple to this structure. The tuple has four elements:
//  `(lint_name, default_lint_level, message, help)`
//
//  where:
//   lint_name: Name of the lint.
//   default_lint_level: Default level for the lint, can be overriden by the user in qsharp.json.
//   message: Message shown in the editor when hovering over the squiggle generated by the lint.
//   help: A user friendly message explaining how to fix the lint.
//
//  Then, add an `impl lint_name` block adding the lint logic.
//
//  After adding a lint you need to go language_service/code_action.rs and add a Quickfix
//  for the newly added lint (or opt out of the Quickfix) in the match statement in that file.
//
//  For more details on how to add a lint, please refer to the crate-level documentation
//  in qsc_linter/lib.rs.
declare_hir_lints! {
    (NeedlessOperation, LintLevel::Allow, "operation does not contain any quantum operations", "this callable can be declared as a function instead"),
    (DeprecatedFunctionConstructor, LintLevel::Warn, "deprecated function constructors", "function constructors for struct types are deprecated, use `new` instead"),
    (DeprecatedWithOperator, LintLevel::Warn, "deprecated `w/` and `w/=` operators for structs", "`w/` and `w/=` operators for structs are deprecated, use `new` instead"),
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
impl HirLintPass<'_> for NeedlessOperation<'_> {
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

/// Crates a lint for deprecated function constructors of structs.
impl HirLintPass<'_> for DeprecatedFunctionConstructor<'_> {
    fn check_expr(&self, expr: &Expr, buffer: &mut Vec<Lint>) {
        if let ExprKind::Var(Res::Item(item_id), _) = &expr.kind {
            let (item, _, _) = resolve_item_relative_to_user_package(
                *item_id,
                self.user_package_id,
                self.package_store,
            );
            if let ItemKind::Ty(_, udt) = &item.kind {
                if udt.is_struct() {
                    buffer.push(lint!(self, expr.span));
                }
            }
        }
    }
}

/// Creates a lint for deprecated `w/` and `w/=` operators for structs.
impl HirLintPass<'_> for DeprecatedWithOperator<'_> {
    // fn check_expr(&self, expr: &Expr, buffer: &mut Vec<Lint>) {
    //     match &expr.kind {
    //         ExprKind::UpdateField(container, _field, _value)
    //         | ExprKind::AssignField(container, _field, _value) => {
    //             if let Ty::Udt(_name, Res::Item(item_id)) = &container.ty {
    //                 let (item, _, _) = resolve_item_relative_to_user_package(
    //                     *item_id,
    //                     self.user_package_id,
    //                     self.package_store,
    //                 );
    //                 if let ItemKind::Ty(_, udt) = &item.kind {
    //                     if udt.is_struct() {
    //                         buffer.push(lint!(self, expr.span));
    //                     }
    //                 }
    //             }
    //         }
    //         _ => {}
    //     }
    // }
}

fn resolve_item_relative_to_user_package(
    item_id: ItemId,
    user_package_id: PackageId,
    package_store: &PackageStore,
) -> (&Item, &Package, ItemId) {
    resolve_item(package_store, user_package_id, item_id)
}

fn resolve_item(
    package_store: &PackageStore,
    local_package_id: PackageId,
    item_id: ItemId,
) -> (&Item, &Package, ItemId) {
    // If the `ItemId` contains a package id, use that.
    // Lack of a package id means the item is in the
    // same package as the one this `ItemId` reference
    // came from. So use the local package id passed in.
    let package_id = item_id.package.unwrap_or(local_package_id);
    let package = &package_store
        .get(package_id)
        .expect("package should exist in store")
        .package;
    (
        package
            .items
            .get(item_id.item)
            .expect("item id should exist"),
        package,
        ItemId {
            package: Some(package_id),
            item: item_id.item,
        },
    )
}
