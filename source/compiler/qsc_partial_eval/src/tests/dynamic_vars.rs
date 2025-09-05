// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes, clippy::similar_names)]

use super::{assert_blocks, assert_callable, get_rir_program};
use expect_test::expect;
use indoc::indoc;
use qsc_rir::rir::CallableId;

#[test]
fn dynamic_int_from_if_expression_with_single_measurement_comparison_and_classical_blocks() {
    let program = get_rir_program(indoc! {
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[107-150]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[174-183]
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false) !dbg package_id=2 span=[174-183]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[174-183]
            Block 1:Block:
                Variable(3, Integer) = Store Variable(2, Integer) !dbg package_id=2 span=[167-168]
                Call id(3), args( Integer(0), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]
            Block 2:Block:
                Variable(2, Integer) = Store Integer(0) !dbg package_id=2 span=[184-189]
                Jump(1) !dbg package_id=2 span=[184-189]
            Block 3:Block:
                Variable(2, Integer) = Store Integer(1) !dbg package_id=2 span=[190-200]
                Jump(1) !dbg package_id=2 span=[190-200]"#]],
    );
}

#[test]
fn dynamic_int_from_if_expression_with_single_measurement_comparison_and_non_classical_blocks() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation OpA(q : Qubit) : Unit { body intrinsic; }
            operation OpB(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1) = (Qubit(), Qubit());
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q0);
                let b = if r == Zero {
                    OpA(q1);
                    0
                } else {
                    OpB(q1);
                    1
                };
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
            name: OpA
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
                name: OpB
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[237-280]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[305-314]
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false) !dbg package_id=2 span=[305-314]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[305-314]
            Block 1:Block:
                Variable(3, Integer) = Store Variable(2, Integer) !dbg package_id=2 span=[298-299]
                Call id(5), args( Integer(0), Pointer, ) !dbg package_id=2 span=[162-166]
                Return !dbg package_id=2 span=[162-166]
            Block 2:Block:
                Call id(3), args( Qubit(1), ) !dbg package_id=2 span=[329-332]
                Variable(2, Integer) = Store Integer(0) !dbg package_id=2 span=[315-361]
                Jump(1) !dbg package_id=2 span=[315-361]
            Block 3:Block:
                Call id(4), args( Qubit(1), ) !dbg package_id=2 span=[381-384]
                Variable(2, Integer) = Store Integer(1) !dbg package_id=2 span=[362-413]
                Jump(1) !dbg package_id=2 span=[362-413]"#]],
    );
}

#[test]
fn dynamic_var_across_if_else_static_in_both_branches_constant_folded() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                mutable value = 0;
                use q = Qubit();
                let cond = MResetZ(q);
                if cond == Zero {
                    value -= 1;
                } else {
                    value += 1;
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
                Variable(0, Integer) = Store Integer(0) !dbg package_id=2 span=[82-87]
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Variable(1, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[160-172]
                Variable(2, Boolean) = Icmp Eq, Variable(1, Boolean), Bool(false) !dbg package_id=2 span=[160-172]
                Branch Variable(2, Boolean), 2, 3 !dbg package_id=2 span=[160-172]
            Block 1:Block:
                Call id(3), args( Integer(0), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]
            Block 2:Block:
                Variable(0, Integer) = Store Integer(-1) !dbg package_id=2 span=[187-192]
                Jump(1) !dbg package_id=2 span=[173-208]
            Block 3:Block:
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[228-233]
                Jump(1) !dbg package_id=2 span=[209-249]"#]],
    );
}

