// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use pyo3::prelude::*;

use crate::eval::ExecutionContext;

#[pyclass(unsendable)]
pub(crate) struct Evaluator {
    pub(crate) context: ExecutionContext,
}

#[pymethods]
impl Evaluator {
    #[new]
    /// Initializes a new Q# evaluator.
    pub(crate) fn new(_py: Python) -> Self {
        Self {
            context: crate::eval::create_context(),
        }
    }

    /// Evaluates a Q# expression.
    ///
    /// returns: A tuple of the expression's result and simulation data.
    /// .0 is the result of the expression.
    /// .1 is the output from the simulation.
    /// .2 is the error output.
    #[pyo3(text_signature = "(expr)")]
    fn eval(&mut self, py: Python, expr: String) -> PyResult<(PyObject, PyObject, PyObject)> {
        let (value, out, err) = self.context.eval(expr);
        Ok((value.to_object(py), out.to_object(py), err.to_object(py)))
    }
}

#[pymodule]
fn _native(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Evaluator>()?;

    Ok(())
}
