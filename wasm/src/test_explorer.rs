// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::{compile, PackageType};
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

    package.collect_test_callables()
}
