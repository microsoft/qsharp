// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod codegen;
pub mod compile;
pub mod error;
pub mod incremental;
pub mod interpret;
pub mod location;
pub mod packages;
pub mod target;

pub use qsc_formatter::formatter;

pub use qsc_frontend::compile::{CompileUnit, PackageStore, SourceContents, SourceMap, SourceName};

pub mod resolve {
    pub use qsc_frontend::resolve::{path_as_field_accessor, Local, LocalKind, Locals, Res};
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
    pub use qsc_project::{
        DirEntry, EntryType, Error, FileSystem, Manifest, ManifestDescriptor, PackageCache,
        PackageGraphSources,
    };
}

pub use qsc_data_structures::{
    language_features::LanguageFeatures, namespaces::*, span::Span, target::TargetCapabilityFlags,
};

pub use qsc_passes::{lower_hir_to_fir, PackageType, PassContext};

pub mod line_column {
    pub use qsc_data_structures::line_column::{Encoding, Position, Range};
}

pub use qsc_eval::{
    backend::{Backend, SparseSim},
    noise::PauliNoise,
    state::{
        fmt_basis_state_label, fmt_complex, format_state_id, get_matrix_latex, get_phase,
        get_state_latex,
    },
};

pub mod linter {
    pub use qsc_linter::{run_lints, LintConfig, LintKind, LintLevel};
}

pub use qsc_doc_gen::{display, generate_docs};

pub mod circuit {
    pub use qsc_circuit::{operations::*, Circuit, Operation};
}

pub mod parse {
    pub use qsc_parse::{completion, top_level_nodes};
}

pub mod partial_eval {
    pub use qsc_partial_eval::Error;
}
