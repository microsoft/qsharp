// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

pub mod test_utils;

use expect_test::{expect, Expect};
use indoc::{formatdoc, indoc};
use qsc_rir::rir::{BlockId, CallableId};
use test_utils::{assert_block_instructions, assert_callable, compile_and_partially_evaluate};

fn check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
    intrinsic_name: &str,
    expected_callable: &Expect,
    expected_block: &Expect,
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
    assert_callable(&program, op_callable_id, expected_callable);
    assert_block_instructions(&program, BlockId(0), expected_block);
}

fn check_call_to_single_qubit_rotation_instrinsic_adds_callable_and_generates_instruction(
    intrinsic_name: &str,
    expected_callable: &Expect,
    expected_block: &Expect,
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
    assert_callable(&program, op_callable_id, expected_callable);
    assert_block_instructions(&program, BlockId(0), expected_block);
}

fn check_call_to_two_qubits_rotation_instrinsic_adds_callable_and_generates_instruction(
    intrinsic_name: &str,
    expected_callable: &Expect,
    expected_block: &Expect,
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
    assert_callable(&program, op_callable_id, expected_callable);
    assert_block_instructions(&program, BlockId(0), expected_block);
}

fn check_call_to_two_qubits_instrinsic_adds_callable_and_generates_instruction(
    intrinsic_name: &str,
    expected_callable: &Expect,
    expected_block: &Expect,
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
    assert_callable(&program, op_callable_id, expected_callable);
    assert_block_instructions(&program, BlockId(0), expected_block);
}

fn check_call_to_three_qubits_instrinsic_adds_callable_and_generates_instruction(
    intrinsic_name: &str,
    expected_callable: &Expect,
    expected_block: &Expect,
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
    assert_callable(&program, op_callable_id, expected_callable);
    assert_block_instructions(&program, BlockId(0), expected_block);
}

#[test]
fn call_to_intrinsic_h_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__h__body",
        &expect![[r#"
            Callable:
                name: __quantum__qis__h__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_s_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__s__body",
        &expect![[r#"
            Callable:
                name: __quantum__qis__s__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_adjoint_s_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__s__adj",
        &expect![[r#"
            Callable:
                name: __quantum__qis__s__adj
                call_type: Regular
                input_type:
                    [0]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_t_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__t__body",
        &expect![[r#"
            Callable:
                name: __quantum__qis__t__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_adjoint_t_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__t__adj",
        &expect![[r#"
            Callable:
                name: __quantum__qis__t__adj
                call_type: Regular
                input_type:
                    [0]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_x_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__x__body",
        &expect![[r#"
            Callable:
                name: __quantum__qis__x__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_y_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__y__body",
        &expect![[r#"
            Callable:
                name: __quantum__qis__y__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_z_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__z__body",
        &expect![[r#"
            Callable:
                name: __quantum__qis__z__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_swap_adds_callable_and_generates_instruction() {
    check_call_to_two_qubits_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__swap__body",
        &expect![[r#"
            Callable:
                name: __quantum__qis__swap__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                    [1]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), Qubit(1), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_cx_adds_callable_and_generates_instruction() {
    check_call_to_two_qubits_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__cx__body",
        &expect![[r#"
            Callable:
                name: __quantum__qis__cx__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                    [1]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), Qubit(1), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_cy_adds_callable_and_generates_instruction() {
    check_call_to_two_qubits_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__cy__body",
        &expect![[r#"
            Callable:
                name: __quantum__qis__cy__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                    [1]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), Qubit(1), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_cz_adds_callable_and_generates_instruction() {
    check_call_to_two_qubits_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__cz__body",
        &expect![[r#"
            Callable:
                name: __quantum__qis__cz__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                    [1]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), Qubit(1), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_ccx_adds_callable_and_generates_instruction() {
    check_call_to_three_qubits_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__ccx__body",
        &expect![[r#"
            Callable:
                name: __quantum__qis__ccx__body
                call_type: Regular
                input_type:
                    [0]: Qubit
                    [1]: Qubit
                    [2]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Qubit(0), Qubit(1), Qubit(2), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_rx_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_rotation_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__rx__body",
        &expect![[r#"
            Callable:
                name: __quantum__qis__rx__body
                call_type: Regular
                input_type:
                    [0]: Double
                    [1]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Double(0), Qubit(0), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_rxx_adds_callable_and_generates_instruction() {
    check_call_to_two_qubits_rotation_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__rxx__body",
        &expect![[r#"
            Callable:
                name: __quantum__qis__rxx__body
                call_type: Regular
                input_type:
                    [0]: Double
                    [1]: Qubit
                    [2]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Double(0), Qubit(0), Qubit(1), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_ry_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_rotation_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__ry__body",
        &expect![[r#"
            Callable:
                name: __quantum__qis__ry__body
                call_type: Regular
                input_type:
                    [0]: Double
                    [1]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Double(0), Qubit(0), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_ryy_adds_callable_and_generates_instruction() {
    check_call_to_two_qubits_rotation_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__ryy__body",
        &expect![[r#"
            Callable:
                name: __quantum__qis__ryy__body
                call_type: Regular
                input_type:
                    [0]: Double
                    [1]: Qubit
                    [2]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Double(0), Qubit(0), Qubit(1), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_rz_adds_callable_and_generates_instruction() {
    check_call_to_single_qubit_rotation_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__rz__body",
        &expect![[r#"
            Callable:
                name: __quantum__qis__rz__body
                call_type: Regular
                input_type:
                    [0]: Double
                    [1]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Double(0), Qubit(0), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
    );
}

#[test]
fn call_to_intrinsic_rzz_adds_callable_and_generates_instruction() {
    check_call_to_two_qubits_rotation_instrinsic_adds_callable_and_generates_instruction(
        "__quantum__qis__rzz__body",
        &expect![[r#"
            Callable:
                name: __quantum__qis__rzz__body
                call_type: Regular
                input_type:
                    [0]: Double
                    [1]: Qubit
                    [2]: Qubit
                output_type: <VOID>
                body: <NONE>"#]],
        &expect![[r#"
            Block:
                Call id(1), args( Double(0), Qubit(0), Qubit(1), )
                Call id(2), args( Integer(0), Pointer, )
                Return"#]],
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
    assert_callable(
        &program,
        op_callable_id,
        &expect![[r#"
        Callable:
            name: __quantum__qis__reset__body
            call_type: Reset
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
            Call id(2), args( Integer(0), Pointer, )
            Return"#]],
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
        &expect![[r#"
        Callable:
            name: __quantum__qis__mz__body
            call_type: Measurement
            input_type:
                [0]: Qubit
                [1]: Result
            output_type: <VOID>
            body: <NONE>"#]],
    );
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
        Block:
            Call id(1), args( Qubit(0), Result(0), )
            Call id(2), args( Integer(0), Pointer, )
            Return"#]],
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
    assert_block_instructions(
        &program,
        BlockId(0),
        &expect![[r#"
        Block:
            Call id(1), args( Qubit(0), Result(0), )
            Call id(2), args( Integer(0), Pointer, )
            Return"#]],
    );
}
