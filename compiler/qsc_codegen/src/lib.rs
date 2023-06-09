// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

mod llvm;

use llvm::{
    function::Function,
    module::{Linkage, Visibility},
    types::Builder,
    BasicBlock, Module, Name,
};
use qsc_frontend::compile::CompileUnit;
use qsc_hir::{
    hir::{CallableDecl, Package},
    mut_visit::{self, MutVisitor},
};

#[must_use]
pub fn emit_qir(unit: &mut CompileUnit /*, std: Option<&mut CompileUnit>*/) -> String {
    let mut gen = QirGenerator {
        module: Module {
            name: "ll_module".to_string(),
            source_file_name: String::new(),
            functions: vec![],
            func_declarations: vec![],
            global_vars: vec![],
            global_aliases: vec![],
            ty_builder: Builder::new(),
        },
    };
    gen.visit_package(&mut unit.package);
    format!("{}", gen.module)
}

struct QirGenerator {
    module: Module,
}

impl MutVisitor for QirGenerator {
    fn visit_package(&mut self, package: &mut Package) {
        mut_visit::walk_package(self, package);
    }

    fn visit_callable_decl(&mut self, decl: &mut CallableDecl) {
        let mut func = Function {
            name: decl.name.name.to_string(),
            parameters: Vec::new(),
            is_var_arg: false,
            return_type: self.module.ty_builder.map_ty(&decl.output),
            basic_blocks: Vec::new(),
            function_attributes: Vec::new(),
            return_attributes: Vec::new(),
            linkage: Linkage::Internal,
            visibility: Visibility::Default,
            debugloc: None,
        };
        let bb = BasicBlock::new(Name::Name("entry".into()));
        func.basic_blocks.push(bb);
        self.module.functions.push(func);

        mut_visit::walk_callable_decl(self, decl);
    }
}
