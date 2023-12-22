// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Tests the memory usage of the compiler.

// TODO:
// - different memory benchmarks
// - generate report for PR
// - compare against base branch

use jemalloc_ctl::{epoch, stats};
use libc::{c_char, c_void};
use qsc::compile;
use qsc_frontend::compile::{PackageStore, RuntimeCapabilityFlags};
use std::ptr::{null, null_mut};

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub fn library() {
    let store = PackageStore::new(compile::core());
    compile::std(&store, RuntimeCapabilityFlags::all());
}

extern "C" fn write_cb(_: *mut c_void, message: *const c_char) {
    print!(
        "{}",
        String::from_utf8_lossy(unsafe {
            std::ffi::CStr::from_ptr(message as *const i8).to_bytes()
        })
    );
}

fn mem_print() {
    unsafe { jemalloc_sys::malloc_stats_print(Some(write_cb), null_mut(), null()) }
}

fn main() {
    epoch::advance().unwrap();

    let before_allocated = stats::allocated::read().unwrap();
    library();
    epoch::advance().unwrap();

    let after_allocated = stats::allocated::read().unwrap();
    println!(
        "{} allocated during compilation",
        after_allocated - before_allocated
    );
}
