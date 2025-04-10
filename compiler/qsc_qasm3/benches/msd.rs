// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use criterion::{criterion_group, criterion_main, Criterion};
use qsc_qasm3::{
    compile_to_qsharp_ast_with_config, io::InMemorySourceResolver, CompilerConfig, OutputSemantics,
    ProgramType, QubitSemantics,
};

const SOURCE: &str = include_str!("./msd.qasm");

pub fn msd(c: &mut Criterion) {
    c.bench_function("msd sample compilation", |b| {
        let all_sources = [("msd.qasm".into(), SOURCE.into())];
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

criterion_group!(benches, msd);
criterion_main!(benches);
