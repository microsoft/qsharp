// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

mod test_utils;

use indoc::indoc;
use qsc_rir::rir::{
    BlockId, Callable, CallableId, CallableType, Instruction, Literal, Operand, Ty,
};
use test_utils::{assert_block_instructions, assert_callable, compile_and_partially_evaluate};

fn double_to_unit_intrinsic_op() -> Callable {
    Callable {
        name: "op".to_string(),
        input_type: vec![Ty::Double],
        output_type: None,
        body: None,
        call_type: CallableType::Regular,
    }
}

#[test]
fn call_to_intrinsic_operation_using_double_literal() {
    let program = compile_and_partially_evaluate(indoc! {r#"
        namespace Test {
            operation op(d : Double) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                op(1.0);
            }
        }
    "#});
    let op_callable_id = CallableId(1);
    assert_callable(&program, op_callable_id, &double_to_unit_intrinsic_op());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Double(1.0))],
                None,
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn calls_to_intrinsic_operation_using_inline_expressions() {
    let program = compile_and_partially_evaluate(indoc! {r#"
        namespace Test {
            function PI() : Double { 3.14159 }
            operation op(d : Double) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                op(2.71828 * 0.0);
                op(PI() / PI());
                op((PI() + PI()) / (2.0 * PI()));
            }
        }
    "#});
    let op_callable_id = CallableId(1);
    assert_callable(&program, op_callable_id, &double_to_unit_intrinsic_op());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Double(0.0))],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Double(1.0))],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Double(1.0))],
                None,
            ),
            Instruction::Return,
        ],
    );
}

#[test]
fn calls_to_intrinsic_operation_using_variables() {
    let program = compile_and_partially_evaluate(indoc! {r#"
        namespace Test {
            operation op(d : Double) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                let pi = 4.0;
                let pi_over_two = pi / 2.0;
                op(pi_over_two);
                mutable n_pi = 1.0 * pi;
                op(n_pi);
                set n_pi = 2.0 * pi;
                op(n_pi);
            }
        }
    "#});
    let op_callable_id = CallableId(1);
    assert_callable(&program, op_callable_id, &double_to_unit_intrinsic_op());
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Double(2.0))],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Double(4.0))],
                None,
            ),
            Instruction::Call(
                op_callable_id,
                vec![Operand::Literal(Literal::Double(8.0))],
                None,
            ),
            Instruction::Return,
        ],
    );
}
