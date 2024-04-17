// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_codegen::qir::fir_to_qir;
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_frontend::compile::{PackageStore, SourceMap, TargetCapabilityFlags};
use qsc_partial_eval::ProgramEntry;
use qsc_passes::{PackageType, PassContext};
use qsc_rca::Analyzer;

use crate::compile;

pub fn get_qir(
    sources: SourceMap,
    language_features: LanguageFeatures,
    capabilities: TargetCapabilityFlags,
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
    let (fir_store, fir_package_id) = qsc_passes::lower_hir_to_fir(&package_store, package_id);
    let package = fir_store.get(fir_package_id);
    let entry = ProgramEntry {
        exec_graph: package.entry_exec_graph.clone(),
        expr: (
            fir_package_id,
            package
                .entry
                .expect("package must have an entry expression"),
        )
            .into(),
    };

    let compute_properties = if capabilities == TargetCapabilityFlags::empty() {
        // baseprofchk already handled compliance, run the analyzer to get the compute properties.
        let analyzer = Analyzer::init(&fir_store);
        Ok(analyzer.analyze_all())
    } else {
        PassContext::run_fir_passes_on_fir(&fir_store, fir_package_id, capabilities)
    };

    let Ok(compute_properties) = compute_properties else {
        // This should never happen, as the program should be checked for errors before trying to
        // generate code for it. But just in case, simply report the failure.
        return Err("Failed to generate QIR".to_string());
    };

    fir_to_qir(&fir_store, capabilities, Some(compute_properties), &entry)
        .map_err(|e| e.to_string())
}
