// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Unit tests for [`cleanup_consumed_closures`], exercising each filter step
//! that decides whether a consumed closure expression is replaced with `Unit`.
//!
//! Tests compile Q# to monomorphized FIR (closures still present, before
//! defunctionalization), then call `cleanup_consumed_closures` directly with
//! crafted `specialized_targets` / `skip_items` inputs to drive one branch of
//! the filter and assert the returned replacement count and the resulting
//! expression kinds. Dependency tests also check the exact assignment mappings
//! and live expression set, since conservative local-read roots can hide lost
//! precision in the replacement count alone.

use super::*;
use crate::defunctionalize::{
    cleanup_consumed_closures, collect_assignment_definitions, collect_live_call_argument_exprs,
};
use crate::walk_utils::CallableNode;
use qsc_fir::fir::{ExprId, ExprKind, LocalItemId, Package, PackageLookup, PatKind};
use qsc_fir::ty::Ty;
use rustc_hash::{FxHashMap, FxHashSet};

/// Compiles `source` to monomorphized FIR and returns the store, the user
/// package id, and the reachable local callable ids (the scope passed to
/// `cleanup_consumed_closures`).
fn setup(source: &str) -> (fir::PackageStore, fir::PackageId, Vec<LocalItemId>) {
    let (fir_store, fir_pkg_id) = compile_to_monomorphized_fir(source);
    let reachable = collect_reachable_from_entry(&fir_store, fir_pkg_id);
    let package = fir_store.get(fir_pkg_id);
    let reachable_item_ids: Vec<LocalItemId> =
        reachable_local_callables(package, fir_pkg_id, &reachable)
            .map(|(id, _)| id)
            .collect();
    (fir_store, fir_pkg_id, reachable_item_ids)
}

/// Collects every `(closure expr id, target callable id)` pair reachable from
/// the entry-reachable callables and the entry expression.
fn all_closures(
    package: &Package,
    reachable_item_ids: &[LocalItemId],
) -> Vec<(ExprId, LocalItemId)> {
    let mut found: Vec<(ExprId, LocalItemId)> = Vec::new();
    for &item_id in reachable_item_ids {
        if let ItemKind::Callable(decl) = &package.get_item(item_id).kind {
            crate::walk_utils::for_each_expr_in_callable_impl(
                package,
                &decl.implementation,
                &mut |expr_id, expr| {
                    if let ExprKind::Closure(_, target) = &expr.kind {
                        found.push((expr_id, *target));
                    }
                },
            );
        }
    }
    if let Some(entry_id) = package.entry {
        crate::walk_utils::for_each_expr(package, entry_id, &mut |expr_id, expr| {
            if let ExprKind::Closure(_, target) = &expr.kind {
                found.push((expr_id, *target));
            }
        });
    }
    found
}

/// Returns the single closure expr id and its target callable id, asserting
/// that exactly one closure is present in the reachable scope.
fn single_closure(package: &Package, reachable_item_ids: &[LocalItemId]) -> (ExprId, LocalItemId) {
    let closures = all_closures(package, reachable_item_ids);
    assert_eq!(
        closures.len(),
        1,
        "expected exactly one closure in the reachable scope, found {}",
        closures.len()
    );
    closures[0]
}

/// Finds the reachable callable item with the given display name.
fn find_callable(package: &Package, reachable_item_ids: &[LocalItemId], name: &str) -> LocalItemId {
    for &item_id in reachable_item_ids {
        if let ItemKind::Callable(decl) = &package.get_item(item_id).kind
            && decl.name.name.as_ref() == name
        {
            return item_id;
        }
    }
    panic!("callable {name} not found in reachable scope");
}

/// True when the expression has been rewritten to the empty-tuple `Unit` value.
fn is_unit_tuple(package: &Package, expr_id: ExprId) -> bool {
    let expr = package.get_expr(expr_id);
    let kind_is_empty_tuple = matches!(&expr.kind, ExprKind::Tuple(elems) if elems.is_empty());
    let ty_is_unit = matches!(&expr.ty, Ty::Tuple(elems) if elems.is_empty());
    kind_is_empty_tuple && ty_is_unit
}

