// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    displayable_output::{DisplayableOutput, DisplayableState},
    fs::file_system,
    interop::{
        compile_qasm3_to_qir, compile_qasm3_to_qsharp, compile_qasm_enriching_errors,
        map_entry_compilation_errors, resource_estimate_qasm3, run_ast, run_qasm3, ImportResolver,
    },
    noisy_simulator::register_noisy_simulator_submodule,
};
use miette::{Diagnostic, Report};
use num_bigint::BigUint;
use num_complex::Complex64;
use pyo3::{
    create_exception,
    exceptions::PyException,
    prelude::*,
    types::{PyComplex, PyDict, PyList, PyTuple},
};
use qsc::{
    fir,
    interpret::{
        self,
        output::{Error, Receiver},
        CircuitEntryPoint, Value,
    },
    packages::BuildableProgram,
    project::{FileSystem, PackageCache, PackageGraphSources},
    target::Profile,
    LanguageFeatures, PackageType, SourceMap,
};

use resource_estimator::{self as re, estimate_expr};
use std::{cell::RefCell, fmt::Write, path::PathBuf, rc::Rc};

/// If the classes are not Send, the Python interpreter
/// will not be able to use them in a separate thread.
///
/// This function is used to verify that the classes are Send.
/// The code will fail to compile if the classes are not Send.
///
/// ### Note
/// `QSharpError`, and `QasmError` are not `Send`, *BUT*
/// we return `QasmError::new_err` or `QSharpError::new_err` which
/// actually returns a `PyErr` that is `Send` and the args passed
/// into the `new_err` call must also impl `Send`.
/// Because of this, we don't need to check the `Send`-ness of
/// them. On the Python side, the `PyErr` is converted into the
/// corresponding exception.
fn verify_classes_are_sendable() {
    fn is_send<T: Send>() {}
    is_send::<OutputSemantics>();
    is_send::<ProgramType>();
    is_send::<TargetProfile>();
    is_send::<Result>();
    is_send::<Pauli>();
    is_send::<Output>();
    is_send::<StateDumpData>();
    is_send::<Circuit>();
}

#[pymodule]
fn _native<'a>(py: Python<'a>, m: &Bound<'a, PyModule>) -> PyResult<()> {
    verify_classes_are_sendable();
    m.add_class::<OutputSemantics>()?;
    m.add_class::<ProgramType>()?;
    m.add_class::<TargetProfile>()?;
    m.add_class::<Interpreter>()?;
    m.add_class::<Result>()?;
    m.add_class::<Pauli>()?;
    m.add_class::<Output>()?;
    m.add_class::<StateDumpData>()?;
    m.add_class::<Circuit>()?;
    m.add_function(wrap_pyfunction!(physical_estimates, m)?)?;
    m.add("QSharpError", py.get_type_bound::<QSharpError>())?;
    register_noisy_simulator_submodule(py, m)?;
    // QASM3 interop
    m.add("QasmError", py.get_type_bound::<QasmError>())?;
    m.add_function(wrap_pyfunction!(resource_estimate_qasm3, m)?)?;
    m.add_function(wrap_pyfunction!(run_qasm3, m)?)?;
    m.add_function(wrap_pyfunction!(compile_qasm3_to_qir, m)?)?;
    m.add_function(wrap_pyfunction!(compile_qasm3_to_qsharp, m)?)?;
    Ok(())
}

// This ordering must match the _native.pyi file.
#[derive(Clone, Copy, PartialEq)]
#[pyclass(eq, eq_int)]
#[allow(non_camel_case_types)]
/// A Q# target profile.
///
/// A target profile describes the capabilities of the hardware or simulator
/// which will be used to run the Q# program.
pub(crate) enum TargetProfile {
    /// Target supports the minimal set of capabilities required to run a quantum program.
    ///
    /// This option maps to the Base Profile as defined by the QIR specification.
    Base,
    /// Target supports the Adaptive profile with integer computation and qubit reset capabilities.
    ///
    /// This profile includes all of the required Adaptive Profile
    /// capabilities, as well as the optional integer computation and qubit
    /// reset capabilities, as defined by the QIR specification.
    Adaptive_RI,
    /// Target supports the full set of capabilities required to run any Q# program.
    ///
    /// This option maps to the Full Profile as defined by the QIR specification.
    Unrestricted,
}

