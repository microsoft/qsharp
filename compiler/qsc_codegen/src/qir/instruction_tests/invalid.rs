// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::qir::ToQir;
use qsc_rir::rir;

#[test]
#[should_panic(expected = "mismatched input types (i64, f64) for add")]
fn add_mismatched_literal_input_tys_should_panic() {
    let inst = rir::Instruction::Add(
        rir::Value::Literal(rir::Literal::Integer(2)),
        rir::Value::Literal(rir::Literal::Double(1.0)),
        rir::Variable {
            variable_id: rir::VariableId(0),
            ty: rir::Ty::Integer,
        },
    );
    let _ = &inst.to_qir(&rir::Program::default());
}

#[test]
#[should_panic(expected = "mismatched input/output types (i64, f64) for add")]
fn add_mismatched_literal_input_output_tys_should_panic() {
    let inst = rir::Instruction::Add(
        rir::Value::Literal(rir::Literal::Integer(2)),
        rir::Value::Literal(rir::Literal::Integer(5)),
        rir::Variable {
            variable_id: rir::VariableId(0),
            ty: rir::Ty::Double,
        },
    );
    let _ = &inst.to_qir(&rir::Program::default());
}

#[test]
#[should_panic(expected = "mismatched input types (i64, f64) for add")]
fn add_mismatched_variable_input_tys_should_panic() {
    let inst = rir::Instruction::Add(
        rir::Value::Variable(rir::Variable {
            variable_id: rir::VariableId(1),
            ty: rir::Ty::Integer,
        }),
        rir::Value::Variable(rir::Variable {
            variable_id: rir::VariableId(2),
            ty: rir::Ty::Double,
        }),
        rir::Variable {
            variable_id: rir::VariableId(0),
            ty: rir::Ty::Integer,
        },
    );
    let _ = &inst.to_qir(&rir::Program::default());
}

#[test]
#[should_panic(expected = "mismatched input/output types (i64, f64) for add")]
fn add_mismatched_variable_input_output_tys_should_panic() {
    let inst = rir::Instruction::Add(
        rir::Value::Variable(rir::Variable {
            variable_id: rir::VariableId(1),
            ty: rir::Ty::Integer,
        }),
        rir::Value::Variable(rir::Variable {
            variable_id: rir::VariableId(2),
            ty: rir::Ty::Integer,
        }),
        rir::Variable {
            variable_id: rir::VariableId(0),
            ty: rir::Ty::Double,
        },
    );
    let _ = &inst.to_qir(&rir::Program::default());
}

#[test]
#[should_panic(expected = "mismatched input types (i64, f64) for and")]
fn bitwise_and_mismatched_literal_input_tys_should_panic() {
    let inst = rir::Instruction::BitwiseAnd(
        rir::Value::Literal(rir::Literal::Integer(2)),
        rir::Value::Literal(rir::Literal::Double(1.0)),
        rir::Variable {
            variable_id: rir::VariableId(0),
            ty: rir::Ty::Integer,
        },
    );
    let _ = &inst.to_qir(&rir::Program::default());
}

#[test]
#[should_panic(expected = "mismatched input/output types (i64, f64) for and")]
fn bitwise_and_mismatched_literal_input_output_tys_should_panic() {
    let inst = rir::Instruction::BitwiseAnd(
        rir::Value::Literal(rir::Literal::Integer(2)),
        rir::Value::Literal(rir::Literal::Integer(5)),
        rir::Variable {
            variable_id: rir::VariableId(0),
            ty: rir::Ty::Double,
        },
    );
    let _ = &inst.to_qir(&rir::Program::default());
}

#[test]
#[should_panic(expected = "mismatched input types (i64, f64) for and")]
fn bitwise_and_mismatched_variable_input_tys_should_panic() {
    let inst = rir::Instruction::BitwiseAnd(
        rir::Value::Variable(rir::Variable {
            variable_id: rir::VariableId(1),
            ty: rir::Ty::Integer,
        }),
        rir::Value::Variable(rir::Variable {
            variable_id: rir::VariableId(2),
            ty: rir::Ty::Double,
        }),
        rir::Variable {
            variable_id: rir::VariableId(0),
            ty: rir::Ty::Integer,
        },
    );
    let _ = &inst.to_qir(&rir::Program::default());
}

#[test]
#[should_panic(expected = "mismatched input/output types (i64, f64) for and")]
fn bitwise_and_mismatched_variable_input_output_tys_should_panic() {
    let inst = rir::Instruction::BitwiseAnd(
        rir::Value::Variable(rir::Variable {
            variable_id: rir::VariableId(1),
            ty: rir::Ty::Integer,
        }),
        rir::Value::Variable(rir::Variable {
            variable_id: rir::VariableId(2),
            ty: rir::Ty::Integer,
        }),
        rir::Variable {
            variable_id: rir::VariableId(0),
            ty: rir::Ty::Double,
        },
    );
    let _ = &inst.to_qir(&rir::Program::default());
}

#[test]
#[should_panic(expected = "unsupported type i1 for add")]
fn add_bool_should_panic() {
    let inst = rir::Instruction::Add(
        rir::Value::Literal(rir::Literal::Bool(true)),
        rir::Value::Literal(rir::Literal::Bool(false)),
        rir::Variable {
            variable_id: rir::VariableId(0),
            ty: rir::Ty::Boolean,
        },
    );
    let _ = &inst.to_qir(&rir::Program::default());
}

#[test]
#[should_panic(expected = "mismatched types (i64 [... i1]) for phi")]
fn phi_with_mismatched_args() {
    let args = [
        (
            rir::Value::Variable(rir::Variable {
                variable_id: rir::VariableId(13),
                ty: rir::Ty::Integer,
            }),
            rir::BlockId(3),
        ),
        (
            rir::Value::Variable(rir::Variable {
                variable_id: rir::VariableId(2),
                ty: rir::Ty::Boolean,
            }),
            rir::BlockId(7),
        ),
    ];
    let inst = rir::Instruction::Phi(
        args.to_vec(),
        rir::Variable {
            variable_id: rir::VariableId(0),
            ty: rir::Ty::Integer,
        },
    );
    let _ = &inst.to_qir(&rir::Program::default());
}
