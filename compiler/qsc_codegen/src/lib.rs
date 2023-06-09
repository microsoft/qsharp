// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

mod llvm;

use std::{collections::HashMap, rc::Rc};

use crate::llvm::types::NamedStructDef;
use llvm::{
    function::{Declaration, Function, Parameter},
    module::{Linkage, Visibility},
    types::Builder,
    BasicBlock, Module, Name,
};
use qsc_frontend::compile::CompileUnit;
use qsc_hir::{
    hir::{CallableDecl, Item, ItemKind, LocalItemId, Package, Pat, PatKind, SpecBody, SpecGen},
    visit::{self, Visitor},
};

#[must_use]
pub fn emit_qir(unit: &mut CompileUnit /*, std: Option<&mut CompileUnit>*/) -> String {
    let mut gen = QirGenerator {
        module: Module {
            name: "qir_module".to_string(),
            source_file_name: String::new(),
            functions: Vec::new(),
            func_declarations: Vec::new(),
            global_vars: Vec::new(),
            global_aliases: Vec::new(),
            ty_builder: Builder::new(),
        },
        namespace_map: HashMap::new(),
        curr_namespace: None,
    };
    gen.module
        .ty_builder
        .add_named_struct_def("Qubit".to_string(), NamedStructDef::Opaque);
    gen.module
        .ty_builder
        .add_named_struct_def("Result".to_string(), NamedStructDef::Opaque);
    gen.visit_package(&unit.package);
    format!("{}", gen.module)
}

struct QirGenerator {
    module: Module,
    namespace_map: HashMap<LocalItemId, Rc<str>>,
    curr_namespace: Option<Rc<str>>,
}

impl QirGenerator {
    fn function_name(&self, decl: &CallableDecl) -> String {
        format!(
            "{}.{}",
            self.curr_namespace
                .as_ref()
                .expect("should have current namespace for function name"),
            decl.name.name
        )
    }

    fn pat_to_params(&mut self, pat: &Pat, drop_names: bool) -> Vec<Parameter> {
        match &pat.kind {
            PatKind::Bind(ident) => vec![Parameter {
                name: if drop_names {
                    None
                } else {
                    Some(Name::Name(Rc::clone(&ident.name)))
                },
                ty: self.module.ty_builder.map_ty(&pat.ty),
                attributes: Vec::new(),
            }],
            PatKind::Discard => vec![Parameter {
                name: None,
                ty: self.module.ty_builder.map_ty(&pat.ty),
                attributes: Vec::new(),
            }],
            PatKind::Elided => panic!("elided should not be present in parameters"),
            PatKind::Tuple(tup) => {
                let mut params = Vec::new();
                for pat in tup {
                    params.append(&mut self.pat_to_params(pat, drop_names));
                }
                params
            }
        }
    }
}

impl Visitor<'_> for QirGenerator {
    fn visit_package(&mut self, package: &Package) {
        visit::walk_package(self, package);
    }

    fn visit_item(&mut self, item: &Item) {
        match &item.kind {
            ItemKind::Callable(_) => {
                self.curr_namespace = Some(Rc::clone(
                    self.namespace_map
                        .get(&item.id)
                        .expect("namespace should be present for callable"),
                ));
            }
            ItemKind::Namespace(ident, items) => {
                for item in items {
                    self.namespace_map.insert(*item, Rc::clone(&ident.name));
                }
            }
            ItemKind::Ty(_, _) => todo!(),
        }
        visit::walk_item(self, item);
        self.curr_namespace.take();
    }

    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        if is_intrinsic(decl) {
            let func_decl = Declaration {
                name: decl.name.name.to_string(),
                parameters: self.pat_to_params(&decl.input, true),
                is_var_arg: false,
                return_type: self.module.ty_builder.map_ty(&decl.output),
                return_attributes: Vec::new(),
                linkage: Linkage::External,
                visibility: Visibility::Default,
                debugloc: None,
            };
            self.module.func_declarations.push(func_decl);
        } else {
            let mut func = Function {
                name: self.function_name(decl),
                parameters: self.pat_to_params(&decl.input, false),
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

            visit::walk_callable_decl(self, decl);
        }
    }
}

fn is_intrinsic(decl: &CallableDecl) -> bool {
    decl.body.body == SpecBody::Gen(SpecGen::Intrinsic)
}
