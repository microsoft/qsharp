// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::boxed::Box;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use cmake::Config;

// 1.8.2
//static ALLOCATOR_MIMALLOC_TAG: &str = "b66e3214d8a104669c2ec05ae91ebc26a8f5ab78";
// 2.1.2
static ALLOCATOR_MIMALLOC_TAG: &str = "43ce4bd7fd34bcc730c1c7471c99995597415488";

fn main() -> Result<(), Box<dyn Error>> {
    let dst = download_mimalloc()?;
    compile_mimalloc(&dst);
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=CMakeLists.txt");
    Ok(())
}

// Compile mimalloc source code and link it to the crate.
// The cc crate is used to compile the source code into a static library.
// The cmake crate is used to download the source code and stage it in the build directory.
// We don't use the cmake crate to compile the source code because the mimalloc build system
// loads extra libraries, changes the name and path around, and does other things that are
// difficult to handle. The cc crate is much simpler and more predictable.
fn compile_mimalloc(dst: &Path) {
    let src_dir = dst
        .join("build")
        .join("mimalloc-prefix")
        .join("src")
        .join("mimalloc");

    let mut build = cc::Build::new();

    build.include(src_dir.join("include"));
    build.include(src_dir.join("src"));
    build.file(src_dir.join("src/static.c"));

    if build.get_compiler().is_like_msvc() {
        build.cpp(true);
        build.static_crt(true);
    }
    // turn off debug mode
    build.define("MI_DEBUG", "0");

    // turning on optimizations doesn't seem to make a difference
    //build.opt_level(3);

    build.compile("mimalloc");

    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=mimalloc");
}

// Use cmake to download mimalloc source code and stage
// it in the build directory.
fn download_mimalloc() -> Result<PathBuf, Box<dyn Error>> {
    let build_dir = get_build_dir()?;
    let mut config = Config::new(build_dir);

    config
        .no_build_target(true)
        .env("ALLOCATOR_MIMALLOC_TAG", ALLOCATOR_MIMALLOC_TAG)
        .very_verbose(true);

    let dst = config.build();

    Ok(dst)
}

fn get_build_dir() -> Result<PathBuf, Box<dyn Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let build_dir = PathBuf::from(manifest_dir.as_str());
    let normalized_build_dir = fs::canonicalize(build_dir)?;
    Ok(normalized_build_dir)
}
