// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    displayable_output::{DisplayableOutput, DisplayableState},
    fs::file_system,
};
use miette::Report;
use num_bigint::BigUint;
use num_complex::Complex64;
use pyo3::{
    create_exception,
    exceptions::PyException,
    prelude::*,
    pyclass::CompareOp,
    types::PyList,
    types::{PyDict, PyString, PyTuple},
};
use qsc::{
    fir,
    interpret::{
        self,
        output::{Error, Receiver},
        Value,
    },
    project::{FileSystem, Manifest, ManifestDescriptor},
    target::Profile,
    PackageType, SourceMap,
};
use resource_estimator::{self as re, estimate_expr};
use rustc_hash::FxHashMap;
use std::fmt::Write;

#[pymodule]
fn _native(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<TargetProfile>()?;
    m.add_class::<Interpreter>()?;
    m.add_class::<Result>()?;
    m.add_class::<Pauli>()?;
    m.add_class::<Output>()?;
    m.add_class::<StateDump>()?;
    m.add_function(wrap_pyfunction!(physical_estimates, m)?)?;
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
    Unrestricted,
    /// Target supports the minimal set of capabilities required to run a quantum program.
    ///
    /// This option maps to the Base Profile as defined by the QIR specification.
    Base,
}

#[pyclass(unsendable)]
pub(crate) struct Interpreter {
    pub(crate) interpreter: interpret::Interpreter,
}

pub(crate) struct PyManifestDescriptor(ManifestDescriptor);

impl FromPyObject<'_> for PyManifestDescriptor {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let dict = ob.downcast::<PyDict>()?;
        let manifest_dir = get_dict_opt_string(dict, "manifest_dir")?.ok_or(
            PyException::new_err("missing key `manifest_dir` in manifest descriptor"),
        )?;
        let manifest = dict
            .get_item("manifest")?
            .ok_or(PyException::new_err(
                "missing key `manifest` in manifest descriptor",
            ))?
            .downcast::<PyDict>()?;

        Ok(Self(ManifestDescriptor {
            manifest: Manifest {
                author: get_dict_opt_string(manifest, "author")?,
                license: get_dict_opt_string(manifest, "license")?,
            },
            manifest_dir: manifest_dir.into(),
        }))
    }
}

