// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(all(
    not(target_family = "wasm"),
    not(all(system = "windows", target_arch = "arm"))
))]
pub mod mimalloc;

/// Declare a global allocator if the platform supports it.
#[macro_export]
macro_rules! assign_global {
    () => {
        #[cfg(all(
            not(target_family = "wasm"),
            not(all(system = "windows", target_arch = "arm"))
        ))]
        #[global_allocator]
        static GLOBAL: allocator::mimalloc::Mimalloc = allocator::mimalloc::Mimalloc;
    };
}
