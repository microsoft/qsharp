// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic)]

use std::ffi::OsString;

use pyo3::prelude::*;

use crate::IntoPyErr;

fn map_args(args: Option<Vec<String>>) -> Vec<OsString> {
    match args {
        Some(args) => {
            // when invoked from python, but not from the command line
            args.into_iter()
                .map(std::convert::Into::into)
                .collect::<Vec<_>>()
        }
        _ => {
            // We are being invoked from the command line
            // skip the first arg, which is the name of the python executable
            std::env::args_os().skip(1).collect::<Vec<_>>()
        }
    }
}
#[pyfunction]
#[pyo3(text_signature = "(args)")]
pub(crate) fn exec_subcommand(args: Option<Vec<String>>) -> PyResult<()> {
    let args = map_args(args);

    if let Err(err) = ::qsc::cli::exec_subcommand(args) {
        Err(IntoPyErr::into_py_err(err))
    } else {
        Ok(())
    }
}

#[pyfunction]
#[pyo3(text_signature = "(args)")]
pub(crate) fn exec_qsc(args: Option<Vec<String>>) -> PyResult<()> {
    let args = map_args(args);

    if let Err(err) = ::qsc::cli::exec_qsc(args) {
        Err(IntoPyErr::into_py_err(err))
    } else {
        Ok(())
    }
}

#[pyfunction]
#[pyo3(text_signature = "(args)")]
pub(crate) fn exec_qsi(args: Option<Vec<String>>) -> PyResult<()> {
    let args = map_args(args);

    if let Err(err) = ::qsc::cli::exec_qsi(args) {
        Err(IntoPyErr::into_py_err(err))
    } else {
        Ok(())
    }
}
