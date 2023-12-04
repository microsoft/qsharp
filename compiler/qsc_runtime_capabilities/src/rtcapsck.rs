use crate::single_pass_analysis::SinglePassAnalyzer;
use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_fir::fir::{PackageId, PackageStore};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("target does not support capability {0}")]
    #[diagnostic(code("Qsc.RuntimeCapabilities.UnsupportedCapability"))]
    UnsupportedCapability(String, #[label] Span),
}

pub fn check_target_capabilities_compatibility(
    package_store: &PackageStore,
    main_package_id: PackageId,
) -> Vec<Error> {
    let store_compute_props = SinglePassAnalyzer::run(package_store);
    store_compute_props.persist();
    let _main_package = package_store
        .0
        .get(main_package_id)
        .expect("Package should exist");
    Vec::new()
}
