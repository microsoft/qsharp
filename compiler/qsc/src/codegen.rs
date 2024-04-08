// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_codegen::qir::hir_to_qir;
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_frontend::compile::{PackageStore, RuntimeCapabilityFlags, SourceMap};
use qsc_passes::{PackageType, PassContext};

use crate::compile;

pub fn get_qir(
    sources: SourceMap,
    language_features: LanguageFeatures,
    capabilities: RuntimeCapabilityFlags,
) -> Result<String, String> {
    let core = compile::core();
    let mut package_store = PackageStore::new(core);
    let std = compile::std(&package_store, capabilities);
    let std = package_store.insert(std);

    let (unit, errors) = crate::compile::compile(
        &package_store,
        &[std],
        sources,
        PackageType::Exe,
        capabilities,
        language_features,
    );

    // Ensure it compiles before trying to add it to the store.
    if !errors.is_empty() {
        // This should never happen, as the program should be checked for errors before trying to
        // generate code for it. But just in case, simply report the failure.
        return Err("Failed to generate QIR".to_string());
    }

    let package_id = package_store.insert(unit);

    let caps_results = PassContext::run_fir_passes_on_hir(&package_store, package_id, capabilities);
    // Ensure it compiles before trying to add it to the store.
    match caps_results {
        Ok(compute_properties) => hir_to_qir(
            &package_store,
            package_id,
            capabilities,
            Some(compute_properties),
        )
        .map_err(|e| e.to_string()),
        Err(_) => {
            // This should never happen, as the program should be checked for errors before trying to
            // generate code for it. But just in case, simply report the failure.
            Err("Failed to generate QIR".to_string())
        }
    }
}
