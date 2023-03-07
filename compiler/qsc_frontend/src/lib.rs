// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

mod compile;
pub mod id;
mod lex;
mod parse;
pub mod resolve;

pub use compile::{compile, CompileUnit, Context, Error, FileIndex, PackageStore};
