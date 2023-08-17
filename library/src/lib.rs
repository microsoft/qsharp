// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub const CORE_LIB: &[(&str, &str)] = &[
    ("core.qs", include_str!("../core/core.qs")),
    ("qir.qs", include_str!("../core/qir.qs")),
];

pub const STD_LIB: &[(&str, &str)] = &[
    ("arithmetic.qs", include_str!("../std/arithmetic.qs")),
    ("arrays.qs", include_str!("../std/arrays.qs")),
    ("canon.qs", include_str!("../std/canon.qs")),
    ("convert.qs", include_str!("../std/convert.qs")),
    ("core.qs", include_str!("../std/core.qs")),
    ("diagnostics.qs", include_str!("../std/diagnostics.qs")),
    ("internal.qs", include_str!("../std/internal.qs")),
    ("intrinsic.qs", include_str!("../std/intrinsic.qs")),
    ("math.qs", include_str!("../std/math.qs")),
    ("measurement.qs", include_str!("../std/measurement.qs")),
    ("qir.qs", include_str!("../std/qir.qs")),
    ("random.qs", include_str!("../std/random.qs")),
];
