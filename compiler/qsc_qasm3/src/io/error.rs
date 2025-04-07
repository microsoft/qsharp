// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::Diagnostic;
use thiserror::Error;

use super::Cycle;

#[derive(Clone, Debug, Diagnostic, Eq, Error, PartialEq)]
#[error(transparent)]
#[diagnostic(transparent)]
pub struct Error(pub ErrorKind);

#[derive(Clone, Debug, Diagnostic, Eq, Error, PartialEq)]
pub enum ErrorKind {
    #[error("Not Found {0}")]
    NotFound(String),
    #[error("IO Error: {0}")]
    IO(String),
    #[error("Multiple include: {0}")]
    MultipleInclude(String),
    #[error("Cyclic include:\n{0}")]
    CyclicInclude(Cycle),
}

impl From<Error> for crate::Error {
    fn from(val: Error) -> Self {
        crate::Error(crate::ErrorKind::IO(val))
    }
}
