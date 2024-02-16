// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    fir::{Block, BlockId, Expr, ExprId, Package, Pat, PatId, Stmt, StmtId},
    visit::Visitor,
};

pub struct Validator<'a> {
    pub package: &'a Package,
}

pub fn validate(package: &Package) {
    let mut v = Validator { package };
    v.validate();
}

/// Validates that the FIR is well-formed.
/// Running `validate` will validate the entire package.
impl Validator<'_> {
    pub fn validate(&mut self) {
        self.visit_package(self.package);
    }
}

impl<'a> Visitor<'a> for Validator<'a> {
    fn get_block(&self, id: BlockId) -> &'a Block {
        self.package.blocks.get(id).expect("block not found")
    }

    fn get_expr(&self, id: ExprId) -> &'a Expr {
        self.package.exprs.get(id).expect("expr not found")
    }

    fn get_pat(&self, id: PatId) -> &'a Pat {
        self.package.pats.get(id).expect("pat not found")
    }

    fn get_stmt(&self, id: StmtId) -> &'a Stmt {
        self.package.stmts.get(id).expect("stmt not found")
    }
}
