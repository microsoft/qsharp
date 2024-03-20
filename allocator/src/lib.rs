// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(not(any(
    target_family = "wasm",
    all(target_family = "windows", target_arch = "aarch64")
)))]
pub mod mimalloc;

/// Declare a global allocator if the platform supports it.
#[macro_export]
macro_rules! assign_global {
    () => {
        #[cfg(not(any(
            target_family = "wasm",
            all(target_family = "windows", target_arch = "aarch64")
        )))]
        #[global_allocator]
        static GLOBAL: allocator::mimalloc::Mimalloc = allocator::mimalloc::Mimalloc;
    };
}
