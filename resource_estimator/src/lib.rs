// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod counts;
mod estimates;

pub use estimates::estimate_physical_resources_from_json;

use counts::LogicalCounter;
use estimates::estimate_physical_resources;
use miette::Diagnostic;
use qsc::interpret::{self, GenericReceiver, Interpreter};
use thiserror::Error;

#[derive(Debug)]
pub struct LogicalResources {
    pub num_qubits: usize,
    pub t_count: usize,
    pub rotation_count: usize,
    pub rotation_depth: usize,
    pub ccz_count: usize,
    pub measurement_count: usize,
}

#[derive(Debug, Diagnostic, Error)]
#[error(transparent)]
#[diagnostic(transparent)]
pub enum Error {
    Interpreter(interpret::Error),
    Estimation(estimates::Error),
}

pub fn estimate_entry(interpreter: &mut Interpreter, params: &str) -> Result<String, Vec<Error>> {
    let mut counter = LogicalCounter::default();
    let mut stdout = std::io::sink();
    let mut out = GenericReceiver::new(&mut stdout);
    interpreter
        .eval_entry_with_sim(&mut counter, &mut out)
        .map_err(|e| e.into_iter().map(Error::Interpreter).collect::<Vec<_>>())?;
    estimate_physical_resources(&counter.logical_resources(), params)
        .map_err(|e| vec![Error::Estimation(e)])
}

pub fn estimate_expr(
    interpreter: &mut Interpreter,
    expr: &str,
    params: &str,
) -> Result<String, Vec<Error>> {
    let mut counter = LogicalCounter::default();
    let mut stdout = std::io::sink();
    let mut out = GenericReceiver::new(&mut stdout);
    interpreter
        .run_with_sim(&mut counter, &mut out, expr)
        .map_err(|e| e.into_iter().map(Error::Interpreter).collect::<Vec<_>>())?
        .map_err(|e| vec![Error::Interpreter(e[0].clone())])?;
    estimate_physical_resources(&counter.logical_resources(), params)
        .map_err(|e| vec![Error::Estimation(e)])
}
