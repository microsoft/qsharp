// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::{
    compile::{self, Error},
    hir::{Item, ItemId, Package, PackageId},
    CompileUnit, PackageStore, PackageType, SourceMap, Span, TargetProfile,
};

pub(crate) const QSHARP_LIBRARY_URI_SCHEME: &str = "qsharp-library-source";

/// Represents an immutable compilation state that can be used
/// to implement language service features.
pub(crate) struct Compilation {
    pub package_store: PackageStore,
    pub std_package_id: PackageId,
    pub unit: CompileUnit,
    pub errors: Vec<Error>,
}

pub(crate) fn compile_document(
    source_name: &str,
    source_contents: &str,
    package_type: PackageType,
) -> Compilation {
    let mut package_store = PackageStore::new(compile::core());
    let std_package_id = package_store.insert(compile::std(&package_store, TargetProfile::Full));

    // Source map only contains the current document.
    let source_map = SourceMap::new([(source_name.into(), source_contents.into())], None);
    let (unit, errors) = compile::compile(
        &package_store,
        &[std_package_id],
        source_map,
        package_type,
        TargetProfile::Full,
    );
    Compilation {
        package_store,
        std_package_id,
        unit,
        errors,
    }
}

pub(crate) fn span_contains(span: Span, offset: u32) -> bool {
    offset >= span.lo && offset < span.hi
}

pub(crate) fn map_offset(source_map: &SourceMap, source_name: &str, source_offset: u32) -> u32 {
    source_map
        .find_by_name(source_name)
        .expect("source should exist in the source map")
        .offset
        + source_offset
}

pub(crate) fn find_item<'a>(
    compilation: &'a Compilation,
    id: &ItemId,
) -> (Option<&'a Item>, Option<&'a Package>) {
    let package = if let Some(package_id) = id.package {
        match compilation.package_store.get(package_id) {
            Some(compilation) => &compilation.package,
            None => return (None, None),
        }
    } else {
        &compilation.unit.package
    };
    (package.items.get(id.item), Some(package))
}
