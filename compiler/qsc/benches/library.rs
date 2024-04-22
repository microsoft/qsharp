// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

allocator::assign_global!();

use criterion::{criterion_group, criterion_main, Criterion};
use qsc::compile;
use qsc_frontend::compile::{PackageStore, TargetCapabilityFlags};

pub fn library(c: &mut Criterion) {
    c.bench_function("Core + Standard library compilation", |b| {
        b.iter(|| {
            let store = PackageStore::new(compile::core());
            compile::std(&store, TargetCapabilityFlags::all())
        });
    });
}

criterion_group!(benches, library);
criterion_main!(benches);
