// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use super::common::{
    check, CALL_TO_CICLYC_FUNCTION_WITH_CLASSICAL_ARGUMENT,
    CALL_TO_CICLYC_FUNCTION_WITH_DYNAMIC_ARGUMENT,
    CALL_TO_CICLYC_OPERATION_WITH_CLASSICAL_ARGUMENT,
    CALL_TO_CICLYC_OPERATION_WITH_DYNAMIC_ARGUMENT, MINIMAL, USE_DYNAMICALLY_SIZED_ARRAY,
    USE_DYNAMIC_BIG_INT, USE_DYNAMIC_BOOLEAN, USE_DYNAMIC_DOUBLE, USE_DYNAMIC_FUNCTION,
    USE_DYNAMIC_INT, USE_DYNAMIC_PAULI, USE_DYNAMIC_QUBIT, USE_DYNAMIC_RANGE, USE_DYNAMIC_STRING,
    USE_DYNAMIC_UDT,
};
use expect_test::{expect, Expect};
use qsc_frontend::compile::RuntimeCapabilityFlags;

fn check_profile(source: &str, expect: &Expect) {
    check(source, expect, RuntimeCapabilityFlags::empty());
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
fn use_of_dynamic_boolean_yields_error() {
    check_profile(
        USE_DYNAMIC_BOOLEAN,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 104,
                        hi: 116,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn use_of_dynamic_int_yields_errors() {
    check_profile(
        USE_DYNAMIC_INT,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 246,
                        hi: 271,
                    },
                ),
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
fn use_of_dynamic_pauli_yields_errors() {
    // In the case of if expressions, if either the condition or the blocks yield errors, the errors yielded by the
    // whole if expression are not surfaced to avoid too error churn.
    // For this reason, the "use of dynamic pauli" error is not yielded in this case.
    check_profile(
        USE_DYNAMIC_PAULI,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 104,
                        hi: 116,
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
                UseOfDynamicBool(
                    Span {
                        lo: 108,
                        hi: 137,
                    },
                ),
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
                UseOfDynamicBool(
                    Span {
                        lo: 246,
                        hi: 284,
                    },
                ),
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
                UseOfDynamicBool(
                    Span {
                        lo: 105,
                        hi: 123,
                    },
                ),
                UseOfDynamicQubit(
                    Span {
                        lo: 142,
                        hi: 158,
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
                UseOfDynamicBool(
                    Span {
                        lo: 247,
                        hi: 285,
                    },
                ),
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
                UseOfDynamicBool(
                    Span {
                        lo: 130,
                        hi: 144,
                    },
                ),
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
fn use_of_dynamically_sized_array_yields_errors() {
    check_profile(
        USE_DYNAMICALLY_SIZED_ARRAY,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 104,
                        hi: 136,
                    },
                ),
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
                UseOfDynamicBool(
                    Span {
                        lo: 283,
                        hi: 335,
                    },
                ),
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
    // In the case of if expressions, if either the condition or the blocks yield errors, the errors yielded by the
    // whole if expression are not surfaced to avoid too much error churn.
    // For this reason, the "use of dynamic function" error is not yielded in this case.
    check_profile(
        USE_DYNAMIC_FUNCTION,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 141,
                        hi: 153,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn call_cyclic_function_with_classical_argument_yields_no_errors() {
    check_profile(
        CALL_TO_CICLYC_FUNCTION_WITH_CLASSICAL_ARGUMENT,
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn call_cyclic_function_with_dynamic_argument_yields_errors() {
    check_profile(
        CALL_TO_CICLYC_FUNCTION_WITH_DYNAMIC_ARGUMENT,
        &expect![[r#"
            [
                UseOfDynamicBool(
                    Span {
                        lo: 211,
                        hi: 243,
                    },
                ),
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
        CALL_TO_CICLYC_OPERATION_WITH_CLASSICAL_ARGUMENT,
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
        CALL_TO_CICLYC_OPERATION_WITH_DYNAMIC_ARGUMENT,
        &expect![[r#"
            [
                CyclicOperationSpec(
                    Span {
                        lo: 15,
                        hi: 23,
                    },
                ),
                UseOfDynamicBool(
                    Span {
                        lo: 212,
                        hi: 244,
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
