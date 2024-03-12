// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(not(target_family = "wasm"))]
#[global_allocator]
static GLOBAL: allocator::Mimalloc = allocator::Mimalloc;

mod displayable_output;
mod fs;
mod interpreter;
