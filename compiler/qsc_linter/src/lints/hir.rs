// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::rc::Rc;

use qsc_data_structures::span::Span;
use qsc_frontend::compile::PackageStore;
use qsc_hir::{
    hir::{
        CallableDecl, CallableKind, Expr, ExprKind, Field, Item, ItemId, ItemKind, Package,
        PackageId, Res, SpecBody, SpecDecl, Stmt, StmtKind,
    },
    ty::Ty,
    visit::{self, Visitor},
};

use crate::linter::{hir::declare_hir_lints, Context};

use super::lint;

// Read Me:
//  To add a new lint add a new tuple to this structure. The tuple has four elements:
//  `(lint_name, default_lint_level, message, help)`
//
//  where:
//   lint_name: Name of the lint.
//   default_lint_level: Default level for the lint, can be overridden by the user in qsharp.json.
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
    (DeprecatedWithOperator, LintLevel::Warn, "deprecated `w/` and `w/=` operators for structs", "`w/` and `w/=` operators for structs are deprecated, use `new` instead", Option<WithOperatorLint>),
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
    fn check_callable_decl(
        &mut self,
        decl: &CallableDecl,
        buffer: &mut Vec<Lint>,
        _context: Context,
    ) {
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
impl HirLintPass for DeprecatedFunctionConstructor {
    fn check_expr(&mut self, expr: &Expr, buffer: &mut Vec<Lint>, context: Context) {
        if let ExprKind::Var(Res::Item(item_id), _) = &expr.kind {
            let (item, _, _) = resolve_item_relative_to_user_package(
                *item_id,
                context.user_package_id,
                context.package_store,
            );
            if let ItemKind::Ty(_, udt) = &item.kind {
                if udt.is_struct() {
                    buffer.push(lint!(self, expr.span));
                }
            }
        }
    }
}

struct WithOperatorLint {
    span: Span,
    ty_name: Rc<str>,
    is_w_eq: bool,
    field_assigns: Vec<(Rc<str>, Rc<str>)>,
}

impl DeprecatedWithOperator {
    /// Returns a substring of the user code's `SourceMap` in the range `lo..hi`.
    fn get_source_code(span: Span, context: Context) -> String {
        let unit = context
            .package_store
            .get(context.user_package_id)
            .expect("user package should exist");

        let source = unit
            .sources
            .find_by_offset(span.lo)
            .expect("source should exist");

        let lo = (span.lo - source.offset) as usize;
        let hi = (span.hi - source.offset) as usize;
        source.contents[lo..hi].to_string()
    }

    /// Returns the indentation at the given offset.
    fn indentation_at_offset(offset: u32, context: Context) -> u32 {
        let unit = context
            .package_store
            .get(context.user_package_id)
            .expect("user package should exist");

        let source = unit
            .sources
            .find_by_offset(offset)
            .expect("source should exist");

        let mut indentation = 0;
        for c in source.contents[..(offset - source.offset) as usize]
            .chars()
            .rev()
        {
            if c == '\n' {
                break;
            } else if c == ' ' {
                indentation += 1;
            } else if c == '\t' {
                indentation += 4;
            } else {
                indentation = 0;
            }
        }
        indentation
    }
}

/// Creates a lint for deprecated `w/` and `w/=` operators for structs.
impl HirLintPass for DeprecatedWithOperator {
    fn check_expr(&mut self, expr: &Expr, buffer: &mut Vec<Lint>, context: Context) {
        match &expr.kind {
            ExprKind::UpdateField(container, field, value)
            | ExprKind::AssignField(container, field, value) => {
                if let Ty::Udt(ty_name, Res::Item(item_id)) = &container.ty {
                    let (item, _, _) = resolve_item_relative_to_user_package(
                        *item_id,
                        context.user_package_id,
                        context.package_store,
                    );
                    if let ItemKind::Ty(_, udt) = &item.kind {
                        if udt.is_struct() {
                            let field_name = if let Field::Path(path) = field {
                                udt.find_field(path)
                                    .expect("field should exist in struct")
                                    .name
                                    .as_ref()
                                    .expect("struct fields always have names")
                                    .clone()
                            } else {
                                panic!("field should be a path");
                            };
                            let field_value = Rc::from(Self::get_source_code(value.span, context));
                            let field_info = (field_name, field_value);

                            match &mut self.lint_info {
                                Some(existing_info) => {
                                    existing_info.field_assigns.push(field_info);
                                }
                                None => {
                                    self.lint_info = Some(WithOperatorLint {
                                        span: expr.span,
                                        ty_name: ty_name.clone(),
                                        is_w_eq: matches!(expr.kind, ExprKind::AssignField(..)),
                                        field_assigns: vec![field_info],
                                    });
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                if let Some(info) = &self.lint_info {
                    // Construct a Struct constructor expr and print it back into Q# code
                    let indentation =
                        (Self::indentation_at_offset(info.span.lo, context) + 4) as usize;
                    let innermost_expr = Self::get_source_code(expr.span, context);
                    let mut new_expr = if info.is_w_eq {
                        format!("set {} = new {} {{\n", innermost_expr, info.ty_name)
                    } else {
                        format!("new {} {{\n", info.ty_name)
                    };
                    new_expr.push_str(&format!(
                        "{:indent$}...{},\n",
                        "",
                        innermost_expr,
                        indent = indentation
                    ));
                    for (field, value) in info.field_assigns.iter().rev() {
                        new_expr.push_str(&format!(
                            "{:indent$}{} = {},\n",
                            "",
                            field,
                            value,
                            indent = indentation
                        ));
                    }
                    new_expr.push_str(&format!("{:indent$}}}", "", indent = indentation - 4));
                    let code_action_edits = vec![(new_expr, info.span)];

                    lint!(self, info.span, code_action_edits);

                    buffer.push(lint!(self, info.span));
                    self.lint_info = None;
                }
            }
        }
    }
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
