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

    let callables = items_with_test_attribute
        .filter(|(_, item)| matches!(item.kind, qsc::hir::ItemKind::Callable(_)));

    let callable_names = callables
        .filter_map(|(_, item)| -> Option<Result<String, String>>{
            if let qsc::hir::ItemKind::Callable(callable) = &item.kind {
                if !callable.generics.is_empty() {
                    return Some(Err(format!("Callable {} has generic type parameters. Test callables cannot have generic type parameters.", callable.name.name)));
                }
                if callable.input.kind != PatKind::Tuple(vec![]) {
                    return Some(Err(format!("Callable {} has input parameters. Test callables cannot have input parameters.", callable.name.name)));
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

                Some(Ok(name))
            } else {
                None
            }
        })
        .collect::<Result<_, _>>()?;

    Ok(callable_names)
}
