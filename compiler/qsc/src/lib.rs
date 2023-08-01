// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

pub mod compile;
mod error;
pub mod interpret;

pub use qsc_frontend::compile::{CompileUnit, PackageStore, SourceContents, SourceMap, SourceName};

pub mod resolve {
    pub use qsc_frontend::resolve::Res;
}

pub mod hir {
    pub use qsc_hir::{hir::*, *};
}

pub mod ast {
    pub use qsc_ast::{ast::*, *};
}

pub use qsc_data_structures::span::Span;

pub use qsc_passes::PackageType;
