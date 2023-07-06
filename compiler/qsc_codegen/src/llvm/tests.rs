// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use super::{
    function::Parameter,
    instruction::{Add, Call, Mul},
    module::{Linkage, Module},
    operand::Operand,
    terminator::Ret,
    types::Builder,
    BasicBlock, Constant, ConstantRef, Function, Instruction, Terminator,
};

#[test]
#[allow(clippy::too_many_lines)]
fn test_module() {
    let mut module = Module {
        name: "test".to_string(),
        source_file_name: "test.qs".to_string(),
        functions: Vec::new(),
        func_declarations: Vec::new(),
        global_vars: Vec::new(),
        ty_builder: Builder::new(),
        function_attribute_groups: Vec::new(),
    };

    module.functions.push(Function {
        name: "foo".to_string(),
        parameters: vec![
            Parameter {
                name: Some("x".into()),
                ty: module.ty_builder.i64(),
            },
            Parameter {
                name: Some("y".into()),
                ty: module.ty_builder.i64(),
            },
        ],
        is_var_arg: false,
        return_type: module.ty_builder.i64(),
        basic_blocks: vec![BasicBlock {
            name: "entry".into(),
            instrs: vec![
                Instruction::Add(Add {
                    operand0: Operand::LocalOperand {
                        name: "x".into(),
                        ty: module.ty_builder.i64(),
                    },
                    operand1: Operand::LocalOperand {
                        name: "y".into(),
                        ty: module.ty_builder.i64(),
                    },
                    dest: 0.into(),
                    debugloc: None,
                }),
                Instruction::Mul(Mul {
                    operand0: Operand::LocalOperand {
                        name: 0.into(),
                        ty: module.ty_builder.i64(),
                    },
                    operand1: Operand::ConstantOperand(ConstantRef::new(Constant::Int {
                        bits: 64,
                        value: 2,
                    })),
                    dest: 1.into(),
                    debugloc: None,
                }),
            ],
            term: Terminator::Ret(Ret {
                return_operand: Some(Operand::LocalOperand {
                    name: 1.into(),
                    ty: module.ty_builder.i64(),
                }),
                debugloc: None,
            }),
        }],
        function_attributes: Vec::new(),
        linkage: Linkage::Internal,
        debugloc: None,
    });

    module.functions.push(Function {
        name: "main".to_string(),
        parameters: Vec::new(),
        is_var_arg: false,
        return_type: module.ty_builder.i64(),
        basic_blocks: vec![
            BasicBlock {
                name: "entry".into(),
                instrs: vec![Instruction::Call(Call {
                    function: Operand::ConstantOperand(ConstantRef::new(
                        Constant::GlobalReference {
                            name: "foo".into(),
                            ty: module.ty_builder.i64(),
                        },
                    )),
                    arguments: vec![
                        Operand::ConstantOperand(ConstantRef::new(Constant::Int {
                            bits: 64,
                            value: 1,
                        })),
                        Operand::ConstantOperand(ConstantRef::new(Constant::Int {
                            bits: 64,
                            value: 2,
                        })),
                    ],
                    dest: Some(0.into()),
                    function_attributes: Vec::new(),
                    is_tail_call: false,
                    debugloc: None,
                })],
                term: Terminator::Ret(Ret {
                    return_operand: Some(Operand::LocalOperand {
                        name: 0.into(),
                        ty: module.ty_builder.i64(),
                    }),
                    debugloc: None,
                }),
            },
            BasicBlock::new("exit".into()),
        ],
        function_attributes: Vec::new(),
        linkage: Linkage::External,
        debugloc: None,
    });

    (expect![[r#"
        ; ModuleID = '<test>'


        define internal i64 @foo(i64 %x, i64 %y) {
        entry:
          %0 = add i64 %x, %y
          %1 = mul i64 %0, 2
          ret i64 %1
        }

        define external i64 @main() {
        entry:
          %0 = call i64 @foo(i64 1, i64 2)
          ret i64 %0
        exit:
          unreachable
        }

    "#]])
    .assert_eq(&module.to_string());
}