/// True when the expression is still a closure.
fn is_closure(package: &Package, expr_id: ExprId) -> bool {
    matches!(package.get_expr(expr_id).kind, ExprKind::Closure(_, _))
}

/// Compiles Q# to monomorphized FIR and checks the assignment dependencies
/// collected from `Main`, excluding local declaration initializers.
///
/// `expected` maps each written local's name to the exact source snippets of its
/// dependency expressions. Sort entries by name and each snippet list
/// lexicographically, retaining duplicates. Comparing these mappings directly
/// detects lost precision that conservative liveness roots can otherwise hide.
fn check_assignment_definitions(source: &str, expected: &[(&str, &[&str])]) {
    let (store, package_id, reachable) = setup(source);
    let package = store.get(package_id);
    let main = find_callable(package, &reachable, "Main");
    let ItemKind::Callable(decl) = &package.get_item(main).kind else {
        panic!("Main must be a callable");
    };
    let mut names = FxHashMap::default();
    let mut definitions = FxHashMap::default();
    crate::walk_utils::for_each_node_in_callable(package, decl, &mut |node| match node {
        CallableNode::Pat(pat) => {
            if let PatKind::Bind(ident) = &package.get_pat(pat).kind {
                names.insert(ident.id, ident.name.clone());
            }
        }
        CallableNode::Expr(expr) => {
            collect_assignment_definitions(package, expr, &mut definitions);
        }
        CallableNode::Stmt(_) | CallableNode::Block(_) => {}
    });
    let mut actual: Vec<(&str, Vec<&str>)> = definitions
        .iter()
        .map(|(local, exprs)| {
            let mut sources: Vec<_> = exprs
                .iter()
                .map(|&expr| {
                    let span = package.get_expr(expr).span;
                    &source[span.lo as usize..span.hi as usize]
                })
                .collect();
            sources.sort_unstable();
            (
                names
                    .get(local)
                    .expect("written local must have a binding")
                    .as_ref(),
                sources,
            )
        })
        .collect();
    actual.sort_unstable_by_key(|(name, _)| *name);
    let expected: Vec<_> = expected
        .iter()
        .map(|(name, sources)| (*name, sources.to_vec()))
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn assignment_definitions_track_simple_rhs_and_reassignments() {
    check_assignment_definitions(
        r#"
        operation Main() : Unit {
            mutable value = 0;
            set value = 1;
            set value = 2;
        }
        "#,
        &[("value", &["1", "2"])],
    );
}

#[test]
fn assignment_definitions_match_nested_tuple_elements() {
    check_assignment_definitions(
        r#"
        operation Main() : Unit {
            mutable (first, (second, third)) = (0, (0, 0));
            set (first, (second, third)) = (1, (2, 3));
        }
        "#,
        &[("first", &["1"]), ("second", &["2"]), ("third", &["3"])],
    );
}

#[test]
fn assignment_definitions_preserve_nonliteral_tuple_rhs() {
    for rhs in ["pair", "Pair()", "{ pair }"] {
        let source = format!(
            r#"
            function Pair() : (Int, (Int, Int)) {{ (1, (2, 3)) }}
            operation Main() : Unit {{
                mutable (first, (second, third)) = (0, (0, 0));
                let pair = Pair();
                set (first, (second, third)) = {rhs};
            }}
            "#
        );
        check_assignment_definitions(
            &source,
            &[("first", &[rhs]), ("second", &[rhs]), ("third", &[rhs])],
        );
    }
}

#[test]
fn assignment_definitions_mix_precise_and_nonliteral_tuple_elements() {
    check_assignment_definitions(
        r#"
        operation Main() : Unit {
            mutable (first, (second, third)) = (0, (0, 0));
            let pair = (2, 3);
            set (first, (second, third)) = (1, pair);
        }
        "#,
        &[
            ("first", &["1"]),
            ("second", &["pair"]),
            ("third", &["pair"]),
        ],
    );
}

#[test]
fn assignment_definitions_preserve_whole_update_expressions() {
    check_assignment_definitions(
        r#"
        struct Wrapper { Value : Int }
        operation Main() : Unit {
            mutable value = 0;
            mutable values = [0];
            mutable wrapper = new Wrapper { Value = 0 };
            set value += 1;
            set values w/= 0 <- 2;
            set wrapper w/= Value <- 3;
        }
        "#,
        &[
            ("value", &["set value += 1"]),
            ("values", &["set values w/= 0 <- 2"]),
            ("wrapper", &["set wrapper w/= Value <- 3"]),
        ],
    );
}

#[test]
fn assignment_definitions_preserve_projected_destination_operands() {
    let source = r#"
        struct Wrapper { Value : Int }
        operation Main() : Unit {
            mutable values = [0];
            mutable wrapper = new Wrapper { Value = 0 };
            set values w/= 0 <- 2;
            set wrapper w/= Value <- 3;
        }
    "#;
    let (mut store, package_id, _) = setup(source);
    let package = store.get_mut(package_id);
    let updates: Vec<_> = package
        .exprs
        .iter()
        .filter(|(_, expr)| {
            matches!(
                expr.kind,
                ExprKind::AssignField(..) | ExprKind::AssignIndex(..)
            )
        })
        .map(|(id, expr)| (id, expr.clone()))
        .collect();
    assert_eq!(updates.len(), 2);
    let mut assigner = qsc_fir::assigner::Assigner::from_package(package);
    for (id, expr) in updates {
        let locals = defunc_analysis::assignment_written_locals(package, &expr);
        assert_eq!(locals.len(), 1);
        let (lhs_kind, rhs) = match expr.kind {
            ExprKind::AssignField(base, field, rhs) => (ExprKind::Field(base, field), rhs),
            ExprKind::AssignIndex(base, index, rhs) => (ExprKind::Index(base, index), rhs),
            _ => unreachable!("only field and index updates were collected"),
        };
        let lhs = crate::fir_builder::alloc_expr(
            package,
            &mut assigner,
            package.get_expr(rhs).ty.clone(),
            lhs_kind,
            expr.span,
        );
        package
            .exprs
            .get_mut(id)
            .expect("assignment must exist")
            .kind = ExprKind::Assign(lhs, rhs);
        let mut definitions = FxHashMap::default();
        collect_assignment_definitions(package, id, &mut definitions);
        assert_eq!(definitions, FxHashMap::from_iter([(locals[0], vec![id])]));
    }
}

#[test]
fn assignment_liveness_does_not_reintroduce_whole_tuple_dependencies() {
    let source = r#"
        operation Main() : Unit {
            mutable (first, (second, third)) = (0, (0, 0));
            set (first, (second, third)) = (1, (2, 3));
        }
    "#;
    let (store, package_id, reachable) = setup(source);
    let package = store.get(package_id);
    let main = find_callable(package, &reachable, "Main");
    let ItemKind::Callable(decl) = &package.get_item(main).kind else {
        panic!("Main must be a callable");
    };
    let mut nodes = Vec::new();
    crate::walk_utils::for_each_node_in_callable(package, decl, &mut |node| nodes.push(node));
    let mut live = FxHashSet::default();
    collect_live_call_argument_exprs(package, package_id, &nodes, &mut live);
    let assignments: Vec<_> = nodes
        .iter()
        .filter_map(|node| {
            if let CallableNode::Expr(id) = node
                && let ExprKind::Assign(_, rhs) = package.get_expr(*id).kind
            {
                Some((*id, rhs))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(assignments.len(), 1);
    let (assignment, rhs) = assignments[0];
    assert!(
        !live.contains(&assignment),
        "only assigned values should be roots"
    );
    crate::walk_utils::for_each_expr(package, rhs, &mut |id, expr| {
        assert_eq!(
            live.contains(&id),
            !matches!(expr.kind, ExprKind::Tuple(_)),
            "tuple values must be protected without protecting their wrappers"
        );
    });
}

/// A closure passed directly as an argument to an ordinary (non-UDT) HOF call,
/// which after monomorphization is `ApplyOp_Empty_(closure, q)`.
const CALL_ARG_SOURCE: &str = r#"
    operation ApplyOp(op : Qubit => Unit, q : Qubit) : Unit {
        op(q);
    }
    operation Main() : Unit {
        use q = Qubit();
        ApplyOp(x => H(x), q);
    }
    "#;

/// A closure bound to a `let` local (`let f = closure; f(q)`), so the closure
/// expression is not nested inside any call-argument subtree.
const LET_BOUND_SOURCE: &str = r#"
    operation Main() : Unit {
        let f = x => H(x);
        use q = Qubit();
        f(q);
    }
    "#;

/// A closure passed to a `newtype` UDT constructor (`let w = W(closure)`), so
/// the closure sits inside a UDT-constructor call-argument subtree.
const UDT_CTOR_SOURCE: &str = r#"
    newtype W = (Qubit => Unit);
    operation Main() : Unit {
        let w = W(x => H(x));
        use q = Qubit();
        (w!)(q);
    }
    "#;

/// No reads can protect this binding, so tests isolate the cleanup filters.
const UNUSED_LET_BOUND_SOURCE: &str = r#"
    operation Main() : Unit {
        let offset = 17;
        let f = x -> x+offset;
    }
    "#;

/// Filter step: the `specialized_targets.is_empty()` early return. With no
/// consumed targets, the function returns 0 and leaves every closure intact.
#[test]
fn empty_specialized_targets_returns_zero_and_preserves_closure() {
    let (mut fir_store, fir_pkg_id, reachable_item_ids) = setup(UNUSED_LET_BOUND_SOURCE);
    let (closure_expr, _target) = single_closure(fir_store.get(fir_pkg_id), &reachable_item_ids);

    let package = fir_store.get_mut(fir_pkg_id);
    let replaced = cleanup_consumed_closures(
        package,
        fir_pkg_id,
        &FxHashSet::default(),
        &FxHashSet::default(),
        &reachable_item_ids,
    );

    assert_eq!(
        replaced, 0,
        "no targets specialized, nothing should be cleaned"
    );
    assert!(
        is_closure(package, closure_expr),
        "closure must be preserved when no targets are specialized"
    );
}

#[test]
fn callable_argument_aliases_preserve_their_closure_initializers() {
    let source = r#"
        function Apply(f : Int -> Int, x : Int) : Int { f(x) }
        operation Main() : Int {
            let offset = 17;
            let f = x -> x+offset;
            let firstAlias = f;
            let secondAlias = firstAlias;
            Apply(secondAlias, 3)
        }
    "#;
    let (mut store, package_id, reachable) = setup(source);
    let (closure, target) = single_closure(store.get(package_id), &reachable);
    let replaced = cleanup_consumed_closures(
        store.get_mut(package_id),
        package_id,
        &FxHashSet::from_iter([target]),
        &FxHashSet::default(),
        &reachable,
    );
    assert_eq!(replaced, 0, "a live aliased argument is not consumed");
    assert!(is_closure(store.get(package_id), closure));
    crate::test_utils::check_semantic_equivalence(source);
}

#[test]
fn callable_argument_assignments_preserve_all_reaching_closures() {
    let source = r#"
        function Apply(f : Int -> Int, x : Int) : Int { f(x) }
        operation Main() : Int {
            let offset = 17;
            mutable f = x -> x+offset;
            set f = x -> x-offset;
            Apply(f, 3)
        }
    "#;
    let (mut store, package_id, reachable) = setup(source);
    let closures = all_closures(store.get(package_id), &reachable);
    assert_eq!(closures.len(), 2);
    let targets = closures.iter().map(|(_, target)| *target).collect();
    let replaced = cleanup_consumed_closures(
        store.get_mut(package_id),
        package_id,
        &targets,
        &FxHashSet::default(),
        &reachable,
    );
    assert_eq!(
        replaced, 0,
        "both definitions of a live argument must survive"
    );
    for (closure, _) in closures {
        assert!(is_closure(store.get(package_id), closure));
    }
    crate::test_utils::check_semantic_equivalence(source);
}

fn check_callable_update_preserves_closures(source: &str) {
    let (mut store, package_id, reachable) = setup(source);
    let closures = all_closures(store.get(package_id), &reachable);
    assert_eq!(closures.len(), 2, "expected initial and update closures");
    let targets = closures.iter().map(|(_, target)| *target).collect();
    let replaced = cleanup_consumed_closures(
        store.get_mut(package_id),
        package_id,
        &targets,
        &FxHashSet::default(),
        &reachable,
    );
    assert_eq!(
        replaced, 0,
        "cleanup must preserve both the initial and update closures"
    );
    for (closure, _) in closures {
        assert!(
            is_closure(store.get(package_id), closure),
            "cleanup must leave each callable value as a closure"
        );
    }
}

#[test]
fn callable_array_append_preserves_initial_and_appended_closures() {
    check_callable_update_preserves_closures(
        r#"
        operation Main() : Unit {
            let offset = 17;
            mutable values = [x -> x+offset];
            set values += [x -> x-offset];
        }
        "#,
    );
}

#[test]
fn callable_array_index_update_preserves_initial_and_replacement_closures() {
    check_callable_update_preserves_closures(
        r#"
        operation Main() : Unit {
            let offset = 17;
            mutable values = [x -> x+offset];
            set values w/= 0 <- x -> x-offset;
        }
        "#,
    );
}

#[test]
fn callable_struct_field_update_preserves_initial_and_replacement_closures() {
    check_callable_update_preserves_closures(
        r#"
        struct Wrapper { Callable : Int -> Int }
        operation Main() : Unit {
            let offset = 17;
            mutable wrapper = new Wrapper { Callable = x -> x+offset };
            set wrapper w/= Callable <- x -> x-offset;
        }
        "#,
    );
}

#[test]
fn tuple_bound_argument_preserves_transitively_captured_closure() {
    let source = r#"
        function Apply(f : Int -> Int, x : Int) : Int { f(x) }
        operation Main() : Int {
            let offset = 17;
            let f = x -> x+offset;
            let (_, g) = (0, x -> 2*f(x));
            Apply(g, 3)
        }
    "#;
    let (mut store, package_id, reachable) = setup(source);
    let closures = all_closures(store.get(package_id), &reachable);
    assert_eq!(closures.len(), 2);
    let targets = closures.iter().map(|(_, target)| *target).collect();
    let replaced = cleanup_consumed_closures(
        store.get_mut(package_id),
        package_id,
        &targets,
        &FxHashSet::default(),
        &reachable,
    );
    assert_eq!(
        replaced, 0,
        "the argument and its captured callable remain live"
    );
    for (closure, _) in closures {
        assert!(is_closure(store.get(package_id), closure));
    }
    crate::test_utils::check_semantic_equivalence(source);
}

#[test]
fn unused_nested_tuple_binding_does_not_keep_sibling_closure_live() {
    let source = r#"
        operation Main() : Int {
            let offset = 17;
            let ((f, _), value) = ((x -> x+offset, 0), 3);
            value + 1
        }
    "#;
    let (mut store, package_id, reachable) = setup(source);
    let (closure, target) = single_closure(store.get(package_id), &reachable);
    let replaced = cleanup_consumed_closures(
        store.get_mut(package_id),
        package_id,
        &FxHashSet::from_iter([target]),
        &FxHashSet::default(),
        &reachable,
    );
    assert_eq!(
        replaced, 1,
        "reading one tuple element should not keep a sibling closure live"
    );
    assert!(is_unit_tuple(store.get(package_id), closure));
    crate::test_utils::check_semantic_equivalence(source);
}

#[test]
fn tuple_assignment_preserves_all_reaching_closures() {
    let source = r#"
        function Apply(f : Int -> Int, x : Int) : Int { f(x) }
        operation Main() : Int {
            let offset = 17;
            mutable (f, value) = (x -> x+offset, 3);
            set (f, value) = (x -> x-offset, 4);
            Apply(f, value)
        }
    "#;
    let (mut store, package_id, reachable) = setup(source);
    let closures = all_closures(store.get(package_id), &reachable);
    assert_eq!(closures.len(), 2);
    let targets = closures.iter().map(|(_, target)| *target).collect();
    let replaced = cleanup_consumed_closures(
        store.get_mut(package_id),
        package_id,
        &targets,
        &FxHashSet::default(),
        &reachable,
    );
    assert_eq!(
        replaced, 0,
        "tuple assignments must keep every reaching callable definition live"
    );
    for (closure, _) in closures {
        assert!(is_closure(store.get(package_id), closure));
    }
    crate::test_utils::check_semantic_equivalence(source);
}

#[test]
fn cyclic_callable_alias_assignments_preserve_live_closures_and_terminate() {
    let source = r#"
        function Apply(f : Int -> Int, x : Int) : Int { f(x) }
        operation Main() : Int {
            let offset = 17;
            mutable first = x -> x+offset;
            mutable second = first;
            set first = second;
            set second = first;
            Apply(second, 3)
        }
    "#;
    let (mut store, package_id, reachable) = setup(source);
    let (closure, target) = single_closure(store.get(package_id), &reachable);
    let replaced = cleanup_consumed_closures(
        store.get_mut(package_id),
        package_id,
        &FxHashSet::from_iter([target]),
        &FxHashSet::default(),
        &reachable,
    );
    assert_eq!(replaced, 0);
    assert!(is_closure(store.get(package_id), closure));
    crate::test_utils::check_semantic_equivalence(source);
}

#[test]
fn unresolved_direct_callee_preserves_its_closure_initializer() {
    let source = r#"
        operation Main() : Int {
            let offset = 17;
            let f = x -> x+offset;
            f(3)
        }
    "#;
    let (mut store, package_id, reachable) = setup(source);
    let (closure, target) = single_closure(store.get(package_id), &reachable);
    let replaced = cleanup_consumed_closures(
        store.get_mut(package_id),
        package_id,
        &FxHashSet::from_iter([target]),
        &FxHashSet::default(),
        &reachable,
    );
    assert_eq!(replaced, 0, "the callee still reads its closure value");
    assert!(is_closure(store.get(package_id), closure));
    crate::test_utils::check_semantic_equivalence(source);
}

#[test]
fn live_let_bound_callables_are_preserved_with_and_without_udt_wrapping() {
    for source in [LET_BOUND_SOURCE, UDT_CTOR_SOURCE] {
        let (mut store, package_id, reachable) = setup(source);
        let (closure, target) = single_closure(store.get(package_id), &reachable);
        let replaced = cleanup_consumed_closures(
            store.get_mut(package_id),
            package_id,
            &FxHashSet::from_iter([target]),
            &FxHashSet::default(),
            &reachable,
        );
        assert_eq!(
            replaced, 0,
            "a surviving callable usage needs its initializer"
        );
        assert!(is_closure(store.get(package_id), closure));
    }
}

/// Filter step: the `specialized_targets.contains(target)` membership test.
/// When the set holds an unrelated callable id, the closure's target does not
/// match, so the closure is preserved.
#[test]
fn non_matching_target_preserves_closure() {
    let (mut fir_store, fir_pkg_id, reachable_item_ids) = setup(UNUSED_LET_BOUND_SOURCE);
    let package = fir_store.get(fir_pkg_id);
    let (closure_expr, target) = single_closure(package, &reachable_item_ids);
    // Use `Main` as a real-but-unrelated id that is not the closure target.
    let unrelated = find_callable(package, &reachable_item_ids, "Main");
    assert_ne!(
        unrelated, target,
        "Main must differ from the closure target"
    );

    let mut specialized_targets = FxHashSet::default();
    specialized_targets.insert(unrelated);

    let package = fir_store.get_mut(fir_pkg_id);
    let replaced = cleanup_consumed_closures(
        package,
        fir_pkg_id,
        &specialized_targets,
        &FxHashSet::default(),
        &reachable_item_ids,
    );

    assert_eq!(replaced, 0, "closure target is not in specialized set");
    assert!(
        is_closure(package, closure_expr),
        "closure must be preserved when its target is not specialized"
    );
}

/// Filter step: the positive cleanup path. A consumed closure that is not a
/// read or live call argument is replaced with the empty-tuple `Unit` value.
#[test]
fn consumed_closure_outside_call_arg_is_cleaned() {
    let (mut fir_store, fir_pkg_id, reachable_item_ids) = setup(UNUSED_LET_BOUND_SOURCE);
    let package = fir_store.get(fir_pkg_id);
    let (closure_expr, target) = single_closure(package, &reachable_item_ids);

    let mut specialized_targets = FxHashSet::default();
    specialized_targets.insert(target);

    let package = fir_store.get_mut(fir_pkg_id);
    let replaced = cleanup_consumed_closures(
        package,
        fir_pkg_id,
        &specialized_targets,
        &FxHashSet::default(),
        &reachable_item_ids,
    );

    assert_eq!(
        replaced, 1,
        "the consumed let-bound closure should be cleaned"
    );
    assert!(
        is_unit_tuple(package, closure_expr),
        "cleaned closure must become an empty-tuple Unit value"
    );
}

/// Filter step: the `skip_items.contains(item_id)` guard. Even when the
/// closure's target is specialized, a closure inside a skipped (freshly
/// specialized) item is left untouched.
#[test]
fn closure_in_skipped_item_is_preserved() {
    let (mut fir_store, fir_pkg_id, reachable_item_ids) = setup(UNUSED_LET_BOUND_SOURCE);
    let package = fir_store.get(fir_pkg_id);
    let (closure_expr, target) = single_closure(package, &reachable_item_ids);
    // The closure lives in `Main`'s body; skipping `Main` must suppress cleanup.
    let main_id = find_callable(package, &reachable_item_ids, "Main");

    let mut specialized_targets = FxHashSet::default();
    specialized_targets.insert(target);
    let mut skip_items = FxHashSet::default();
    skip_items.insert(main_id);

    let package = fir_store.get_mut(fir_pkg_id);
    let replaced = cleanup_consumed_closures(
        package,
        fir_pkg_id,
        &specialized_targets,
        &skip_items,
        &reachable_item_ids,
    );

    assert_eq!(replaced, 0, "closure in a skipped item must not be cleaned");
    assert!(
        is_closure(package, closure_expr),
        "closure must be preserved when its enclosing item is skipped"
    );
}

/// Filter step: the `!call_arg_exprs.contains(expr_id)` guard. A consumed
/// closure that is still a live argument of an ordinary HOF call must survive
/// so a later fixpoint iteration can specialize on it.
#[test]
fn live_call_arg_closure_is_preserved() {
    let (mut fir_store, fir_pkg_id, reachable_item_ids) = setup(CALL_ARG_SOURCE);
    let package = fir_store.get(fir_pkg_id);
    let (closure_expr, target) = single_closure(package, &reachable_item_ids);

    let mut specialized_targets = FxHashSet::default();
    specialized_targets.insert(target);

    let package = fir_store.get_mut(fir_pkg_id);
    let replaced = cleanup_consumed_closures(
        package,
        fir_pkg_id,
        &specialized_targets,
        &FxHashSet::default(),
        &reachable_item_ids,
    );

    assert_eq!(
        replaced, 0,
        "a live call-argument closure must not be cleaned"
    );
    assert!(
        is_closure(package, closure_expr),
        "closure passed as a live HOF argument must be preserved"
    );
}

/// Filter step: the `is_udt_ctor_call` exception. A closure inside a UDT
/// constructor call-argument subtree is a structural wrapper, not a live HOF
/// argument, so it remains eligible for cleanup when the wrapper is unused.
#[test]
fn udt_ctor_wrapped_closure_is_cleaned() {
    let source = r#"
        newtype W = (Int -> Int);
        operation Main() : Unit {
            let offset = 17;
            let w = W(x -> x+offset);
        }
    "#;
    let (mut fir_store, fir_pkg_id, reachable_item_ids) = setup(source);
    let package = fir_store.get(fir_pkg_id);
    let (closure_expr, target) = single_closure(package, &reachable_item_ids);

    let mut specialized_targets = FxHashSet::default();
    specialized_targets.insert(target);

    let package = fir_store.get_mut(fir_pkg_id);
    let replaced = cleanup_consumed_closures(
        package,
        fir_pkg_id,
        &specialized_targets,
        &FxHashSet::default(),
        &reachable_item_ids,
    );

    assert_eq!(
        replaced, 1,
        "closure inside a UDT-constructor call must be cleaned"
    );
    assert!(
        is_unit_tuple(package, closure_expr),
        "cleaned UDT-wrapped closure must become an empty-tuple Unit value"
    );
}
