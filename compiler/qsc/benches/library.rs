// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use criterion::{criterion_group, criterion_main, Criterion};
use qsc_frontend::compile;
use qsc_passes::run_default_passes;

pub fn library(c: &mut Criterion) {
    c.bench_function("Standard library", |b| {
        b.iter(|| {
            let mut std = compile::std();
            run_default_passes(&mut std);
        })
    });
}

criterion_group!(benches, library);
criterion_main!(benches);
