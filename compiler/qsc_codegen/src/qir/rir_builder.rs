// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_rir::rir;

pub fn x_decl() -> rir::Callable {
    rir::Callable {
        name: "__quantum__qis__x__body".to_string(),
        input_type: vec![rir::Ty::Qubit],
        output_type: None,
        body: None,
    }
}

pub fn z_decl() -> rir::Callable {
    rir::Callable {
        name: "__quantum__qis__z__body".to_string(),
        input_type: vec![rir::Ty::Qubit],
        output_type: None,
        body: None,
    }
}

pub fn h_decl() -> rir::Callable {
    rir::Callable {
        name: "__quantum__qis__h__body".to_string(),
        input_type: vec![rir::Ty::Qubit],
        output_type: None,
        body: None,
    }
}

pub fn cx_decl() -> rir::Callable {
    rir::Callable {
        name: "__quantum__qis__cx__body".to_string(),
        input_type: vec![rir::Ty::Qubit, rir::Ty::Qubit],
        output_type: None,
        body: None,
    }
}

pub fn rx_decl() -> rir::Callable {
    rir::Callable {
        name: "__quantum__qis__rx__body".to_string(),
        input_type: vec![rir::Ty::Double, rir::Ty::Qubit],
        output_type: None,
        body: None,
    }
}

pub fn mz_decl() -> rir::Callable {
    rir::Callable {
        name: "__quantum__qis__mz__body".to_string(),
        input_type: vec![rir::Ty::Qubit, rir::Ty::Result],
        output_type: None,
        body: None,
    }
}

pub fn mresetz_decl() -> rir::Callable {
    rir::Callable {
        name: "__quantum__qis__mresetz__body".to_string(),
        input_type: vec![rir::Ty::Qubit, rir::Ty::Result],
        output_type: None,
        body: None,
    }
}

pub fn read_result_decl() -> rir::Callable {
    rir::Callable {
        name: "__quantum__qis__read_result__body".to_string(),
        input_type: vec![rir::Ty::Result],
        output_type: Some(rir::Ty::Boolean),
        body: None,
    }
}

pub fn result_record_decl() -> rir::Callable {
    rir::Callable {
        name: "__quantum__rt__result_record_output".to_string(),
        input_type: vec![rir::Ty::Result, rir::Ty::Pointer],
        output_type: None,
        body: None,
    }
}

pub fn array_record_decl() -> rir::Callable {
    rir::Callable {
        name: "__quantum__rt__array_record_output".to_string(),
        input_type: vec![rir::Ty::Integer, rir::Ty::Pointer],
        output_type: None,
        body: None,
    }
}

