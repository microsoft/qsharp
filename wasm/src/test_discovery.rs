// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::serializable_type;

serializable_type! {
    TestDescriptor,
    {
        pub callable_name: String,
        pub location: crate::line_column::Location,
    },
    r#"export interface ITestDescriptor {
        callableName: string; 
        location: ILocation;
    }"#,
    ITestDescriptor
}
