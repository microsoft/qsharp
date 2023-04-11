// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// TODO: merge: I'm sure there are unused stuff here
use pyo3::{exceptions::PyException, prelude::*, types::PyList, types::PyTuple};
use qsc_eval::{output::CursorReceiver, stateful::Interpreter, val::Value};
use std::io::Cursor;

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
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let mut receiver = CursorReceiver::new(&mut cursor);
        let results = self
            .interpreter
            .line(&mut receiver, expr)
            .collect::<Vec<_>>();

        // TODO: hoooo boy
        return match results[0].to_owned() {
            Ok(value) => Ok((
                ValueWrapper(value).into_py(py),
                receiver.dump().to_object(py),
                PyList::empty(py).to_object(py),
            )),
            Err(err) => Ok((
                PyTuple::empty(py).to_object(py),
                receiver.dump().to_object(py),
                {
                    let list = PyList::empty(py);
                    err.0
                        .into_iter()
                        .map(|e| match e {
                            qsc_eval::stateful::Error::Compile(e) => {
                                panic!("Did not expect compilation error {}", e.to_string())
                            }
                            qsc_eval::stateful::Error::Eval(e) => ExecutionError {
                                error_type: "RuntimeError".to_string(),
                                message: e.to_string(),
                            },
                            qsc_eval::stateful::Error::Incremental(e) => ExecutionError {
                                error_type: "CompilationError".to_string(),
                                message: e.to_string(),
                            },
                        })
                        .for_each(|e| {
                            list.append(e.into_py(py)).unwrap(); // TODO: Argh
                            ()
                        });
                    list.to_object(py)
                },
            )),
        };
    }
}

#[pymodule]
fn _native(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Evaluator>()?;
    m.add_class::<Result>()?;
    m.add_class::<Pauli>()?;

    Ok(())
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
        format!("{}: {}", self.error_type, self.message)
    }
}

struct ValueWrapper(Value);

impl IntoPy<PyObject> for ValueWrapper {
    fn into_py(self, py: Python) -> PyObject {
        match self.0 {
            Value::Int(val) => val.into_py(py),
            Value::Result(val) => if val { Result::One } else { Result::Zero }.into_py(py),
            Value::Bool(val) => val.into_py(py),
            Value::Pauli(val) => PauliWrapper(val).into_py(py),
            _ => format!("<{}> {}", Value::type_name(&self.0), &self.0).into_py(py),
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

struct PauliWrapper(qsc_ast::ast::Pauli);

impl IntoPy<PyObject> for PauliWrapper {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self.0 {
            qsc_ast::ast::Pauli::I => Pauli::I.into_py(py),
            qsc_ast::ast::Pauli::X => Pauli::X.into_py(py),
            qsc_ast::ast::Pauli::Y => Pauli::Y.into_py(py),
            qsc_ast::ast::Pauli::Z => Pauli::Z.into_py(py),
        }
    }
}
