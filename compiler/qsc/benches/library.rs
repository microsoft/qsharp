// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

allocator::assign_global!();

use criterion::{criterion_group, criterion_main, Criterion};
use qsc::compile;
use qsc_frontend::compile::{PackageStore, RuntimeCapabilityFlags};

pub fn library(c: &mut Criterion) {
    c.bench_function("Core + Standard library compilation", |b| {
        let store = PackageStore::new(compile::core());
        b.iter(|| compile::std(&store, RuntimeCapabilityFlags::all()));
    });
}

criterion_group!(benches, library);
criterion_main!(benches);
