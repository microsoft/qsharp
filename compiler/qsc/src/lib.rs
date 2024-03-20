// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod compile;
pub mod error;
pub mod incremental;
pub mod interpret;
pub mod location;
pub mod target;

pub use qsc_formatter::formatter;

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

pub use qsc_data_structures::{language_features::LanguageFeatures, span::Span};

pub use qsc_passes::{PackageType, PassContext};

pub mod line_column {
    pub use qsc_data_structures::line_column::{Encoding, Position, Range};
}

pub use qsc_eval::{
    backend::{Backend, SparseSim},
    state::{fmt_basis_state_label, fmt_complex, format_state_id, get_latex, get_phase},
};

pub mod linter {
    pub use qsc_linter::{run_lints, LintConfig, LintKind, LintLevel};
}

pub use qsc_doc_gen::{display, generate_docs};

pub mod circuit {
    pub use qsc_circuit::{operations::*, Circuit, Operation};
}
