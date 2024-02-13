// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

#[cfg(test)]
mod tests;

// The core prefix on the name is needed to disambiguate from the std
// files of the same name. This comes in during debugging when we need
// to load a core/std file from the library.
pub const CORE_LIB: &[(&str, &str)] = &[
    ("core/core.qs", include_str!("../core/core.qs")),
    ("core/qir.qs", include_str!("../core/qir.qs")),
];

pub const STD_LIB: &[(&str, &str)] = &[
    ("arrays.qs", include_str!("../std/arrays.qs")),
    ("canon.qs", include_str!("../std/canon.qs")),
    ("convert.qs", include_str!("../std/convert.qs")),
    ("core.qs", include_str!("../std/core.qs")),
    ("diagnostics.qs", include_str!("../std/diagnostics.qs")),
    ("internal.qs", include_str!("../std/internal.qs")),
    ("intrinsic.qs", include_str!("../std/intrinsic.qs")),
    ("logical.qs", include_str!("../std/logical.qs")),
    ("math.qs", include_str!("../std/math.qs")),
    ("measurement.qs", include_str!("../std/measurement.qs")),
    ("qir.qs", include_str!("../std/qir.qs")),
    ("random.qs", include_str!("../std/random.qs")),
    ("re.qs", include_str!("../std/re.qs")),
    (
        "unstable_arithmetic.qs",
        include_str!("../std/unstable_arithmetic.qs"),
    ),
    (
        "unstable_arithmetic_internal.qs",
        include_str!("../std/unstable_arithmetic_internal.qs"),
    ),
    (
        "unstable_state_preparation.qs",
        include_str!("../std/unstable_state_preparation.qs"),
    ),
    (
        "unstable_table_lookup.qs",
        include_str!("../std/unstable_table_lookup.qs"),
    ),
];
