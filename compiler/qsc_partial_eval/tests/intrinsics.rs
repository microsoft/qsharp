// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

pub mod test_utils;

use indoc::{formatdoc, indoc};
use qsc_rir::rir::{
    BlockId, Callable, CallableId, CallableType, Instruction, Literal, Operand, Ty,
};
use test_utils::{assert_block_instructions, assert_callable, compile_and_partially_evaluate};

fn check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
    intrinsic_name: &str,
) {
    let program = compile_and_partially_evaluate(
        formatdoc! {
            r#"
            namespace Test {{
                @EntryPoint()
                operation Main() : Unit {{
                    use q = Qubit();
                    QIR.Intrinsic.{intrinsic_name}(q);
                }}
            }}
            "#,
            intrinsic_name = intrinsic_name
        }
        .as_str(),
    );
    let op_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_callable_id,
        &single_qubit_intrinsic_op(intrinsic_name),
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Return,
        ],
    );
}

fn check_call_to_single_qubit_rotation_instrinsic_adds_callable_and_generates_instruction(
    intrinsic_name: &str,
) {
    let program = compile_and_partially_evaluate(
        formatdoc! {
            r#"
            namespace Test {{
                @EntryPoint()
                operation Main() : Unit {{
                    use q = Qubit();
                    QIR.Intrinsic.{intrinsic_name}(0.0, q);
                }}
            }}
            "#,
            intrinsic_name = intrinsic_name
        }
        .as_str(),
    );
    let op_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_callable_id,
        &single_qubit_rotation_intrinsic_op(intrinsic_name),
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![
                    Operand::Literal(Literal::Double(0.0)),
                    Operand::Literal(Literal::Qubit(0)),
                ],
                None,
            ),
            Instruction::Return,
        ],
    );
}

fn check_call_to_two_qubits_rotation_instrinsic_adds_callable_and_generates_instruction(
    intrinsic_name: &str,
) {
    let program = compile_and_partially_evaluate(
        formatdoc! {
            r#"
            namespace Test {{
                @EntryPoint()
                operation Main() : Unit {{
                    use (q0, q1) = (Qubit(), Qubit());
                    QIR.Intrinsic.{intrinsic_name}(0.0, q0, q1);
                }}
            }}
            "#,
            intrinsic_name = intrinsic_name
        }
        .as_str(),
    );
    let op_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_callable_id,
        &two_qubits_rotation_intrinsic_op(intrinsic_name),
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![
                    Operand::Literal(Literal::Double(0.0)),
                    Operand::Literal(Literal::Qubit(0)),
                    Operand::Literal(Literal::Qubit(1)),
                ],
                None,
            ),
            Instruction::Return,
        ],
    );
}

fn check_call_to_two_qubits_instrinsic_adds_callable_and_generates_instruction(
    intrinsic_name: &str,
) {
    let program = compile_and_partially_evaluate(
        formatdoc! {
            r#"
            namespace Test {{
                @EntryPoint()
                operation Main() : Unit {{
                    use (q0, q1) = (Qubit(), Qubit());
                    QIR.Intrinsic.{intrinsic_name}(q0, q1);
                }}
            }}
            "#,
            intrinsic_name = intrinsic_name
        }
        .as_str(),
    );
    let op_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_callable_id,
        &two_qubits_intrinsic_op(intrinsic_name),
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![
                    Operand::Literal(Literal::Qubit(0)),
                    Operand::Literal(Literal::Qubit(1)),
                ],
                None,
            ),
            Instruction::Return,
        ],
    );
}

fn check_call_to_three_qubits_instrinsic_adds_callable_and_generates_instruction(
    intrinsic_name: &str,
) {
    let program = compile_and_partially_evaluate(
        formatdoc! {
            r#"
            namespace Test {{
                @EntryPoint()
                operation Main() : Unit {{
                    use (q0, q1, q2) = (Qubit(), Qubit(), Qubit());
                    QIR.Intrinsic.{intrinsic_name}(q0, q1, q2);
                }}
            }}
            "#,
            intrinsic_name = intrinsic_name
        }
        .as_str(),
    );
    let op_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_callable_id,
        &three_qubits_intrinsic_op(intrinsic_name),
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![
                    Operand::Literal(Literal::Qubit(0)),
                    Operand::Literal(Literal::Qubit(1)),
                    Operand::Literal(Literal::Qubit(2)),
                ],
                None,
            ),
            Instruction::Return,
        ],
    );
}

fn measurement_intrinsic_op(name: &str) -> Callable {
    Callable {
        name: name.to_string(),
        input_type: vec![Ty::Qubit, Ty::Result],
        output_type: None,
        body: None,
        call_type: CallableType::Measurement,
    }
}