#[test]
fn dynamic_var_across_if_else_in_loop_constant_folded_in_first_iteration() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                mutable value = 0;
                use q = Qubit();
                let cond = MResetZ(q);
                for _ in 0..1 {
                    if cond == Zero {
                        value -= 1;
                    } else {
                        value += 1;
                    }
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
                Variable(0, Integer) = Store Integer(0) !dbg package_id=2 span=[82-87]
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Variable(1, Integer) = Store Integer(0) !dbg package_id=2 span=[166-170]
                Variable(2, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[188-200]
                Variable(3, Boolean) = Icmp Eq, Variable(2, Boolean), Bool(false) !dbg package_id=2 span=[188-200]
                Branch Variable(3, Boolean), 2, 3 !dbg package_id=2 span=[188-200]
            Block 1:Block:
                Variable(1, Integer) = Store Integer(1) !dbg package_id=2 span=[166-170]
                Variable(4, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[188-200]
                Variable(5, Boolean) = Icmp Eq, Variable(4, Boolean), Bool(false) !dbg package_id=2 span=[188-200]
                Branch Variable(5, Boolean), 5, 6 !dbg package_id=2 span=[188-200]
            Block 2:Block:
                Variable(0, Integer) = Store Integer(-1) !dbg package_id=2 span=[219-224]
                Jump(1) !dbg package_id=2 span=[201-244]
            Block 3:Block:
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[268-273]
                Jump(1) !dbg package_id=2 span=[245-293]
            Block 4:Block:
                Variable(1, Integer) = Store Integer(2) !dbg package_id=2 span=[166-170]
                Call id(3), args( Integer(0), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]
            Block 5:Block:
                Variable(6, Integer) = Sub Variable(0, Integer), Integer(1) !dbg package_id=2 span=[219-229]
                Variable(0, Integer) = Store Variable(6, Integer) !dbg package_id=2 span=[219-224]
                Jump(4) !dbg package_id=2 span=[201-244]
            Block 6:Block:
                Variable(7, Integer) = Add Variable(0, Integer), Integer(1) !dbg package_id=2 span=[268-278]
                Variable(0, Integer) = Store Variable(7, Integer) !dbg package_id=2 span=[268-273]
                Jump(4) !dbg package_id=2 span=[245-293]"#]],
    );
}

#[test]
fn dynamic_var_within_if_else_in_loop_constant_folded_in_every_iteration() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let cond = MResetZ(q);
                for _ in 0..1 {
                    mutable value = 0;
                    if cond == Zero {
                        value -= 1;
                    } else {
                        value += 1;
                    }
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Variable(0, Integer) = Store Integer(0) !dbg package_id=2 span=[139-143]
                Variable(1, Integer) = Store Integer(0) !dbg package_id=2 span=[166-171]
                Variable(2, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[192-204]
                Variable(3, Boolean) = Icmp Eq, Variable(2, Boolean), Bool(false) !dbg package_id=2 span=[192-204]
                Branch Variable(3, Boolean), 2, 3 !dbg package_id=2 span=[192-204]
            Block 1:Block:
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[139-143]
                Variable(4, Integer) = Store Integer(0) !dbg package_id=2 span=[166-171]
                Variable(5, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[192-204]
                Variable(6, Boolean) = Icmp Eq, Variable(5, Boolean), Bool(false) !dbg package_id=2 span=[192-204]
                Branch Variable(6, Boolean), 5, 6 !dbg package_id=2 span=[192-204]
            Block 2:Block:
                Variable(1, Integer) = Store Integer(-1) !dbg package_id=2 span=[223-228]
                Jump(1) !dbg package_id=2 span=[205-248]
            Block 3:Block:
                Variable(1, Integer) = Store Integer(1) !dbg package_id=2 span=[272-277]
                Jump(1) !dbg package_id=2 span=[249-297]
            Block 4:Block:
                Variable(0, Integer) = Store Integer(2) !dbg package_id=2 span=[139-143]
                Call id(3), args( Integer(0), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]
            Block 5:Block:
                Variable(4, Integer) = Store Integer(-1) !dbg package_id=2 span=[223-228]
                Jump(4) !dbg package_id=2 span=[205-248]
            Block 6:Block:
                Variable(4, Integer) = Store Integer(1) !dbg package_id=2 span=[272-277]
                Jump(4) !dbg package_id=2 span=[249-297]"#]],
    );
}

#[test]
fn dynamic_var_updated_twice_in_same_branch_constant_folded() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                mutable value = 0;
                use q = Qubit();
                let cond = MResetZ(q);
                if cond == Zero {
                    value -= 1;
                    value += 3;
                } else {
                    value += 1;
                    value -= 3;
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
                Variable(0, Integer) = Store Integer(0) !dbg package_id=2 span=[82-87]
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Variable(1, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[160-172]
                Variable(2, Boolean) = Icmp Eq, Variable(1, Boolean), Bool(false) !dbg package_id=2 span=[160-172]
                Branch Variable(2, Boolean), 2, 3 !dbg package_id=2 span=[160-172]
            Block 1:Block:
                Call id(3), args( Integer(0), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]
            Block 2:Block:
                Variable(0, Integer) = Store Integer(-1) !dbg package_id=2 span=[187-192]
                Variable(0, Integer) = Store Integer(2) !dbg package_id=2 span=[211-216]
                Jump(1) !dbg package_id=2 span=[173-232]
            Block 3:Block:
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[252-257]
                Variable(0, Integer) = Store Integer(-2) !dbg package_id=2 span=[276-281]
                Jump(1) !dbg package_id=2 span=[233-297]"#]],
    );
}

#[test]
fn dynamic_var_updated_to_same_value_in_different_branches_constant_folded_after_if() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                mutable value = 0;
                use q = Qubit();
                let cond = MResetZ(q);
                if cond == Zero {
                    value -= 1;
                    value += 2;
                } else {
                    value += 1;
                    value /= 1;
                }
                value += 1;
            }
        }
        "#,
    });

    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Variable(0, Integer) = Store Integer(0) !dbg package_id=2 span=[82-87]
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Variable(1, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[160-172]
                Variable(2, Boolean) = Icmp Eq, Variable(1, Boolean), Bool(false) !dbg package_id=2 span=[160-172]
                Branch Variable(2, Boolean), 2, 3 !dbg package_id=2 span=[160-172]
            Block 1:Block:
                Variable(0, Integer) = Store Integer(2) !dbg package_id=2 span=[306-311]
                Call id(3), args( Integer(0), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]
            Block 2:Block:
                Variable(0, Integer) = Store Integer(-1) !dbg package_id=2 span=[187-192]
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[211-216]
                Jump(1) !dbg package_id=2 span=[173-232]
            Block 3:Block:
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[252-257]
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[276-281]
                Jump(1) !dbg package_id=2 span=[233-297]"#]],
    );
}

