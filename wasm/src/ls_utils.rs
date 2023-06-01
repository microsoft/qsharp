// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::{compile, hir::PackageId, PackageStore, SourceMap};
use qsc_hir::hir::Package;

pub(crate) fn span_contains(span: qsc_data_structures::span::Span, offset: u32) -> bool {
    offset >= span.lo && offset < span.hi
}

pub(crate) fn get_compilation(
    source_path: &str,
    code: &str,
) -> (
    Package,
    Package,
    SourceMap,
    bool,
    std::vec::Vec<qsc::compile::Error>,
) {
    thread_local! {
        static STORE_STD: (PackageStore, PackageId) = {
            let mut store = PackageStore::new(compile::core());
            let std = store.insert(compile::std(&store));
            (store, std)
        };
    }

    STORE_STD.with(|(store, std)| {
        let std_compile_unit = store.get(*std).expect("expected to find std package");

        let sources = SourceMap::new([(source_path.into(), code.into())], None);
        // Argh, cloning the source map :(
        let (compile_unit, errors) = compile::compile(store, &[*std], sources.clone());

        let no_compilation = compile_unit.package.items.values().next().is_none();
        (
            std_compile_unit.package.clone(),
            compile_unit.package,
            sources,
            no_compilation,
            errors,
        )
    })
}
