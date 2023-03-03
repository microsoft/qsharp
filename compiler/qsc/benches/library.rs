// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use criterion::{criterion_group, criterion_main, Criterion};
use qsc_frontend::compile;

static CANON: &str = include_str!("../../../library/canon.qs");
static CORE: &str = include_str!("../../../library/core.qs");
static DIAGNOSTICS: &str = include_str!("../../../library/diagnostics.qs");
static INTERNAL: &str = include_str!("../../../library/internal.qs");
static INTRINSIC: &str = include_str!("../../../library/intrinsic.qs");
static MATH: &str = include_str!("../../../library/math.qs");
static QIR: &str = include_str!("../../../library/qir.qs");

pub fn library(c: &mut Criterion) {
    c.bench_function("Standard library", |b| {
        b.iter(|| {
            compile(
                [CANON, CORE, DIAGNOSTICS, INTERNAL, INTRINSIC, MATH, QIR],
                "",
            );
        })
    });
}

criterion_group!(benches, library);
criterion_main!(benches);
