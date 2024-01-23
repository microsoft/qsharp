// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_fir::{
    fir::{
        Block, BlockId, Expr, ExprId, ExprKind, Item, ItemKind, LocalItemId, Package, Pat, PatId,
        Res, Stmt, StmtId,
    },
    visit::Visitor,
};
use rustc_hash::FxHashSet;

#[derive(Debug)]
struct CycleDetector<'a> {
    package: &'a Package,
    callables_with_cycles: FxHashSet<LocalItemId>,
    current_callables_in_stack: FxHashSet<LocalItemId>,
}

impl<'a> CycleDetector<'a> {
    fn new(package: &'a Package) -> Self {
        Self {
            package,
            callables_with_cycles: FxHashSet::<LocalItemId>::default(),
            current_callables_in_stack: FxHashSet::<LocalItemId>::default(),
        }
    }

    fn detect_callables_with_cycles(&mut self) {
        self.visit_package(self.package);
    }

    fn determine_callee_item(&mut self, expr_id: ExprId) -> &Item {
        let expr = self.get_expr(expr_id);
        let local_item_id = match &expr.kind {
            ExprKind::Closure(_, local_item_id) => local_item_id,
            ExprKind::Var(res, _) => match res {
                Res::Item(item_id) => &item_id.item,
                Res::Local(_) => panic!("should deal with this case somehow"),
                Res::Err => panic!("resolution should not be error"),
            },
            _ => panic!("cannot determine callee from expression"),
        };
        self.package
            .items
            .get(*local_item_id)
            .expect("item should exist")
    }

    fn get_callables_with_cycles(&self) -> &FxHashSet<LocalItemId> {
        &self.callables_with_cycles
    }
}

// TODO (cesarzc): implement visitor pattern for CycleDetector.
impl<'a> Visitor<'a> for CycleDetector<'a> {
    fn get_block(&mut self, id: BlockId) -> &'a Block {
        self.package
            .blocks
            .get(id)
            .expect("couldn't find block in FIR")
    }

    fn get_expr(&mut self, id: ExprId) -> &'a Expr {
        self.package
            .exprs
            .get(id)
            .expect("couldn't find expr in FIR")
    }

    fn get_pat(&mut self, id: PatId) -> &'a Pat {
        self.package.pats.get(id).expect("couldn't find pat in FIR")
    }

    fn get_stmt(&mut self, id: StmtId) -> &'a Stmt {
        self.package
            .stmts
            .get(id)
            .expect("couldn't find stmt in FIR")
    }

    fn visit_expr(&mut self, expr: ExprId) {
        let expr = self.get_expr(expr);
        // We are only interested in call expressions.
        match &expr.kind {
            ExprKind::Call(callee, _) => {
                // TODO (cesarzc): do this when not having non-mutable getters.
                //let item = self.determine_callee_item(*callee);
                //self.visit_item(item);
            }
            _ => {}
        };
    }

    fn visit_item(&mut self, item: &'a Item) {
        // We are only interested in visiting callables.
        let ItemKind::Callable(callable) = &item.kind else {
            return;
        };

        // If cycles have already been identified for the callable, there is no need to continue.
        if self.callables_with_cycles.contains(&item.id) {
            return;
        }

        // If the callable is already in the stack, it means the callable has a cycle.
        if self.current_callables_in_stack.contains(&item.id) {
            self.callables_with_cycles.insert(item.id);
            return;
        }

        // Insert the item as a callable in the stack, visit it and then remove it from the stack.
        self.current_callables_in_stack.insert(item.id);
        self.visit_callable_decl(callable);
        self.current_callables_in_stack.remove(&item.id);
    }

    fn visit_package(&mut self, package: &'a Package) {
        // We are only interested in visiting items.
        package.items.values().for_each(|i| self.visit_item(i));
    }
}

pub fn detect_callables_with_cycles(package: &Package) -> FxHashSet<LocalItemId> {
    let mut cycle_detector = CycleDetector::new(package);
    cycle_detector.detect_callables_with_cycles();
    cycle_detector.get_callables_with_cycles().clone()
}
