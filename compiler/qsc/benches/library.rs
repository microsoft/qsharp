// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use criterion::{criterion_group, criterion_main, Criterion};
use qsc_frontend::{compile, PackageStore};

const CANON: &str = include_str!("../../../library/canon.qs");
const CORE: &str = include_str!("../../../library/core.qs");
const DIAGNOSTICS: &str = include_str!("../../../library/diagnostics.qs");
const INTERNAL: &str = include_str!("../../../library/internal.qs");
const INTRINSIC: &str = include_str!("../../../library/intrinsic.qs");
const MATH: &str = include_str!("../../../library/math.qs");
const QIR: &str = include_str!("../../../library/qir.qs");

pub fn library(c: &mut Criterion) {
    c.bench_function("Standard library", |b| {
        b.iter(|| {
            compile(
                &PackageStore::new(),
                &[CANON, CORE, DIAGNOSTICS, INTERNAL, INTRINSIC, MATH, QIR],
                "",
                Vec::new(),
            );
        })
    });
}

criterion_group!(benches, library);
criterion_main!(benches);
