// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use criterion::{criterion_group, criterion_main, Criterion};

pub fn library(c: &mut Criterion) {
    c.bench_function("Standard library", |b| b.iter(qsc::compile::std));
}

criterion_group!(benches, library);
criterion_main!(benches);
