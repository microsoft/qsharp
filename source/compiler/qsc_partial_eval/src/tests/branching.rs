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
                Call id(1), args( Qubit(0), ) !dbg package_id=2 span=[177-180]
                Call id(2), args( Integer(0), Pointer, ) !dbg package_id=2 span=[106-110]
                Return !dbg package_id=2 span=[106-110]"#]],
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
                Call id(1), args( Integer(0), Pointer, ) !dbg package_id=2 span=[106-110]
                Return !dbg package_id=2 span=[106-110]"#]],
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
                Call id(1), args( Qubit(0), ) !dbg package_id=2 span=[233-236]
                Call id(2), args( Integer(0), Pointer, ) !dbg package_id=2 span=[162-166]
                Return !dbg package_id=2 span=[162-166]"#]],
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
                Call id(1), args( Qubit(0), ) !dbg package_id=2 span=[271-274]
                Call id(2), args( Integer(0), Pointer, ) !dbg package_id=2 span=[162-166]
                Return !dbg package_id=2 span=[162-166]"#]],
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
                Call id(1), args( Qubit(0), ) !dbg package_id=2 span=[332-335]
                Call id(2), args( Integer(0), Pointer, ) !dbg package_id=2 span=[218-222]
                Return !dbg package_id=2 span=[218-222]"#]],
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[163-206]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[222-231]
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false) !dbg package_id=2 span=[222-231]
                Branch Variable(1, Boolean), 2, 1 !dbg package_id=2 span=[222-231]
            Block 1:Block:
                Call id(4), args( Integer(0), Pointer, ) !dbg package_id=2 span=[106-110]
                Return !dbg package_id=2 span=[106-110]
            Block 2:Block:
                Call id(3), args( Qubit(0), ) !dbg package_id=2 span=[246-249]
                Jump(1) !dbg package_id=2 span=[232-263]"#]],
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[219-262]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[278-286]
                Variable(1, Boolean) = Store Variable(0, Boolean) !dbg package_id=2 span=[278-286]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[278-286]
            Block 1:Block:
                Call id(5), args( Integer(0), Pointer, ) !dbg package_id=2 span=[162-166]
                Return !dbg package_id=2 span=[162-166]
            Block 2:Block:
                Call id(3), args( Qubit(0), ) !dbg package_id=2 span=[301-304]
                Jump(1) !dbg package_id=2 span=[287-318]
            Block 3:Block:
                Call id(4), args( Qubit(0), ) !dbg package_id=2 span=[338-341]
                Jump(1) !dbg package_id=2 span=[319-355]"#]],
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[307-350]
                Call id(1), args( Qubit(1), Result(1), ) !dbg package_id=2 span=[373-416]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[433-442]
                Variable(1, Boolean) = Store Variable(0, Boolean) !dbg package_id=2 span=[433-442]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[433-442]
            Block 1:Block:
                Call id(6), args( Integer(0), Pointer, ) !dbg package_id=2 span=[218-222]
                Return !dbg package_id=2 span=[218-222]
            Block 2:Block:
                Call id(3), args( Qubit(2), ) !dbg package_id=2 span=[457-460]
                Jump(1) !dbg package_id=2 span=[443-475]
            Block 3:Block:
                Variable(2, Boolean) = Call id(2), args( Result(1), ) !dbg package_id=2 span=[481-490]
                Variable(3, Boolean) = Store Variable(2, Boolean) !dbg package_id=2 span=[481-490]
                Branch Variable(3, Boolean), 5, 6 !dbg package_id=2 span=[481-490]
            Block 4:Block:
                Jump(1) !dbg package_id=2 span=[476-561]
            Block 5:Block:
                Call id(4), args( Qubit(2), ) !dbg package_id=2 span=[505-508]
                Jump(4) !dbg package_id=2 span=[491-523]
            Block 6:Block:
                Call id(5), args( Qubit(2), ) !dbg package_id=2 span=[543-546]
                Jump(4) !dbg package_id=2 span=[524-561]"#]],
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[163-206]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[222-231]
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false) !dbg package_id=2 span=[222-231]
                Branch Variable(1, Boolean), 2, 1 !dbg package_id=2 span=[222-231]
            Block 1:Block:
                Call id(4), args( Integer(0), Pointer, ) !dbg package_id=2 span=[106-110]
                Return !dbg package_id=2 span=[106-110]
            Block 2:Block:
                Call id(3), args( Qubit(0), ) !dbg package_id=2 span=[272-275]
                Jump(1) !dbg package_id=2 span=[232-303]"#]],
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[163-206]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[222-231]
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false) !dbg package_id=2 span=[222-231]
                Branch Variable(1, Boolean), 2, 1 !dbg package_id=2 span=[222-231]
            Block 1:Block:
                Call id(3), args( Integer(0), Pointer, ) !dbg package_id=2 span=[106-110]
                Return !dbg package_id=2 span=[106-110]
            Block 2:Block:
                Jump(1) !dbg package_id=2 span=[232-304]"#]],
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[219-262]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[278-286]
                Variable(1, Boolean) = Store Variable(0, Boolean) !dbg package_id=2 span=[278-286]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[278-286]
            Block 1:Block:
                Call id(5), args( Integer(0), Pointer, ) !dbg package_id=2 span=[162-166]
                Return !dbg package_id=2 span=[162-166]
            Block 2:Block:
                Call id(3), args( Qubit(0), ) !dbg package_id=2 span=[301-304]
                Jump(1) !dbg package_id=2 span=[287-318]
            Block 3:Block:
                Call id(4), args( Qubit(0), ) !dbg package_id=2 span=[364-367]
                Jump(1) !dbg package_id=2 span=[319-395]"#]],
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[219-262]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[278-286]
                Variable(1, Boolean) = Store Variable(0, Boolean) !dbg package_id=2 span=[278-286]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[278-286]
            Block 1:Block:
                Call id(4), args( Integer(0), Pointer, ) !dbg package_id=2 span=[162-166]
                Return !dbg package_id=2 span=[162-166]
            Block 2:Block:
                Call id(3), args( Qubit(0), ) !dbg package_id=2 span=[301-304]
                Jump(1) !dbg package_id=2 span=[287-318]
            Block 3:Block:
                Jump(1) !dbg package_id=2 span=[319-396]"#]],
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[195-238]
                Call id(1), args( Qubit(1), Result(1), ) !dbg package_id=2 span=[261-304]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[321-331]
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false) !dbg package_id=2 span=[321-331]
                Branch Variable(1, Boolean), 2, 1 !dbg package_id=2 span=[321-331]
            Block 1:Block:
                Call id(4), args( Integer(0), Pointer, ) !dbg package_id=2 span=[106-110]
                Return !dbg package_id=2 span=[106-110]
            Block 2:Block:
                Variable(2, Boolean) = Call id(2), args( Result(1), ) !dbg package_id=2 span=[349-358]
                Variable(3, Boolean) = Store Variable(2, Boolean) !dbg package_id=2 span=[349-358]
                Branch Variable(3, Boolean), 4, 3 !dbg package_id=2 span=[349-358]
            Block 3:Block:
                Jump(1) !dbg package_id=2 span=[332-409]
            Block 4:Block:
                Call id(3), args( Qubit(2), ) !dbg package_id=2 span=[377-380]
                Jump(3) !dbg package_id=2 span=[359-399]"#]],
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[363-406]
                Call id(1), args( Qubit(1), Result(1), ) !dbg package_id=2 span=[429-472]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[489-499]
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false) !dbg package_id=2 span=[489-499]
                Branch Variable(1, Boolean), 2, 6 !dbg package_id=2 span=[489-499]
            Block 1:Block:
                Call id(7), args( Integer(0), Pointer, ) !dbg package_id=2 span=[274-278]
                Return !dbg package_id=2 span=[274-278]
            Block 2:Block:
                Variable(2, Boolean) = Call id(2), args( Result(1), ) !dbg package_id=2 span=[517-527]
                Variable(3, Boolean) = Icmp Eq, Variable(2, Boolean), Bool(false) !dbg package_id=2 span=[517-527]
                Branch Variable(3, Boolean), 4, 5 !dbg package_id=2 span=[517-527]
            Block 3:Block:
                Jump(1) !dbg package_id=2 span=[500-624]
            Block 4:Block:
                Call id(3), args( Qubit(2), ) !dbg package_id=2 span=[546-549]
                Jump(3) !dbg package_id=2 span=[528-568]
            Block 5:Block:
                Call id(4), args( Qubit(2), ) !dbg package_id=2 span=[592-595]
                Jump(3) !dbg package_id=2 span=[569-614]
            Block 6:Block:
                Variable(4, Boolean) = Call id(2), args( Result(1), ) !dbg package_id=2 span=[647-656]
                Variable(5, Boolean) = Store Variable(4, Boolean) !dbg package_id=2 span=[647-656]
                Branch Variable(5, Boolean), 8, 9 !dbg package_id=2 span=[647-656]
            Block 7:Block:
                Jump(1) !dbg package_id=2 span=[625-752]
            Block 8:Block:
                Call id(5), args( Qubit(2), ) !dbg package_id=2 span=[674-677]
                Jump(7) !dbg package_id=2 span=[656-696]
            Block 9:Block:
                Call id(6), args( Qubit(2), ) !dbg package_id=2 span=[720-723]
                Jump(7) !dbg package_id=2 span=[697-742]"#]],
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[219-262]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[278-287]
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false) !dbg package_id=2 span=[278-287]
                Branch Variable(1, Boolean), 2, 1 !dbg package_id=2 span=[278-287]
            Block 1:Block:
                Call id(4), args( Qubit(0), ) !dbg package_id=2 span=[328-331]
                Call id(5), args( Integer(0), Pointer, ) !dbg package_id=2 span=[162-166]
                Return !dbg package_id=2 span=[162-166]
            Block 2:Block:
                Call id(3), args( Qubit(0), ) !dbg package_id=2 span=[302-305]
                Jump(1) !dbg package_id=2 span=[288-319]"#]],
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[275-318]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[334-342]
                Variable(1, Boolean) = Store Variable(0, Boolean) !dbg package_id=2 span=[334-342]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[334-342]
            Block 1:Block:
                Call id(5), args( Qubit(0), ) !dbg package_id=2 span=[420-423]
                Call id(6), args( Integer(0), Pointer, ) !dbg package_id=2 span=[218-222]
                Return !dbg package_id=2 span=[218-222]
            Block 2:Block:
                Call id(3), args( Qubit(0), ) !dbg package_id=2 span=[357-360]
                Jump(1) !dbg package_id=2 span=[343-374]
            Block 3:Block:
                Call id(4), args( Qubit(0), ) !dbg package_id=2 span=[394-397]
                Jump(1) !dbg package_id=2 span=[375-411]"#]],
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

