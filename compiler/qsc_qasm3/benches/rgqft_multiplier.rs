// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qsc_qasm3::{
    compile_to_qsharp_ast_with_config, io::InMemorySourceResolver, CompilerConfig, OutputSemantics,
    ProgramType, QubitSemantics,
};

pub fn rgqft_multiplier_1q(c: &mut Criterion) {
    const SOURCE: &str = include_str!("./rgqft_multiplier_1q.qasm");

    c.bench_function("rgqft_multiplier_1q sample compilation", |b| {
        let all_sources = [("rgqft_multiplier_1q.qasm".into(), SOURCE.into())];
        let mut resolver = InMemorySourceResolver::from_iter(all_sources);

        b.iter(move || {
            let config = CompilerConfig::new(
                QubitSemantics::Qiskit,
                OutputSemantics::Qiskit,
                ProgramType::File,
                Some("Test".into()),
                None,
            );
            black_box(compile_to_qsharp_ast_with_config(
                SOURCE,
                "",
                Some(&mut resolver),
                config,
            ))
        });
    });
}

pub fn rgqft_multiplier_4q(c: &mut Criterion) {
    const SOURCE: &str = include_str!("./rgqft_multiplier_4q.qasm");

    c.bench_function("rgqft_multiplier_4q sample compilation", |b| {
        let all_sources = [("rgqft_multiplier_4q.qasm".into(), SOURCE.into())];
        let mut resolver = InMemorySourceResolver::from_iter(all_sources);

        b.iter(move || {
            let config = CompilerConfig::new(
                QubitSemantics::Qiskit,
                OutputSemantics::Qiskit,
                ProgramType::File,
                Some("Test".into()),
                None,
            );
            black_box(compile_to_qsharp_ast_with_config(
                SOURCE,
                "",
                Some(&mut resolver),
                config,
            ))
        });
    });
}

criterion_group!(benches, rgqft_multiplier_1q, rgqft_multiplier_4q);
criterion_main!(benches);
