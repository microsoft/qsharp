// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes, clippy::similar_names)]

pub mod test_utils;

use expect_test::expect;
use indoc::indoc;
use qsc_rir::rir::CallableId;
use test_utils::{assert_callable, compile_and_partially_evaluate};

use crate::test_utils::assert_blocks;

#[test]
fn dynamic_int_from_if_expression_with_single_measurement_comparison_and_classical_blocks() {
    let program = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                let b = if r == Zero { 0 } else { 1 };
            }
        }
        "#,
    });
    println!("{program}");

    // Verify the callables added to the program.
    let mresetz_callable_id = CallableId(1);
    assert_callable(
        &program,
        mresetz_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__mresetz__body
            call_type: Measurement
            input_type:
                [0]: Qubit
                [1]: Result
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let read_result_callable_id = CallableId(2);
    assert_callable(
        &program,
        read_result_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__read_result__body
            call_type: Readout
            input_type:
                [0]: Result
            output_type: Boolean
            body: <NONE>"#]],
    );

    assert_blocks(&program, &expect![[r#"
        Blocks:
        Block 0:Block:
            Call id(1), args( Qubit(0), Result(0), )
            Variable(0, Boolean) = Call id(2), args( Result(0), )
            Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
            Branch Variable(1, Boolean), 2, 3
        Block 1:Block:
            Call id(3), args( Integer(0), Pointer, )
            Return
        Block 2:Block:
            Variable(2, Integer) = Store Integer(0)
            Jump(1)
        Block 3:Block:
            Variable(2, Integer) = Store Integer(1)
            Jump(1)"#]]);
}

#[test]
#[should_panic(expected = "() cannot be mapped to a RIR operand")]
fn dynamic_int_from_if_expression_with_single_measurement_comparison_and_non_classical_blocks() {
    let _ = compile_and_partially_evaluate(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1) = (Qubit(), Qubit());
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q0);
                let b = if r == Zero {
                    opA(q1);
                    0
                } else {
                    opB(q1);
                    1
                };
            }
        }
        "#,
    });
}
