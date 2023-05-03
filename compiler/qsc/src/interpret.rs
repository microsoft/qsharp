// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod debug;
pub mod stateful;
pub mod stateless;

pub use qsc_eval::{
    output::{self, GenericReceiver},
    val::Value,
};
