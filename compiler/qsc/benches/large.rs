// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use criterion::{criterion_group, criterion_main, Criterion};
use qsc::compile::{self, compile};
use qsc_frontend::compile::{PackageStore, SourceMap, TargetProfile};
use qsc_passes::PackageType;

const INPUT: &str = include_str!("./large.qs");

pub fn large_file(c: &mut Criterion) {
    c.bench_function("Large input file", |b| {
        b.iter(|| {
            let mut store = PackageStore::new(compile::core());
            let std = store.insert(compile::std(&store, TargetProfile::Full));
            let sources = SourceMap::new([("large.qs".into(), INPUT.into())], None);
            let (_, reports) = compile(
                &store,
                &[std],
                sources,
                PackageType::Exe,
                TargetProfile::Full,
            );
            assert!(reports.is_empty());
        })
    });
}

criterion_group!(benches, large_file);
criterion_main!(benches);
