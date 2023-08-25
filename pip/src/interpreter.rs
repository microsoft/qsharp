// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::displayable_output::{DisplayableOutput, DisplayableState};
use miette::Report;
use num_bigint::BigUint;
use num_complex::Complex64;
use pyo3::{
    create_exception, exceptions::PyException, prelude::*, pyclass::CompareOp, types::PyList,
    types::PyTuple,
};
use qsc::{
    fir,
    interpret::{
        output::{Error, Receiver},
        stateful::{self, LineError},
        Value,
    },
    PackageType, SourceMap,
};
use std::{fmt::Write, sync::Arc};

#[pymodule]
fn _native(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<TargetProfile>()?;
    m.add_class::<Interpreter>()?;
    m.add_class::<Result>()?;
    m.add_class::<Pauli>()?;
    m.add_class::<Output>()?;
    m.add("QSharpError", py.get_type::<QSharpError>())?;

    Ok(())
}

#[derive(Clone, Copy)]
#[pyclass(unsendable)]
/// A Q# target profile.
///
/// A target profile describes the capabilities of the hardware or simulator
/// which will be used to run the Q# program.
pub(crate) enum TargetProfile {
    /// Target supports the full set of capabilities required to run any Q# program.
    ///
    /// This option maps to the Full Profile as defined by the QIR specification.
    Full,
    /// Target supports the minimal set of capabilities required to run a quantum program.
    ///
    /// This option maps to the Base Profile as defined by the QIR specification.
    Base,
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
    pub(crate) fn new(_py: Python, target: TargetProfile) -> PyResult<Self> {
        let target = match target {
            TargetProfile::Full => qsc::TargetProfile::Full,
            TargetProfile::Base => qsc::TargetProfile::Base,
        };
        match stateful::Interpreter::new(true, SourceMap::default(), PackageType::Lib, target) {
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

    #[allow(clippy::unused_self)]
    #[allow(clippy::unnecessary_wraps)]
    fn run(
        &mut self,
        py: Python,
        entry_expr: &str,
        shots: u32,
        callback: Option<PyObject>,
    ) -> PyResult<Py<PyList>> {
        let mut receiver = OptionalCallbackReceiver { callback, py };
        match self.interpreter.run(&mut receiver, entry_expr, shots) {
            Ok(results) => Ok(PyList::new(
                py,
                results.into_iter().map(|res| match res {
                    Ok(v) => ValueWrapper(v).into_py(py),
                    Err(errors) => {
                        QSharpError::new_err(format_errors(entry_expr, errors)).into_py(py)
                    }
                }),
            )
            .into_py(py)),
            Err(errors) => Err(QSharpError::new_err(format_errors(entry_expr, errors))),
        }
    }

    fn qir(&mut self, _py: Python, entry_expr: &str) -> PyResult<String> {
        match self.interpreter.qirgen(entry_expr) {
            Ok(qir) => Ok(qir),
            Err(errors) => Err(QSharpError::new_err(format_errors(entry_expr, errors))),
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
#[derive(PartialEq)]
/// A Q# measurement result.
pub(crate) enum Result {
    Zero,
    One,
}

#[pymethods]
impl Result {
    fn __repr__(&self) -> String {
        match self {
            Result::Zero => "Zero".to_owned(),
            Result::One => "One".to_owned(),
        }
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    fn __hash__(&self) -> u32 {
        match self {
            Result::Zero => 0,
            Result::One => 1,
        }
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        let this = i32::from(*self == Result::One);
        let other = i32::from(*other == Result::One);
        match op {
            CompareOp::Lt => this < other,
            CompareOp::Le => this <= other,
            CompareOp::Eq => this == other,
            CompareOp::Ne => this != other,
            CompareOp::Gt => this > other,
            CompareOp::Ge => this >= other,
        }
    }
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
            Value::Result(val) => if val.unwrap_bool() {
                Result::One
            } else {
                Result::Zero
            }
            .into_py(py),
            Value::Pauli(val) => match val {
                fir::Pauli::I => Pauli::I.into_py(py),
                fir::Pauli::X => Pauli::X.into_py(py),
                fir::Pauli::Y => Pauli::Y.into_py(py),
                fir::Pauli::Z => Pauli::Z.into_py(py),
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
