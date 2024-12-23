// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::{compile, PackageType};
use wasm_bindgen::prelude::wasm_bindgen;
use serde::{Serialize, Deserialize};


use crate::{
    project_system::{into_qsc_args, ProgramConfig}, serializable_type, STORE_CORE_STD
};

serializable_type! {
    TestDescriptor,
    {
        #[serde(rename = "callableName")]
        pub callable_name: String,
        pub location: crate::line_column::Location,
    },
    r#"export interface ITestDescriptor {
        callableName: string; 
        location: ILocation;
    }"#,
    ITestDescriptor
}

#[wasm_bindgen]
pub fn get_test_callables(config: ProgramConfig) -> Result<Vec<ITestDescriptor>, String> {
    let (source_map, capabilities, language_features, _store, _deps) =
        into_qsc_args(config, None).map_err(super::compile_errors_into_qsharp_errors_json)?;

    let compile_unit = STORE_CORE_STD.with(|(store, std)| {
        let (unit, _errs) = compile::compile(
            store,
            &[(*std, None)],
            source_map,
            PackageType::Lib,
            capabilities,
            language_features,
        ); 
        unit
    });


    let test_descriptors =  qsc::test_callables::get_test_callables(
        &compile_unit
    );

    Ok(test_descriptors.map(|qsc::test_callables::TestDescriptor { callable_name, location }| {
        TestDescriptor {
            callable_name,
            location: location.into(),
        }.into()
    }).collect())

}
