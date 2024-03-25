// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use super::common::{
    check, CALL_TO_CICLYC_FUNCTION_WITH_CLASSICAL_ARGUMENT,
    CALL_TO_CICLYC_FUNCTION_WITH_DYNAMIC_ARGUMENT,
    CALL_TO_CICLYC_OPERATION_WITH_CLASSICAL_ARGUMENT,
    CALL_TO_CICLYC_OPERATION_WITH_DYNAMIC_ARGUMENT, MINIMAL, USE_DYNAMICALLY_SIZED_ARRAY,
    USE_DYNAMIC_BOOLEAN, USE_DYNAMIC_DOUBLE, USE_DYNAMIC_INT, USE_DYNAMIC_PAULI, USE_DYNAMIC_RANGE,
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

// In the case of if expressions, if either the condition or the blocks yield errors, the errors yielded by the whole if
// expression are not surfaced to avoid too error churn.
// Many of the test cases in this file use if expressions and in many of those the condition expression yields errors.
// For this reason, some errors such "use of expected int" or "use of expected double" are not seen in some test cases.

#[test]
fn use_of_dynamic_int_yields_errors() {
    check_profile(
        USE_DYNAMIC_INT,
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
fn use_of_dynamic_pauli_yields_errors() {
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
                        lo: 104,
                        hi: 116,
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
