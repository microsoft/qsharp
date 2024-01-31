// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

pub mod compile;
pub mod error;
pub mod incremental;
pub mod interpret;
pub mod target;

pub use qsc_frontend::compile::{
    CompileUnit, PackageStore, RuntimeCapabilityFlags, SourceContents, SourceMap, SourceName,
};

pub mod resolve {
    pub use qsc_frontend::resolve::{Local, LocalKind, Locals, Res};
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

pub mod project {
    pub use qsc_project::{DirEntry, EntryType, FileSystem, Manifest, ManifestDescriptor};
}

pub use qsc_data_structures::span::Span;

pub use qsc_passes::{PackageType, PassContext};

pub mod line_column {
    pub use qsc_data_structures::line_column::{Encoding, Position, Range};
}

pub use qsc_eval::{
    backend::{Backend, SparseSim},
    output::{fmt_basis_state_label, fmt_complex, format_state_id, get_phase},
};
