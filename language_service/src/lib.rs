// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

pub mod completion;
pub mod definition;
mod display;
pub mod hover;
pub mod protocol;
mod qsc_utils;
#[cfg(test)]
mod test_utils;

pub use protocol::LanguageService;