fn reset_intrinsic_op() -> Callable {
    Callable {
        name: "__quantum__qis__reset__body".to_string(),
        input_type: vec![Ty::Qubit],
        output_type: None,
        body: None,
        call_type: CallableType::Reset,
    }
}

fn single_qubit_intrinsic_op(name: &str) -> Callable {
    Callable {
        name: name.to_string(),
        input_type: vec![Ty::Qubit],
        output_type: None,
        body: None,
        call_type: CallableType::Regular,
    }
}

fn single_qubit_rotation_intrinsic_op(name: &str) -> Callable {
    Callable {
        name: name.to_string(),
        input_type: vec![Ty::Double, Ty::Qubit],
        output_type: None,
        body: None,
        call_type: CallableType::Regular,
    }
}

fn two_qubits_intrinsic_op(name: &str) -> Callable {
    Callable {
        name: name.to_string(),
        input_type: vec![Ty::Qubit, Ty::Qubit],
        output_type: None,
        body: None,
        call_type: CallableType::Regular,
    }
}

fn two_qubits_rotation_intrinsic_op(name: &str) -> Callable {
    Callable {
        name: name.to_string(),
        input_type: vec![Ty::Double, Ty::Qubit, Ty::Qubit],
        output_type: None,
        body: None,
        call_type: CallableType::Regular,
    }
}

fn three_qubits_intrinsic_op(name: &str) -> Callable {
    Callable {
        name: name.to_string(),
        input_type: vec![Ty::Qubit, Ty::Qubit, Ty::Qubit],
        output_type: None,
        body: None,
        call_type: CallableType::Regular,
    }
}

#[test]
fn call_to_intrinsic_h_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__h__body",
    );
}

#[test]
fn call_to_intrinsic_s_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__s__body",
    );
}

#[test]
fn call_to_intrinsic_adjoint_s_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__s__adj",
    );
}

#[test]
fn call_to_intrinsic_t_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__t__body",
    );
}

#[test]
fn call_to_intrinsic_adjoint_t_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__t__adj",
    );
}

#[test]
fn call_to_intrinsic_x_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__x__body",
    );
}

#[test]
fn call_to_intrinsic_y_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__y__body",
    );
}

#[test]
fn call_to_intrinsic_z_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__z__body",
    );
}

#[test]
fn call_to_intrinsic_swap_adds_callable_and_generates_instruction() {
    check_call_to_two_qubits_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__swap__body",
    );
}

#[test]
fn call_to_intrinsic_cx_adds_callable_and_generates_instruction() {
    check_call_to_two_qubits_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__cx__body",
    );
}

#[test]
fn call_to_intrinsic_cy_adds_callable_and_generates_instruction() {
    check_call_to_two_qubits_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__cy__body",
    );
}

#[test]
fn call_to_intrinsic_cz_adds_callable_and_generates_instruction() {
    check_call_to_two_qubits_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__cz__body",
    );
}

#[test]
fn call_to_intrinsic_ccx_adds_callable_and_generates_instruction() {
    check_call_to_three_qubits_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__ccx__body",
    );
}

#[test]
fn call_to_intrinsic_rx_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_rotation_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__rx__body",
    );
}

#[test]
fn call_to_intrinsic_rxx_adds_callable_and_generates_instruction() {
    check_call_to_two_qubits_rotation_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__rxx__body",
    );
}

#[test]
fn call_to_intrinsic_ry_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_rotation_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__ry__body",
    );
}

#[test]
fn call_to_intrinsic_ryy_adds_callable_and_generates_instruction() {
    check_call_to_two_qubits_rotation_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__ryy__body",
    );
}

#[test]
fn call_to_intrinsic_rz_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_rotation_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__rz__body",
    );
}

#[test]
fn call_to_intrinsic_rzz_adds_callable_and_generates_instruction() {
    check_call_to_two_qubits_rotation_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__rzz__body",
    );
}

#[test]
fn check_partial_eval_for_call_to_reset() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                QIR.Intrinsic.__quantum__qis__reset__body(q);
            }
        }
        "#,
    });
    let op_callable_id = CallableId(1);
    assert_callable(&program, op_callable_id, &reset_intrinsic_op());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn call_to_intrinsic_m_adds_callable_and_generates_instruction() {
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
    assert_callable(
        &program,
        op_callable_id,
        &measurement_intrinsic_op("__quantum__qis__mz__body"),
    );
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
fn call_to_intrinsic_mresetz_adds_callable_and_generates_instruction() {
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
    assert_callable(
        &program,
        op_callable_id,
        &measurement_intrinsic_op("__quantum__qis__mresetz__body"),
    );
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
