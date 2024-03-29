// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines, clippy::needless_raw_string_hashes)]

use crate::{qir::ToQir, test_utils::rir_builder};
use expect_test::expect;
use qsc_rir::rir;

use super::defer_measurements;

#[test]
fn measurements_deferred_on_return_block() {
    let mut program = rir::Program::default();
    add_simple_measurement_block(&mut program);

    // Before
    expect![[r#"
        block_0:
          call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
          call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 1 to %Qubit*))
          call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
          call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 2 to %Result*))
          call void @__quantum__rt__array_record_output(i64 3, i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 2 to %Result*), i8* null)
          ret void"#]].assert_eq(&format!("block_0:\n{}", program.blocks.get(0_usize.into()).expect("block should be present").to_qir(&program)));

    // After
    defer_measurements(&mut program);
    expect![[r#"
        block_0:
          call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 1 to %Qubit*))
          call void @__quantum__qis__mz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
          call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
          call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 2 to %Result*))
          call void @__quantum__rt__array_record_output(i64 3, i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)
          call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 2 to %Result*), i8* null)
          ret void"#]].assert_eq(&format!("block_0:\n{}", program.blocks.get(0_usize.into()).expect("block should be present").to_qir(&program)));
}

#[test]
fn branching_block_measurements_deferred_properly() {
    let mut program = rir::Program::default();
    add_branching_measurement_block(&mut program);

    // Before
    expect![[r#"
        block_0:
          call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
          call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
          call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
          %var_0 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 0 to %Result*))
          br i1 %var_0, label %block_1, label %block_2"#]].assert_eq(&format!("block_0:\n{}", program.blocks.get(0_usize.into()).expect("block should be present").to_qir(&program)));

    // After
    defer_measurements(&mut program);
    expect![[r#"
        block_0:
          call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 0 to %Qubit*))
          call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
          call void @__quantum__qis__mresetz__body(%Qubit* inttoptr (i64 0 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))
          %var_0 = call i1 @__quantum__qis__read_result__body(%Result* inttoptr (i64 0 to %Result*))
          br i1 %var_0, label %block_1, label %block_2"#]].assert_eq(&format!("block_0:\n{}", program.blocks.get(0_usize.into()).expect("block should be present").to_qir(&program)));
}

fn add_simple_measurement_block(program: &mut rir::Program) {
    program
        .callables
        .insert(rir::CallableId(0), rir_builder::mz_decl());
    program
        .callables
        .insert(rir::CallableId(1), rir_builder::mresetz_decl());
    program
        .callables
        .insert(rir::CallableId(2), rir_builder::x_decl());
    program
        .callables
        .insert(rir::CallableId(3), rir_builder::array_record_decl());
    program
        .callables
        .insert(rir::CallableId(4), rir_builder::result_record_decl());
    program.blocks.insert(
        rir::BlockId(0),
        rir::Block(vec![
            rir::Instruction::Call(
                rir::CallableId(0),
                vec![
                    rir::Value::Literal(rir::Literal::Qubit(0)),
                    rir::Value::Literal(rir::Literal::Result(0)),
                ],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(2),
                vec![rir::Value::Literal(rir::Literal::Qubit(1))],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(1),
                vec![
                    rir::Value::Literal(rir::Literal::Qubit(1)),
                    rir::Value::Literal(rir::Literal::Result(1)),
                ],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(1),
                vec![
                    rir::Value::Literal(rir::Literal::Qubit(0)),
                    rir::Value::Literal(rir::Literal::Result(2)),
                ],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(3),
                vec![
                    rir::Value::Literal(rir::Literal::Integer(3)),
                    rir::Value::Literal(rir::Literal::Pointer),
                ],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(4),
                vec![
                    rir::Value::Literal(rir::Literal::Result(0)),
                    rir::Value::Literal(rir::Literal::Pointer),
                ],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(4),
                vec![
                    rir::Value::Literal(rir::Literal::Result(1)),
                    rir::Value::Literal(rir::Literal::Pointer),
                ],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(4),
                vec![
                    rir::Value::Literal(rir::Literal::Result(2)),
                    rir::Value::Literal(rir::Literal::Pointer),
                ],
                None,
            ),
            rir::Instruction::Return,
        ]),
    );
}

fn add_branching_measurement_block(program: &mut rir::Program) {
    program
        .callables
        .insert(rir::CallableId(0), rir_builder::mresetz_decl());
    program
        .callables
        .insert(rir::CallableId(1), rir_builder::x_decl());
    program
        .callables
        .insert(rir::CallableId(3), rir_builder::read_result_decl());
    program.blocks.insert(
        rir::BlockId(0),
        rir::Block(vec![
            rir::Instruction::Call(
                rir::CallableId(0),
                vec![
                    rir::Value::Literal(rir::Literal::Qubit(1)),
                    rir::Value::Literal(rir::Literal::Result(0)),
                ],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(1),
                vec![rir::Value::Literal(rir::Literal::Qubit(0))],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(0),
                vec![
                    rir::Value::Literal(rir::Literal::Qubit(0)),
                    rir::Value::Literal(rir::Literal::Result(1)),
                ],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(3),
                vec![rir::Value::Literal(rir::Literal::Result(0))],
                Some(rir::Variable {
                    variable_id: rir::VariableId(0),
                    ty: rir::Ty::Boolean,
                }),
            ),
            rir::Instruction::Branch(
                rir::Value::Variable(rir::Variable {
                    variable_id: rir::VariableId(0),
                    ty: rir::Ty::Boolean,
                }),
                rir::BlockId(1),
                rir::BlockId(2),
            ),
        ]),
    );
}
