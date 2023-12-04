use crate::{
    single_pass_analysis::{AppIdx, ItemComputeProps, PackageComputeProps, SinglePassAnalyzer},
    RuntimeCapability,
};
use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_fir::fir::{Item, ItemKind, LocalItemId, Package, PackageId, PackageStore};
use rustc_hash::FxHashSet;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("target does not support callable capabilities {0}")]
    #[diagnostic(code("Qsc.RuntimeCapabilities.TargetDoesNotSuppoprtCallableCaps"))]
    TargetDoesNotSuppoprtCallableCaps(String, #[label] Span),
}

pub fn check_target_capabilities_compatibility(
    package_store: &PackageStore,
    main_package_id: PackageId,
    target_capabilities: &FxHashSet<RuntimeCapability>,
) -> Vec<Error> {
    let store_compute_props = SinglePassAnalyzer::run(package_store);
    store_compute_props.persist();
    let main_package_compute_props = store_compute_props
        .get_package(main_package_id)
        .expect("Main package compute properties should exist");
    let main_package = package_store
        .0
        .get(main_package_id)
        .expect("Package should exist");
    let mut errors = Vec::new();
    for (item_id, _) in main_package.items.iter() {
        let mut item_errors = check_item_capabilities_compatibility(
            item_id,
            main_package,
            &main_package_compute_props,
            target_capabilities,
        );
        errors.append(&mut item_errors);
    }
    errors
}

fn check_item_capabilities_compatibility(
    item_id: LocalItemId,
    package: &Package,
    package_compute_props: &PackageComputeProps,
    target_capabilities: &FxHashSet<RuntimeCapability>,
) -> Vec<Error> {
    let mut errors = Vec::new();
    let item = package.items.get(item_id).expect("Item should exist");
    if let ItemKind::Callable(callable) = &item.kind {
        let callable_compute_props = match package_compute_props
            .items
            .get(item_id)
            .expect("Item compute properties should exist")
        {
            ItemComputeProps::Callable(callable_compute_props) => callable_compute_props,
            _ => panic!("Item compute properties should be callable compute properties."),
        };

        let all_static_caps = callable_compute_props
            .apps
            .get(AppIdx::all_static_params_idx())
            .expect("Callable application should exist");
        let mut callable_unsupported_caps = Vec::new();
        for rt_cap in all_static_caps.rt_caps.difference(target_capabilities) {
            callable_unsupported_caps.push(rt_cap.clone());
        }
        if !callable_unsupported_caps.is_empty() {
            let mut callable_unsupported_caps_str = String::new();
            for unsupported_cap in callable_unsupported_caps {
                callable_unsupported_caps_str.push_str(&format!(" {unsupported_cap:?}").to_owned());
            }
            let error = Error::TargetDoesNotSuppoprtCallableCaps(
                callable_unsupported_caps_str,
                callable.name.span,
            );
            errors.push(error);
        }
    }
    errors
}
