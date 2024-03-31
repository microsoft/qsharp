// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines, clippy::needless_raw_string_hashes)]

use expect_test::expect;

use crate::{
    builder::{cx_decl, h_decl, mresetz_decl, mz_decl, reset_decl, x_decl},
    rir::{Block, BlockId, CallableId, CallableType, Instruction, Literal, Program, Value},
};

use super::reindex_qubits;

#[test]
fn qubit_reindexed_after_reset_removes_reset() {
    let mut program = Program::new();
    program.num_qubits = 1;
    program.callables.insert(CallableId(0), x_decl());
    program.callables.insert(CallableId(1), reset_decl());
    program.blocks.insert(
        BlockId(0),
        Block(vec![
            Instruction::Call(CallableId(0), vec![Value::Literal(Literal::Qubit(0))], None),
            Instruction::Call(CallableId(1), vec![Value::Literal(Literal::Qubit(0))], None),
            Instruction::Call(CallableId(0), vec![Value::Literal(Literal::Qubit(0))], None),
            Instruction::Call(CallableId(1), vec![Value::Literal(Literal::Qubit(0))], None),
            Instruction::Return,
        ]),
    );

    // Before
    expect![[r#"
        Block:
            Call id(0), args( Qubit(0), )
            Call id(1), args( Qubit(0), )
            Call id(0), args( Qubit(0), )
            Call id(1), args( Qubit(0), )
            Return"#]]
    .assert_eq(&program.get_block(BlockId(0)).to_string());

    // After
    reindex_qubits(&mut program);
    expect![[r#"
        Block:
            Call id(0), args( Qubit(0), )
            Call id(0), args( Qubit(1), )
            Return"#]]
    .assert_eq(&program.get_block(BlockId(0)).to_string());
    assert_eq!(program.num_qubits, 2);

    // Reset callable should be removed.
    for callable in program.callables.values() {
        assert_ne!(callable.call_type, CallableType::Reset);
    }
}

#[test]
fn qubit_reindexed_after_mz() {
    let mut program = Program::new();
    program.num_qubits = 1;
    program.callables.insert(CallableId(0), x_decl());
    program.callables.insert(CallableId(1), mz_decl());
    program.blocks.insert(
        BlockId(0),
        Block(vec![
            Instruction::Call(CallableId(0), vec![Value::Literal(Literal::Qubit(0))], None),
            Instruction::Call(
                CallableId(1),
                vec![
                    Value::Literal(Literal::Qubit(0)),
                    Value::Literal(Literal::Result(0)),
                ],
                None,
            ),
            Instruction::Call(CallableId(0), vec![Value::Literal(Literal::Qubit(0))], None),
            Instruction::Call(
                CallableId(1),
                vec![
                    Value::Literal(Literal::Qubit(0)),
                    Value::Literal(Literal::Result(1)),
                ],
                None,
            ),
            Instruction::Return,
        ]),
    );

    // Before
    expect![[r#"
        Block:
            Call id(0), args( Qubit(0), )
            Call id(1), args( Qubit(0), Result(0), )
            Call id(0), args( Qubit(0), )
            Call id(1), args( Qubit(0), Result(1), )
            Return"#]]
    .assert_eq(&program.get_block(BlockId(0)).to_string());

    // After
    reindex_qubits(&mut program);
    expect![[r#"
        Block:
            Call id(0), args( Qubit(0), )
            Call id(1), args( Qubit(0), Result(0), )
            Call id(0), args( Qubit(1), )
            Call id(1), args( Qubit(1), Result(1), )
            Return"#]]
    .assert_eq(&program.get_block(BlockId(0)).to_string());
    assert_eq!(program.num_qubits, 2);
}

#[test]
fn qubit_reindexed_after_mresetz_and_changed_to_mz() {
    let mut program = Program::new();
    program.num_qubits = 1;
    program.callables.insert(CallableId(0), x_decl());
    program.callables.insert(CallableId(1), mresetz_decl());
    program.blocks.insert(
        BlockId(0),
        Block(vec![
            Instruction::Call(CallableId(0), vec![Value::Literal(Literal::Qubit(0))], None),
            Instruction::Call(
                CallableId(1),
                vec![
                    Value::Literal(Literal::Qubit(0)),
                    Value::Literal(Literal::Result(0)),
                ],
                None,
            ),
            Instruction::Call(CallableId(0), vec![Value::Literal(Literal::Qubit(0))], None),
            Instruction::Call(
                CallableId(1),
                vec![
                    Value::Literal(Literal::Qubit(0)),
                    Value::Literal(Literal::Result(1)),
                ],
                None,
            ),
            Instruction::Return,
        ]),
    );

    // Before
    expect![[r#"
        Block:
            Call id(0), args( Qubit(0), )
            Call id(1), args( Qubit(0), Result(0), )
            Call id(0), args( Qubit(0), )
            Call id(1), args( Qubit(0), Result(1), )
            Return"#]]
    .assert_eq(&program.get_block(BlockId(0)).to_string());

    // After
    reindex_qubits(&mut program);
    expect![[r#"
        Block:
            Call id(0), args( Qubit(0), )
            Call id(2), args( Qubit(0), Result(0), )
            Call id(0), args( Qubit(1), )
            Call id(2), args( Qubit(1), Result(1), )
            Return"#]]
    .assert_eq(&program.get_block(BlockId(0)).to_string());
    assert_eq!(program.num_qubits, 2);
}

#[test]
fn multiple_qubit_reindex() {
    let mut program = Program::new();
    program.num_qubits = 2;
    program.callables.insert(CallableId(0), h_decl());
    program.callables.insert(CallableId(1), mresetz_decl());
    program.callables.insert(CallableId(2), cx_decl());
    program.blocks.insert(
        BlockId(0),
        Block(vec![
            Instruction::Call(CallableId(0), vec![Value::Literal(Literal::Qubit(0))], None),
            Instruction::Call(
                CallableId(1),
                vec![
                    Value::Literal(Literal::Qubit(0)),
                    Value::Literal(Literal::Result(0)),
                ],
                None,
            ),
            Instruction::Call(
                CallableId(2),
                vec![
                    Value::Literal(Literal::Qubit(1)),
                    Value::Literal(Literal::Qubit(0)),
                ],
                None,
            ),
            Instruction::Return,
        ]),
    );

    // Before
    expect![[r#"
        Block:
            Call id(0), args( Qubit(0), )
            Call id(1), args( Qubit(0), Result(0), )
            Call id(2), args( Qubit(1), Qubit(0), )
            Return"#]]
    .assert_eq(&program.get_block(BlockId(0)).to_string());

    // After
    reindex_qubits(&mut program);
    expect![[r#"
        Block:
            Call id(0), args( Qubit(0), )
            Call id(3), args( Qubit(0), Result(0), )
            Call id(2), args( Qubit(1), Qubit(2), )
            Return"#]]
    .assert_eq(&program.get_block(BlockId(0)).to_string());
    assert_eq!(program.num_qubits, 3);
}

#[test]
fn qubit_reindexed_multiple_times() {
    let mut program = Program::new();
    program.num_qubits = 1;
    program.callables.insert(CallableId(0), x_decl());
    program.callables.insert(CallableId(1), mz_decl());
    program.blocks.insert(
        BlockId(0),
        Block(vec![
            Instruction::Call(CallableId(0), vec![Value::Literal(Literal::Qubit(0))], None),
            Instruction::Call(
                CallableId(1),
                vec![
                    Value::Literal(Literal::Qubit(0)),
                    Value::Literal(Literal::Result(0)),
                ],
                None,
            ),
            Instruction::Call(CallableId(0), vec![Value::Literal(Literal::Qubit(0))], None),
            Instruction::Call(
                CallableId(1),
                vec![
                    Value::Literal(Literal::Qubit(0)),
                    Value::Literal(Literal::Result(1)),
                ],
                None,
            ),
            Instruction::Call(CallableId(0), vec![Value::Literal(Literal::Qubit(0))], None),
            Instruction::Call(
                CallableId(1),
                vec![
                    Value::Literal(Literal::Qubit(0)),
                    Value::Literal(Literal::Result(2)),
                ],
                None,
            ),
            Instruction::Call(CallableId(0), vec![Value::Literal(Literal::Qubit(0))], None),
            Instruction::Call(
                CallableId(1),
                vec![
                    Value::Literal(Literal::Qubit(0)),
                    Value::Literal(Literal::Result(3)),
                ],
                None,
            ),
            Instruction::Return,
        ]),
    );

    // Before
    expect![[r#"
        Block:
            Call id(0), args( Qubit(0), )
            Call id(1), args( Qubit(0), Result(0), )
            Call id(0), args( Qubit(0), )
            Call id(1), args( Qubit(0), Result(1), )
            Call id(0), args( Qubit(0), )
            Call id(1), args( Qubit(0), Result(2), )
            Call id(0), args( Qubit(0), )
            Call id(1), args( Qubit(0), Result(3), )
            Return"#]]
    .assert_eq(&program.get_block(BlockId(0)).to_string());

    // After
    reindex_qubits(&mut program);
    expect![[r#"
        Block:
            Call id(0), args( Qubit(0), )
            Call id(1), args( Qubit(0), Result(0), )
            Call id(0), args( Qubit(1), )
            Call id(1), args( Qubit(1), Result(1), )
            Call id(0), args( Qubit(2), )
            Call id(1), args( Qubit(2), Result(2), )
            Call id(0), args( Qubit(3), )
            Call id(1), args( Qubit(3), Result(3), )
            Return"#]]
    .assert_eq(&program.get_block(BlockId(0)).to_string());
    assert_eq!(program.num_qubits, 4);
}
