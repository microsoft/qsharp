// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

mod closure;
pub mod compile;
pub mod incremental;
mod lex;
mod lower;
pub mod parse;
pub mod resolve;
pub mod typeck;
