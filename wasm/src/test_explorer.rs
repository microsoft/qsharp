use qsc::{
    compile,
    hir::{Attr, PatKind},
    PackageType,
};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    project_system::{into_qsc_args, ProgramConfig},
    STORE_CORE_STD,
};

#[wasm_bindgen]
pub fn collect_test_callables(config: ProgramConfig) -> Result<Vec<String>, String> {
    let (source_map, capabilities, language_features, _store, _deps) =
        into_qsc_args(config, None).map_err(super::compile_errors_into_qsharp_errors_json)?;

    let package = STORE_CORE_STD.with(|(store, std)| {
        let (unit, _) = compile::compile(
            store,
            &[(*std, None)],
            source_map,
            PackageType::Lib,
            capabilities,
            language_features,
        );
        unit.package
    });

    let items_with_test_attribute = package
        .items
        .iter()
        .filter(|(_, item)| item.attrs.iter().any(|attr| *attr == Attr::Test));

    let (callables, others): (Vec<_>, Vec<_>) = items_with_test_attribute.partition(|(_, item)| {
        log::info!("item parent: {:?}", item.parent);
        matches!(item.kind, qsc::hir::ItemKind::Callable(_))
    });

    if !others.is_empty() {
        todo!("Return pretty error for non-callable with test attribute")
    }

    let callable_names = callables
        .iter()
        .filter_map(|(_, item)| {
            if let qsc::hir::ItemKind::Callable(callable) = &item.kind {
                if !callable.generics.is_empty() {
                    todo!("Return pretty error for generic callable with test attribute")
                }
                if callable.input.kind != PatKind::Tuple(vec![]) {
                    todo!("Return pretty error for callable with input")
                }
                // this is indeed a test callable, so let's grab its parent name
                let name = match item.parent {
                    None => Default::default(),
                    Some(parent_id) => {
                        let parent_item = package
                            .items
                            .get(parent_id)
                            .expect("Parent item did not exist in package");
                        if let qsc::hir::ItemKind::Namespace(ns, _) = &parent_item.kind {
                            format!("{}.{}", ns.name(), callable.name.name)
                        } else {
                            callable.name.name.to_string()
                        }
                    }
                };

                Some(name)
            } else {
                None
            }
        })
        .collect();

    Ok(callable_names)
}
