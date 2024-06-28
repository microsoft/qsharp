// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(
    clippy::needless_raw_string_hashes,
    clippy::similar_names,
    clippy::too_many_lines
)]

use super::{
    assert_blocks, assert_callable, assert_error, get_partial_evaluation_error, get_rir_program,
};
use expect_test::expect;
use indoc::indoc;
use qsc_rir::rir::CallableId;

#[test]
fn if_expression_with_true_condition() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                if true {
                    opA(q);
                }
            }
        }
        "#,
    });
    let op_a_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_a_callable_id,
        &expect![[r#"
        Callable:
            name: opA
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
        Blocks:
        Block 0:Block:
            Call id(1), args( Qubit(0), )
            Call id(2), args( Integer(0), Pointer, )
            Return"#]],
    );
}

#[test]
fn if_expression_with_false_condition() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                if false {
                    opA(q);
                }
            }
        }
        "#,
    });
    assert_blocks(
        &program,
        &expect![[r#"
        Blocks:
        Block 0:Block:
            Call id(1), args( Integer(0), Pointer, )
            Return"#]],
    );
}

#[test]
fn if_else_expression_with_true_condition() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                if true {
                    opA(q);
                } else {
                    opB(q);
                }
            }
        }
        "#,
    });
    let op_a_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_a_callable_id,
        &expect![[r#"
        Callable:
            name: opA
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
        Blocks:
        Block 0:Block:
            Call id(1), args( Qubit(0), )
            Call id(2), args( Integer(0), Pointer, )
            Return"#]],
    );
}

#[test]
fn if_else_expression_with_false_condition() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                if false {
                    opA(q);
                } else {
                    opB(q);
                }
            }
        }
        "#,
    });
    let op_b_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_b_callable_id,
        &expect![[r#"
        Callable:
            name: opB
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
        Blocks:
        Block 0:Block:
            Call id(1), args( Qubit(0), )
            Call id(2), args( Integer(0), Pointer, )
            Return"#]],
    );
}

#[test]
fn if_elif_else_expression_with_true_elif_condition() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            operation opC(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                if false {
                    opA(q);
                } elif true {
                    opB(q);
                } else {
                    opC(q);
                }
            }
        }
        "#,
    });
    let op_b_callable_id = CallableId(1);
    assert_callable(
        &program,
        op_b_callable_id,
        &expect![[r#"
        Callable:
            name: opB
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
        Blocks:
        Block 0:Block:
            Call id(1), args( Qubit(0), )
            Call id(2), args( Integer(0), Pointer, )
            Return"#]],
    );
}

#[test]
fn if_expression_with_dynamic_condition() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                if r == Zero {
                    opA(q);
                }
            }
        }
        "#,
    });

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
    let op_a_callable_id = CallableId(3);
    assert_callable(
        &program,
        op_a_callable_id,
        &expect![[r#"
        Callable:
            name: opA
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );

    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 1
            Block 1:Block:
                Call id(4), args( Integer(0), Pointer, )
                Return
            Block 2:Block:
                Call id(3), args( Qubit(0), )
                Jump(1)"#]],
    );
}

