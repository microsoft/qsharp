// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use criterion::{criterion_group, criterion_main, Criterion};
use indoc::indoc;
use qsc::{interpret::Interpreter, PackageType};
use qsc_eval::output::GenericReceiver;
use qsc_frontend::compile::{RuntimeCapabilityFlags, SourceMap};

const TELEPORT: &str = include_str!("../../../samples/algorithms/Teleportation.qs");
const DEUTSCHJOZSA: &str = include_str!("../../../samples/algorithms/DeutschJozsa.qs");
const LARGE: &str = include_str!("./large.qs");

pub fn teleport(c: &mut Criterion) {
    c.bench_function("Teleport evaluation", |b| {
        let sources = SourceMap::new([("Teleportation.qs".into(), TELEPORT.into())], None);
        let mut evaluator = Interpreter::new(
            true,
            sources,
            PackageType::Exe,
            RuntimeCapabilityFlags::all(),
        )
        .expect("code should compile");
        b.iter(move || {
            let mut out = Vec::new();
            let mut rec = GenericReceiver::new(&mut out);
            assert!(evaluator.eval_entry(&mut rec).is_ok());
        })
    });
}

pub fn deutsch_jozsa(c: &mut Criterion) {
    c.bench_function("Deutsch-Jozsa evaluation", |b| {
        let sources = SourceMap::new([("DeutschJozsa.qs".into(), DEUTSCHJOZSA.into())], None);
        let mut evaluator = Interpreter::new(
            true,
            sources,
            PackageType::Exe,
            RuntimeCapabilityFlags::all(),
        )
        .expect("code should compile");
        b.iter(move || {
            let mut out = Vec::new();
            let mut rec = GenericReceiver::new(&mut out);
            assert!(evaluator.eval_entry(&mut rec).is_ok());
        })
    });
}

pub fn large_file(c: &mut Criterion) {
    c.bench_function("Large file parity evaluation", |b| {
        let sources = SourceMap::new([("large.qs".into(), LARGE.into())], None);
        let mut evaluator = Interpreter::new(
            true,
            sources,
            PackageType::Exe,
            RuntimeCapabilityFlags::all(),
        )
        .expect("code should compile");
        b.iter(move || {
            let mut out = Vec::new();
            let mut rec = GenericReceiver::new(&mut out);
            assert!(evaluator.eval_entry(&mut rec).is_ok());
        })
    });
}

pub fn array_append(c: &mut Criterion) {
    c.bench_function("Array append evaluation", |b| {
        let sources = SourceMap::new(
            [("none".into(), "".into())],
            Some(
                indoc! {"{
            mutable arr = [];
            for i in 0..999 {
                set arr += [i];
            }
            arr
        }"}
                .into(),
            ),
        );
        let mut evaluator = Interpreter::new(
            true,
            sources,
            PackageType::Exe,
            RuntimeCapabilityFlags::all(),
        )
        .expect("code should compile");
        b.iter(move || {
            let mut out = Vec::new();
            let mut rec = GenericReceiver::new(&mut out);
            assert!(evaluator.eval_entry(&mut rec).is_ok());
        })
    });
}

pub fn array_update(c: &mut Criterion) {
    c.bench_function("Array update evaluation", |b| {
        let sources = SourceMap::new(
            [("none".into(), "".into())],
            Some(
                indoc! {"{
            mutable arr = [0, size = 10000];
            for i in 0..999 {
                set arr w/= i <- i;
            }
            arr
        }"}
                .into(),
            ),
        );
        let mut evaluator = Interpreter::new(
            true,
            sources,
            PackageType::Exe,
            RuntimeCapabilityFlags::all(),
        )
        .expect("code should compile");
        b.iter(move || {
            let mut out = Vec::new();
            let mut rec = GenericReceiver::new(&mut out);
            assert!(evaluator.eval_entry(&mut rec).is_ok());
        })
    });
}

criterion_group!(
    benches,
    teleport,
    deutsch_jozsa,
    large_file,
    array_append,
    array_update
);
criterion_main!(benches);
