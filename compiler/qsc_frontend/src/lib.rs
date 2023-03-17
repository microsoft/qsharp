// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

pub mod compile;
pub mod diagnostic;
pub mod id;
mod lex;
pub mod parse;
pub mod resolve;
