// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::displayable_output::{DisplayableOutput, DisplayableState};
use miette::Report;
use num_bigint::BigUint;
use num_complex::Complex64;
use pyo3::{create_exception, exceptions::PyException, prelude::*, types::PyList, types::PyTuple};
use qsc::{
    hir,
    interpret::{
        output::{Error, Receiver},
        stateful::{self, LineError},
        Value,
    },
    SourceMap,
};
use std::{fmt::Write, sync::Arc};

#[pymodule]
fn _native(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Interpreter>()?;
    m.add_class::<Result>()?;
    m.add_class::<Pauli>()?;
    m.add_class::<Output>()?;
    m.add("QSharpError", py.get_type::<QSharpError>())?;

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
            Err(errors) => {
                let mut message = String::new();
                for error in errors {
                    writeln!(message, "{error}").expect("string should be writable");
                }
                Err(PyException::new_err(message))
            }
        }
    }

    /// Interprets Q# source code.
    ///
    /// :param input: The Q# source code to interpret.
    /// :param output_fn: A callback function that will be called with each output.
    ///
    /// :returns value: The value returned by the last statement in the input.
    ///
    /// :raises QSharpError: If there is an error interpreting the input.
    fn interpret(
        &mut self,
        py: Python,
        input: &str,
        callback: Option<PyObject>,
    ) -> PyResult<PyObject> {
        let mut receiver = OptionalCallbackReceiver { callback, py };
        match self.interpreter.interpret_line(&mut receiver, input) {
            Ok(value) => Ok(ValueWrapper(value).into_py(py)),
            Err(errors) => Err(QSharpError::new_err(format_errors(input, errors))),
        }
    }
}

create_exception!(
    module,
    QSharpError,
    pyo3::exceptions::PyException,
    "An error returned from the Q# interpreter."
);

fn format_errors(expr: &str, errors: Vec<LineError>) -> String {
    errors
        .into_iter()
        .map(|e| {
            let mut message = String::new();
            if let Some(stack_trace) = e.stack_trace() {
                write!(message, "{stack_trace}").unwrap();
            }
            let report = Report::new(e).with_source_code(Arc::new(expr.to_owned()));
            write!(message, "{report:?}").unwrap();
            message
        })
        .collect::<String>()
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
            DisplayableOutput::Message(msg) => format!("<p>{msg}</p>"),
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
                hir::Pauli::I => Pauli::I.into_py(py),
                hir::Pauli::X => Pauli::X.into_py(py),
                hir::Pauli::Y => Pauli::Y.into_py(py),
                hir::Pauli::Z => Pauli::Z.into_py(py),
            },
            Value::Tuple(val) => {
                if val.is_empty() {
                    // Special case Value::unit as None
                    py.None()
                } else {
                    PyTuple::new(py, val.iter().map(|v| ValueWrapper(v.clone()).into_py(py)))
                        .into_py(py)
                }
            }
            Value::Array(val) => {
                PyList::new(py, val.iter().map(|v| ValueWrapper(v.clone()).into_py(py))).into_py(py)
            }
            _ => format!("<{}> {}", Value::type_name(&self.0), &self.0).into_py(py),
        }
    }
}

struct OptionalCallbackReceiver<'a> {
    callback: Option<PyObject>,
    py: Python<'a>,
}

impl Receiver for OptionalCallbackReceiver<'_> {
    fn state(
        &mut self,
        state: Vec<(BigUint, Complex64)>,
        qubit_count: usize,
    ) -> core::result::Result<(), Error> {
        if let Some(callback) = &self.callback {
            let out = DisplayableOutput::State(DisplayableState(state, qubit_count));
            callback
                .call1(
                    self.py,
                    PyTuple::new(
                        self.py,
                        &[Py::new(self.py, Output(out)).expect("should be able to create output")],
                    ),
                )
                .map_err(|_| Error)?;
        }
        Ok(())
    }

    fn message(&mut self, msg: &str) -> core::result::Result<(), Error> {
        if let Some(callback) = &self.callback {
            let out = DisplayableOutput::Message(msg.to_owned());
            callback
                .call1(
                    self.py,
                    PyTuple::new(
                        self.py,
                        &[Py::new(self.py, Output(out)).expect("should be able to create output")],
                    ),
                )
                .map_err(|_| Error)?;
        }
        Ok(())
    }
}
