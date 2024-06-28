// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::Diagnostic;
use thiserror::Error;

#[cfg(feature = "fs")]
#[derive(Error, Debug, Diagnostic)]
pub enum StdFsError {
    #[error("found a qsharp.json file, but it was invalid: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("failed to construct regular expression from excluded file item: {0}")]
    RegexError(#[from] regex_lite::Error),
}