#[pymethods]
/// A Q# interpreter.
impl Interpreter {
    #[new]
    /// Initializes a new Q# interpreter.
    pub(crate) fn new(
        py: Python,
        target: TargetProfile,
        manifest_descriptor: Option<PyManifestDescriptor>,
        read_file: Option<PyObject>,
        list_directory: Option<PyObject>,
    ) -> PyResult<Self> {
        let target = match target {
            TargetProfile::Unrestricted => Profile::Unrestricted,
            TargetProfile::Base => Profile::Base,
        };

        let sources = if let Some(manifest_descriptor) = manifest_descriptor {
            let project = file_system(
                py,
                read_file.expect(
                    "file system hooks should have been passed in with a manifest descriptor",
                ),
                list_directory.expect(
                    "file system hooks should have been passed in with a manifest descriptor",
                ),
            )
            .load_project(&manifest_descriptor.0)
            .map_py_err()?;
            SourceMap::new(project.sources, None)
        } else {
            SourceMap::default()
        };

        match interpret::Interpreter::new(true, sources, PackageType::Lib, target.into()) {
            Ok(interpreter) => Ok(Self { interpreter }),
            Err(errors) => Err(QSharpError::new_err(format_errors(errors))),
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
        match self.interpreter.eval_fragments(&mut receiver, input) {
            Ok(value) => Ok(ValueWrapper(value).into_py(py)),
            Err(errors) => Err(QSharpError::new_err(format_errors(errors))),
        }
    }

    /// Sets the quantum seed for the interpreter.
    fn set_quantum_seed(&mut self, seed: Option<u64>) {
        self.interpreter.set_quantum_seed(seed);
    }

    /// Sets the classical seed for the interpreter.
    fn set_classical_seed(&mut self, seed: Option<u64>) {
        self.interpreter.set_classical_seed(seed);
    }

    /// Dumps the quantum state of the interpreter.
    /// Returns a tuple of (amplitudes, num_qubits), where amplitudes is a dictionary from integer indices to
    /// pairs of real and imaginary amplitudes.
    fn dump_machine(&mut self) -> StateDump {
        let (state, qubit_count) = self.interpreter.get_quantum_state();
        StateDump(DisplayableState(
            state.into_iter().collect::<FxHashMap<_, _>>(),
            qubit_count,
        ))
    }

    fn run(
        &mut self,
        py: Python,
        entry_expr: &str,
        callback: Option<PyObject>,
    ) -> PyResult<PyObject> {
        let mut receiver = OptionalCallbackReceiver { callback, py };
        match self.interpreter.run(&mut receiver, entry_expr) {
            Ok(result) => match result {
                Ok(v) => Ok(ValueWrapper(v).into_py(py)),
                Err(errors) => Err(QSharpError::new_err(format_errors(errors))),
            },
            Err(errors) => Err(QSharpError::new_err(format_errors(errors))),
        }
    }

    fn qir(&mut self, _py: Python, entry_expr: &str) -> PyResult<String> {
        match self.interpreter.qirgen(entry_expr) {
            Ok(qir) => Ok(qir),
            Err(errors) => Err(QSharpError::new_err(format_errors(errors))),
        }
    }

    fn estimate(&mut self, _py: Python, entry_expr: &str, job_params: &str) -> PyResult<String> {
        match estimate_expr(&mut self.interpreter, entry_expr, job_params) {
            Ok(estimate) => Ok(estimate),
            Err(errors) if matches!(errors[0], re::Error::Interpreter(_)) => {
                Err(QSharpError::new_err(format_errors(
                    errors
                        .into_iter()
                        .map(|e| match e {
                            re::Error::Interpreter(e) => e,
                            re::Error::Estimation(_) => unreachable!(),
                        })
                        .collect::<Vec<_>>(),
                )))
            }
            Err(errors) => Err(QSharpError::new_err(
                errors
                    .into_iter()
                    .map(|e| match e {
                        re::Error::Estimation(e) => e.to_string(),
                        re::Error::Interpreter(_) => unreachable!(),
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            )),
        }
    }
}

#[pyfunction]
pub fn physical_estimates(logical_resources: &str, job_params: &str) -> PyResult<String> {
    match re::estimate_physical_resources_from_json(logical_resources, job_params) {
        Ok(estimates) => Ok(estimates),
        Err(error) => Err(QSharpError::new_err(error.to_string())),
    }
}

create_exception!(
    module,
    QSharpError,
    pyo3::exceptions::PyException,
    "An error returned from the Q# interpreter."
);

fn format_errors(errors: Vec<interpret::Error>) -> String {
    errors
        .into_iter()
        .map(|e| {
            let mut message = String::new();
            if let Some(stack_trace) = e.stack_trace() {
                write!(message, "{stack_trace}").unwrap();
            }
            let additional_help = python_help(&e);
            let report = Report::new(e);
            write!(message, "{report:?}").unwrap();
            if let Some(additional_help) = additional_help {
                writeln!(message, "{additional_help}").unwrap();
            }
            message
        })
        .collect::<String>()
}

/// Additional help text for an error specific to the Python module
fn python_help(error: &interpret::Error) -> Option<String> {
    if matches!(error, interpret::Error::UnsupportedRuntimeCapabilities) {
        Some("Unsupported target profile. Initialize Q# by running `qsharp.init(target_profile=qsharp.TargetProfile.Base)` before performing code generation.".into())
    } else {
        None
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
            DisplayableOutput::Message(msg) => format!("<p>{msg}</p>"),
        }
    }

    fn state_dump(&self) -> Option<StateDump> {
        match &self.0 {
            DisplayableOutput::State(state) => Some(StateDump(state.clone())),
            DisplayableOutput::Message(_) => None,
        }
    }
}

#[pyclass(unsendable)]
/// Captured simlation state dump.
pub(crate) struct StateDump(pub(crate) DisplayableState);

#[pymethods]
impl StateDump {
    fn get_dict(&self, py: Python) -> PyResult<Py<PyDict>> {
        Ok(PyDict::from_sequence(
            py,
            PyList::new(
                py,
                self.0
                     .0
                    .iter()
                    .map(|(k, v)| {
                        PyTuple::new(
                            py,
                            &[k.clone().into_py(py), PyTuple::new(py, [v.re, v.im]).into()],
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .into_py(py),
        )?
        .into_py(py))
    }

    #[getter]
    fn get_qubit_count(&self) -> usize {
        self.0 .1
    }

    // Pass by value is needed for compatiblity with the pyo3 API.
    #[allow(clippy::needless_pass_by_value)]
    fn __getitem__(&self, key: BigUint) -> Option<(f64, f64)> {
        self.0 .0.get(&key).map(|state| (state.re, state.im))
    }

    fn __len__(&self) -> usize {
        self.0 .0.len()
    }

    fn __repr__(&self) -> String {
        self.0.to_plain()
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    fn _repr_html_(&self) -> String {
        self.0.to_html()
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
            Value::BigInt(val) => val.into_py(py),
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
            let state = state.into_iter().collect::<FxHashMap<_, _>>();
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

trait MapPyErr<T, E> {
    fn map_py_err(self) -> core::result::Result<T, PyErr>;
}

impl<T, E> MapPyErr<T, E> for core::result::Result<T, E>
where
    E: IntoPyErr,
{
    fn map_py_err(self) -> core::result::Result<T, PyErr>
    where
        E: IntoPyErr,
    {
        self.map_err(IntoPyErr::into_py_err)
    }
}

trait IntoPyErr {
    fn into_py_err(self) -> PyErr;
}

impl IntoPyErr for Report {
    fn into_py_err(self) -> PyErr {
        PyException::new_err(format!("{self:?}"))
    }
}

impl IntoPyErr for Vec<interpret::Error> {
    fn into_py_err(self) -> PyErr {
        let mut message = String::new();
        for error in self {
            writeln!(message, "{error}").expect("string should be writable");
        }
        PyException::new_err(message)
    }
}

fn get_dict_opt_string(dict: &PyDict, key: &str) -> PyResult<Option<String>> {
    let value = dict.get_item(key)?;
    Ok(match value {
        Some(item) => Some(item.downcast::<PyString>()?.to_string_lossy().into()),
        None => None,
    })
}