pub fn bell_program() -> rir::Program {
    let mut program = rir::Program::default();
    program.callables.insert(rir::CallableId(0), h_decl());
    program.callables.insert(rir::CallableId(1), cx_decl());
    program.callables.insert(rir::CallableId(2), mz_decl());
    program
        .callables
        .insert(rir::CallableId(3), array_record_decl());
    program
        .callables
        .insert(rir::CallableId(4), result_record_decl());
    program.callables.insert(
        rir::CallableId(5),
        rir::Callable {
            name: "main".to_string(),
            input_type: vec![],
            output_type: None,
            body: Some(rir::BlockId(0)),
        },
    );
    program.blocks.insert(
        rir::BlockId(0),
        rir::Block(vec![
            rir::Instruction::Call(
                rir::CallableId(0),
                vec![rir::Value::Literal(rir::Literal::Qubit(0))],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(1),
                vec![
                    rir::Value::Literal(rir::Literal::Qubit(0)),
                    rir::Value::Literal(rir::Literal::Qubit(1)),
                ],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(2),
                vec![
                    rir::Value::Literal(rir::Literal::Qubit(0)),
                    rir::Value::Literal(rir::Literal::Result(0)),
                ],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(2),
                vec![
                    rir::Value::Literal(rir::Literal::Qubit(1)),
                    rir::Value::Literal(rir::Literal::Result(1)),
                ],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(3),
                vec![
                    rir::Value::Literal(rir::Literal::Integer(2)),
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
            rir::Instruction::Return,
        ]),
    );
    program.num_qubits = 2;
    program.num_results = 2;
    program.config.defer_measurements = true;
    program.config.remap_qubits_on_reuse = true;
    program
}

#[allow(clippy::too_many_lines)]
pub fn teleport_program() -> rir::Program {
    let mut program = rir::Program::default();
    program.callables.insert(rir::CallableId(0), h_decl());
    program.callables.insert(rir::CallableId(1), z_decl());
    program.callables.insert(rir::CallableId(2), x_decl());
    program.callables.insert(rir::CallableId(3), cx_decl());
    program.callables.insert(rir::CallableId(4), mresetz_decl());
    program
        .callables
        .insert(rir::CallableId(5), read_result_decl());
    program
        .callables
        .insert(rir::CallableId(6), result_record_decl());
    program.callables.insert(
        rir::CallableId(7),
        rir::Callable {
            name: "main".to_string(),
            input_type: vec![],
            output_type: None,
            body: Some(rir::BlockId(0)),
        },
    );
    program.blocks.insert(
        rir::BlockId(0),
        rir::Block(vec![
            rir::Instruction::Call(
                rir::CallableId(2),
                vec![rir::Value::Literal(rir::Literal::Qubit(0))],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(0),
                vec![rir::Value::Literal(rir::Literal::Qubit(2))],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(3),
                vec![
                    rir::Value::Literal(rir::Literal::Qubit(2)),
                    rir::Value::Literal(rir::Literal::Qubit(1)),
                ],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(3),
                vec![
                    rir::Value::Literal(rir::Literal::Qubit(0)),
                    rir::Value::Literal(rir::Literal::Qubit(2)),
                ],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(0),
                vec![rir::Value::Literal(rir::Literal::Qubit(0))],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(4),
                vec![
                    rir::Value::Literal(rir::Literal::Qubit(0)),
                    rir::Value::Literal(rir::Literal::Result(0)),
                ],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(5),
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
    program.blocks.insert(
        rir::BlockId(1),
        rir::Block(vec![
            rir::Instruction::Call(
                rir::CallableId(1),
                vec![rir::Value::Literal(rir::Literal::Qubit(1))],
                None,
            ),
            rir::Instruction::Jump(rir::BlockId(2)),
        ]),
    );
    program.blocks.insert(
        rir::BlockId(2),
        rir::Block(vec![
            rir::Instruction::Call(
                rir::CallableId(4),
                vec![
                    rir::Value::Literal(rir::Literal::Qubit(2)),
                    rir::Value::Literal(rir::Literal::Result(1)),
                ],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(5),
                vec![rir::Value::Literal(rir::Literal::Result(1))],
                Some(rir::Variable {
                    variable_id: rir::VariableId(1),
                    ty: rir::Ty::Boolean,
                }),
            ),
            rir::Instruction::Branch(
                rir::Value::Variable(rir::Variable {
                    variable_id: rir::VariableId(1),
                    ty: rir::Ty::Boolean,
                }),
                rir::BlockId(3),
                rir::BlockId(4),
            ),
        ]),
    );
    program.blocks.insert(
        rir::BlockId(3),
        rir::Block(vec![
            rir::Instruction::Call(
                rir::CallableId(2),
                vec![rir::Value::Literal(rir::Literal::Qubit(1))],
                None,
            ),
            rir::Instruction::Jump(rir::BlockId(4)),
        ]),
    );
    program.blocks.insert(
        rir::BlockId(4),
        rir::Block(vec![
            rir::Instruction::Call(
                rir::CallableId(4),
                vec![
                    rir::Value::Literal(rir::Literal::Qubit(1)),
                    rir::Value::Literal(rir::Literal::Result(2)),
                ],
                None,
            ),
            rir::Instruction::Call(
                rir::CallableId(6),
                vec![
                    rir::Value::Literal(rir::Literal::Result(2)),
                    rir::Value::Literal(rir::Literal::Pointer),
                ],
                None,
            ),
            rir::Instruction::Return,
        ]),
    );
    program.num_qubits = 3;
    program.num_results = 3;
    program
}
