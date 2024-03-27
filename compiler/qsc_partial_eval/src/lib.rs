// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_fir::{
    fir::{Block, BlockId, Expr, ExprId, Package, PackageLookup, Pat, PatId, Stmt, StmtId},
    visit::Visitor,
};
use qsc_rca::PackageComputeProperties;
use qsc_rir::rir::Program;

struct PartialEvaluator<'a> {
    package: &'a Package,
    _compute_properties: &'a PackageComputeProperties,
    program: Program,
}

impl<'a> PartialEvaluator<'a> {
    fn new(package: &'a Package, compute_properties: &'a PackageComputeProperties) -> Self {
        Self {
            package,
            _compute_properties: compute_properties,
            program: Program::new(),
        }
    }

    fn eval(mut self) -> Program {
        let Some(entry_expr_id) = self.package.entry else {
            panic!("package does not have an entry expression");
        };

        self.visit_expr(entry_expr_id);
        self.program
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

#[must_use]
pub fn partially_evaluate(
    package: &Package,
    compute_properties: &PackageComputeProperties,
) -> Program {
    let partial_evaluator = PartialEvaluator::new(package, compute_properties);
    partial_evaluator.eval()
}
