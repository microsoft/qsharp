// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use criterion::{criterion_group, criterion_main, Criterion};
use qsc::compile::{self, compile};
use qsc_frontend::compile::{PackageStore, SourceMap};

const INPUT: &str = include_str!("./nqueens.qs");

pub fn nqueens(c: &mut Criterion) {
    c.bench_function("NQueens large input file", |b| {
        b.iter(|| {
            let mut store = PackageStore::new(compile::core());
            let std = store.insert(compile::std(&store));
            let sources = SourceMap::new([("nqueens.qs".into(), INPUT.into())], None);
            let (_, reports) = compile(&store, &[std], sources);
            assert!(reports.is_empty());
        })
    });
}

criterion_group!(benches, nqueens);
criterion_main!(benches);
