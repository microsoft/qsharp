// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::io::Cursor;

use pyo3::{exceptions::PyException, prelude::*};
use qsc_eval::{output::CursorReceiver, stateful::Interpreter};

#[pyclass(unsendable)]
pub(crate) struct Evaluator {
    pub(crate) interpreter: Interpreter,
}

#[pymethods]
impl Evaluator {
    #[new]
    /// Initializes a new Q# evaluator.
    pub(crate) fn new(_py: Python) -> PyResult<Self> {
        const SOURCES: [&str; 0] = [];
        let result = Interpreter::new(true, SOURCES);
        match result {
            Ok(interpreter) => Ok(Self { interpreter }),
            Err((err, _)) => Err(PyException::new_err(format!("{:?}", err))),
        }
    }

    /// Evaluates a Q# expression.
    ///
    /// returns: A tuple of the expression's result and simulation data.
    /// .0 is the result of the expression.
    /// .1 is the output from the simulation.
    /// .2 is the error output.
    #[pyo3(text_signature = "(expr)")]
    fn eval(&mut self, py: Python, expr: String) -> PyResult<(PyObject, PyObject)> {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let mut receiver = CursorReceiver::new(&mut cursor);
        let results = self
            .interpreter
            .line(&mut receiver, expr)
            .collect::<Vec<_>>();

        // TODO: Handle multiple results.
        let result = &results[0];
        match result {
            Ok(value) => Ok((
                value.to_string().to_object(py),
                receiver.dump().to_object(py),
            )),
            Err(err) => Err(PyException::new_err(format!("{:?}", err))),
        }
    }
}

#[pymodule]
fn _native(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Evaluator>()?;

    Ok(())
}
