// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

pub const QSHARP_LIBRARY_URI_SCHEME: &str = "qsharp-library-source";

// The core prefix on the name is needed to disambiguate from the std
// files of the same name. This comes in during debugging when we need
// to load a core/std file from the library.
pub const CORE_LIB: &[(&str, &str)] = &[
    (
        "qsharp-library-source:core/core.qs",
        include_str!("../qs_source/core/core.qs"),
    ),
    (
        "qsharp-library-source:core/qir.qs",
        include_str!("../qs_source/core/qir.qs"),
    ),
];

pub const STD_LIB: &[(&str, &str)] = &[
    (
        "qsharp-library-source:arrays.qs",
        include_str!("../qs_source/src/std/arrays.qs"),
    ),
    (
        "qsharp-library-source:canon.qs",
        include_str!("../qs_source/src/std/canon.qs"),
    ),
    (
        "qsharp-library-source:convert.qs",
        include_str!("../qs_source/src/std/convert.qs"),
    ),
    (
        "qsharp-library-source:core.qs",
        include_str!("../qs_source/src/std/core.qs"),
    ),
    (
        "qsharp-library-source:diagnostics.qs",
        include_str!("../qs_source/src/std/diagnostics.qs"),
    ),
    (
        "qsharp-library-source:internal.qs",
        include_str!("../qs_source/src/std/internal.qs"),
    ),
    (
        "qsharp-library-source:intrinsic.qs",
        include_str!("../qs_source/src/std/intrinsic.qs"),
    ),
    (
        "qsharp-library-source:logical.qs",
        include_str!("../qs_source/src/std/logical.qs"),
    ),
    (
        "qsharp-library-source:math.qs",
        include_str!("../qs_source/src/std/math.qs"),
    ),
    (
        "qsharp-library-source:measurement.qs",
        include_str!("../qs_source/src/std/measurement.qs"),
    ),
    (
        "qsharp-library-source:qir.qs",
        include_str!("../qs_source/src/std/qir.qs"),
    ),
    (
        "qsharp-library-source:random.qs",
        include_str!("../qs_source/src/std/random.qs"),
    ),
    (
        "qsharp-library-source:re.qs",
        include_str!("../qs_source/src/std/re.qs"),
    ),
    (
        "qsharp-library-source:unstable_arithmetic.qs",
        include_str!("../qs_source/src/std/unstable_arithmetic.qs"),
    ),
    (
        "qsharp-library-source:unstable_arithmetic_internal.qs",
        include_str!("../qs_source/src/std/unstable_arithmetic_internal.qs"),
    ),
    (
        "qsharp-library-source:unstable_state_preparation.qs",
        include_str!("../qs_source/src/std/unstable_state_preparation.qs"),
    ),
    (
        "qsharp-library-source:unstable_table_lookup.qs",
        include_str!("../qs_source/src/std/unstable_table_lookup.qs"),
    ),
];
