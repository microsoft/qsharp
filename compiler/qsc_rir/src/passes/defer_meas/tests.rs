// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines, clippy::needless_raw_string_hashes)]

use crate::{
    builder,
    rir::{
        Block, BlockId, CallableId, Instruction, Literal, Program, Ty, Value, Variable, VariableId,
    },
};
use expect_test::expect;

use super::defer_measurements;

#[test]
fn measurements_deferred_on_return_block() {
    let mut program = Program::default();
    add_simple_measurement_block(&mut program);

    // Before
    expect![[r#"
        Block:
            Call:
                callable_id: 0
                args:
                    [0]: Literal: Qubit(0)
                    [1]: Literal: Result(0)
                    variable: <NONE>
            Call:
                callable_id: 2
                args:
                    [0]: Literal: Qubit(1)
                    variable: <NONE>
            Call:
                callable_id: 1
                args:
                    [0]: Literal: Qubit(1)
                    [1]: Literal: Result(1)
                    variable: <NONE>
            Call:
                callable_id: 1
                args:
                    [0]: Literal: Qubit(0)
                    [1]: Literal: Result(2)
                    variable: <NONE>
            Call:
                callable_id: 3
                args:
                    [0]: Literal: Integer(3)
                    [1]: Literal: Pointer
                    variable: <NONE>
            Call:
                callable_id: 4
                args:
                    [0]: Literal: Result(0)
                    [1]: Literal: Pointer
                    variable: <NONE>
            Call:
                callable_id: 4
                args:
                    [0]: Literal: Result(1)
                    [1]: Literal: Pointer
                    variable: <NONE>
            Call:
                callable_id: 4
                args:
                    [0]: Literal: Result(2)
                    [1]: Literal: Pointer
                    variable: <NONE>
            Return"#]]
    .assert_eq(&program.get_block(BlockId(0)).to_string());

    // After
    defer_measurements(&mut program);
    expect![[r#"
        Block:
            Call:
                callable_id: 2
                args:
                    [0]: Literal: Qubit(1)
                    variable: <NONE>
            Call:
                callable_id: 0
                args:
                    [0]: Literal: Qubit(0)
                    [1]: Literal: Result(0)
                    variable: <NONE>
            Call:
                callable_id: 1
                args:
                    [0]: Literal: Qubit(1)
                    [1]: Literal: Result(1)
                    variable: <NONE>
            Call:
                callable_id: 1
                args:
                    [0]: Literal: Qubit(0)
                    [1]: Literal: Result(2)
                    variable: <NONE>
            Call:
                callable_id: 3
                args:
                    [0]: Literal: Integer(3)
                    [1]: Literal: Pointer
                    variable: <NONE>
            Call:
                callable_id: 4
                args:
                    [0]: Literal: Result(0)
                    [1]: Literal: Pointer
                    variable: <NONE>
            Call:
                callable_id: 4
                args:
                    [0]: Literal: Result(1)
                    [1]: Literal: Pointer
                    variable: <NONE>
            Call:
                callable_id: 4
                args:
                    [0]: Literal: Result(2)
                    [1]: Literal: Pointer
                    variable: <NONE>
            Return"#]]
    .assert_eq(&program.get_block(BlockId(0)).to_string());
}

#[test]
fn branching_block_measurements_deferred_properly() {
    let mut program = Program::default();
    add_branching_measurement_block(&mut program);

    // Before
    expect![[r#"
        Block:
            Call:
                callable_id: 0
                args:
                    [0]: Literal: Qubit(1)
                    [1]: Literal: Result(0)
                    variable: <NONE>
            Call:
                callable_id: 1
                args:
                    [0]: Literal: Qubit(0)
                    variable: <NONE>
            Call:
                callable_id: 0
                args:
                    [0]: Literal: Qubit(0)
                    [1]: Literal: Result(1)
                    variable: <NONE>
            Call:
                callable_id: 3
                args:
                    [0]: Literal: Result(0)
                    variable: Variable:
                        variable_id: 0
                        ty: Boolean
            Branch:
                condition: Variable: Variable:
                    variable_id: 0
                    ty: Boolean
                if_true: 1
                if_false: 2"#]]
    .assert_eq(&program.get_block(BlockId(0)).to_string());

    // After
    defer_measurements(&mut program);
    expect![[r#"
        Block:
            Call:
                callable_id: 1
                args:
                    [0]: Literal: Qubit(0)
                    variable: <NONE>
            Call:
                callable_id: 0
                args:
                    [0]: Literal: Qubit(1)
                    [1]: Literal: Result(0)
                    variable: <NONE>
            Call:
                callable_id: 0
                args:
                    [0]: Literal: Qubit(0)
                    [1]: Literal: Result(1)
                    variable: <NONE>
            Call:
                callable_id: 3
                args:
                    [0]: Literal: Result(0)
                    variable: Variable:
                        variable_id: 0
                        ty: Boolean
            Branch:
                condition: Variable: Variable:
                    variable_id: 0
                    ty: Boolean
                if_true: 1
                if_false: 2"#]]
    .assert_eq(&program.get_block(BlockId(0)).to_string());
}

fn add_simple_measurement_block(program: &mut Program) {
    program.callables.insert(CallableId(0), builder::mz_decl());
    program
        .callables
        .insert(CallableId(1), builder::mresetz_decl());
    program.callables.insert(CallableId(2), builder::x_decl());
    program
        .callables
        .insert(CallableId(3), builder::array_record_decl());
    program
        .callables
        .insert(CallableId(4), builder::result_record_decl());
    program.blocks.insert(
        BlockId(0),
        Block(vec![
            Instruction::Call(
                CallableId(0),
                vec![
                    Value::Literal(Literal::Qubit(0)),
                    Value::Literal(Literal::Result(0)),
                ],
                None,
            ),
            Instruction::Call(CallableId(2), vec![Value::Literal(Literal::Qubit(1))], None),
            Instruction::Call(
                CallableId(1),
                vec![
                    Value::Literal(Literal::Qubit(1)),
                    Value::Literal(Literal::Result(1)),
                ],
                None,
            ),
            Instruction::Call(
                CallableId(1),
                vec![
                    Value::Literal(Literal::Qubit(0)),
                    Value::Literal(Literal::Result(2)),
                ],
                None,
            ),
            Instruction::Call(
                CallableId(3),
                vec![
                    Value::Literal(Literal::Integer(3)),
                    Value::Literal(Literal::Pointer),
                ],
                None,
            ),
            Instruction::Call(
                CallableId(4),
                vec![
                    Value::Literal(Literal::Result(0)),
                    Value::Literal(Literal::Pointer),
                ],
                None,
            ),
            Instruction::Call(
                CallableId(4),
                vec![
                    Value::Literal(Literal::Result(1)),
                    Value::Literal(Literal::Pointer),
                ],
                None,
            ),
            Instruction::Call(
                CallableId(4),
                vec![
                    Value::Literal(Literal::Result(2)),
                    Value::Literal(Literal::Pointer),
                ],
                None,
            ),
            Instruction::Return,
        ]),
    );
}

fn add_branching_measurement_block(program: &mut Program) {
    program
        .callables
        .insert(CallableId(0), builder::mresetz_decl());
    program.callables.insert(CallableId(1), builder::x_decl());
    program
        .callables
        .insert(CallableId(3), builder::read_result_decl());
    program.blocks.insert(
        BlockId(0),
        Block(vec![
            Instruction::Call(
                CallableId(0),
                vec![
                    Value::Literal(Literal::Qubit(1)),
                    Value::Literal(Literal::Result(0)),
                ],
                None,
            ),
            Instruction::Call(CallableId(1), vec![Value::Literal(Literal::Qubit(0))], None),
            Instruction::Call(
                CallableId(0),
                vec![
                    Value::Literal(Literal::Qubit(0)),
                    Value::Literal(Literal::Result(1)),
                ],
                None,
            ),
            Instruction::Call(
                CallableId(3),
                vec![Value::Literal(Literal::Result(0))],
                Some(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
            ),
            Instruction::Branch(
                Value::Variable(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
                BlockId(1),
                BlockId(2),
            ),
        ]),
    );
}
