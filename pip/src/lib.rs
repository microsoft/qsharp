// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

allocator::assign_global!();

mod cli;
mod displayable_output;
mod fs;
mod interpreter;

use miette::Report;
use pyo3::{exceptions::PyException, prelude::*};
use std::fmt::Write;

trait MapPyErr<T, E> {
    fn map_py_err(self) -> core::result::Result<T, PyErr>;
}

impl<T, E> MapPyErr<T, E> for core::result::Result<T, E>
where
    E: IntoPyErr,
{
    fn map_py_err(self) -> core::result::Result<T, PyErr>
    where
        E: IntoPyErr,
    {
        self.map_err(IntoPyErr::into_py_err)
    }
}

trait IntoPyErr {
    fn into_py_err(self) -> PyErr;
}

impl IntoPyErr for Report {
    fn into_py_err(self) -> PyErr {
        PyException::new_err(format!("{self:?}"))
    }
}

impl IntoPyErr for Vec<qsc::interpret::Error> {
    fn into_py_err(self) -> PyErr {
        let mut message = String::new();
        for error in self {
            writeln!(message, "{error}").expect("string should be writable");
        }
        PyException::new_err(message)
    }
}
