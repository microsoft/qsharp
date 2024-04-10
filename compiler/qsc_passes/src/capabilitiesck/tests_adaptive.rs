// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use super::tests_common::{
    check, CALL_DYNAMIC_FUNCTION, CALL_DYNAMIC_OPERATION,
    CALL_TO_CYCLIC_FUNCTION_WITH_CLASSICAL_ARGUMENT, CALL_TO_CYCLIC_FUNCTION_WITH_DYNAMIC_ARGUMENT,
    CALL_TO_CYCLIC_OPERATION_WITH_CLASSICAL_ARGUMENT,
    CALL_TO_CYCLIC_OPERATION_WITH_DYNAMIC_ARGUMENT, CALL_UNRESOLVED_FUNCTION,
    LOOP_WITH_DYNAMIC_CONDITION, MEASUREMENT_WITHIN_DYNAMIC_SCOPE, MINIMAL,
    RETURN_WITHIN_DYNAMIC_SCOPE, USE_CLOSURE_FUNCTION, USE_DYNAMICALLY_SIZED_ARRAY,
    USE_DYNAMIC_BIG_INT, USE_DYNAMIC_BOOLEAN, USE_DYNAMIC_DOUBLE, USE_DYNAMIC_FUNCTION,
    USE_DYNAMIC_INDEX, USE_DYNAMIC_INT, USE_DYNAMIC_OPERATION, USE_DYNAMIC_PAULI,
    USE_DYNAMIC_QUBIT, USE_DYNAMIC_RANGE, USE_DYNAMIC_STRING, USE_DYNAMIC_UDT,
};
use expect_test::{expect, Expect};
use qsc_frontend::compile::RuntimeCapabilityFlags;

fn check_profile(source: &str, expect: &Expect) {
    check(source, expect, RuntimeCapabilityFlags::ForwardBranching);
}

