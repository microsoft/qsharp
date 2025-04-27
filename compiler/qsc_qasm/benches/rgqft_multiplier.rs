// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qsc_qasm::{
    compile_to_qsharp_ast_with_config, io::InMemorySourceResolver, CompilerConfig, OutputSemantics,
    ProgramType, QasmCompileUnit, QubitSemantics,
};

fn rgqft_multiplier(source: &str) -> QasmCompileUnit {
    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::File,
        Some("Test".into()),
        None,
    );
    compile_to_qsharp_ast_with_config(source, "", None::<&mut InMemorySourceResolver>, config)
}

pub fn rgqft_multiplier_1q(c: &mut Criterion) {
    const SOURCE: &str = include_str!("./rgqft_multiplier_1q.qasm");

    c.bench_function("rgqft_multiplier_1q sample compilation", |b| {
        b.iter(move || black_box(rgqft_multiplier(SOURCE)));
    });
}

pub fn rgqft_multiplier_4q(c: &mut Criterion) {
    const SOURCE: &str = include_str!("./rgqft_multiplier_4q.qasm");

    c.bench_function("rgqft_multiplier_4q sample compilation", |b| {
        b.iter(move || black_box(rgqft_multiplier(SOURCE)));
    });
}

criterion_group!(benches, rgqft_multiplier_1q, rgqft_multiplier_4q);
criterion_main!(benches);
