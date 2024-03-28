// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use qsc_fir::{
    fir::{Block, BlockId, Expr, ExprId, Package, PackageLookup, Pat, PatId, Stmt, StmtId},
    visit::Visitor,
};
use qsc_rca::PackageComputeProperties;
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
    package: &'a Package,
    _compute_properties: &'a PackageComputeProperties,
    program: Program,
    assigner: Assigner,
}

impl<'a> PartialEvaluator<'a> {
    fn new(package: &'a Package, compute_properties: &'a PackageComputeProperties) -> Self {
        Self {
            package,
            _compute_properties: compute_properties,
            program: Program::new(),
            assigner: Assigner::default(),
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn eval(mut self) -> Result<Program, Error> {
        let Some(entry_expr_id) = self.package.entry else {
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
        Ok(self.program)
    }
}

impl<'a> Visitor<'a> for PartialEvaluator<'a> {
    fn get_block(&self, id: BlockId) -> &'a Block {
        self.package.get_block(id)
    }

    fn get_expr(&self, id: ExprId) -> &'a Expr {
        self.package.get_expr(id)
    }

    fn get_pat(&self, id: PatId) -> &'a Pat {
        self.package.get_pat(id)
    }

    fn get_stmt(&self, id: StmtId) -> &'a Stmt {
        self.package.get_stmt(id)
    }
}

pub enum Error {
    EvaluationFailed,
}

pub fn partially_evaluate(
    package: &Package,
    compute_properties: &PackageComputeProperties,
) -> Result<Program, Error> {
    let partial_evaluator = PartialEvaluator::new(package, compute_properties);
    partial_evaluator.eval()
}
