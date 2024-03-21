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
                        lo: 96,
                        hi: 125,
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
                        lo: 96,
                        hi: 135,
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
                        lo: 96,
                        hi: 138,
                    },
                ),
                UseOfDynamicRange(
                    Span {
                        lo: 96,
                        hi: 138,
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
                UseOfDynamicDouble(
                    Span {
                        lo: 96,
                        hi: 129,
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
                        lo: 96,
                        hi: 137,
                    },
                ),
                UseOfDynamicallySizedArray(
                    Span {
                        lo: 96,
                        hi: 137,
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
                UseOfDynamicInt(
                    Span {
                        lo: 201,
                        hi: 244,
                    },
                ),
                CallToCyclicFunctionWithDynamicArg(
                    Span {
                        lo: 201,
                        hi: 244,
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
                        lo: 177,
                        hi: 200,
                    },
                ),
                CallToCyclicOperation(
                    Span {
                        lo: 177,
                        hi: 200,
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
                UseOfDynamicInt(
                    Span {
                        lo: 202,
                        hi: 245,
                    },
                ),
                CallToCyclicOperation(
                    Span {
                        lo: 202,
                        hi: 245,
                    },
                ),
            ]
        "#]],
    );
}