#[test]
fn if_expression_with_classical_operand_from_hybrid_results_array_comparing_to_literal_zero() {
    let program = get_rir_program(indoc! {r#"
        @EntryPoint()
        operation Main() : Result[] {
            mutable measurements = [Zero, Zero];
            use (a, b) = (Qubit(), Qubit());
            set measurements w/= 0 <- MResetZ(a);
            // Use a static result in the condition.
            if measurements[1] == Zero {
                X(b);
            }
            set measurements w/= 1 <- MResetZ(b);
            measurements
        }
        "#
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
    let x_callable_id = CallableId(2);
    assert_callable(
        &program,
        x_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__x__body
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let record_array_callable_id = CallableId(3);
    assert_callable(
        &program,
        record_array_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__array_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    let record_result_callable_id = CallableId(4);
    assert_callable(
        &program,
        record_result_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__result_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Result
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Call id(2), args( Qubit(1), ) !dbg package_id=1 span=[132972-132995]
                Call id(1), args( Qubit(1), Result(1), ) !dbg package_id=1 span=[182985-183014]
                Call id(3), args( Integer(2), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(4), args( Result(0), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(4), args( Result(1), Pointer, ) !dbg package_id=2 span=[25-29]
                Return !dbg package_id=2 span=[25-29]"#]],
    );
}

#[test]
fn if_expression_with_classical_operand_from_hybrid_results_array_comparing_to_literal_one() {
    let program = get_rir_program(indoc! {r#"
        @EntryPoint()
        operation Main() : Result[] {
            mutable measurements = [Zero, Zero];
            use (a, b) = (Qubit(), Qubit());
            set measurements w/= 0 <- MResetZ(a);
            // Use a static result in the condition.
            if measurements[1] != One {
                X(b);
            }
            set measurements w/= 1 <- MResetZ(b);
            measurements
        }
        "#
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
    let x_callable_id = CallableId(2);
    assert_callable(
        &program,
        x_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__x__body
            call_type: Regular
            input_type:
                [0]: Qubit
            output_type: <VOID>
            body: <NONE>"#]],
    );
    let record_array_callable_id = CallableId(3);
    assert_callable(
        &program,
        record_array_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__array_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    let record_result_callable_id = CallableId(4);
    assert_callable(
        &program,
        record_result_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__result_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Result
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Call id(2), args( Qubit(1), ) !dbg package_id=1 span=[132972-132995]
                Call id(1), args( Qubit(1), Result(1), ) !dbg package_id=1 span=[182985-183014]
                Call id(3), args( Integer(2), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(4), args( Result(0), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(4), args( Result(1), Pointer, ) !dbg package_id=2 span=[25-29]
                Return !dbg package_id=2 span=[25-29]"#]],
    );
}

#[test]
fn if_expression_with_dynamic_operand_from_hybrid_results_array() {
    let program = get_rir_program(indoc! {r#"
        @EntryPoint()
        operation Main() : Result[] {
            mutable measurements = [Zero, Zero];
            use (a, b) = (Qubit(), Qubit());
            set measurements w/= 0 <- MResetZ(a);
            // Use a dynamic result in the condition.
            if measurements[0] == Zero {
                X(b);
            }
            set measurements w/= 1 <- MResetZ(b);
            measurements
        }
        "#
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
    let x_callable_id = CallableId(3);
    assert_callable(
        &program,
        x_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__qis__x__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
    );
    let record_array_callable_id = CallableId(4);
    assert_callable(
        &program,
        record_array_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__array_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    let record_result_callable_id = CallableId(5);
    assert_callable(
        &program,
        record_result_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__result_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Result
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[218-241]
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false) !dbg package_id=2 span=[218-241]
                Branch Variable(1, Boolean), 2, 1 !dbg package_id=2 span=[218-241]
            Block 1:Block:
                Call id(1), args( Qubit(1), Result(1), ) !dbg package_id=1 span=[182985-183014]
                Call id(4), args( Integer(2), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(5), args( Result(0), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(5), args( Result(1), Pointer, ) !dbg package_id=2 span=[25-29]
                Return !dbg package_id=2 span=[25-29]
            Block 2:Block:
                Call id(3), args( Qubit(1), ) !dbg package_id=1 span=[132972-132995]
                Jump(1) !dbg package_id=2 span=[242-263]"#]],
    );
}

#[test]
fn if_expression_with_classical_operand_from_hybrid_booleans_array() {
    let program = get_rir_program(indoc! {r#"
        @EntryPoint()
        operation Main() : Bool[] {
            mutable flags = [false, false];
            use (a, b) = (Qubit(), Qubit());
            set flags w/= 0 <- MResetZ(a) == One;
            // Use a static Boolean in the condition.
            if flags[1] == false {
                X(b);
            }
            set flags w/= 1 <- MResetZ(b) == One;
            flags
        }
        "#
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
    let x_callable_id = CallableId(3);
    assert_callable(
        &program,
        x_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__qis__x__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
    );
    let record_array_callable_id = CallableId(4);
    assert_callable(
        &program,
        record_array_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__array_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    let record_bool_callable_id = CallableId(5);
    assert_callable(
        &program,
        record_bool_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__bool_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Boolean
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[139-156]
                Variable(1, Boolean) = Store Variable(0, Boolean) !dbg package_id=2 span=[139-156]
                Call id(3), args( Qubit(1), ) !dbg package_id=1 span=[132972-132995]
                Call id(1), args( Qubit(1), Result(1), ) !dbg package_id=1 span=[182985-183014]
                Variable(2, Boolean) = Call id(2), args( Result(1), ) !dbg package_id=2 span=[274-291]
                Variable(3, Boolean) = Store Variable(2, Boolean) !dbg package_id=2 span=[274-291]
                Call id(4), args( Integer(2), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(5), args( Variable(1, Boolean), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(5), args( Variable(3, Boolean), Pointer, ) !dbg package_id=2 span=[25-29]
                Return !dbg package_id=2 span=[25-29]"#]],
    );
}

#[test]
fn if_expression_with_dynamic_operand_from_hybrid_booleans_array() {
    let program = get_rir_program(indoc! {r#"
        @EntryPoint()
        operation Main() : Bool[] {
            mutable flags = [false, false];
            use (a, b) = (Qubit(), Qubit());
            set flags w/= 0 <- MResetZ(a) == One;
            // Use a dynamic Boolean in the condition.
            if flags[0] {
                X(b);
            }
            set flags w/= 1 <- MResetZ(b) == One;
            flags
        }
        "#
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
    let x_callable_id = CallableId(3);
    assert_callable(
        &program,
        x_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__qis__x__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
    );
    let record_array_callable_id = CallableId(4);
    assert_callable(
        &program,
        record_array_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__array_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    let record_bool_callable_id = CallableId(5);
    assert_callable(
        &program,
        record_bool_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__bool_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Boolean
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[139-156]
                Variable(1, Boolean) = Store Variable(0, Boolean) !dbg package_id=2 span=[139-156]
                Branch Variable(1, Boolean), 2, 1 !dbg package_id=2 span=[212-220]
            Block 1:Block:
                Call id(1), args( Qubit(1), Result(1), ) !dbg package_id=1 span=[182985-183014]
                Variable(2, Boolean) = Call id(2), args( Result(1), ) !dbg package_id=2 span=[266-283]
                Variable(3, Boolean) = Store Variable(2, Boolean) !dbg package_id=2 span=[266-283]
                Call id(4), args( Integer(2), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(5), args( Variable(1, Boolean), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(5), args( Variable(3, Boolean), Pointer, ) !dbg package_id=2 span=[25-29]
                Return !dbg package_id=2 span=[25-29]
            Block 2:Block:
                Call id(3), args( Qubit(1), ) !dbg package_id=1 span=[132972-132995]
                Jump(1) !dbg package_id=2 span=[221-242]"#]],
    );
}

#[test]
fn if_expression_with_classical_operand_from_hybrid_integers_array() {
    let program = get_rir_program(indoc! {r#"
        @EntryPoint()
        operation Main() : Int[] {
            mutable integers = [0, 0];
            use (a, b) = (Qubit(), Qubit());
            set integers w/= 0 <- MResetZ(a) == Zero ? 0 | 1;
            // Use a static integer in the condition.
            if integers[1] == 0 {
                X(b);
            }
            set integers w/= 1 <- MResetZ(b) == Zero ? 0 | 1;
            integers
        }
        "#
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
    let x_callable_id = CallableId(3);
    assert_callable(
        &program,
        x_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__qis__x__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
    );
    let record_array_callable_id = CallableId(4);
    assert_callable(
        &program,
        record_array_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__array_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    let record_int_callable_id = CallableId(5);
    assert_callable(
        &program,
        record_int_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__int_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[136-154]
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false) !dbg package_id=2 span=[136-154]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[136-154]
            Block 1:Block:
                Call id(3), args( Qubit(1), ) !dbg package_id=1 span=[132972-132995]
                Call id(1), args( Qubit(1), Result(1), ) !dbg package_id=1 span=[182985-183014]
                Variable(3, Boolean) = Call id(2), args( Result(1), ) !dbg package_id=2 span=[282-300]
                Variable(4, Boolean) = Icmp Eq, Variable(3, Boolean), Bool(false) !dbg package_id=2 span=[282-300]
                Branch Variable(4, Boolean), 5, 6 !dbg package_id=2 span=[282-300]
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0) !dbg package_id=2 span=[157-158]
                Jump(1) !dbg package_id=2 span=[157-158]
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1) !dbg package_id=2 span=[161-162]
                Jump(1) !dbg package_id=2 span=[161-162]
            Block 4:Block:
                Call id(4), args( Integer(2), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(5), args( Variable(2, Integer), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(5), args( Variable(5, Integer), Pointer, ) !dbg package_id=2 span=[25-29]
                Return !dbg package_id=2 span=[25-29]
            Block 5:Block:
                Variable(5, Integer) = Store Integer(0) !dbg package_id=2 span=[303-304]
                Jump(4) !dbg package_id=2 span=[303-304]
            Block 6:Block:
                Variable(5, Integer) = Store Integer(1) !dbg package_id=2 span=[307-308]
                Jump(4) !dbg package_id=2 span=[307-308]"#]],
    );
}

#[test]
fn if_expression_with_dynamic_operand_from_hybrid_integers_array() {
    let program = get_rir_program(indoc! {r#"
        @EntryPoint()
        operation Main() : Int[] {
            mutable integers = [0, 0];
            use (a, b) = (Qubit(), Qubit());
            set integers w/= 0 <- MResetZ(a) == Zero ? 0 | 1;
            // Use a dynamic integer in the condition.
            if integers[0] == 0 {
                X(b);
            }
            set integers w/= 1 <- MResetZ(b) == Zero ? 0 | 1;
            integers
        }
        "#
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
    let x_callable_id = CallableId(3);
    assert_callable(
        &program,
        x_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__qis__x__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
    );
    let record_array_callable_id = CallableId(4);
    assert_callable(
        &program,
        record_array_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__array_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    let record_int_callable_id = CallableId(5);
    assert_callable(
        &program,
        record_int_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__int_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[136-154]
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false) !dbg package_id=2 span=[136-154]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[136-154]
            Block 1:Block:
                Variable(3, Boolean) = Icmp Eq, Variable(2, Integer), Integer(0) !dbg package_id=2 span=[218-234]
                Branch Variable(3, Boolean), 5, 4 !dbg package_id=2 span=[218-234]
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0) !dbg package_id=2 span=[157-158]
                Jump(1) !dbg package_id=2 span=[157-158]
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1) !dbg package_id=2 span=[161-162]
                Jump(1) !dbg package_id=2 span=[161-162]
            Block 4:Block:
                Call id(1), args( Qubit(1), Result(1), ) !dbg package_id=1 span=[182985-183014]
                Variable(4, Boolean) = Call id(2), args( Result(1), ) !dbg package_id=2 span=[283-301]
                Variable(5, Boolean) = Icmp Eq, Variable(4, Boolean), Bool(false) !dbg package_id=2 span=[283-301]
                Branch Variable(5, Boolean), 7, 8 !dbg package_id=2 span=[283-301]
            Block 5:Block:
                Call id(3), args( Qubit(1), ) !dbg package_id=1 span=[132972-132995]
                Jump(4) !dbg package_id=2 span=[235-256]
            Block 6:Block:
                Call id(4), args( Integer(2), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(5), args( Variable(2, Integer), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(5), args( Variable(6, Integer), Pointer, ) !dbg package_id=2 span=[25-29]
                Return !dbg package_id=2 span=[25-29]
            Block 7:Block:
                Variable(6, Integer) = Store Integer(0) !dbg package_id=2 span=[304-305]
                Jump(6) !dbg package_id=2 span=[304-305]
            Block 8:Block:
                Variable(6, Integer) = Store Integer(1) !dbg package_id=2 span=[308-309]
                Jump(6) !dbg package_id=2 span=[308-309]"#]],
    );
}

#[test]
fn if_expression_with_classical_operand_from_hybrid_doubles_array() {
    let program = get_rir_program(indoc! {r#"
        @EntryPoint()
        operation Main() : Double[] {
            mutable doubles = [0.0, 0.0];
            use (a, b) = (Qubit(), Qubit());
            set doubles w/= 0 <- MResetZ(a) == Zero ? 0.1 | 1.1;
            // Use a static double in the condition.
            if doubles[1] == 0.0 {
                X(b);
            }
            set doubles w/= 1 <- MResetZ(b) == Zero ? 0.1 | 1.1;
            doubles
        }
        "#
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
    let x_callable_id = CallableId(3);
    assert_callable(
        &program,
        x_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__qis__x__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
    );
    let record_array_callable_id = CallableId(4);
    assert_callable(
        &program,
        record_array_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__array_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    let record_int_callable_id = CallableId(5);
    assert_callable(
        &program,
        record_int_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__double_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Double
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[141-159]
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false) !dbg package_id=2 span=[141-159]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[141-159]
            Block 1:Block:
                Call id(3), args( Qubit(1), ) !dbg package_id=1 span=[132972-132995]
                Call id(1), args( Qubit(1), Result(1), ) !dbg package_id=1 span=[182985-183014]
                Variable(3, Boolean) = Call id(2), args( Result(1), ) !dbg package_id=2 span=[290-308]
                Variable(4, Boolean) = Icmp Eq, Variable(3, Boolean), Bool(false) !dbg package_id=2 span=[290-308]
                Branch Variable(4, Boolean), 5, 6 !dbg package_id=2 span=[290-308]
            Block 2:Block:
                Variable(2, Double) = Store Double(0.1) !dbg package_id=2 span=[162-165]
                Jump(1) !dbg package_id=2 span=[162-165]
            Block 3:Block:
                Variable(2, Double) = Store Double(1.1) !dbg package_id=2 span=[168-171]
                Jump(1) !dbg package_id=2 span=[168-171]
            Block 4:Block:
                Call id(4), args( Integer(2), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(5), args( Variable(2, Double), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(5), args( Variable(5, Double), Pointer, ) !dbg package_id=2 span=[25-29]
                Return !dbg package_id=2 span=[25-29]
            Block 5:Block:
                Variable(5, Double) = Store Double(0.1) !dbg package_id=2 span=[311-314]
                Jump(4) !dbg package_id=2 span=[311-314]
            Block 6:Block:
                Variable(5, Double) = Store Double(1.1) !dbg package_id=2 span=[317-320]
                Jump(4) !dbg package_id=2 span=[317-320]"#]],
    );
}

#[test]
fn if_expression_with_dynamic_operand_from_hybrid_doubles_array() {
    let program = get_rir_program(indoc! {r#"
        @EntryPoint()
        operation Main() : Double[] {
            mutable doubles = [0.0, 0.0];
            use (a, b) = (Qubit(), Qubit());
            set doubles w/= 0 <- MResetZ(a) == Zero ? 0.1 | 1.1;
            // Use a dynamic double in the condition.
            if doubles[0] == 0.0 {
                X(b);
            }
            set doubles w/= 1 <- MResetZ(b) == Zero ? 0.1 | 1.1;
            doubles
        }
        "#
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
    let x_callable_id = CallableId(3);
    assert_callable(
        &program,
        x_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__qis__x__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
    );
    let record_array_callable_id = CallableId(4);
    assert_callable(
        &program,
        record_array_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__array_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Integer
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    let record_int_callable_id = CallableId(5);
    assert_callable(
        &program,
        record_int_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__double_record_output
                call_type: OutputRecording
                input_type:
                    [0]: Double
                    [1]: Pointer
                output_type: <VOID>
                body: <NONE>"#]],
    );
    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[141-159]
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false) !dbg package_id=2 span=[141-159]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[141-159]
            Block 1:Block:
                Variable(3, Boolean) = Fcmp Oeq, Variable(2, Double), Double(0) !dbg package_id=2 span=[226-243]
                Branch Variable(3, Boolean), 5, 4 !dbg package_id=2 span=[226-243]
            Block 2:Block:
                Variable(2, Double) = Store Double(0.1) !dbg package_id=2 span=[162-165]
                Jump(1) !dbg package_id=2 span=[162-165]
            Block 3:Block:
                Variable(2, Double) = Store Double(1.1) !dbg package_id=2 span=[168-171]
                Jump(1) !dbg package_id=2 span=[168-171]
            Block 4:Block:
                Call id(1), args( Qubit(1), Result(1), ) !dbg package_id=1 span=[182985-183014]
                Variable(4, Boolean) = Call id(2), args( Result(1), ) !dbg package_id=2 span=[291-309]
                Variable(5, Boolean) = Icmp Eq, Variable(4, Boolean), Bool(false) !dbg package_id=2 span=[291-309]
                Branch Variable(5, Boolean), 7, 8 !dbg package_id=2 span=[291-309]
            Block 5:Block:
                Call id(3), args( Qubit(1), ) !dbg package_id=1 span=[132972-132995]
                Jump(4) !dbg package_id=2 span=[244-265]
            Block 6:Block:
                Call id(4), args( Integer(2), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(5), args( Variable(2, Double), Pointer, ) !dbg package_id=2 span=[25-29]
                Call id(5), args( Variable(6, Double), Pointer, ) !dbg package_id=2 span=[25-29]
                Return !dbg package_id=2 span=[25-29]
            Block 7:Block:
                Variable(6, Double) = Store Double(0.1) !dbg package_id=2 span=[312-315]
                Jump(6) !dbg package_id=2 span=[312-315]
            Block 8:Block:
                Variable(6, Double) = Store Double(1.1) !dbg package_id=2 span=[318-321]
                Jump(6) !dbg package_id=2 span=[318-321]"#]],
    );
}

#[test]
fn if_expression_with_implicit_return_in_callable_supported() {
    let program = get_rir_program(indoc! {r#"
        function Choose(r : Result) : Int {
            if r == One {
                1
            } else {
                0
            }
        }
        @EntryPoint()
        operation Main() : Int {
            use q = Qubit();
            Choose(MResetZ(q))
        }
        "#
    });

    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[44-52]
                Variable(1, Boolean) = Store Variable(0, Boolean) !dbg package_id=2 span=[44-52]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[44-52]
            Block 1:Block:
                Variable(3, Integer) = Store Variable(2, Integer) !dbg package_id=2 span=[160-178]
                Call id(3), args( Variable(3, Integer), Pointer, ) !dbg package_id=2 span=[120-124]
                Return !dbg package_id=2 span=[120-124]
            Block 2:Block:
                Variable(2, Integer) = Store Integer(1) !dbg package_id=2 span=[53-70]
                Jump(1) !dbg package_id=2 span=[53-70]
            Block 3:Block:
                Variable(2, Integer) = Store Integer(0) !dbg package_id=2 span=[71-93]
                Jump(1) !dbg package_id=2 span=[71-93]"#]],
    );
}

#[test]
fn if_expression_with_explicit_return_in_callable_fails() {
    let error = get_partial_evaluation_error(indoc! {r#"
        function Choose(r : Result) : Int {
            if r == One {
                return 1;
            } else {
                return 0;
            }
        }
        @EntryPoint()
        operation Main() : Int {
            use q = Qubit();
            Choose(MResetZ(q))
        }
        "#
    });

    assert_error(
        &error,
        &expect![[
            r#"Unimplemented("early return", PackageSpan { package: PackageId(2), span: Span { lo: 53, hi: 78 } })"#
        ]],
    );
}

#[test]
fn if_expression_with_explicit_return_in_one_branch_and_fallthrough_else_in_callable_fails() {
    let error = get_partial_evaluation_error(indoc! {r#"
        function Choose(r : Result) : Int {
            if r == One {
                return 1;
            }

            return 3;
        }
        @EntryPoint()
        operation Main() : Int {
            use q = Qubit();
            Choose(MResetZ(q))
        }
        "#
    });

    assert_error(
        &error,
        &expect![[
            r#"Unimplemented("early return", PackageSpan { package: PackageId(2), span: Span { lo: 53, hi: 78 } })"#
        ]],
    );
}
