// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

// The core prefix on the name is needed to disambiguate from the std
// files of the same name. This comes in during debugging when we need
// to load a core/std file from the library.
pub const CORE_LIB: &[(&str, &str)] = &[
    ("core/core.qs", include_str!("../qs_source/core/core.qs")),
    ("core/qir.qs", include_str!("../qs_source/core/qir.qs")),
];

pub const STD_LIB: &[(&str, &str)] = &[
    ("arrays.qs", include_str!("../qs_source/src/std/arrays.qs")),
    ("canon.qs", include_str!("../qs_source/src/std/canon.qs")),
    (
        "convert.qs",
        include_str!("../qs_source/src/std/convert.qs"),
    ),
    ("core.qs", include_str!("../qs_source/src/std/core.qs")),
    (
        "diagnostics.qs",
        include_str!("../qs_source/src/std/diagnostics.qs"),
    ),
    (
        "internal.qs",
        include_str!("../qs_source/src/std/internal.qs"),
    ),
    (
        "intrinsic.qs",
        include_str!("../qs_source/src/std/intrinsic.qs"),
    ),
    (
        "logical.qs",
        include_str!("../qs_source/src/std/logical.qs"),
    ),
    ("math.qs", include_str!("../qs_source/src/std/math.qs")),
    (
        "measurement.qs",
        include_str!("../qs_source/src/std/measurement.qs"),
    ),
    ("qir.qs", include_str!("../qs_source/src/std/qir.qs")),
    ("random.qs", include_str!("../qs_source/src/std/random.qs")),
    ("re.qs", include_str!("../qs_source/src/std/re.qs")),
    (
        "unstable_arithmetic.qs",
        include_str!("../qs_source/src/std/unstable_arithmetic.qs"),
    ),
    (
        "unstable_arithmetic_internal.qs",
        include_str!("../qs_source/src/std/unstable_arithmetic_internal.qs"),
    ),
    (
        "unstable_state_preparation.qs",
        include_str!("../qs_source/src/std/unstable_state_preparation.qs"),
    ),
    (
        "unstable_table_lookup.qs",
        include_str!("../qs_source/src/std/unstable_table_lookup.qs"),
    ),
];
