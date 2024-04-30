// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::qir::ToQir;
use qsc_rir::rir;

#[test]
#[should_panic(expected = "unsupported type f64 for add")]
fn add_double_literals() {
    let inst = rir::Instruction::Add(
        rir::Operand::Literal(rir::Literal::Double(core::f64::consts::PI)),
        rir::Operand::Literal(rir::Literal::Double(core::f64::consts::E)),
        rir::Variable {
            id: rir::VariableId(0),
            ty: rir::Ty::Double,
        },
    );
    let _ = &inst.to_qir(&rir::Program::default());
}

#[test]
#[should_panic(expected = "unsupported type f64 for ashr")]
fn ashr_double_literals() {
    let inst = rir::Instruction::Ashr(
        rir::Operand::Literal(rir::Literal::Double(core::f64::consts::PI)),
        rir::Operand::Literal(rir::Literal::Double(core::f64::consts::E)),
        rir::Variable {
            id: rir::VariableId(0),
            ty: rir::Ty::Double,
        },
    );
    let _ = &inst.to_qir(&rir::Program::default());
}

#[test]
#[should_panic(expected = "unsupported type f64 for and")]
fn bitwise_and_double_literals() {
    let inst = rir::Instruction::BitwiseAnd(
        rir::Operand::Literal(rir::Literal::Double(core::f64::consts::PI)),
        rir::Operand::Literal(rir::Literal::Double(core::f64::consts::E)),
        rir::Variable {
            id: rir::VariableId(0),
            ty: rir::Ty::Double,
        },
    );
    let _ = &inst.to_qir(&rir::Program::default());
}

#[test]
#[should_panic(expected = "unsupported type f64 for not")]
fn bitwise_not_double_literals() {
    let inst = rir::Instruction::BitwiseNot(
        rir::Operand::Literal(rir::Literal::Double(core::f64::consts::PI)),
        rir::Variable {
            id: rir::VariableId(0),
            ty: rir::Ty::Double,
        },
    );
    let _ = &inst.to_qir(&rir::Program::default());
}

#[test]
#[should_panic(expected = "unsupported type f64 for or")]
fn bitwise_or_double_literals() {
    let inst = rir::Instruction::BitwiseOr(
        rir::Operand::Literal(rir::Literal::Double(core::f64::consts::PI)),
        rir::Operand::Literal(rir::Literal::Double(core::f64::consts::E)),
        rir::Variable {
            id: rir::VariableId(0),
            ty: rir::Ty::Double,
        },
    );
    let _ = &inst.to_qir(&rir::Program::default());
}

#[test]
#[should_panic(expected = "unsupported type f64 for xor")]
fn bitwise_xor_double_literals() {
    let inst = rir::Instruction::BitwiseXor(
        rir::Operand::Literal(rir::Literal::Double(core::f64::consts::PI)),
        rir::Operand::Literal(rir::Literal::Double(core::f64::consts::E)),
        rir::Variable {
            id: rir::VariableId(0),
            ty: rir::Ty::Double,
        },
    );
    let _ = &inst.to_qir(&rir::Program::default());
}
