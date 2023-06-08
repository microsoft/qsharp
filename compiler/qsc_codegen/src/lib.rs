// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

mod llvm;

use llvm::{function::Function, types::Builder, BasicBlock, Module, Name};
use qsc_frontend::compile::CompileUnit;
use qsc_hir::{
    hir::Package,
    mut_visit::{self, MutVisitor},
};

#[must_use]
pub fn emit_qir(unit: &mut CompileUnit /*, std: Option<&mut CompileUnit>*/) -> String {
    // todo: impl visitor for types builder instead of using this
    let types = Builder::new().build();
    let mut g = QirGenerator {
        module: Module {
            name: "ll_module".to_string(),
            source_file_name: String::new(),
            functions: vec![],
            func_declarations: vec![],
            global_vars: vec![],
            global_aliases: vec![],
            types,
        },
    };
    g.visit_package(&mut unit.package);
    format!("{}", g.module)
}

struct QirGenerator {
    module: Module,
}

impl MutVisitor for QirGenerator {
    fn visit_package(&mut self, package: &mut Package) {
        mut_visit::walk_package(self, package);
    }

    fn visit_callable_decl(&mut self, decl: &mut qsc_hir::hir::CallableDecl) {
        let mut func = Function::new(decl.name.name.to_string());
        func.return_type = self.module.types.void();
        let bb = BasicBlock::new(Name::Name("entry".into()));
        func.basic_blocks.push(bb);
        self.module.functions.push(func);

        mut_visit::walk_callable_decl(self, decl);
    }
}
