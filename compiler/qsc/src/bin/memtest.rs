// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Tests the memory usage of the compiler.

// TODO:
// - different memory benchmarks
// - generate report for PR
// - compare against base branch

use jemalloc_ctl::{epoch, stats};
use qsc::{compile, CompileUnit};
use qsc_frontend::compile::{PackageStore, RuntimeCapabilityFlags};

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub fn library() -> CompileUnit {
    let store = PackageStore::new(compile::core());
    compile::std(&store, RuntimeCapabilityFlags::all())
}

fn main() {
    epoch::advance().unwrap();

    let before_allocated = stats::allocated::read().unwrap();
    let _ = library();
    epoch::advance().unwrap();

    let after_allocated = stats::allocated::read().unwrap();
    let std = after_allocated - before_allocated;
    println!(
        r#"# Memory Report
| Test         | This Branch | On Main |
|--------------|-------------|---------|
| standard lib | {std}       |         |
"#
    );
}
