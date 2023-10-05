// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

mod closure;
pub mod compile;
pub mod error;
pub mod incremental;
mod lower;
pub mod resolve;
pub mod typeck;
