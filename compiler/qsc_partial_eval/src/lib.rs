// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use qsc_fir::{
    fir::{
        Block, BlockId, Expr, ExprId, PackageId, PackageStore, PackageStoreLookup, Pat, PatId,
        Stmt, StmtId, StoreBlockId, StoreExprId, StorePatId, StoreStmtId,
    },
    visit::Visitor,
};
use qsc_rca::PackageStoreComputeProperties;
use qsc_rir::rir::{self, Program};
use std::result::Result;

#[derive(Debug, Default)]
struct Assigner {
    next_callable: rir::CallableId,
    next_block: rir::BlockId,
}

impl Assigner {
    pub fn next_block(&mut self) -> rir::BlockId {
        let id = self.next_block;
        self.next_block = id.successor();
        id
    }

    pub fn next_callable(&mut self) -> rir::CallableId {
        let id = self.next_callable;
        self.next_callable = id.successor();
        id
    }
}

struct PartialEvaluator<'a> {
    package_store: &'a PackageStore,
    _compute_properties: &'a PackageStoreComputeProperties,
    program: Program,
    assigner: Assigner,
    current_package: Option<PackageId>,
}

impl<'a> PartialEvaluator<'a> {
    fn new(
        package_store: &'a PackageStore,
        compute_properties: &'a PackageStoreComputeProperties,
    ) -> Self {
        Self {
            package_store,
            _compute_properties: compute_properties,
            program: Program::new(),
            assigner: Assigner::default(),
            current_package: None,
        }
    }

    fn clear_current_package(&mut self) -> PackageId {
        self.current_package
            .take()
            .expect("there is no package to clear")
    }

    #[allow(clippy::unnecessary_wraps)]
    fn eval(mut self, package_id: PackageId) -> Result<Program, Error> {
        self.set_current_package(package_id);
        let entry_package = self.package_store.get(package_id);
        let Some(entry_expr_id) = entry_package.entry else {
            panic!("package does not have an entry expression");
        };

        // Create entry-point callable.
        let entry_block_id = self.assigner.next_block();
        let entry_block = rir::Block(Vec::new());
        self.program.blocks.insert(entry_block_id, entry_block);
        let entry_point_id = self.assigner.next_callable();
        let entry_point = rir::Callable {
            name: "main".into(),
            input_type: Vec::new(),
            output_type: None,
            body: Some(entry_block_id),
        };
        self.program.callables.insert(entry_point_id, entry_point);
        self.program.entry = entry_point_id;

        // Visit the entry point expression.
        self.visit_expr(entry_expr_id);
        self.clear_current_package();
        Ok(self.program)
    }

    fn get_current_package(&self) -> PackageId {
        self.current_package.expect("current package is not set")
    }

    fn set_current_package(&mut self, package_id: PackageId) {
        assert!(self.current_package.is_none());
        self.current_package = Some(package_id);
    }
}

impl<'a> Visitor<'a> for PartialEvaluator<'a> {
    fn get_block(&self, id: BlockId) -> &'a Block {
        let block_id = StoreBlockId::from((self.get_current_package(), id));
        self.package_store.get_block(block_id)
    }

    fn get_expr(&self, id: ExprId) -> &'a Expr {
        let expr_id = StoreExprId::from((self.get_current_package(), id));
        self.package_store.get_expr(expr_id)
    }

    fn get_pat(&self, id: PatId) -> &'a Pat {
        let pat_id = StorePatId::from((self.get_current_package(), id));
        self.package_store.get_pat(pat_id)
    }

    fn get_stmt(&self, id: StmtId) -> &'a Stmt {
        let stmt_id = StoreStmtId::from((self.get_current_package(), id));
        self.package_store.get_stmt(stmt_id)
    }
}

pub enum Error {
    EvaluationFailed,
}

pub fn partially_evaluate(
    package_id: PackageId,
    package_store: &PackageStore,
    compute_properties: &PackageStoreComputeProperties,
) -> Result<Program, Error> {
    let partial_evaluator = PartialEvaluator::new(package_store, compute_properties);
    partial_evaluator.eval(package_id)
}