#[test]
fn dynamic_var_updated_in_nested_branches_constant_folded_when_value_matches_across_all_branches() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                mutable value = 0;
                use q = Qubit();
                let cond = MResetZ(q);
                if cond == Zero {
                    value -= 1;
                    if cond == Zero {
                        value += 2;
                    } else {
                        value -= 2;
                        value /= 3;
                        value *= -1;
                }
                } else {
                    value += 1;
                    if cond == Zero {
                        value += 2;
                        value /= 3;
                    } else {
                        value -= 2;
                        value *= -1;
                    }
                }
                return value;
            }
        }
        "#,
    });

    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Variable(0, Integer) = Store Integer(0) !dbg package_id=2 span=[81-86]
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Variable(1, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[159-171]
                Variable(2, Boolean) = Icmp Eq, Variable(1, Boolean), Bool(false) !dbg package_id=2 span=[159-171]
                Branch Variable(2, Boolean), 2, 6 !dbg package_id=2 span=[159-171]
            Block 1:Block:
                Call id(3), args( Integer(1), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]
            Block 2:Block:
                Variable(0, Integer) = Store Integer(-1) !dbg package_id=2 span=[186-191]
                Variable(3, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[213-225]
                Variable(4, Boolean) = Icmp Eq, Variable(3, Boolean), Bool(false) !dbg package_id=2 span=[213-225]
                Branch Variable(4, Boolean), 4, 5 !dbg package_id=2 span=[213-225]
            Block 3:Block:
                Jump(1) !dbg package_id=2 span=[172-381]
            Block 4:Block:
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[244-249]
                Jump(3) !dbg package_id=2 span=[226-269]
            Block 5:Block:
                Variable(0, Integer) = Store Integer(-3) !dbg package_id=2 span=[293-298]
                Variable(0, Integer) = Store Integer(-1) !dbg package_id=2 span=[321-326]
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[349-354]
                Jump(3) !dbg package_id=2 span=[270-371]
            Block 6:Block:
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[401-406]
                Variable(5, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[428-440]
                Variable(6, Boolean) = Icmp Eq, Variable(5, Boolean), Bool(false) !dbg package_id=2 span=[428-440]
                Branch Variable(6, Boolean), 8, 9 !dbg package_id=2 span=[428-440]
            Block 7:Block:
                Jump(1) !dbg package_id=2 span=[382-600]
            Block 8:Block:
                Variable(0, Integer) = Store Integer(3) !dbg package_id=2 span=[459-464]
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[487-492]
                Jump(7) !dbg package_id=2 span=[441-512]
            Block 9:Block:
                Variable(0, Integer) = Store Integer(-1) !dbg package_id=2 span=[536-541]
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[564-569]
                Jump(7) !dbg package_id=2 span=[513-590]"#]],
    );
}

#[test]
fn dynamic_var_set_to_static_after_dynamism_still_constant_folded() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                mutable value = 0;
                use q = Qubit();
                let cond = MResetZ(q);
                if cond == Zero {
                    value -= 1;
                }
                value *= 2;
                value = 3;
                value += 1;
            }
        }
        "#,
    });

    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Variable(0, Integer) = Store Integer(0) !dbg package_id=2 span=[82-87]
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Variable(1, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[160-172]
                Variable(2, Boolean) = Icmp Eq, Variable(1, Boolean), Bool(false) !dbg package_id=2 span=[160-172]
                Branch Variable(2, Boolean), 2, 1 !dbg package_id=2 span=[160-172]
            Block 1:Block:
                Variable(3, Integer) = Mul Variable(0, Integer), Integer(2) !dbg package_id=2 span=[217-227]
                Variable(0, Integer) = Store Variable(3, Integer) !dbg package_id=2 span=[217-222]
                Variable(0, Integer) = Store Integer(3) !dbg package_id=2 span=[237-242]
                Variable(0, Integer) = Store Integer(4) !dbg package_id=2 span=[256-261]
                Call id(3), args( Integer(0), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]
            Block 2:Block:
                Variable(0, Integer) = Store Integer(-1) !dbg package_id=2 span=[187-192]
                Jump(1) !dbg package_id=2 span=[173-208]"#]],
    );
}