#[test]
fn minimal_program_yields_no_errors() {
    check_profile(
        MINIMAL,
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn use_of_dynamic_boolean_yields_no_errors() {
    check_profile(
        USE_DYNAMIC_BOOLEAN,
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn use_of_dynamic_int_yields_error() {
    check_profile(
        USE_DYNAMIC_INT,
        &expect![[r#"
            [
                UseOfDynamicInt(
                    Span {
                        lo: 246,
                        hi: 271,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_pauli_yields_error() {
    check_profile(
        USE_DYNAMIC_PAULI,
        &expect![[r#"
            [
                UseOfDynamicPauli(
                    Span {
                        lo: 104,
                        hi: 134,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_range_yields_errors() {
    check_profile(
        USE_DYNAMIC_RANGE,
        &expect![[r#"
            [
                UseOfDynamicInt(
                    Span {
                        lo: 108,
                        hi: 137,
                    },
                ),
                UseOfDynamicRange(
                    Span {
                        lo: 108,
                        hi: 137,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_double_yields_errors() {
    check_profile(
        USE_DYNAMIC_DOUBLE,
        &expect![[r#"
            [
                UseOfDynamicInt(
                    Span {
                        lo: 246,
                        hi: 284,
                    },
                ),
                UseOfDynamicDouble(
                    Span {
                        lo: 246,
                        hi: 284,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_qubit_yields_errors() {
    check_profile(
        USE_DYNAMIC_QUBIT,
        &expect![[r#"
            [
                UseOfDynamicQubit(
                    Span {
                        lo: 146,
                        hi: 162,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_big_int_yields_errors() {
    check_profile(
        USE_DYNAMIC_BIG_INT,
        &expect![[r#"
            [
                UseOfDynamicInt(
                    Span {
                        lo: 247,
                        hi: 285,
                    },
                ),
                UseOfDynamicBigInt(
                    Span {
                        lo: 247,
                        hi: 285,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_string_yields_errors() {
    check_profile(
        USE_DYNAMIC_STRING,
        &expect![[r#"
            [
                UseOfDynamicString(
                    Span {
                        lo: 130,
                        hi: 144,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamically_sized_array_yields_error() {
    check_profile(
        USE_DYNAMICALLY_SIZED_ARRAY,
        &expect![[r#"
            [
                UseOfDynamicInt(
                    Span {
                        lo: 104,
                        hi: 136,
                    },
                ),
                UseOfDynamicallySizedArray(
                    Span {
                        lo: 104,
                        hi: 136,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_udt_yields_errors() {
    check_profile(
        USE_DYNAMIC_UDT,
        &expect![[r#"
            [
                UseOfDynamicInt(
                    Span {
                        lo: 283,
                        hi: 335,
                    },
                ),
                UseOfDynamicDouble(
                    Span {
                        lo: 283,
                        hi: 335,
                    },
                ),
                UseOfDynamicUdt(
                    Span {
                        lo: 283,
                        hi: 335,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_function_yields_errors() {
    check_profile(
        USE_DYNAMIC_FUNCTION,
        &expect![[r#"
            [
                UseOfDynamicArrowFunction(
                    Span {
                        lo: 142,
                        hi: 166,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_operation_yields_errors() {
    check_profile(
        USE_DYNAMIC_OPERATION,
        &expect![[r#"
            [
                UseOfDynamicArrowOperation(
                    Span {
                        lo: 142,
                        hi: 162,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn call_cyclic_function_with_classical_argument_yields_no_errors() {
    check_profile(
        CALL_TO_CYCLIC_FUNCTION_WITH_CLASSICAL_ARGUMENT,
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn call_cyclic_function_with_dynamic_argument_yields_errors() {
    check_profile(
        CALL_TO_CYCLIC_FUNCTION_WITH_DYNAMIC_ARGUMENT,
        &expect![[r#"
            [
                UseOfDynamicInt(
                    Span {
                        lo: 211,
                        hi: 243,
                    },
                ),
                CallToCyclicFunctionWithDynamicArg(
                    Span {
                        lo: 211,
                        hi: 243,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn call_cyclic_operation_with_classical_argument_yields_errors() {
    check_profile(
        CALL_TO_CYCLIC_OPERATION_WITH_CLASSICAL_ARGUMENT,
        &expect![[r#"
            [
                CyclicOperationSpec(
                    Span {
                        lo: 15,
                        hi: 23,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 187,
                        hi: 199,
                    },
                ),
                CallToCyclicOperation(
                    Span {
                        lo: 187,
                        hi: 199,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn call_cyclic_operation_with_dynamic_argument_yields_errors() {
    check_profile(
        CALL_TO_CYCLIC_OPERATION_WITH_DYNAMIC_ARGUMENT,
        &expect![[r#"
            [
                CyclicOperationSpec(
                    Span {
                        lo: 15,
                        hi: 23,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 212,
                        hi: 244,
                    },
                ),
                CallToCyclicOperation(
                    Span {
                        lo: 212,
                        hi: 244,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn call_to_dynamic_function_yields_errors() {
    check_profile(
        CALL_DYNAMIC_FUNCTION,
        &expect![[r#"
            [
                UseOfDynamicArrowFunction(
                    Span {
                        lo: 142,
                        hi: 166,
                    },
                ),
                UseOfDynamicDouble(
                    Span {
                        lo: 180,
                        hi: 188,
                    },
                ),
                UseOfDynamicArrowFunction(
                    Span {
                        lo: 180,
                        hi: 188,
                    },
                ),
                CallToDynamicCallee(
                    Span {
                        lo: 180,
                        hi: 188,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn call_to_dynamic_operation_yields_errors() {
    check_profile(
        CALL_DYNAMIC_OPERATION,
        &expect![[r#"
            [
                UseOfDynamicArrowOperation(
                    Span {
                        lo: 142,
                        hi: 162,
                    },
                ),
                UseOfDynamicArrowOperation(
                    Span {
                        lo: 176,
                        hi: 181,
                    },
                ),
                CallToDynamicCallee(
                    Span {
                        lo: 176,
                        hi: 181,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn call_to_unresolved_yields_errors() {
    check_profile(
        CALL_UNRESOLVED_FUNCTION,
        &expect![[r#"
            [
                UseOfDynamicDouble(
                    Span {
                        lo: 172,
                        hi: 180,
                    },
                ),
                CallToUnresolvedCallee(
                    Span {
                        lo: 172,
                        hi: 180,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn measurement_within_dynamic_scope_yields_no_errors() {
    check_profile(
        MEASUREMENT_WITHIN_DYNAMIC_SCOPE,
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn use_of_dynamic_index_yields_errors() {
    check_profile(
        USE_DYNAMIC_INDEX,
        &expect![[r#"
            [
                UseOfDynamicInt(
                    Span {
                        lo: 246,
                        hi: 271,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 319,
                        hi: 323,
                    },
                ),
                UseOfDynamicIndex(
                    Span {
                        lo: 319,
                        hi: 323,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn return_within_dynamic_scope_yields_no_errors() {
    check_profile(
        RETURN_WITHIN_DYNAMIC_SCOPE,
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn loop_with_dynamic_condition_yields_errors() {
    check_profile(
        LOOP_WITH_DYNAMIC_CONDITION,
        &expect![[r#"
            [
                UseOfDynamicInt(
                    Span {
                        lo: 106,
                        hi: 127,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 141,
                        hi: 159,
                    },
                ),
                UseOfDynamicRange(
                    Span {
                        lo: 141,
                        hi: 159,
                    },
                ),
                LoopWithDynamicCondition(
                    Span {
                        lo: 141,
                        hi: 159,
                    },
                ),
                UseOfDynamicInt(
                    Span {
                        lo: 150,
                        hi: 156,
                    },
                ),
                UseOfDynamicRange(
                    Span {
                        lo: 150,
                        hi: 156,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_closure_yields_errors() {
    check_profile(
        USE_CLOSURE_FUNCTION,
        &expect![[r#"
            [
                UseOfClosure(
                    Span {
                        lo: 149,
                        hi: 168,
                    },
                ),
            ]
        "#]],
    );
}
