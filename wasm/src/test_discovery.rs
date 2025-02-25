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
        pub compilation_uri: String,
        pub friendly_name: String,
    },
    r#"export interface ITestDescriptor {
        callableName: string;
        location: ILocation;
        compilationUri: string;
        friendlyName: string;
    }"#,
    ITestDescriptor
}
