// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use criterion::{criterion_group, criterion_main, Criterion};
use qsc::{compile, PackageStore};

pub fn library(c: &mut Criterion) {
    let store = PackageStore::new(compile::core());
    c.bench_function("Standard library", |b| {
        b.iter(|| compile::std(&store, qsc::TargetProfile::Full))
    });
}

criterion_group!(benches, library);
criterion_main!(benches);