#[test]
fn dynamic_var_updated_in_loop_constant_folded_when_every_iteration_results_in_same_value() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Int {
                mutable value = 0;
                use q = Qubit();
                let cond = MResetZ(q);
                for _ in 0..1 {
                    if cond == Zero {
                        value -= 1;
                        value ^= 2;
                    } else {
                        value += 1;
                    }
                    value -= 1;
                }
                return value
            }
        }
        "#,
    });

    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Variable(0, Integer) = Store Integer(0) !dbg package_id=2 span=[81-86]
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Variable(1, Integer) = Store Integer(0) !dbg package_id=2 span=[165-169]
                Variable(2, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[187-199]
                Variable(3, Boolean) = Icmp Eq, Variable(2, Boolean), Bool(false) !dbg package_id=2 span=[187-199]
                Branch Variable(3, Boolean), 2, 3 !dbg package_id=2 span=[187-199]
            Block 1:Block:
                Variable(0, Integer) = Store Integer(0) !dbg package_id=2 span=[333-338]
                Variable(1, Integer) = Store Integer(1) !dbg package_id=2 span=[165-169]
                Variable(4, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[187-199]
                Variable(5, Boolean) = Icmp Eq, Variable(4, Boolean), Bool(false) !dbg package_id=2 span=[187-199]
                Branch Variable(5, Boolean), 5, 6 !dbg package_id=2 span=[187-199]
            Block 2:Block:
                Variable(0, Integer) = Store Integer(-1) !dbg package_id=2 span=[218-223]
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[246-251]
                Jump(1) !dbg package_id=2 span=[200-271]
            Block 3:Block:
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[295-300]
                Jump(1) !dbg package_id=2 span=[272-320]
            Block 4:Block:
                Variable(0, Integer) = Store Integer(0) !dbg package_id=2 span=[333-338]
                Variable(1, Integer) = Store Integer(2) !dbg package_id=2 span=[165-169]
                Call id(3), args( Integer(0), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]
            Block 5:Block:
                Variable(0, Integer) = Store Integer(-1) !dbg package_id=2 span=[218-223]
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[246-251]
                Jump(4) !dbg package_id=2 span=[200-271]
            Block 6:Block:
                Variable(0, Integer) = Store Integer(1) !dbg package_id=2 span=[295-300]
                Jump(4) !dbg package_id=2 span=[272-320]"#]],
    );
}

