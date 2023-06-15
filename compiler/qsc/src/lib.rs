// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

pub mod compile;
mod error;
pub mod interpret;

pub use qsc_frontend::compile::{PackageStore, SourceContents, SourceMap, SourceName};

pub mod hir {
    pub use qsc_hir::{hir::*, *};
}

pub use qsc_data_structures::span::Span;
