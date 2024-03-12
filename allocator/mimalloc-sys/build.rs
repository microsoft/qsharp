// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::boxed::Box;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use cmake::Config;

// 1.8.2
//static ALLOCATOR_MIMALLOC_TAG: &str = "b66e3214d8a104669c2ec05ae91ebc26a8f5ab78";
// 2.1.2
static ALLOCATOR_MIMALLOC_TAG: &str = "43ce4bd7fd34bcc730c1c7471c99995597415488";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=CMakeLists.txt");

    compile_mimalloc()?;

    Ok(())
}

fn compile_mimalloc() -> Result<(), Box<dyn Error>> {
    let build_dir = get_build_dir()?;
    let mut config = Config::new(build_dir);

    config
        .define("CMAKE_BUILD_TYPE", "MinSizeRel")
        .define("MI_INSTALL_TOPLEVEL", "ON")
        .build_target("mimalloc")
        .env("ALLOCATOR_MIMALLOC_TAG", ALLOCATOR_MIMALLOC_TAG);

    let dst = config.build();
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=mimalloc");

    Ok(())
}

fn get_build_dir() -> Result<PathBuf, Box<dyn Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let build_dir = PathBuf::from(manifest_dir.as_str());
    let normalized_build_dir = fs::canonicalize(build_dir)?;
    Ok(normalized_build_dir)
}