impl From<TargetProfile> for Profile {
    fn from(profile: TargetProfile) -> Self {
        match profile {
            TargetProfile::Base => Profile::Base,
            TargetProfile::Adaptive_RI => Profile::AdaptiveRI,
            TargetProfile::Unrestricted => Profile::Unrestricted,
        }
    }
}

// This ordering must match the _native.pyi file.
#[derive(Clone, Copy, PartialEq)]
#[pyclass(eq, eq_int)]
#[allow(non_camel_case_types)]
/// Represents the output semantics for OpenQASM 3 compilation.
/// Each has implications on the output of the compilation
/// and the semantic checks that are performed.
pub(crate) enum OutputSemantics {
    /// The output is in Qiskit format meaning that the output
    /// is all of the classical registers, in reverse order
    /// in which they were added to the circuit with each
    /// bit within each register in reverse order.
    Qiskit,
    /// [OpenQASM 3 has two output modes](https://openqasm.com/language/directives.html#input-output)
    /// - If the programmer provides one or more `output` declarations, then
    ///     variables described as outputs will be returned as output.
    ///     The spec make no mention of endianness or order of the output.
    /// - Otherwise, assume all of the declared variables are returned as output.
    OpenQasm,
    /// No output semantics are applied. The entry point returns `Unit`.
    ResourceEstimation,
}

impl From<OutputSemantics> for qsc_qasm3::OutputSemantics {
    fn from(output_semantics: OutputSemantics) -> Self {
        match output_semantics {
            OutputSemantics::Qiskit => qsc_qasm3::OutputSemantics::Qiskit,
            OutputSemantics::OpenQasm => qsc_qasm3::OutputSemantics::OpenQasm,
            OutputSemantics::ResourceEstimation => qsc_qasm3::OutputSemantics::ResourceEstimation,
        }
    }
}

// This ordering must match the _native.pyi file.
#[derive(Clone, PartialEq)]
#[pyclass(eq)]
#[allow(non_camel_case_types)]
/// Represents the type of compilation output to create
pub enum ProgramType {
    /// Creates an operation in a namespace as if the program is a standalone
    /// file. Inputs are lifted to the operation params. Output are lifted to
    /// the operation return type. The operation is marked as `@EntryPoint`
    /// as long as there are no input parameters.
    File,
    /// Programs are compiled to a standalone function. Inputs are lifted to
    /// the operation params. Output are lifted to the operation return type.
    Operation,
    /// Creates a list of statements from the program. This is useful for
    /// interactive environments where the program is a list of statements
    /// imported into the current scope.
    /// This is also useful for testing individual statements compilation.
    Fragments,
}

impl From<ProgramType> for qsc_qasm3::ProgramType {
    fn from(output_semantics: ProgramType) -> Self {
        match output_semantics {
            ProgramType::File => qsc_qasm3::ProgramType::File,
            ProgramType::Operation => qsc_qasm3::ProgramType::Operation,
            ProgramType::Fragments => qsc_qasm3::ProgramType::Fragments,
        }
    }
}

#[pyclass(unsendable)]
pub(crate) struct Interpreter {
    pub(crate) interpreter: interpret::Interpreter,
}

thread_local! { static PACKAGE_CACHE: Rc<RefCell<PackageCache>> = Rc::default(); }

