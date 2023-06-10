// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

mod llvm;

use std::{collections::HashMap, rc::Rc};

use crate::llvm::types::NamedStructDef;
use llvm::{
    constant::Float,
    function::{Declaration, Function, Parameter},
    module::{Linkage, Visibility},
    terminator::{Br, Ret},
    types::Builder,
    BasicBlock, Constant, ConstantRef, Module, Name, Operand, Terminator,
};
use qsc_frontend::compile::CompileUnit;
use qsc_hir::{
    hir::{
        CallableDecl, Expr, ExprKind, Item, ItemKind, Lit, LocalItemId, Package, Pat, PatKind,
        SpecBody, SpecGen, Stmt, StmtKind, Ty,
    },
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
        curr_func: None,
        operand: None,
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
    curr_func: Option<Function>,
    operand: Option<Operand>,
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

    fn curr_block(&mut self) -> &mut BasicBlock {
        self.curr_func
            .as_mut()
            .expect("current function should be set")
            .basic_blocks
            .last_mut()
            .expect("blocks should be non-empty")
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

            let body_name = "body".into();
            let exit_name = "exit".into();

            let mut entry = BasicBlock::new(Name::Name("entry".into()));
            entry.term = Terminator::Br(Br {
                dest: Name::Name(Rc::clone(&body_name)),
                debugloc: None,
            });
            func.basic_blocks.push(entry);

            let mut body = BasicBlock::new(Name::Name(body_name));
            body.term = Terminator::Br(Br {
                dest: Name::Name(Rc::clone(&exit_name)),
                debugloc: None,
            });
            func.basic_blocks.push(body);

            self.curr_func = Some(func);
            visit::walk_callable_decl(self, decl);

            let mut exit = BasicBlock::new(Name::Name(exit_name));
            if let Some(op) = self.operand.take() {
                exit.term = Terminator::Ret(Ret {
                    return_operand: Some(op),
                    debugloc: None,
                });
            } else if decl.output == Ty::UNIT {
                exit.term = Terminator::Ret(Ret {
                    return_operand: None,
                    debugloc: None,
                });
            }
            let mut func = self.curr_func.take().expect("function should be set above");
            func.basic_blocks.push(exit);
            self.module.functions.push(func);
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Expr(expr) => self.visit_expr(expr),
            StmtKind::Item(_) | StmtKind::Local(_, _, _) | StmtKind::Qubit(_, _, _, _) => {}
            StmtKind::Semi(expr) => {
                self.visit_expr(expr);
                self.operand.take();
            }
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Array(_) => todo!(),
            ExprKind::ArrayRepeat(_, _) => todo!(),
            ExprKind::Assign(_, _) => todo!(),
            ExprKind::AssignOp(_, _, _) => todo!(),
            ExprKind::AssignField(_, _, _) => todo!(),
            ExprKind::AssignIndex(_, _, _) => todo!(),
            ExprKind::BinOp(_, _, _) => todo!(),
            ExprKind::Block(_) => todo!(),
            ExprKind::Call(_, _) => todo!(),
            ExprKind::Closure(_, _) => todo!(),
            ExprKind::Conjugate(_, _) => todo!(),
            ExprKind::Fail(_) => todo!(),
            ExprKind::Field(_, _) => todo!(),
            ExprKind::For(_, _, _) => todo!(),
            ExprKind::Hole => todo!(),
            ExprKind::If(_, _, _) => todo!(),
            ExprKind::Index(_, _) => todo!(),
            ExprKind::Lit(lit) if matches!(lit, Lit::Result(..)) => todo!(),
            ExprKind::Lit(lit) => {
                let const_op = literal_to_const_op(lit);
                self.operand.replace(Operand::ConstantOperand(const_op));
            }
            ExprKind::Range(_, _, _) => todo!(),
            ExprKind::Repeat(_, _, _) => todo!(),
            ExprKind::Return(expr) => {
                self.visit_expr(expr);
                self.curr_block().term = Terminator::Ret(Ret {
                    return_operand: self.operand.take(),
                    debugloc: None,
                });
            }
            ExprKind::String(_) => todo!(),
            ExprKind::TernOp(_, _, _, _) => todo!(),
            ExprKind::Tuple(tup) if tup.is_empty() => {}
            ExprKind::Tuple(_) => todo!(),
            ExprKind::UnOp(_, _) => todo!(),
            ExprKind::UpdateField(_, _, _) => todo!(),
            ExprKind::Var(_) => todo!(),
            ExprKind::While(_, _) => todo!(),
            ExprKind::Err => todo!(),
        }
    }
}

fn literal_to_const_op(lit: &Lit) -> ConstantRef {
    ConstantRef(
        match lit {
            Lit::BigInt(_) => todo!(),
            Lit::Bool(b) => Constant::Int {
                bits: 1,
                value: u64::from(*b),
            },
            Lit::Double(d) => Constant::Float(Float::Double(*d)),
            #[allow(clippy::cast_sign_loss)]
            Lit::Int(i) => Constant::Int {
                bits: 64,
                value: *i as u64,
            },
            Lit::Pauli(_) => todo!(),
            Lit::Result(_) => todo!(),
        }
        .into(),
    )
}

fn is_intrinsic(decl: &CallableDecl) -> bool {
    decl.body.body == SpecBody::Gen(SpecGen::Intrinsic)
}
