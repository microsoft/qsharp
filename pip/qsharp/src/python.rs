// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::formatting::{DisplayableOutput, FormattingReceiver};
use pyo3::{exceptions::PyException, prelude::*, types::PyList, types::PyTuple};
use qsc_eval::{
    stateful::{Error, Interpreter},
    val::Value,
};

#[pymodule]
fn _native(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Evaluator>()?;
    m.add_class::<Result>()?;
    m.add_class::<Pauli>()?;
    m.add_class::<Output>()?;
    m.add_class::<ExecutionError>()?;

    Ok(())
}

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
    fn eval(&mut self, py: Python, expr: String) -> PyResult<(PyObject, PyObject, PyObject)> {
        let mut receiver = FormattingReceiver::new();
        let results = self
            .interpreter
            .line(&mut receiver, expr)
            .collect::<Vec<_>>();
        let outputs = receiver.outputs;

        // TODO: Figure out what to do with multiple statements
        let (value, errors) = match results.last() {
            Some(r) => match r.to_owned() {
                Ok(value) => (value, Vec::<Error>::new()),
                Err(err) => (Value::UNIT, { err.0 }),
            },
            None => (Value::UNIT, Vec::<Error>::new()),
        };

        Ok((
            ValueWrapper(value).into_py(py),
            PyList::new(
                py,
                outputs.into_iter().map(|o| Py::new(py, Output(o)).unwrap()),
            )
            .into_py(py),
            PyList::new(
                py,
                errors
                    .into_iter()
                    .map(|e| Py::new(py, ExecutionError::from(e)).unwrap()),
            )
            .into_py(py),
        ))
    }
}

#[pyclass(unsendable)]
pub(crate) struct ExecutionError {
    #[pyo3(get, set)]
    error_type: String,
    #[pyo3(get, set)]
    message: String,
}

#[pymethods]
impl ExecutionError {
    fn __repr__(&self) -> String {
        format!("{}: {}", self.error_type, self.message)
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

impl From<Error> for ExecutionError {
    fn from(e: Error) -> ExecutionError {
        match e {
            Error::Compile(e) => {
                panic!("Did not expect compilation error {}", e.to_string())
            }
            Error::Eval(e) => ExecutionError {
                error_type: String::from("RuntimeError"),
                message: e.to_string(),
            },
            Error::Incremental(e) => ExecutionError {
                error_type: String::from("CompilationError"),
                message: e.to_string(),
            },
        }
    }
}

#[pyclass(unsendable)]
pub(crate) struct Output(DisplayableOutput);

#[pymethods]
impl Output {
    fn __repr__(&self) -> String {
        match &self.0 {
            DisplayableOutput::State(state) => state.to_plain(),
            DisplayableOutput::Message(msg) => msg.clone(),
        }
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    fn _repr_html_(&self) -> String {
        match &self.0 {
            DisplayableOutput::State(state) => state.to_html(),
            DisplayableOutput::Message(msg) => msg.clone(),
        }
    }
}

#[pyclass(unsendable)]
pub(crate) enum Result {
    Zero,
    One,
}

#[pyclass(unsendable)]
pub(crate) enum Pauli {
    I,
    X,
    Y,
    Z,
}

// Mapping of Q# value types to Python value types.
struct ValueWrapper(Value);

impl IntoPy<PyObject> for ValueWrapper {
    fn into_py(self, py: Python) -> PyObject {
        match self.0 {
            Value::Int(val) => val.into_py(py),
            Value::Bool(val) => val.into_py(py),
            Value::String(val) => val.into_py(py),
            Value::Result(val) => if val { Result::One } else { Result::Zero }.into_py(py),
            Value::Pauli(val) => match val {
                qsc_ast::ast::Pauli::I => Pauli::I.into_py(py),
                qsc_ast::ast::Pauli::X => Pauli::X.into_py(py),
                qsc_ast::ast::Pauli::Y => Pauli::Y.into_py(py),
                qsc_ast::ast::Pauli::Z => Pauli::Z.into_py(py),
            },
            Value::Tuple(val) => {
                PyTuple::new(py, val.into_iter().map(|v| ValueWrapper(v).into_py(py))).into_py(py)
            }
            Value::Array(val) => {
                PyList::new(py, val.into_iter().map(|v| ValueWrapper(v).into_py(py))).into_py(py)
            }
            _ => format!("<{}> {}", Value::type_name(&self.0), &self.0).into_py(py),
        }
    }
}
