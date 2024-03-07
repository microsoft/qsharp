// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use criterion::{criterion_group, criterion_main, Criterion};
use qsc::incremental::Compiler;
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_eval::{debug::map_hir_package_to_fir, lower::Lowerer};
use qsc_fir::fir::PackageStore;
use qsc_frontend::compile::{RuntimeCapabilityFlags, SourceMap};
use qsc_passes::PackageType;
use qsc_rca::Analyzer;

const TELEPORT: &str = include_str!("../../../samples/algorithms/Teleportation.qs");
const DEUTSCHJOZSA: &str = include_str!("../../../samples/algorithms/DeutschJozsa.qs");
const LARGE: &str = include_str!("./large.qs");

pub fn teleport(c: &mut Criterion) {
    c.bench_function(
        "Perform Runtime Capabilities Analysis (RCA) on teleport sample",
        |b| {
            let sources = SourceMap::new([("Teleportation.qs".into(), TELEPORT.into())], None);
            let fir_store = compile_and_lower_to_fir(sources);
            b.iter(|| {
                let analyzer = Analyzer::init(&fir_store);
                let _compute_properties = analyzer.analyze_all();
            });
        },
    );
}

pub fn deutsch_jozsa(c: &mut Criterion) {
    c.bench_function(
        "Perform Runtime Capabilities Analysis (RCA) on Deutsch-Jozsa sample",
        |b| {
            let sources = SourceMap::new([("DeutschJozsa.qs".into(), DEUTSCHJOZSA.into())], None);
            let fir_store = compile_and_lower_to_fir(sources);
            b.iter(|| {
                let analyzer = Analyzer::init(&fir_store);
                let _compute_properties = analyzer.analyze_all();
            });
        },
    );
}

pub fn large_file(c: &mut Criterion) {
    c.bench_function(
        "Perform Runtime Capabilities Analysis (RCA) on large file sample",
        |b| {
            let sources = SourceMap::new([("Large.qs".into(), LARGE.into())], None);
            let fir_store = compile_and_lower_to_fir(sources);
            b.iter(|| {
                let analyzer = Analyzer::init(&fir_store);
                let _compute_properties = analyzer.analyze_all();
            });
        },
    );
}

fn compile_and_lower_to_fir(sources: SourceMap) -> PackageStore {
    let compiler = Compiler::new(
        true,
        sources,
        PackageType::Lib,
        RuntimeCapabilityFlags::all(),
        LanguageFeatures::default(),
    )
    .expect("should be able to create a new compiler");
    let mut lowerer = Lowerer::new();
    let mut fir_store = PackageStore::new();
    for (id, unit) in compiler.package_store() {
        fir_store.insert(
            map_hir_package_to_fir(id),
            lowerer.lower_package(&unit.package),
        );
    }
    fir_store
}

criterion_group!(benches, teleport, deutsch_jozsa, large_file);
criterion_main!(benches);
