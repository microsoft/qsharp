// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(not(target_family = "wasm"))]
pub mod mimalloc;

/// Declare a global allocator if the platform supports it.
#[macro_export]
macro_rules! assign_global {
    () => {
        #[cfg(not(target_family = "wasm"))]
        #[global_allocator]
        static GLOBAL: allocator::mimalloc::Mimalloc = allocator::mimalloc::Mimalloc;
    };
}
