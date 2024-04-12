// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

pub mod test_utils;

use indoc::indoc;
use qsc_rir::rir::{
    BlockId, Callable, CallableId, CallableType, Instruction, Literal, Operand, Ty,
};
use test_utils::{assert_block_instructions, assert_callable, compile_and_partially_evaluate};

fn single_qubit_unitary_intrinsic_callable() -> Callable {
    Callable {
        name: "Op".to_string(),
        input_type: vec![Ty::Qubit],
        output_type: None,
        body: None,
        call_type: CallableType::Regular,
    }
}

#[test]
fn call_to_single_qubit_unitary() {
    let program = compile_and_partially_evaluate(indoc! {r#"
        namespace Test {
            operation Op(q : Qubit) : Unit { body intrinsic; }
            operation OpSquared(q : Qubit) : Unit {
                Op(q);
                Op(q);
            }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                OpSquared(q);
            }
        }
    "#});

    let single_qubit_unitary_intrinsic_callable_id = CallableId(1);
    assert_callable(
        &program,
        single_qubit_unitary_intrinsic_callable_id,
        &single_qubit_unitary_intrinsic_callable(),
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &[
            Instruction::Call(
                single_qubit_unitary_intrinsic_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Call(
                single_qubit_unitary_intrinsic_callable_id,
                vec![Operand::Literal(Literal::Qubit(0))],
                None,
            ),
            Instruction::Return,
        ],
    );
}
