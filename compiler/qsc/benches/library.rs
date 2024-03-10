// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(not(target_family = "wasm"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use criterion::{criterion_group, criterion_main, Criterion};
use qsc::compile;
use qsc_frontend::compile::{PackageStore, RuntimeCapabilityFlags};

pub fn library(c: &mut Criterion) {
    let store = PackageStore::new(compile::core());
    c.bench_function("Standard library", |b| {
        b.iter(|| compile::std(&store, RuntimeCapabilityFlags::all()));
    });
}

criterion_group!(benches, library);
criterion_main!(benches);
