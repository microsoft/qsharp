// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::data_structures::{
    derive_callable_input_elements, derive_callable_input_map, derive_callable_input_params,
    CallableVariable, CallableVariableKind,
};
use qsc_data_structures::index_map::IndexMap;
use qsc_fir::{
    fir::{
        Block, BlockId, CallableDecl, Expr, ExprId, ExprKind, Item, ItemKind, LocalItemId, NodeId,
        Package, PackageId, PackageLookup, Pat, PatId, PatKind, Res, SpecDecl, Stmt, StmtId,
        StmtKind,
    },
    visit::Visitor,
};
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Default)]
struct CallableStack {
    callables_set: FxHashSet<LocalItemId>,
    stack: Vec<LocalItemId>,
}

impl CallableStack {
    fn contains(&self, value: &LocalItemId) -> bool {
        self.callables_set.contains(value)
    }
    fn peak(&self) -> LocalItemId {
        *self.stack.last().expect("stack should not be empty")
    }

    fn pop(&mut self) -> LocalItemId {
        let popped = self.stack.pop().expect("stack should not be empty");
        self.callables_set.remove(&popped);
        popped
    }

    fn push(&mut self, value: LocalItemId) {
        self.callables_set.insert(value);
        self.stack.push(value);
    }
}

struct CycleDetector<'a> {
    package_id: PackageId,
    package: &'a Package,
    stack: CallableStack,
    node_maps: IndexMap<LocalItemId, FxHashMap<NodeId, CallableVariable>>,
    callables_with_cycles: FxHashSet<LocalItemId>,
}

impl<'a> CycleDetector<'a> {
    fn new(package_id: PackageId, package: &'a Package) -> Self {
        Self {
            package_id,
            package,
            stack: CallableStack::default(),
            node_maps: IndexMap::default(),
            callables_with_cycles: FxHashSet::<LocalItemId>::default(),
        }
    }

    fn detect_callables_with_cycles(&mut self) {
        self.visit_package(self.package);
    }

    fn get_callables_with_cycles(&self) -> &FxHashSet<LocalItemId> {
        &self.callables_with_cycles
    }

    fn map_pat_to_expr(&mut self, pat_id: PatId, expr_id: ExprId) {
        let pat = self.get_pat(pat_id);
        match &pat.kind {
            PatKind::Bind(ident) => {
                let callable_id = self.stack.peak();
                let node_map = self
                    .node_maps
                    .get_mut(callable_id)
                    .expect("node map should exist");
                node_map.insert(
                    ident.id,
                    CallableVariable {
                        node: ident.id,
                        pat: pat_id,
                        ty: pat.ty.clone(),
                        kind: CallableVariableKind::Local(expr_id),
                    },
                );
            }
            PatKind::Tuple(_) => {
                // TODO (cesarzc): implement correctly.
            }
            PatKind::Discard => {}
        }
    }

    fn resolve_callee(&self, expr_id: ExprId) -> Option<LocalItemId> {
        let expr = self.get_expr(expr_id);
        match &expr.kind {
            ExprKind::Closure(_, local_item_id) => Some(*local_item_id),
            // TODO (cesarzc): need to take into account the specific functor in order to avoid false positive.
            ExprKind::UnOp(_, expr_id) => self.resolve_callee(*expr_id),
            ExprKind::Var(res, _) => match res {
                Res::Item(item_id) => match item_id.package {
                    Some(package_id) => {
                        if package_id == self.package_id {
                            Some(item_id.item)
                        } else {
                            None
                        }
                    }
                    None => Some(item_id.item),
                },
                Res::Local(node_id) => {
                    let callable_id = self.stack.peak();
                    let node_map = self
                        .node_maps
                        .get(callable_id)
                        .expect("node map should exist");
                    if let Some(callable_variable) = node_map.get(node_id) {
                        match &callable_variable.kind {
                            CallableVariableKind::InputParam(_) => None,
                            CallableVariableKind::Local(expr_id) => self.resolve_callee(*expr_id),
                        }
                    } else {
                        panic!("cannot determine callee from resolution")
                    }
                }
                Res::Err => panic!("resolution should not be error"),
            },
            _ => panic!("cannot determine callee from expression"),
        }
    }
}

// TODO (cesarzc): implement visitor pattern for CycleDetector.
impl<'a> Visitor<'a> for CycleDetector<'a> {
    fn get_block(&self, id: BlockId) -> &'a Block {
        self.package
            .blocks
            .get(id)
            .expect("couldn't find block in FIR")
    }

    fn get_expr(&self, id: ExprId) -> &'a Expr {
        self.package
            .exprs
            .get(id)
            .expect("couldn't find expr in FIR")
    }

    fn get_pat(&self, id: PatId) -> &'a Pat {
        self.package.pats.get(id).expect("couldn't find pat in FIR")
    }

    fn get_stmt(&self, id: StmtId) -> &'a Stmt {
        self.package
            .stmts
            .get(id)
            .expect("couldn't find stmt in FIR")
    }

    fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
        // If not already computed, initialize the node map with its inputs.
        let callable_id = self.stack.peak();
        if !self.node_maps.contains_key(callable_id) {
            let input_elements = derive_callable_input_elements(decl, &self.package.pats);
            let input_params = derive_callable_input_params(input_elements.iter());
            let input_map = derive_callable_input_map(input_params.iter());
            self.node_maps.insert(callable_id, input_map);
        }

        self.visit_callable_impl(&decl.implementation);
    }

    fn visit_expr(&mut self, expr_id: ExprId) {
        let expr = self.get_expr(expr_id);
        // We are only interested in call expressions.
        if let ExprKind::Call(callee, _) = expr.kind {
            // TODO (cesarzc): if passing any function, it needs to check whether the function being passed is already
            // in the stack.
            // Example of this behavior: Microsoft.Quantum.Arrays.Sorted.

            // Visit the callee only if it resolves to a local item.
            if let Some(local_item_id) = self.resolve_callee(callee) {
                let item = self.package.get_item(local_item_id);
                self.visit_item(item);
            }
        }
    }

    fn visit_item(&mut self, item: &'a Item) {
        // We are only interested in visiting callables.
        let ItemKind::Callable(callable) = &item.kind else {
            return;
        };

        // If the callable is already in the stack, it means the callable has a cycle.
        if self.stack.contains(&item.id) {
            self.callables_with_cycles.insert(item.id);
            return;
        }

        // Insert the item as a callable in the stack, visit it and then remove it from the stack.
        self.stack.push(item.id);
        self.visit_callable_decl(callable);
        _ = self.stack.pop();
    }

    fn visit_package(&mut self, package: &'a Package) {
        // We are only interested in visiting items.
        package.items.values().for_each(|i| self.visit_item(i));
    }

    fn visit_spec_decl(&mut self, decl: &'a SpecDecl) {
        self.visit_block(decl.block);
    }

    fn visit_stmt(&mut self, stmt_id: StmtId) {
        let stmt = self.get_stmt(stmt_id);
        match &stmt.kind {
            StmtKind::Item(_) => {}
            StmtKind::Expr(expr_id) | StmtKind::Semi(expr_id) => self.visit_expr(*expr_id),
            StmtKind::Local(_, pat_id, expr_id) => self.map_pat_to_expr(*pat_id, *expr_id),
        }
    }
}

pub fn detect_callables_with_cycles(
    package_id: PackageId,
    package: &Package,
) -> FxHashSet<LocalItemId> {
    let mut cycle_detector = CycleDetector::new(package_id, package);
    cycle_detector.detect_callables_with_cycles();
    cycle_detector.get_callables_with_cycles().clone()
}