#[test]
fn immutable_bind_of_dynamic_var_should_be_point_in_time_copy() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : (Int, Int) {
                mutable value = 0;
                use q = Qubit();
                let cond = MResetZ(q);
                if cond == Zero {
                    value -= 1;
                }
                let copy = value;
                value += 1;
                (copy, value)
            }
        }
        "#,
    });

    assert_blocks(
        &program,
        &expect![[r#"
            Blocks:
            Block 0:Block:
                Variable(0, Integer) = Store Integer(0) !dbg package_id=2 span=[88-93]
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=1 span=[182985-183014]
                Variable(1, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[166-178]
                Variable(2, Boolean) = Icmp Eq, Variable(1, Boolean), Bool(false) !dbg package_id=2 span=[166-178]
                Branch Variable(2, Boolean), 2, 1 !dbg package_id=2 span=[166-178]
            Block 1:Block:
                Variable(3, Integer) = Store Variable(0, Integer) !dbg package_id=2 span=[227-231]
                Variable(4, Integer) = Add Variable(0, Integer), Integer(1) !dbg package_id=2 span=[249-259]
                Variable(0, Integer) = Store Variable(4, Integer) !dbg package_id=2 span=[249-254]
                Call id(3), args( Integer(2), Pointer, ) !dbg package_id=2 span=[50-54]
                Call id(4), args( Variable(3, Integer), Pointer, ) !dbg package_id=2 span=[50-54]
                Call id(4), args( Variable(0, Integer), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]
            Block 2:Block:
                Variable(0, Integer) = Store Integer(-1) !dbg package_id=2 span=[193-198]
                Jump(1) !dbg package_id=2 span=[179-214]"#]],
    );
}

#[test]
fn dynamic_double_from_if_expression_with_single_measurement_comparison_and_classical_blocks() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            @EntryPoint()
            operation Main() : Unit {
                use q = Qubit();
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q);
                let b = if r == Zero { 0.1 } else { 1.1 };
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[107-150]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[174-183]
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false) !dbg package_id=2 span=[174-183]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[174-183]
            Block 1:Block:
                Variable(3, Double) = Store Variable(2, Double) !dbg package_id=2 span=[167-168]
                Call id(3), args( Integer(0), Pointer, ) !dbg package_id=2 span=[50-54]
                Return !dbg package_id=2 span=[50-54]
            Block 2:Block:
                Variable(2, Double) = Store Double(0.1) !dbg package_id=2 span=[184-191]
                Jump(1) !dbg package_id=2 span=[184-191]
            Block 3:Block:
                Variable(2, Double) = Store Double(1.1) !dbg package_id=2 span=[192-204]
                Jump(1) !dbg package_id=2 span=[192-204]"#]],
    );
}

