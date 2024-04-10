// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

mod test_utils;

use indoc::indoc;
use qsc_rir::rir::{
    BlockId, Callable, CallableId, CallableType, ConditionCode, Instruction, Literal, Operand, Ty,
    Variable, VariableId,
};
use test_utils::{assert_block_instructions, assert_callable, compile_and_partially_evaluate};

fn m_intrinsic_op() -> Callable {
    Callable {
        name: "__quantum__qis__mz__body".to_string(),
        input_type: vec![Ty::Qubit, Ty::Result],
        output_type: None,
        body: None,
        call_type: CallableType::Measurement,
    }
}

fn mresetz_intrinsic_op() -> Callable {
    Callable {
        name: "__quantum__qis__mresetz__body".to_string(),
        input_type: vec![Ty::Qubit, Ty::Result],
        output_type: None,
        body: None,
        call_type: CallableType::Measurement,
    }
}

fn read_reasult_intrinsic_op() -> Callable {
    Callable {
        name: "__quantum__rt__read_result__body".to_string(),
        input_type: vec![Ty::Result],
        output_type: Some(Ty::Boolean),
        body: None,
        call_type: CallableType::Readout,
    }
}

#[test]
fn result_ids_are_correct_for_measuring_and_resetting_one_qubit() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                QIR.Intrinsic.__quantum__qis__mresetz__body(q);
            }
        }
        "#,
    });
    let op_callable_id = CallableId(1);
    assert_callable(&program, op_callable_id, &mresetz_intrinsic_op());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![
                    Operand::Literal(Literal::Qubit(0)),
                    Operand::Literal(Literal::Result(0)),
                ],
                None,
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn result_ids_are_correct_for_measuring_one_qubit() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                QIR.Intrinsic.__quantum__qis__m__body(q);
            }
        }
        "#,
    });
    let op_callable_id = CallableId(1);
    assert_callable(&program, op_callable_id, &m_intrinsic_op());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![
                    Operand::Literal(Literal::Qubit(0)),
                    Operand::Literal(Literal::Result(0)),
                ],
                None,
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn result_ids_are_correct_for_measuring_one_qubit_multiple_times() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                QIR.Intrinsic.__quantum__qis__m__body(q);
                QIR.Intrinsic.__quantum__qis__m__body(q);
                QIR.Intrinsic.__quantum__qis__m__body(q);
            }
        }
        "#,
    });
    let op_callable_id = CallableId(1);
    assert_callable(&program, op_callable_id, &m_intrinsic_op());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![
                    Operand::Literal(Literal::Qubit(0)),
                    Operand::Literal(Literal::Result(0)),
                ],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![
                    Operand::Literal(Literal::Qubit(0)),
                    Operand::Literal(Literal::Result(1)),
                ],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![
                    Operand::Literal(Literal::Qubit(0)),
                    Operand::Literal(Literal::Result(2)),
                ],
                None,
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn result_ids_are_correct_for_measuring_multiple_qubits() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1, q2) = (Qubit(), Qubit(), Qubit());
                QIR.Intrinsic.__quantum__qis__m__body(q0);
                QIR.Intrinsic.__quantum__qis__m__body(q1);
                QIR.Intrinsic.__quantum__qis__m__body(q2);
            }
        }
        "#,
    });
    let op_callable_id = CallableId(1);
    assert_callable(&program, op_callable_id, &m_intrinsic_op());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![
                    Operand::Literal(Literal::Qubit(0)),
                    Operand::Literal(Literal::Result(0)),
                ],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![
                    Operand::Literal(Literal::Qubit(1)),
                    Operand::Literal(Literal::Result(1)),
                ],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![
                    Operand::Literal(Literal::Qubit(2)),
                    Operand::Literal(Literal::Result(2)),
                ],
                None,
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn comparing_measurement_results_for_equality_adds_read_result_and_comparison_instructions() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1) = (Qubit(), Qubit());
                let r0 = QIR.Intrinsic.__quantum__qis__m__body(q0);
                let r1 = QIR.Intrinsic.__quantum__qis__m__body(q1);
                let b = r0 == r1;
            }
        }
        "#,
    });
    let measurement_callable_id = CallableId(1);
    assert_callable(&program, measurement_callable_id, &m_intrinsic_op());
    let readout_callable_id = CallableId(2);
    assert_callable(&program, readout_callable_id, &read_reasult_intrinsic_op());
    assert_block_instructions(
        &program,
        BlockId(0),
        &vec![
            Instruction::Call(
                measurement_callable_id,
                vec![
                    Operand::Literal(Literal::Qubit(0)),
                    Operand::Literal(Literal::Result(0)),
                ],
                None,
            ),
            Instruction::Call(
                measurement_callable_id,
                vec![
                    Operand::Literal(Literal::Qubit(1)),
                    Operand::Literal(Literal::Result(1)),
                ],
                None,
            ),
            Instruction::Call(
                readout_callable_id,
                vec![Operand::Literal(Literal::Result(0))],
                Some(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
            ),
            Instruction::Call(
                readout_callable_id,
                vec![Operand::Literal(Literal::Result(1))],
                Some(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
            ),
            Instruction::Icmp(
                ConditionCode::Eq,
                Operand::Variable(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn comparing_measurement_results_for_inequality_adds_read_result_and_comparison_instructions() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1) = (Qubit(), Qubit());
                let r0 = QIR.Intrinsic.__quantum__qis__m__body(q0);
                let r1 = QIR.Intrinsic.__quantum__qis__m__body(q1);
                let b = r0 != r1;
            }
        }
        "#,
    });
    let measurement_callable_id = CallableId(1);
    assert_callable(&program, measurement_callable_id, &m_intrinsic_op());
    let readout_callable_id = CallableId(2);
    assert_callable(&program, readout_callable_id, &read_reasult_intrinsic_op());
    assert_block_instructions(
        &program,
        BlockId(0),
        &vec![
            Instruction::Call(
                measurement_callable_id,
                vec![
                    Operand::Literal(Literal::Qubit(0)),
                    Operand::Literal(Literal::Result(0)),
                ],
                None,
            ),
            Instruction::Call(
                measurement_callable_id,
                vec![
                    Operand::Literal(Literal::Qubit(1)),
                    Operand::Literal(Literal::Result(1)),
                ],
                None,
            ),
            Instruction::Call(
                readout_callable_id,
                vec![Operand::Literal(Literal::Result(0))],
                Some(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
            ),
            Instruction::Call(
                readout_callable_id,
                vec![Operand::Literal(Literal::Result(1))],
                Some(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
            ),
            Instruction::Icmp(
                ConditionCode::Ne,
                Operand::Variable(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
                Operand::Variable(Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                }),
                Variable {
                    variable_id: VariableId(2),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn comparing_measurement_result_against_result_literal_for_equality_adds_read_result_and_comparison_instructions(
) {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__m__body(q);
                let b = r == One;
            }
        }
        "#,
    });
    let measurement_callable_id = CallableId(1);
    assert_callable(&program, measurement_callable_id, &m_intrinsic_op());
    let readout_callable_id = CallableId(2);
    assert_callable(&program, readout_callable_id, &read_reasult_intrinsic_op());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                measurement_callable_id,
                vec![
                    Operand::Literal(Literal::Qubit(0)),
                    Operand::Literal(Literal::Result(0)),
                ],
                None,
            ),
            Instruction::Call(
                readout_callable_id,
                vec![Operand::Literal(Literal::Result(0))],
                Some(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
            ),
            Instruction::Icmp(
                ConditionCode::Eq,
                Operand::Variable(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
                Operand::Literal(Literal::Bool(true)),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn comparing_measurement_result_against_result_literal_for_inequality_adds_read_result_and_comparison_instructions(
) {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__m__body(q);
                let b = r != Zero;
            }
        }
        "#,
    });
    let measurement_callable_id = CallableId(1);
    assert_callable(&program, measurement_callable_id, &m_intrinsic_op());
    let readout_callable_id = CallableId(2);
    assert_callable(&program, readout_callable_id, &read_reasult_intrinsic_op());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                measurement_callable_id,
                vec![
                    Operand::Literal(Literal::Qubit(0)),
                    Operand::Literal(Literal::Result(0)),
                ],
                None,
            ),
            Instruction::Call(
                readout_callable_id,
                vec![Operand::Literal(Literal::Result(0))],
                Some(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
            ),
            Instruction::Icmp(
                ConditionCode::Ne,
                Operand::Variable(Variable {
                    variable_id: VariableId(0),
                    ty: Ty::Boolean,
                }),
                Operand::Literal(Literal::Bool(false)),
                Variable {
                    variable_id: VariableId(1),
                    ty: Ty::Boolean,
                },
            ),
            Instruction::Return,
        ],
    );
}
