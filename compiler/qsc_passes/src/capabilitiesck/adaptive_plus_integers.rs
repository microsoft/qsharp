// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use crate::capabilitiesck::common::USE_DYNAMIC_RANGE;

use super::common::{
    check, CALL_TO_CICLYC_FUNCTION_WITH_CLASSICAL_ARGUMENT,
    CALL_TO_CICLYC_FUNCTION_WITH_DYNAMIC_ARGUMENT,
    CALL_TO_CICLYC_OPERATION_WITH_CLASSICAL_ARGUMENT,
    CALL_TO_CICLYC_OPERATION_WITH_DYNAMIC_ARGUMENT, MINIMAL, USE_DYNAMICALLY_SIZED_ARRAY,
    USE_DYNAMIC_BIG_INT, USE_DYNAMIC_BOOLEAN, USE_DYNAMIC_DOUBLE, USE_DYNAMIC_FUNCTION,
    USE_DYNAMIC_INT, USE_DYNAMIC_PAULI, USE_DYNAMIC_QUBIT, USE_DYNAMIC_STRING, USE_DYNAMIC_UDT,
};
use expect_test::{expect, Expect};
use qsc_frontend::compile::RuntimeCapabilityFlags;

fn check_profile(source: &str, expect: &Expect) {
    check(
        source,
        expect,
        RuntimeCapabilityFlags::ForwardBranching | RuntimeCapabilityFlags::IntegerComputations,
    );
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
fn use_of_dynamic_int_yields_no_errors() {
    check_profile(
        USE_DYNAMIC_INT,
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn use_of_dynamic_pauli_yields_no_errors() {
    check_profile(
        USE_DYNAMIC_PAULI,
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn use_of_dynamic_range_yields_no_errors() {
    check_profile(
        USE_DYNAMIC_RANGE,
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn use_of_dynamic_double_yields_error() {
    check_profile(
        USE_DYNAMIC_DOUBLE,
        &expect![[r#"
            [
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

#[ignore = "investigation in progress"]
#[test]
fn use_of_dynamic_function_yields_errors() {
    check_profile(
        USE_DYNAMIC_FUNCTION,
        &expect![[r#"
            []
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
fn call_cyclic_function_with_dynamic_argument_yields_error() {
    check_profile(
        CALL_TO_CICLYC_FUNCTION_WITH_DYNAMIC_ARGUMENT,
        &expect![[r#"
            [
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
