// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

mod compile;
mod id;
mod lex;
pub mod parse;
pub mod symbol;

pub use compile::{compile, Context, Error, SourceId};
