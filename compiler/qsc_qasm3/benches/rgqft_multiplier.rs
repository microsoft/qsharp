// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use criterion::{criterion_group, criterion_main, Criterion};
use qsc_qasm3::{
    compile_to_qsharp_ast_with_config, io::InMemorySourceResolver, CompilerConfig, OutputSemantics,
    ProgramType, QubitSemantics,
};

const SOURCE: &str = include_str!("./rgqft_multiplier.qasm");

pub fn rgqft_multiplier(c: &mut Criterion) {
    c.bench_function("rgqft_multiplier sample compilation", |b| {
        let all_sources = [("rgqft_multiplier.qasm".into(), SOURCE.into())];
        let mut resolver = InMemorySourceResolver::from_iter(all_sources);

        b.iter(move || {
            let config = CompilerConfig::new(
                QubitSemantics::Qiskit,
                OutputSemantics::Qiskit,
                ProgramType::File,
                Some("Test".into()),
                None,
            );
            compile_to_qsharp_ast_with_config(SOURCE, "", Some(&mut resolver), config);
        });
    });
}

criterion_group!(benches, rgqft_multiplier);
criterion_main!(benches);
