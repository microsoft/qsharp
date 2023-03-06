// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

mod compile;
pub mod id;
mod lex;
mod parse;
pub mod symbol;

pub use compile::{compile, CompiledPackage, Context, Error, FileIndex, PackageStore};
