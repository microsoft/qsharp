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

pub use qsc_eval::backend::{Backend, SparseSim};

/// # Panics
/// oh it panics
#[must_use]
pub fn compile_to_qir(qsharp: &str, entry_expr: Option<&str>, target: TargetProfile) -> String {
    let mut core = qsc_frontend::compile::core();
    assert!(qsc_passes::run_core_passes(&mut core).is_empty());
    let mut store = qsc_frontend::compile::PackageStore::new(core);
    let mut std = compile::std(&store, target);
    assert!(
        qsc_passes::run_default_passes(store.core(), &mut std, PackageType::Lib, target).is_empty()
    );
    let std = store.insert(std);

    let sources = SourceMap::new(
        [("test.qs".into(), qsharp.into())],
        entry_expr.map(std::convert::Into::into),
    );

    let mut unit = qsc_frontend::compile::compile(&store, &[std], sources, target);
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);
    assert!(
        qsc_passes::run_default_passes(store.core(), &mut unit, PackageType::Exe, target)
            .is_empty()
    );
    let package = store.insert(unit);

    qsc_codegen::qir_base::generate_qir(&store, package).expect("expected success!")
}
