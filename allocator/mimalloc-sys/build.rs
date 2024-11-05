// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::boxed::Box;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    compile_mimalloc();
    let build_dir = get_build_dir()?;
    println!(
        "cargo:rerun-if-changed={}",
        build_dir.join("mimalloc").display()
    );
    Ok(())
}

// Compile mimalloc source code and link it to the crate.
// The cc crate is used to compile the source code into a static library.
// We don't use the cmake crate to compile the source code because the mimalloc build system
// loads extra libraries, changes the name and path around, and does other things that are
// difficult to handle. The cc crate is much simpler and more predictable.
fn compile_mimalloc() {
    let mimalloc_vendor_dir = PathBuf::from("mimalloc");

    let mut build = cc::Build::new();

    let include_dir = mimalloc_vendor_dir.join("include");
    let src_dir = mimalloc_vendor_dir.join("src");
    let static_file = src_dir.join("static.c");

    assert!(include_dir.exists(), "include_dir: {include_dir:?}");
    assert!(src_dir.exists(), "src_dir: {src_dir:?}");
    assert!(static_file.exists(), "static_file: {static_file:?}");

    build.include(include_dir);
    build.include(src_dir);
    build.file(static_file);

    if build.get_compiler().is_like_msvc() {
        build.static_crt(true);
    }
    // turn off debug mode
    build.define("MI_DEBUG", "0");

    // turning on optimizations doesn't seem to make a difference
    //build.opt_level(3);

    // log the command that will be run
    build.cargo_debug(true);

    // turn off warnings from the mimalloc code
    build.cargo_warnings(false);

    build.compile("mimalloc");
}

fn get_build_dir() -> Result<PathBuf, Box<dyn Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let build_dir = PathBuf::from(manifest_dir.as_str());
    let normalized_build_dir = fs::canonicalize(build_dir)?;
    Ok(normalized_build_dir)
}