#[test]
fn if_else_expression_with_dynamic_condition() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                if r == One {
                    opA(q);
                } else {
                    opB(q);
                }
            }
        }
        "#,
    });

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
    let op_a_callable_id = CallableId(3);
    assert_callable(
        &program,
        op_a_callable_id,
        &expect![[r#"
        Callable:
            name: opA
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let op_b_callable_id = CallableId(4);
    assert_callable(
        &program,
        op_b_callable_id,
        &expect![[r#"
        Callable:
            name: opB
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );

    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Store Variable(0, Boolean)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Call id(5), args( Integer(0), Pointer, )
                Return
            Block 2:Block:
                Call id(3), args( Qubit(0), )
                Jump(1)
            Block 3:Block:
                Call id(4), args( Qubit(0), )
                Jump(1)"#]],
    );
}

#[test]
fn if_elif_else_expression_with_dynamic_condition() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            operation opC(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1, q2) = (Qubit(), Qubit(), Qubit());
                let r0 = QIR.Intrinsic.__quantum__qis__mresetz__body(q0);
                let r1 = QIR.Intrinsic.__quantum__qis__mresetz__body(q1);
                if r0 == One {
                    opA(q2);
                } elif r1 == One {
                    opB(q2);
                } else {
                    opC(q2);
                }
            }
        }
        "#,
    });

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
    let op_a_callable_id = CallableId(3);
    assert_callable(
        &program,
        op_a_callable_id,
        &expect![[r#"
        Callable:
            name: opA
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let op_b_callable_id = CallableId(4);
    assert_callable(
        &program,
        op_b_callable_id,
        &expect![[r#"
        Callable:
            name: opB
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let op_c_callable_id = CallableId(5);
    assert_callable(
        &program,
        op_c_callable_id,
        &expect![[r#"
        Callable:
            name: opC
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );

    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Call id(1), args( Qubit(1), Result(1), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Store Variable(0, Boolean)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Call id(6), args( Integer(0), Pointer, )
                Return
            Block 2:Block:
                Call id(3), args( Qubit(2), )
                Jump(1)
            Block 3:Block:
                Variable(2, Boolean) = Call id(2), args( Result(1), )
                Variable(3, Boolean) = Store Variable(2, Boolean)
                Branch Variable(3, Boolean), 5, 6
            Block 4:Block:
                Jump(1)
            Block 5:Block:
                Call id(4), args( Qubit(2), )
                Jump(4)
            Block 6:Block:
                Call id(5), args( Qubit(2), )
                Jump(4)"#]],
    );
}

#[test]
fn if_expression_with_dynamic_condition_and_nested_if_expression_with_true_condition() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                if r == Zero {
                    if true {
                        opA(q);
                    }
                }
            }
        }
        "#,
    });

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
    let op_a_callable_id = CallableId(3);
    assert_callable(
        &program,
        op_a_callable_id,
        &expect![[r#"
        Callable:
            name: opA
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );

    assert_blocks(
        &program,
        &expect![[r#"
        Blocks:
        Block 0:Block:
            Call id(1), args( Qubit(0), Result(0), )
            Variable(0, Boolean) = Call id(2), args( Result(0), )
            Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
            Branch Variable(1, Boolean), 2, 1
        Block 1:Block:
            Call id(4), args( Integer(0), Pointer, )
            Return
        Block 2:Block:
            Call id(3), args( Qubit(0), )
            Jump(1)"#]],
    );
}

#[test]
fn if_expression_with_dynamic_condition_and_nested_if_expression_with_false_condition() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                if r == Zero {
                    if false {
                        opA(q);
                    }
                }
            }
        }
        "#,
    });

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

    assert_blocks(
        &program,
        &expect![[r#"
        Blocks:
        Block 0:Block:
            Call id(1), args( Qubit(0), Result(0), )
            Variable(0, Boolean) = Call id(2), args( Result(0), )
            Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
            Branch Variable(1, Boolean), 2, 1
        Block 1:Block:
            Call id(3), args( Integer(0), Pointer, )
            Return
        Block 2:Block:
            Jump(1)"#]],
    );
}

#[test]
fn if_else_expression_with_dynamic_condition_and_nested_if_expression_with_true_condition() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                if r == One {
                    opA(q);
                } else {
                    if true {
                        opB(q);
                    }
                }
            }
        }
        "#,
    });

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
    let op_a_callable_id = CallableId(3);
    assert_callable(
        &program,
        op_a_callable_id,
        &expect![[r#"
        Callable:
            name: opA
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let op_b_callable_id = CallableId(4);
    assert_callable(
        &program,
        op_b_callable_id,
        &expect![[r#"
        Callable:
            name: opB
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );

    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Store Variable(0, Boolean)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Call id(5), args( Integer(0), Pointer, )
                Return
            Block 2:Block:
                Call id(3), args( Qubit(0), )
                Jump(1)
            Block 3:Block:
                Call id(4), args( Qubit(0), )
                Jump(1)"#]],
    );
}

#[test]
fn if_else_expression_with_dynamic_condition_and_nested_if_expression_with_false_condition() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                if r == One {
                    opA(q);
                } else {
                    if false {
                        opB(q);
                    }
                }
            }
        }
        "#,
    });

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
    let op_a_callable_id = CallableId(3);
    assert_callable(
        &program,
        op_a_callable_id,
        &expect![[r#"
        Callable:
            name: opA
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );

    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Store Variable(0, Boolean)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Call id(4), args( Integer(0), Pointer, )
                Return
            Block 2:Block:
                Call id(3), args( Qubit(0), )
                Jump(1)
            Block 3:Block:
                Jump(1)"#]],
    );
}

