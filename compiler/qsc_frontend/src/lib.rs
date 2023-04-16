// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

pub mod compile;
pub mod diagnostic;
pub mod incremental;
mod lex;
mod lower;
mod parse;
pub mod resolve;
pub mod typeck;
mod validate;
