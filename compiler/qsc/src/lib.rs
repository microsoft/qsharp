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

pub mod fir {
    pub use qsc_fir::{fir::*, *};
}

pub mod hir {
    pub use qsc_hir::{hir::*, *};
}

pub mod ast {
    pub use qsc_ast::{ast::*, *};
}

pub use qsc_data_structures::span::Span;

pub use qsc_frontend::compile::TargetProfile;

pub use qsc_passes::PackageType;

pub use qsc_eval::{
    backend::{Backend, SparseSim},
    output::{fmt_basis_state_label, fmt_complex, format_state_id, get_phase},
};
