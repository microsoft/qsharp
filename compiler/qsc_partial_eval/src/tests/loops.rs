// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{assert_block_instructions, assert_callable, get_rir_program};
use expect_test::expect;
use indoc::indoc;
use qsc_rir::rir::{BlockId, CallableId};

#[test]
fn unitary_call_within_a_for_loop() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation op(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                for _ in 1..3 {
                    op(q);
                }
            }
        }
        "#,
    });

    let op_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_callable_id,
        &expect![[r#"
        Callable:
            name: op
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
                Variable(0, Integer) = Store Integer(1)
                Call id(1), args( Qubit(0), )
                Variable(0, Integer) = Store Integer(2)
                Call id(1), args( Qubit(0), )
                Variable(0, Integer) = Store Integer(3)
                Call id(1), args( Qubit(0), )
                Variable(0, Integer) = Store Integer(4)
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn unitary_call_within_a_while_loop() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation op(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                mutable idx = 0;
                while idx < 3 {
                    op(q);
                    set idx += 1;
                }
            }
        }
        "#,
    });

    let rotation_callable_id = CallableId(1);
    assert_callable(
        &program,
        rotation_callable_id,
        &expect![[r#"
        Callable:
            name: op
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
                Variable(0, Integer) = Store Integer(0)
                Call id(1), args( Qubit(0), )
                Variable(0, Integer) = Store Integer(1)
                Call id(1), args( Qubit(0), )
                Variable(0, Integer) = Store Integer(2)
                Call id(1), args( Qubit(0), )
                Variable(0, Integer) = Store Integer(3)
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn unitary_call_within_a_repeat_until_loop() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation op(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                mutable idx = 0;
                repeat {
                    op(q);
                    set idx += 1;
                } until idx >= 3;
            }
        }
        "#,
    });

    let op_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_callable_id,
        &expect![[r#"
        Callable:
            name: op
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
                Variable(0, Integer) = Store Integer(0)
                Variable(1, Boolean) = Store Bool(true)
                Call id(1), args( Qubit(0), )
                Variable(0, Integer) = Store Integer(1)
                Variable(1, Boolean) = Store Bool(true)
                Call id(1), args( Qubit(0), )
                Variable(0, Integer) = Store Integer(2)
                Variable(1, Boolean) = Store Bool(true)
                Call id(1), args( Qubit(0), )
                Variable(0, Integer) = Store Integer(3)
                Variable(1, Boolean) = Store Bool(false)
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn rotation_call_within_a_for_loop() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation rotation(theta : Double, q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                for theta in [0.0, 1.0, 2.0] {
                    rotation(theta, q);
                }
            }
        }
        "#,
    });

    let rotation_callable_id = CallableId(1);
    assert_callable(
        &program,
        rotation_callable_id,
        &expect![[r#"
        Callable:
            name: rotation
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
                Variable(0, Integer) = Store Integer(0)
                Call id(1), args( Double(0), Qubit(0), )
                Variable(0, Integer) = Store Integer(1)
                Call id(1), args( Double(1), Qubit(0), )
                Variable(0, Integer) = Store Integer(2)
                Call id(1), args( Double(2), Qubit(0), )
                Variable(0, Integer) = Store Integer(3)
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn rotation_call_within_a_while_loop() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation rotation(theta : Double, q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let angles = [0.0, 1.0, 2.0];
                mutable idx = 0;
                while idx < 3 {
                    rotation(angles[idx], q);
                    set idx += 1;
                }
            }
        }
        "#,
    });

    let op_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_callable_id,
        &expect![[r#"
        Callable:
            name: rotation
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
                Variable(0, Integer) = Store Integer(0)
                Call id(1), args( Double(0), Qubit(0), )
                Variable(0, Integer) = Store Integer(1)
                Call id(1), args( Double(1), Qubit(0), )
                Variable(0, Integer) = Store Integer(2)
                Call id(1), args( Double(2), Qubit(0), )
                Variable(0, Integer) = Store Integer(3)
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn rotation_call_within_a_repeat_until_loop() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation rotation(theta : Double, q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let angles = [0.0, 1.0, 2.0];
                mutable idx = 0;
                repeat {
                    rotation(angles[idx], q);
                    set idx += 1;
                } until idx >= 3;
            }
        }
        "#,
    });

    let rotation_callable_id = CallableId(1);
    assert_callable(
        &program,
        rotation_callable_id,
        &expect![[r#"
        Callable:
            name: rotation
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
                Variable(0, Integer) = Store Integer(0)
                Variable(1, Boolean) = Store Bool(true)
                Call id(1), args( Double(0), Qubit(0), )
                Variable(0, Integer) = Store Integer(1)
                Variable(1, Boolean) = Store Bool(true)
                Call id(1), args( Double(1), Qubit(0), )
                Variable(0, Integer) = Store Integer(2)
                Variable(1, Boolean) = Store Bool(true)
                Call id(1), args( Double(2), Qubit(0), )
                Variable(0, Integer) = Store Integer(3)
                Variable(1, Boolean) = Store Bool(false)
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn mutable_bool_updated_in_loop() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                mutable flag = false;
                for _ in 1..3 {
                    if not flag {
                        set flag = MResetZ(q) == One;
                    }
                }
            }
        }
        "#,
    });

    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Variable(0, Boolean) = Store Bool(false)
                Variable(1, Integer) = Store Integer(1)
                Call id(1), args( Qubit(0), Result(0), )
                Variable(2, Boolean) = Call id(2), args( Result(0), )
                Variable(3, Boolean) = Store Variable(2, Boolean)
                Variable(0, Boolean) = Store Variable(3, Boolean)
                Variable(1, Integer) = Store Integer(2)
                Variable(4, Boolean) = LogicalNot Variable(0, Boolean)
                Branch Variable(4, Boolean), 2, 1"#]],
    );
}

#[test]
fn mutable_int_updated_in_loop() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                mutable count = 1;
                for _ in 1..3 {
                    if count > 0 and MResetZ(q) == One {
                        set count = -count;
                    }
                }
            }
        }
        "#,
    });

    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
            Block:
                Variable(0, Integer) = Store Integer(1)
                Variable(1, Integer) = Store Integer(1)
                Call id(1), args( Qubit(0), Result(0), )
                Variable(2, Boolean) = Call id(2), args( Result(0), )
                Variable(3, Boolean) = Store Variable(2, Boolean)
                Branch Variable(3, Boolean), 2, 1"#]],
    );
}