#[test]
fn dynamic_double_from_if_expression_with_single_measurement_comparison_and_non_classical_blocks() {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation OpA(q : Qubit) : Unit { body intrinsic; }
            operation OpB(q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1) = (Qubit(), Qubit());
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q0);
                let b = if r == Zero {
                    OpA(q1);
                    0.1
                } else {
                    OpB(q1);
                    1.1
                };
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
            name: OpA
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
                name: OpB
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[237-280]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[305-314]
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false) !dbg package_id=2 span=[305-314]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[305-314]
            Block 1:Block:
                Variable(3, Double) = Store Variable(2, Double) !dbg package_id=2 span=[298-299]
                Call id(5), args( Integer(0), Pointer, ) !dbg package_id=2 span=[162-166]
                Return !dbg package_id=2 span=[162-166]
            Block 2:Block:
                Call id(3), args( Qubit(1), ) !dbg package_id=2 span=[329-332]
                Variable(2, Double) = Store Double(0.1) !dbg package_id=2 span=[315-363]
                Jump(1) !dbg package_id=2 span=[315-363]
            Block 3:Block:
                Call id(4), args( Qubit(1), ) !dbg package_id=2 span=[383-386]
                Variable(2, Double) = Store Double(1.1) !dbg package_id=2 span=[364-417]
                Jump(1) !dbg package_id=2 span=[364-417]"#]],
    );
}

#[test]
fn dynamic_double_from_if_expression_with_single_measurement_comparison_pass_dynamic_double_to_intrinsic()
 {
    let program = get_rir_program(indoc! {
        r#"
        namespace Test {
            operation OpA(theta: Double, q : Qubit) : Unit { body intrinsic; }
            @EntryPoint()
            operation Main() : Unit {
                use (q0, q1) = (Qubit(), Qubit());
                let r = QIR.Intrinsic.__quantum__qis__mresetz__body(q0);
                let b = if r == Zero {
                    0.1
                } else {
                    1.1
                };
                OpA(b, q1);
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
                name: OpA
                call_type: Regular
                input_type:
                    [0]: Double
                    [1]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
    );
    let op_b_callable_id = CallableId(4);
    assert_callable(
        &program,
        op_b_callable_id,
        &expect![[r#"
            Callable:
                name: __quantum__rt__tuple_record_output
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
                Call id(1), args( Qubit(0), Result(0), ) !dbg package_id=2 span=[196-239]
                Variable(0, Boolean) = Call id(2), args( Result(0), ) !dbg package_id=2 span=[264-273]
                Variable(1, Boolean) = Icmp Eq, Variable(0, Boolean), Bool(false) !dbg package_id=2 span=[264-273]
                Branch Variable(1, Boolean), 2, 3 !dbg package_id=2 span=[264-273]
            Block 1:Block:
                Variable(3, Double) = Store Variable(2, Double) !dbg package_id=2 span=[257-258]
                Call id(3), args( Variable(3, Double), Qubit(1), ) !dbg package_id=2 span=[344-347]
                Call id(4), args( Integer(0), Pointer, ) !dbg package_id=2 span=[121-125]
                Return !dbg package_id=2 span=[121-125]
            Block 2:Block:
                Variable(2, Double) = Store Double(0.1) !dbg package_id=2 span=[274-301]
                Jump(1) !dbg package_id=2 span=[274-301]
            Block 3:Block:
                Variable(2, Double) = Store Double(1.1) !dbg package_id=2 span=[302-334]
                Jump(1) !dbg package_id=2 span=[302-334]"#]],
    );
}