#[test]
fn if_expression_with_dynamic_condition_and_nested_if_expression_with_dynamic_condition() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1, q2) = (Qubit(), Qubit(), Qubit());
                let r0 = QIR.Intrinsic.__quantum__qis__mresetz__body(q0);
                let r1 = QIR.Intrinsic.__quantum__qis__mresetz__body(q1);
                if r0 == Zero {
                    if r1 == One {
                        opA(q2);
                    }
                }
            }
        }
        "#,
    });

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
    let op_a_callable_id = CallableId(3);
    assert_callable(
        &program,
        op_a_callable_id,
        &expect![[r#"
        Callable:
            name: opA
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );

    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Call id(1), args( Qubit(1), Result(1), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 1
            Block 1:Block:
                Call id(4), args( Integer(0), Pointer, )
                Return
            Block 2:Block:
                Variable(2, Boolean) = Call id(2), args( Result(1), )
                Variable(3, Boolean) = Store Variable(2, Boolean)
                Branch Variable(3, Boolean), 4, 3
            Block 3:Block:
                Jump(1)
            Block 4:Block:
                Call id(3), args( Qubit(2), )
                Jump(3)"#]],
    );
}

#[allow(clippy::too_many_lines)]
#[test]
fn doubly_nested_if_else_expressions_with_dynamic_conditions() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            operation opC(q : Qubit) : Unit { body intrinsic; }
            operation opD(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1, q2) = (Qubit(), Qubit(), Qubit());
                let r0 = QIR.Intrinsic.__quantum__qis__mresetz__body(q0);
                let r1 = QIR.Intrinsic.__quantum__qis__mresetz__body(q1);
                if r0 == Zero {
                    if r1 == Zero {
                        opA(q2);
                    } else {
                        opB(q2);
                    }
                } else {
                    if r1 == One{
                        opC(q2);
                    } else {
                        opD(q2);
                    }
                }
            }
        }
        "#,
    });

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
    let op_a_callable_id = CallableId(3);
    assert_callable(
        &program,
        op_a_callable_id,
        &expect![[r#"
        Callable:
            name: opA
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let op_b_callable_id = CallableId(4);
    assert_callable(
        &program,
        op_b_callable_id,
        &expect![[r#"
        Callable:
            name: opB
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let op_c_callable_id = CallableId(5);
    assert_callable(
        &program,
        op_c_callable_id,
        &expect![[r#"
        Callable:
            name: opC
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let op_d_callable_id = CallableId(6);
    assert_callable(
        &program,
        op_d_callable_id,
        &expect![[r#"
        Callable:
            name: opD
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );

    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Call id(1), args( Qubit(1), Result(1), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
                Branch Variable(1, Boolean), 2, 6
            Block 1:Block:
                Call id(7), args( Integer(0), Pointer, )
                Return
            Block 2:Block:
                Variable(2, Boolean) = Call id(2), args( Result(1), )
                Variable(3, Boolean) = Icmp Eq, Variable(2, Boolean), Bool(false)
                Branch Variable(3, Boolean), 4, 5
            Block 3:Block:
                Jump(1)
            Block 4:Block:
                Call id(3), args( Qubit(2), )
                Jump(3)
            Block 5:Block:
                Call id(4), args( Qubit(2), )
                Jump(3)
            Block 6:Block:
                Variable(4, Boolean) = Call id(2), args( Result(1), )
                Variable(5, Boolean) = Store Variable(4, Boolean)
                Branch Variable(5, Boolean), 8, 9
            Block 7:Block:
                Jump(1)
            Block 8:Block:
                Call id(5), args( Qubit(2), )
                Jump(7)
            Block 9:Block:
                Call id(6), args( Qubit(2), )
                Jump(7)"#]],
    );
}

#[test]
fn if_expression_with_dynamic_condition_and_subsequent_call_to_operation() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                if r == Zero {
                    opA(q);
                }
                opB(q);
            }
        }
        "#,
    });

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
    let op_a_callable_id = CallableId(3);
    assert_callable(
        &program,
        op_a_callable_id,
        &expect![[r#"
        Callable:
            name: opA
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let op_b_callable_id = CallableId(4);
    assert_callable(
        &program,
        op_b_callable_id,
        &expect![[r#"
        Callable:
            name: opB
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );

    assert_blocks(
        &program,
        &expect![[r#"
        Blocks:
        Block 0:Block:
            Call id(1), args( Qubit(0), Result(0), )
            Variable(0, Boolean) = Call id(2), args( Result(0), )
            Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false)
            Branch Variable(1, Boolean), 2, 1
        Block 1:Block:
            Call id(4), args( Qubit(0), )
            Call id(5), args( Integer(0), Pointer, )
            Return
        Block 2:Block:
            Call id(3), args( Qubit(0), )
            Jump(1)"#]],
    );
}

#[test]
fn if_else_expression_with_dynamic_condition_and_subsequent_call_to_operation() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation opA(q : Qubit) : Unit { body intrinsic; }
            operation opB(q : Qubit) : Unit { body intrinsic; }
            operation opC(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                if r == One {
                    opA(q);
                } else {
                    opB(q);
                }
                opC(q);
            }
        }
        "#,
    });

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
    let op_a_callable_id = CallableId(3);
    assert_callable(
        &program,
        op_a_callable_id,
        &expect![[r#"
        Callable:
            name: opA
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let op_b_callable_id = CallableId(4);
    assert_callable(
        &program,
        op_b_callable_id,
        &expect![[r#"
        Callable:
            name: opB
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let op_c_callable_id = CallableId(5);
    assert_callable(
        &program,
        op_c_callable_id,
        &expect![[r#"
        Callable:
            name: opC
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );

    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), )
                Variable(0, Boolean) = Call id(2), args( Result(0), )
                Variable(1, Boolean) = Store Variable(0, Boolean)
                Branch Variable(1, Boolean), 2, 3
            Block 1:Block:
                Call id(5), args( Qubit(0), )
                Call id(6), args( Integer(0), Pointer, )
                Return
            Block 2:Block:
                Call id(3), args( Qubit(0), )
                Jump(1)
            Block 3:Block:
                Call id(4), args( Qubit(0), )
                Jump(1)"#]],
    );
}

#[test]
fn if_else_expression_with_result_literal_fails() {
    let error = get_partial_evaluation_error(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Result {
                use q = Qubit();
                MResetZ(q) == One ? One | MResetZ(q)
            }
        }
        "#,
    });

    assert_error(
        &error,
        &expect![[
            r#"Unexpected("dynamic value of type Result in conditional expression", PackageSpan { package: PackageId(2), span: Span { lo: 101, hi: 137 } })"#
        ]],
    );
}
