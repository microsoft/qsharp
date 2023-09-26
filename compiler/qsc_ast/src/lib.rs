// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

pub mod assigner;
pub mod ast;
pub mod mut_visit;
pub mod validate;
pub mod visit;
