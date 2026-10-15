// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Native bindings of ⑤ `qdk.simulation.tensornetwork_qir`: turn the Python
//! queries into calls to the ④ MPS method, and its results back into Python.
//!
//! ```text
//! _tensor_network.py ─ program dict ─► _tensor_network_sample       ─► run_mps_shots          ─► shot records
//!                    ─ query dicts ──► _tensor_network_state_query  ─► evaluate_state_queries ─► one value per query
//! ```
//!
//! - Both functions prepare the program with ③'s `PreparedAdaptiveProgram` and
//!   hand it to ④ (`qdk_cutensornet`), which validates it on the host, before
//!   device discovery, and runs it on cuTensorNet. Nothing here touches the
//!   GPU or checks the program beyond decoding it.
//! - Errors keep ④'s classification: `MpsExecutionError::is_environment_error`
//!   (no library, no device, a resource limit) becomes `OSError`, and anything
//!   else (unsupported operation, invalid input) `ValueError`.
//! - The GIL is held during GPU work.
//! - The Python side (`qdk/simulation/_tensor_network.py`) validates arguments,
//!   lowers QIR to 64-bit bytecode, encodes the query dicts and maps shot
//!   records through output recording. A new query or option changes both
//!   files and `qdk/_native.pyi`; `qdk_cutensornet`'s `execution.rs` lists the
//!   steps.

use super::{adaptive_program_from_pydict, cpu_simulators::output_records_to_pylist};
use num_complex::Complex64;
use pyo3::{
    exceptions::{PyOSError, PyValueError},
    prelude::*,
    types::{PyDict, PyList},
};
use qdk_cutensornet::{
    MpsCost, MpsExecutionError, StateMethod, StateQuery, StateQueryValue, evaluate_state_queries,
    run_mps_shots,
};
use qdk_simulators::execution::{PauliSum, PreparedAdaptiveProgram};

/// Samples `shots` shots of a Base program from one MPS (the batch strategy)
/// and returns each shot's output records.
///
/// `input` is the program as `AdaptiveProfilePass(Bytecode.Bit64)` lowers it.
/// `seed` reproduces the shots; `None` draws one from entropy.
/// `max_bond_dimension` caps the MPS bonds at χ; `None` keeps the default.
#[pyfunction]
#[pyo3(signature = (input, shots, seed=None, max_bond_dimension=None))]
pub(crate) fn _tensor_network_sample<'py>(
    py: Python<'py>,
    input: &Bound<'py, PyDict>,
    shots: u32,
    seed: Option<u32>,
    max_bond_dimension: Option<u32>,
) -> PyResult<Py<PyAny>> {
    let prepared = prepare(input)?;
    let output =
        run_mps_shots(&prepared, shots, seed, max_bond_dimension).map_err(mps_execution_error)?;
    output_records_to_pylist(py, output)
}

/// Evaluates `Expectation` queries on |ψ⟩, the MPS a Base program reaches
/// before its terminal measurements, and reports `Cost` for that MPS. Returns
/// one value per query, in order.
///
/// `queries` holds the dicts built by `qdk.simulation.tensornetwork_qir`.
/// `max_bond_dimension` caps the MPS bonds at χ; `None` keeps the default.
#[pyfunction]
#[pyo3(signature = (input, queries, max_bond_dimension=None))]
pub(crate) fn _tensor_network_state_query<'py>(
    py: Python<'py>,
    input: &Bound<'py, PyDict>,
    queries: &Bound<'py, PyList>,
    max_bond_dimension: Option<u32>,
) -> PyResult<Bound<'py, PyList>> {
    let mut state_queries = Vec::with_capacity(queries.len());
    for query in queries {
        let kind: String = query.get_item("kind")?.extract()?;
        state_queries.push(match kind.as_str() {
            "expectation" => StateQuery::Expectation(pauli_sum(&query)?),
            "cost" => StateQuery::Cost,
            _ => {
                return Err(PyValueError::new_err(format!(
                    "unsupported state query: {kind}"
                )));
            }
        });
    }

    let prepared = prepare(input)?;
    let method = StateMethod::Mps { max_bond_dimension };
    let values =
        evaluate_state_queries(&prepared, &state_queries, method).map_err(mps_execution_error)?;
    let results = PyList::empty(py);
    for value in values {
        match value {
            StateQueryValue::Expectation(value) => results.append(value)?,
            StateQueryValue::Cost(cost) => results.append(mps_cost_dict(py, cost)?)?,
        }
    }
    Ok(results)
}

fn prepare(input: &Bound<'_, PyDict>) -> PyResult<PreparedAdaptiveProgram<u64>> {
    let program = adaptive_program_from_pydict::<u64>(input)?;
    PreparedAdaptiveProgram::new(program).map_err(|error| PyValueError::new_err(error.to_string()))
}

fn pauli_sum(query: &Bound<'_, PyAny>) -> PyResult<PauliSum> {
    let terms: Vec<(String, Vec<usize>, Complex64)> = query.get_item("terms")?.extract()?;
    let mut sum = PauliSum::new();
    for (labels, qubits, coefficient) in terms {
        sum.push_labels(coefficient, &labels, &qubits)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
    }
    Ok(sum)
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "used directly with Result::map_err"
)]
fn mps_execution_error(error: MpsExecutionError) -> PyErr {
    if error.is_environment_error() {
        PyOSError::new_err(error.to_string())
    } else {
        PyValueError::new_err(error.to_string())
    }
}

fn mps_cost_dict(py: Python<'_>, cost: MpsCost) -> PyResult<Bound<'_, PyDict>> {
    let report = PyDict::new(py);
    report.set_item("max_bond_dimension", cost.max_bond_dimension)?;
    report.set_item("state_bytes", cost.state_bytes)?;
    report.set_item("workspace_bytes", cost.workspace_bytes)?;
    Ok(report)
}
