// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::formatting::{DisplayableOutput, FormattingReceiver};
use pyo3::{exceptions::PyException, prelude::*, types::PyList, types::PyTuple};
use qsc::stateful;
use qsc_eval::val::Value;
use qsc_frontend::compile::SourceMap;

#[pymodule]
fn _native(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Interpreter>()?;
    m.add_class::<Result>()?;
    m.add_class::<Pauli>()?;
    m.add_class::<Output>()?;
    m.add_class::<Error>()?;

    Ok(())
}

#[pyclass(unsendable)]
pub(crate) struct Interpreter {
    pub(crate) interpreter: stateful::Interpreter,
}

#[pymethods]
/// A Q# interpreter.
impl Interpreter {
    #[new]
    /// Initializes a new Q# interpreter.
    pub(crate) fn new(_py: Python) -> PyResult<Self> {
        match stateful::Interpreter::new(true, SourceMap::default()) {
            Ok(interpreter) => Ok(Self { interpreter }),
            Err(errors) => Err(PyException::new_err(format!("{errors:?}"))),
        }
    }

    /// Interprets a line of Q#.
    ///
    /// :param expr: The line of Q# to interpret.
    ///
    /// :returns (value, outputs, errors):
    ///    value: The value of the last statement in the line.
    ///    outputs: A list of outputs from the line. An output can be a state or a message.
    ///    errors: A list of errors from the line. Errors can be compilation or runtime errors.
    #[pyo3(text_signature = "(expr)")]
    fn interpret(&mut self, py: Python, expr: &str) -> PyResult<(PyObject, PyObject, PyObject)> {
        let mut receiver = FormattingReceiver::new();
        let (value, errors) = match self.interpreter.line(expr, &mut receiver) {
            Ok(value) => (value, Vec::<stateful::Error>::new()),
            Err(errs) => (Value::unit(), errs),
        };
        let outputs = receiver.outputs;

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
                    .map(|e| Py::new(py, Error::from(e)).unwrap()),
            )
            .into_py(py),
        ))
    }
}

#[pyclass(unsendable)]
/// An error returned from the Q# interpreter.
pub(crate) struct Error {
    #[pyo3(get, set)]
    error_type: String,
    #[pyo3(get, set)]
    message: String,
}

#[pymethods]
/// An error returned from the Q# interpreter.
impl Error {
    fn __repr__(&self) -> String {
        format!("{}: {}", self.error_type, self.message)
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

impl From<stateful::Error> for Error {
    fn from(e: stateful::Error) -> Error {
        match e {
            stateful::Error::Compile(e) => {
                panic!("Did not expect compilation error {}", e)
            }
            stateful::Error::Eval(e) => Error {
                error_type: String::from("RuntimeError"),
                message: e.to_string(),
            },
            stateful::Error::Incremental(e) => Error {
                error_type: String::from("CompilationError"),
                message: e.to_string(),
            },
        }
    }
}

#[pyclass(unsendable)]
pub(crate) struct Output(DisplayableOutput);

#[pymethods]
/// An output returned from the Q# interpreter.
/// Outputs can be a state dumps or messages. These are normally printed to the console.
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
/// A Q# measurement result.
pub(crate) enum Result {
    Zero,
    One,
}

#[pyclass(unsendable)]
/// A Q# Pauli operator.
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
            Value::Double(val) => val.into_py(py),
            Value::Bool(val) => val.into_py(py),
            Value::String(val) => val.into_py(py),
            Value::Result(val) => if val { Result::One } else { Result::Zero }.into_py(py),
            Value::Pauli(val) => match val {
                qsc_hir::hir::Pauli::I => Pauli::I.into_py(py),
                qsc_hir::hir::Pauli::X => Pauli::X.into_py(py),
                qsc_hir::hir::Pauli::Y => Pauli::Y.into_py(py),
                qsc_hir::hir::Pauli::Z => Pauli::Z.into_py(py),
            },
            Value::Tuple(val) => {
                PyTuple::new(py, val.iter().map(|v| ValueWrapper(v.clone()).into_py(py)))
                    .into_py(py)
            }
            Value::Array(val) => {
                PyList::new(py, val.iter().map(|v| ValueWrapper(v.clone()).into_py(py))).into_py(py)
            }
            _ => format!("<{}> {}", Value::type_name(&self.0), &self.0).into_py(py),
        }
    }
}
