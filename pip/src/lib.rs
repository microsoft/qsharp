// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(not(target_family = "wasm"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod displayable_output;
mod fs;
mod interpreter;
