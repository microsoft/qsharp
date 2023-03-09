// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use criterion::{criterion_group, criterion_main, Criterion};
use qsc_frontend::compile::{compile, PackageStore};

const INPUT: &str = include_str!("./nqueens.qs");

pub fn nqueens(c: &mut Criterion) {
    c.bench_function("NQueens large input file", |b| {
        b.iter(|| compile(&PackageStore::new(), &[], &[INPUT], ""))
    });
}

criterion_group!(benches, nqueens);
criterion_main!(benches);