#[pymethods]
/// A Q# interpreter.
impl Interpreter {
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::needless_pass_by_value)]
    #[pyo3(signature = (target_profile, language_features=None, project_root=None, read_file=None, list_directory=None, resolve_path=None, fetch_github=None))]
    #[new]
    /// Initializes a new Q# interpreter.
    pub(crate) fn new(
        py: Python,
        target_profile: TargetProfile,
        language_features: Option<Vec<String>>,
        project_root: Option<String>,
        read_file: Option<PyObject>,
        list_directory: Option<PyObject>,
        resolve_path: Option<PyObject>,
        fetch_github: Option<PyObject>,
    ) -> PyResult<Self> {
        let target = Into::<Profile>::into(target_profile).into();

        let language_features = LanguageFeatures::from_iter(language_features.unwrap_or_default());

        let package_cache = PACKAGE_CACHE.with(Clone::clone);

        let buildable_program = if let Some(project_root) = project_root {
            if let (Some(read_file), Some(list_directory), Some(resolve_path), Some(fetch_github)) =
                (read_file, list_directory, resolve_path, fetch_github)
            {
                let project =
                    file_system(py, read_file, list_directory, resolve_path, fetch_github)
                        .load_project(&PathBuf::from(project_root), Some(&package_cache))
                        .map_err(IntoPyErr::into_py_err)?;

                if !project.errors.is_empty() {
                    return Err(project.errors.into_py_err());
                }

                BuildableProgram::new(target, project.package_graph_sources)
            } else {
                panic!("file system hooks should have been passed in with a manifest descriptor")
            }
        } else {
            let graph = PackageGraphSources::with_no_dependencies(
                Vec::default(),
                LanguageFeatures::from_iter(language_features),
                None,
            );
            BuildableProgram::new(target, graph)
        };

        match interpret::Interpreter::new(
            SourceMap::new(buildable_program.user_code.sources, None),
            PackageType::Lib,
            target,
            buildable_program.user_code.language_features,
            buildable_program.store,
            &buildable_program.user_code_dependencies,
        ) {
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
    #[pyo3(signature=(input, callback=None))]
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
    #[pyo3(signature=(seed=None))]
    fn set_quantum_seed(&mut self, seed: Option<u64>) {
        self.interpreter.set_quantum_seed(seed);
    }

    /// Sets the classical seed for the interpreter.
    #[pyo3(signature=(seed=None))]
    fn set_classical_seed(&mut self, seed: Option<u64>) {
        self.interpreter.set_classical_seed(seed);
    }

    /// Dumps the quantum state of the interpreter.
    /// Returns a tuple of (amplitudes, num_qubits), where amplitudes is a dictionary from integer indices to
    /// pairs of real and imaginary amplitudes.
    fn dump_machine(&mut self) -> StateDumpData {
        let (state, qubit_count) = self.interpreter.get_quantum_state();
        StateDumpData(DisplayableState(state, qubit_count))
    }

    /// Dumps the current circuit state of the interpreter.
    ///
    /// This circuit will contain the gates that have been applied
    /// in the simulator up to the current point.
    fn dump_circuit(&mut self, py: Python) -> PyObject {
        Circuit(self.interpreter.get_circuit()).into_py(py)
    }

    #[pyo3(signature=(entry_expr=None, callback=None))]
    fn run(
        &mut self,
        py: Python,
        entry_expr: Option<&str>,
        callback: Option<PyObject>,
    ) -> PyResult<PyObject> {
        let mut receiver = OptionalCallbackReceiver { callback, py };
        match self.interpreter.run(&mut receiver, entry_expr) {
            Ok(value) => Ok(ValueWrapper(value).into_py(py)),
            Err(errors) => Err(QSharpError::new_err(format_errors(errors))),
        }
    }

    fn qir(&mut self, _py: Python, entry_expr: &str) -> PyResult<String> {
        match self.interpreter.qirgen(entry_expr) {
            Ok(qir) => Ok(qir),
            Err(errors) => Err(QSharpError::new_err(format_errors(errors))),
        }
    }

    /// Synthesizes a circuit for a Q# program. Either an entry
    /// expression or an operation must be provided.
    ///
    /// :param entry_expr: An entry expression.
    ///
    /// :param operation: The operation to synthesize. This can be a name of
    /// an operation of a lambda expression. The operation must take only
    /// qubits or arrays of qubits as parameters.
    ///
    /// :raises QSharpError: If there is an error synthesizing the circuit.
    #[pyo3(signature=(entry_expr=None, operation=None))]
    fn circuit(
        &mut self,
        py: Python,
        entry_expr: Option<String>,
        operation: Option<String>,
    ) -> PyResult<PyObject> {
        let entrypoint = match (entry_expr, operation) {
            (Some(entry_expr), None) => CircuitEntryPoint::EntryExpr(entry_expr),
            (None, Some(operation)) => CircuitEntryPoint::Operation(operation),
            _ => {
                return Err(PyException::new_err(
                    "either entry_expr or operation must be specified",
                ))
            }
        };

        match self.interpreter.circuit(entrypoint, false) {
            Ok(circuit) => Ok(Circuit(circuit).into_py(py)),
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

    #[allow(clippy::too_many_arguments)]
    #[pyo3(
        signature = (source, callback=None, read_file=None, list_directory=None, resolve_path=None, fetch_github=None, **kwargs)
    )]
    pub fn _run_qasm3(
        &mut self,
        py: Python,
        source: &str,
        callback: Option<PyObject>,
        read_file: Option<PyObject>,
        list_directory: Option<PyObject>,
        resolve_path: Option<PyObject>,
        fetch_github: Option<PyObject>,
        kwargs: Option<Bound<'_, PyDict>>,
    ) -> PyResult<PyObject> {
        let mut receiver = OptionalCallbackReceiver { callback, py };

        let kwargs = kwargs.unwrap_or_else(|| PyDict::new_bound(py));

        let operation_name = crate::interop::get_operation_name(&kwargs)?;
        let seed = crate::interop::get_seed(&kwargs);
        let shots = crate::interop::get_shots(&kwargs)?;
        let search_path = crate::interop::get_search_path(&kwargs)?;
        let program_type = crate::interop::get_program_type(&kwargs)?;
        let output_semantics = crate::interop::get_output_semantics(&kwargs)?;

        let fs = crate::interop::create_filesystem_from_py(
            py,
            read_file,
            list_directory,
            resolve_path,
            fetch_github,
        );
        let resolver = ImportResolver::new(fs, PathBuf::from(search_path));

        let (package, _source_map, signature) = compile_qasm_enriching_errors(
            source,
            &operation_name,
            &resolver,
            program_type.clone(),
            output_semantics,
            false,
        )?;

        let value = self
            .interpreter
            .eval_ast_fragments(&mut receiver, source, package)
            .map_err(|errors| QSharpError::new_err(format_errors(errors)))?;

        match program_type {
            ProgramType::File => {
                let entry_expr = signature.create_entry_expr_from_params(String::new());
                self.interpreter
                    .set_entry_expr(&entry_expr)
                    .map_err(|errors| map_entry_compilation_errors(errors, &signature))?;

                match run_ast(&mut self.interpreter, &mut receiver, shots, seed) {
                    Ok(result) => Ok(PyList::new_bound(
                        py,
                        result.iter().map(|v| ValueWrapper(v.clone()).into_py(py)),
                    )
                    .into_py(py)),
                    Err(errors) => Err(QSharpError::new_err(format_errors(errors))),
                }
            }
            _ => Ok(ValueWrapper(value).into_py(py)),
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

create_exception!(
    module,
    QasmError,
    pyo3::exceptions::PyException,
    "An error returned from the OpenQASM parser."
);

pub(crate) fn format_errors(errors: Vec<interpret::Error>) -> String {
    errors
        .into_iter()
        .map(|e| format_error(&e))
        .collect::<String>()
}

pub(crate) fn format_error(e: &interpret::Error) -> String {
    let mut message = String::new();
    if let Some(stack_trace) = e.stack_trace() {
        write!(message, "{stack_trace}").unwrap();
    }
    let additional_help = python_help(e);
    let report = Report::new(e.clone());
    write!(message, "{report:?}")
        .unwrap_or_else(|err| panic!("writing error failed: {err} error was: {e:?}"));
    if let Some(additional_help) = additional_help {
        writeln!(message, "{additional_help}").unwrap();
    }
    message
}

/// Additional help text for an error specific to the Python module
fn python_help(error: &interpret::Error) -> Option<String> {
    if matches!(error, interpret::Error::UnsupportedRuntimeCapabilities) {
        Some("Unsupported target profile. Initialize Q# by running `qsharp.init(target_profile=qsharp.TargetProfile.Base)` before performing code generation.".into())
    } else {
        None
    }
}

#[pyclass]
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

    fn _repr_latex_(&self) -> Option<String> {
        match &self.0 {
            DisplayableOutput::State(state) => state.to_latex(),
            DisplayableOutput::Message(_) => None,
        }
    }

    fn state_dump(&self) -> Option<StateDumpData> {
        match &self.0 {
            DisplayableOutput::State(state) => Some(StateDumpData(state.clone())),
            DisplayableOutput::Message(_) => None,
        }
    }
}

#[pyclass]
/// Captured simlation state dump.
pub(crate) struct StateDumpData(pub(crate) DisplayableState);

#[pymethods]
impl StateDumpData {
    fn get_dict<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyDict>> {
        PyDict::from_sequence_bound(&PyList::new_bound(
            py,
            self.0
                 .0
                .iter()
                .map(|(k, v)| {
                    PyTuple::new_bound(
                        py,
                        &[
                            k.clone().into_py(py),
                            PyComplex::from_doubles_bound(py, v.re, v.im).into(),
                        ],
                    )
                })
                .collect::<Vec<_>>(),
        ))
    }

    #[getter]
    fn get_qubit_count(&self) -> usize {
        self.0 .1
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

    fn _repr_latex_(&self) -> Option<String> {
        self.0.to_latex()
    }
}

#[derive(Clone, Copy, PartialEq)]
#[pyclass(eq, eq_int)]
/// A Q# measurement result.
pub(crate) enum Result {
    Zero,
    One,
}

#[pymethods]
impl Result {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn __repr__(&self) -> String {
        match self {
            Result::Zero => "Zero".to_owned(),
            Result::One => "One".to_owned(),
        }
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn __str__(&self) -> String {
        self.__repr__()
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn __hash__(&self) -> u32 {
        match self {
            Result::Zero => 0,
            Result::One => 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
#[pyclass(eq, eq_int)]
/// A Q# Pauli operator.
pub(crate) enum Pauli {
    I,
    X,
    Y,
    Z,
}

// Mapping of Q# value types to Python value types.
pub(crate) struct ValueWrapper(pub(crate) Value);

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
                    PyTuple::new_bound(py, val.iter().map(|v| ValueWrapper(v.clone()).into_py(py)))
                        .into_py(py)
                }
            }
            Value::Array(val) => {
                PyList::new_bound(py, val.iter().map(|v| ValueWrapper(v.clone()).into_py(py)))
                    .into_py(py)
            }
            _ => format!("<{}> {}", Value::type_name(&self.0), &self.0).into_py(py),
        }
    }
}

pub(crate) struct OptionalCallbackReceiver<'a> {
    pub(crate) callback: Option<PyObject>,
    pub(crate) py: Python<'a>,
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
                    PyTuple::new_bound(
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
                    PyTuple::new_bound(
                        self.py,
                        &[Py::new(self.py, Output(out)).expect("should be able to create output")],
                    ),
                )
                .map_err(|_| Error)?;
        }
        Ok(())
    }
}

#[pyclass]
struct Circuit(pub qsc::circuit::Circuit);

#[pymethods]
impl Circuit {
    fn __repr__(&self) -> String {
        self.0.to_string()
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    fn json(&self, _py: Python) -> PyResult<String> {
        serde_json::to_string(&self.0).map_err(|e| PyException::new_err(e.to_string()))
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

impl<E> IntoPyErr for Vec<E>
where
    E: Diagnostic + Send + Sync + 'static,
{
    fn into_py_err(self) -> PyErr {
        let mut message = String::new();
        for diag in self {
            let report = Report::new(diag);
            writeln!(message, "{report:?}").expect("string should be writable");
        }
        PyException::new_err(message)
    }
}
