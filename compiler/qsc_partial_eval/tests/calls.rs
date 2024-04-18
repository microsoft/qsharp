// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(
    clippy::needless_raw_string_hashes,
    clippy::similar_names,
    clippy::too_many_lines
)]

pub mod test_utils;

use expect_test::expect;
use indoc::indoc;
use qsc_rir::rir::{BlockId, CallableId};
use test_utils::{assert_block_instructions, assert_callable, compile_and_partially_evaluate};

#[test]
fn call_to_single_qubit_unitary_with_two_calls_to_the_same_intrinsic() {
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
    let op_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_callable_id,
        &expect![[r#"
        Callable:
            name: Op
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
        Block:
            Call id(1), args( Qubit(0), )
            Call id(1), args( Qubit(0), )
            Call id(2), args( Integer(0), Pointer, )
            Return"#]],
    );
}

#[test]
fn call_to_single_qubit_unitary_with_calls_to_different_intrinsics() {
    let program = compile_and_partially_evaluate(indoc! {r#"
        namespace Test {
            operation OpA(q : Qubit) : Unit { body intrinsic; }
            operation OpB(q : Qubit) : Unit { body intrinsic; }
            operation Combined(q : Qubit) : Unit {
                OpA(q);
                OpB(q);
            }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                Combined(q);
            }
        }
    "#});
    let op_a_callable_id = CallableId(1);
    let op_b_callable_id = CallableId(2);
    assert_callable(
        &program,
        op_a_callable_id,
        &expect![[r#"
        Callable:
            name: OpA
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_callable(
        &program,
        op_b_callable_id,
        &expect![[r#"
        Callable:
            name: OpB
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
        Block:
            Call id(1), args( Qubit(0), )
            Call id(2), args( Qubit(0), )
            Call id(3), args( Integer(0), Pointer, )
            Return"#]],
    );
}

#[test]
fn call_to_two_qubit_unitary() {
    let program = compile_and_partially_evaluate(indoc! {r#"
        namespace Test {
            operation Op(q0 : Qubit, q1 : Qubit) : Unit { body intrinsic; }
            operation ApplyOpCombinations(q0 : Qubit, q1 : Qubit) : Unit {
                Op(q0, q1);
                Op(q1, q0);
            }
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1) = (Qubit(), Qubit());
                ApplyOpCombinations(q0, q1);
            }
        }
    "#});
    let op_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_callable_id,
        &expect![[r#"
        Callable:
            name: Op
            call_type: Regular
            input_type:
                [0]: Qubit
                [1]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
        Block:
            Call id(1), args( Qubit(0), Qubit(1), )
            Call id(1), args( Qubit(1), Qubit(0), )
            Call id(2), args( Integer(0), Pointer, )
            Return"#]],
    );
}

#[test]
fn call_to_unitary_that_receives_double_and_qubit() {
    let program = compile_and_partially_evaluate(indoc! {r#"
        namespace Test {
            operation DoubleFirst(d : Double, q : Qubit) : Unit { body intrinsic; }
            operation QubitFirst(q : Qubit, d : Double) : Unit { body intrinsic; }
            operation Op(d : Double, q : Qubit) : Unit {
                DoubleFirst(d, q);
                QubitFirst(q, d);
            }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                Op(1.0, q);
            }
        }
    "#});
    let double_first_callable_id = CallableId(1);
    let qubit_first_callable_id = CallableId(2);
    assert_callable(
        &program,
        double_first_callable_id,
        &expect![[r#"
        Callable:
            name: DoubleFirst
            call_type: Regular
            input_type:
                [0]: Double
                [1]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_callable(
        &program,
        qubit_first_callable_id,
        &expect![[r#"
        Callable:
            name: QubitFirst
            call_type: Regular
            input_type:
                [0]: Qubit
                [1]: Double
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
        Block:
            Call id(1), args( Double(1), Qubit(0), )
            Call id(2), args( Qubit(0), Double(1), )
            Call id(3), args( Integer(0), Pointer, )
            Return"#]],
    );
}

#[test]
fn call_to_unitary_rotation_unitary_with_computation() {
    let program = compile_and_partially_evaluate(indoc! {r#"
        namespace Test {
            operation Rotation(d : Double, q : Qubit) : Unit { body intrinsic; }
            operation RotationWithComputation(d : Double, q : Qubit) : Unit {
                Rotation(2.0 * d, q);
            }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                RotationWithComputation(2.0, q);
                RotationWithComputation(3.0, q);
            }
        }
    "#});
    let rotation_callable_id = CallableId(1);
    assert_callable(
        &program,
        rotation_callable_id,
        &expect![[r#"
        Callable:
            name: Rotation
            call_type: Regular
            input_type:
                [0]: Double
                [1]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
        Block:
            Call id(1), args( Double(4), Qubit(0), )
            Call id(1), args( Double(6), Qubit(0), )
            Call id(2), args( Integer(0), Pointer, )
            Return"#]],
    );
}
