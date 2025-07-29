// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    displayable_output::{DisplayableMatrix, DisplayableOutput, DisplayableState},
    fs::file_system,
    generic_estimator::register_generic_estimator_submodule,
    interop::{
        circuit_qasm_program, compile_qasm_program_to_qir, compile_qasm_to_qsharp,
        create_filesystem_from_py, get_operation_name, get_output_semantics, get_program_type,
        get_search_path, resource_estimate_qasm_program, run_qasm_program,
    },
    noisy_simulator::register_noisy_simulator_submodule,
};
use miette::{Diagnostic, Report};
use num_bigint::{BigInt, BigUint};
use num_complex::Complex64;
use pyo3::{
    IntoPyObjectExt, create_exception,
    exceptions::{PyException, PyValueError},
    prelude::*,
    types::{PyDict, PyList, PyString, PyTuple, PyType},
};
use qsc::{
    LanguageFeatures, PackageType, SourceMap,
    error::WithSource,
    fir::{self},
    hir::ty::{Prim, Ty},
    interpret::{
        self, CircuitEntryPoint, PauliNoise, Value,
        output::{Error, Receiver},
    },
    packages::BuildableProgram,
    project::{FileSystem, PackageCache, PackageGraphSources, ProjectType},
    qasm::{CompilerConfig, QubitSemantics, compiler::compile_to_qsharp_ast_with_config},
    target::Profile,
};

use resource_estimator::{self as re, estimate_call, estimate_expr};
use rustc_hash::FxHashMap;
use std::{cell::RefCell, fmt::Write, path::PathBuf, rc::Rc, str::FromStr, sync::Arc};

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
    is_send::<ValueIR>();
    is_send::<UdtValue>();
    is_send::<TypeIR>();
    is_send::<TypeKind>();
    is_send::<PrimitiveKind>();
    is_send::<UdtIR>();
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
    m.add_class::<GlobalCallable>()?;
    m.add_class::<ValueIR>()?;
    m.add_class::<UdtValue>()?;
    m.add_class::<TypeIR>()?;
    m.add_class::<TypeKind>()?;
    m.add_class::<PrimitiveKind>()?;
    m.add_class::<UdtIR>()?;
    m.add_function(wrap_pyfunction!(physical_estimates, m)?)?;
    m.add("QSharpError", py.get_type::<QSharpError>())?;
    register_noisy_simulator_submodule(py, m)?;
    register_generic_estimator_submodule(m)?;
    // QASM interop
    m.add("QasmError", py.get_type::<QasmError>())?;
    m.add_function(wrap_pyfunction!(resource_estimate_qasm_program, m)?)?;
    m.add_function(wrap_pyfunction!(run_qasm_program, m)?)?;
    m.add_function(wrap_pyfunction!(circuit_qasm_program, m)?)?;
    m.add_function(wrap_pyfunction!(compile_qasm_program_to_qir, m)?)?;
    m.add_function(wrap_pyfunction!(compile_qasm_to_qsharp, m)?)?;
    Ok(())
}

// This ordering must match the _native.pyi file.
#[derive(Clone, Copy, Default, PartialEq)]
#[pyclass(eq, eq_int, module = "qsharp._native")]
#[allow(non_camel_case_types)]
/// A Q# target profile.
///
/// A target profile describes the capabilities of the hardware or simulator
/// which will be used to run the Q# program.
pub(crate) enum TargetProfile {
    /// Target supports the minimal set of capabilities required to run a quantum program.
    ///
    /// This option maps to the Base Profile as defined by the QIR specification.
    #[default]
    Base,
    /// Target supports the Adaptive profile with the integer computation extension.
    ///
    /// This profile includes all of the required Adaptive Profile
    /// capabilities, as well as the optional integer computation
    /// extension defined by the QIR specification.
    Adaptive_RI,
    /// Target supports the Adaptive profile with integer & floating-point
    /// computation extensions.
    ///
    /// This profile includes all required Adaptive Profile and `Adaptive_RI`
    /// capabilities, as well as the optional floating-point computation
    /// extension defined by the QIR specification.
    Adaptive_RIF,
    /// Target supports the full set of capabilities required to run any Q# program.
    ///
    /// This option maps to the Full Profile as defined by the QIR specification.
    Unrestricted,
}

#[pymethods]
impl TargetProfile {
    #[new]
    // We need to define `new` so that instances of `TargetProfile` can be created by Python
    pub(crate) fn new() -> Self {
        Self::default()
    }

    // called and the returned object is pickled as the contents for the instance
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn __getstate__(&self) -> PyResult<isize> {
        Ok(self.__pyo3__int__())
    }

    // called with the unpickled state and the instance is updated in place
    // This is what requires `new` to be implemented as we can't hydrate an
    // unininitialized instance in Python.
    fn __setstate__(&mut self, state: i32) -> PyResult<()> {
        (*self) = match state {
            0 => Self::Base,
            1 => Self::Adaptive_RI,
            2 => Self::Adaptive_RIF,
            3 => Self::Unrestricted,
            _ => return Err(PyValueError::new_err("invalid state")),
        };
        Ok(())
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn __str__(&self) -> String {
        Into::<Profile>::into(*self).to_str().to_owned()
    }

    /// Creates a target profile from a string.
    /// :param value: The string to parse.
    /// :raises ValueError: If the string does not match any target profile.
    #[classmethod]
    #[allow(clippy::needless_pass_by_value)]
    fn from_str(_cls: &Bound<'_, PyType>, key: String) -> pyo3::PyResult<Self> {
        let profile = Profile::from_str(key.as_str())
            .map_err(|()| PyValueError::new_err(format!("{key} is not a valid target profile")))?;
        Ok(TargetProfile::from(profile))
    }
}

impl From<Profile> for TargetProfile {
    fn from(profile: Profile) -> Self {
        match profile {
            Profile::Base => TargetProfile::Base,
            Profile::AdaptiveRI => TargetProfile::Adaptive_RI,
            Profile::AdaptiveRIF => TargetProfile::Adaptive_RIF,
            Profile::Unrestricted => TargetProfile::Unrestricted,
        }
    }
}

impl From<TargetProfile> for Profile {
    fn from(profile: TargetProfile) -> Self {
        match profile {
            TargetProfile::Base => Profile::Base,
            TargetProfile::Adaptive_RI => Profile::AdaptiveRI,
            TargetProfile::Adaptive_RIF => Profile::AdaptiveRIF,
            TargetProfile::Unrestricted => Profile::Unrestricted,
        }
    }
}

// This ordering must match the _native.pyi file.
#[derive(Clone, Copy, Default, PartialEq)]
#[pyclass(eq, eq_int, module = "qsharp._native")]
#[allow(non_camel_case_types)]
/// Represents the output semantics for OpenQASM 3 compilation.
/// Each has implications on the output of the compilation
/// and the semantic checks that are performed.
pub(crate) enum OutputSemantics {
    /// The output is in Qiskit format meaning that the output
    /// is all of the classical registers, in reverse order
    /// in which they were added to the circuit with each
    /// bit within each register in reverse order.
    #[default]
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

#[pymethods]
impl OutputSemantics {
    #[new]
    // We need to define `new` so that instances of `TargetProfile` can be created by Python
    pub(crate) fn new() -> Self {
        Self::default()
    }

    // called and the returned object is pickled as the contents for the instance
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn __getstate__(&self) -> PyResult<isize> {
        Ok(self.__pyo3__int__())
    }

    // called with the unpickled state and the instance is updated in place
    // This is what requires `new` to be implemented as we can't hydrate an
    // unininitialized instance in Python.
    fn __setstate__(&mut self, state: i32) -> PyResult<()> {
        (*self) = match state {
            0 => Self::Qiskit,
            1 => Self::OpenQasm,
            2 => Self::ResourceEstimation,
            _ => return Err(PyValueError::new_err("invalid state")),
        };
        Ok(())
    }
}

impl From<OutputSemantics> for qsc::qasm::OutputSemantics {
    fn from(output_semantics: OutputSemantics) -> Self {
        match output_semantics {
            OutputSemantics::Qiskit => qsc::qasm::OutputSemantics::Qiskit,
            OutputSemantics::OpenQasm => qsc::qasm::OutputSemantics::OpenQasm,
            OutputSemantics::ResourceEstimation => qsc::qasm::OutputSemantics::ResourceEstimation,
        }
    }
}

// This ordering must match the _native.pyi file.
#[derive(Clone, Copy, Default, PartialEq)]
#[pyclass(eq, eq_int, module = "qsharp._native")]
#[allow(non_camel_case_types)]
/// Represents the type of compilation output to create
pub enum ProgramType {
    /// Creates an operation in a namespace as if the program is a standalone
    /// file. Inputs are lifted to the operation params. Output are lifted to
    /// the operation return type. The operation is marked as `@EntryPoint`
    /// as long as there are no input parameters.
    #[default]
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

#[pymethods]
impl ProgramType {
    #[new]
    // We need to define `new` so that instances of `TargetProfile` can be created by Python
    pub(crate) fn new() -> Self {
        Self::default()
    }

    // called and the returned object is pickled as the contents for the instance
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn __getstate__(&self) -> PyResult<isize> {
        Ok(self.__pyo3__int__())
    }

    // called with the unpickled state and the instance is updated in place
    // This is what requires `new` to be implemented as we can't hydrate an
    // unininitialized instance in Python.
    fn __setstate__(&mut self, state: i32) -> PyResult<()> {
        (*self) = match state {
            0 => Self::File,
            1 => Self::Operation,
            2 => Self::Fragments,
            _ => return Err(PyValueError::new_err("invalid state")),
        };
        Ok(())
    }
}

impl From<ProgramType> for qsc::qasm::ProgramType {
    fn from(output_semantics: ProgramType) -> Self {
        match output_semantics {
            ProgramType::File => qsc::qasm::ProgramType::File,
            ProgramType::Operation => qsc::qasm::ProgramType::Operation,
            ProgramType::Fragments => qsc::qasm::ProgramType::Fragments,
        }
    }
}

#[allow(clippy::struct_field_names)]
#[pyclass(unsendable)]
pub(crate) struct Interpreter {
    pub(crate) interpreter: interpret::Interpreter,
    /// The Python function to call to create a new function wrapping a callable invocation.
    pub(crate) make_callable: Option<PyObject>,
    /// The Python function to call to create a class representing a qsharp struct.
    pub(crate) make_class: Option<PyObject>,
}

thread_local! { static PACKAGE_CACHE: Rc<RefCell<PackageCache>> = Rc::default(); }

#[pymethods]
/// A Q# interpreter.
impl Interpreter {
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::needless_pass_by_value)]
    #[pyo3(signature = (target_profile, language_features=None, project_root=None, read_file=None, list_directory=None, resolve_path=None, fetch_github=None, make_callable=None, make_class=None))]
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
        make_callable: Option<PyObject>,
        make_class: Option<PyObject>,
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
                let ProjectType::QSharp(package_graph_sources) = project.project_type else {
                    unreachable!("Project type should be Q#")
                };
                BuildableProgram::new(target, package_graph_sources)
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
            Ok(interpreter) => {
                if let Some(make_callable) = &make_callable {
                    // Add any global callables from the user source as Python functions to the environment.
                    let exported_items = interpreter.user_globals();
                    for (namespace, name, val) in exported_items {
                        create_py_callable(py, make_callable, &namespace, &name, val)?;
                    }
                }
                if let Some(make_class) = &make_class {
                    // Add any global structs from the user source as Python classes to the environment.
                    let exported_items = interpreter.user_types();
                    for (namespace, name, udt) in exported_items {
                        let ty = Ty::Udt(name.clone(), qsc::hir::Res::Item(udt));
                        create_py_class(&interpreter, py, make_class, &namespace, &name, &ty)?;
                    }
                }
                Ok(Self {
                    interpreter,
                    make_callable,
                    make_class,
                })
            }
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
            Ok((value, ty)) => {
                if let Some(make_callable) = &self.make_callable {
                    // Get any global callables from the evaluated input and add them to the environment. This will grab
                    // every callable that was defined in the input and by previous calls that added to the open package.
                    // This is safe because either the callable will be replaced with itself or a new callable with the
                    // same name will shadow the previous one, which is the expected behavior.
                    let new_items = self.interpreter.source_globals();
                    for (namespace, name, val) in new_items {
                        create_py_callable(py, make_callable, &namespace, &name, val)?;
                    }
                }
                if let Some(make_class) = &self.make_class {
                    // Get any global UDTs from the evaluated input and add them to the environment. This will grab
                    // every UDT that was defined in the input and by previous calls that added to the open package.
                    // This is safe because either the UDT will be replaced with itself or a new UDT with the
                    // same name will shadow the previous one, which is the expected behavior.
                    let new_items = self.interpreter.source_types();
                    for (namespace, name, udt) in new_items {
                        let ty = Ty::Udt(name.clone(), qsc::hir::Res::Item(udt));
                        create_py_class(&self.interpreter, py, make_class, &namespace, &name, &ty)?;
                    }
                }
                typed_value_to_value_ir(&self.interpreter, &value, &ty)?.into_py_any(py)
            }
            Err(errors) => Err(QSharpError::new_err(format_errors(errors))),
        }
    }

    /// Imports OpenQASM source code into the active Q# interpreter.
    ///
    /// Args:
    ///     source (str): An OpenQASM program or fragment.
    ///     output_fn: The function to handle the output of the execution.
    ///     read_file: A callable that reads a file and returns its content and path.
    ///     list_directory: A callable that lists the contents of a directory.
    ///     resolve_path: A callable that resolves a file path given a base path and a relative path.
    ///     fetch_github: A callable that fetches a file from GitHub.
    ///     **kwargs: Additional keyword arguments to pass to the execution.
    ///         - name (str): The name of the program. This is used as the entry point for the program.
    ///         - search_path (Optional[str]): The optional search path for resolving file references.
    ///         - output_semantics (OutputSemantics, optional): The output semantics for the compilation.
    ///         - program_type (ProgramType, optional): The type of program compilation to perform.
    ///
    /// Returns:
    ///     value: The value returned by the last statement in the source code.
    ///
    /// Raises:
    ///     QasmError: If there is an error generating, parsing, or analyzing the OpenQASM source.
    ///     QSharpError: If there is an error compiling the program.
    ///     QSharpError: If there is an error evaluating the source code.
    #[pyo3(signature=(input, output_fn, read_file, list_directory, resolve_path, fetch_github, **kwargs))]
    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::too_many_arguments)]
    fn import_qasm(
        &mut self,
        py: Python,
        input: &str,
        output_fn: Option<PyObject>,
        read_file: Option<PyObject>,
        list_directory: Option<PyObject>,
        resolve_path: Option<PyObject>,
        fetch_github: Option<PyObject>,
        kwargs: Option<Bound<'_, PyDict>>,
    ) -> PyResult<PyObject> {
        let kwargs = kwargs.unwrap_or_else(|| PyDict::new(py));

        let operation_name = get_operation_name(&kwargs)?;
        let search_path = get_search_path(&kwargs)?;
        let program_ty = get_program_type(&kwargs, || ProgramType::Operation)?;
        let output_semantics = get_output_semantics(&kwargs, || OutputSemantics::OpenQasm)?;

        let fs =
            create_filesystem_from_py(py, read_file, list_directory, resolve_path, fetch_github);
        let file_path = PathBuf::from_str(&search_path)
            .expect("from_str is infallible")
            .join("program.qasm");
        let project = fs.load_openqasm_project(&file_path, Some(Arc::<str>::from(input)));
        let ProjectType::OpenQASM(sources) = project.project_type else {
            return Err(QasmError::new_err(
                "Expected OpenQASM project, but got a different type".to_string(),
            ));
        };

        let config = CompilerConfig::new(
            QubitSemantics::Qiskit,
            output_semantics.into(),
            program_ty.into(),
            Some(operation_name.into()),
            None,
        );
        let res = qsc::qasm::semantic::parse_sources(&sources);
        let unit = compile_to_qsharp_ast_with_config(res, config);
        let (sources, errors, package, _) = unit.into_tuple();

        if !errors.is_empty() {
            let errors = errors
                .iter()
                .map(|e| {
                    use qsc::compile::ErrorKind;
                    use qsc::interpret::Error;
                    let error = e.error().clone();
                    let kind = ErrorKind::OpenQasm(error);
                    let v = WithSource::from_map(&sources, kind);
                    Error::Compile(v)
                })
                .collect();
            return Err(QSharpError::new_err(format_errors(errors)));
        }
        let mut receiver = OptionalCallbackReceiver {
            callback: output_fn,
            py,
        };

        match self
            .interpreter
            .eval_ast_fragments(&mut receiver, input, package)
        {
            Ok(value) => {
                if let Some(make_callable) = &self.make_callable {
                    // Get any global callables from the evaluated input and add them to the environment. This will grab
                    // every callable that was defined in the input and by previous calls that added to the open package.
                    // This is safe because either the callable will be replaced with itself or a new callable with the
                    // same name will shadow the previous one, which is the expected behavior.
                    let new_items = self.interpreter.source_globals();
                    for (namespace, name, val) in new_items {
                        create_py_callable(py, make_callable, &namespace, &name, val)?;
                    }
                }
                Ok(ValueWrapper(value).into_pyobject(py)?.unbind())
            }
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
    fn dump_circuit(&mut self, py: Python) -> PyResult<PyObject> {
        Circuit(self.interpreter.get_circuit()).into_py_any(py)
    }

    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature=(entry_expr=None, callback=None, noise=None, qubit_loss=None, callable=None, args=None))]
    fn run(
        &mut self,
        py: Python,
        entry_expr: Option<&str>,
        callback: Option<PyObject>,
        noise: Option<(f64, f64, f64)>,
        qubit_loss: Option<f64>,
        callable: Option<GlobalCallable>,
        args: Option<PyObject>,
    ) -> PyResult<PyObject> {
        let mut receiver = OptionalCallbackReceiver { callback, py };

        let noise = match noise {
            None => None,
            Some((px, py, pz)) => match PauliNoise::from_probabilities(px, py, pz) {
                Ok(noise_struct) => Some(noise_struct),
                Err(error_message) => return Err(PyException::new_err(error_message)),
            },
        };

        let mut result_ty = None;
        let result = match callable {
            Some(callable) => {
                let (input_ty, output_ty) = self
                    .interpreter
                    .global_callable_ty(&callable.0)
                    .ok_or(QSharpError::new_err("callable not found"))?;
                let args = args_to_values(&self.interpreter, py, args, &input_ty, &output_ty)?;
                result_ty = Some(output_ty);
                self.interpreter.invoke_with_noise(
                    &mut receiver,
                    callable.0,
                    args,
                    noise,
                    qubit_loss,
                )
            }
            _ => self
                .interpreter
                .run(&mut receiver, entry_expr, noise, qubit_loss),
        };

        match result {
            Ok(value) => {
                if let Some(ty) = result_ty {
                    typed_value_to_value_ir(&self.interpreter, &value, &ty)?.into_py_any(py)
                } else {
                    Ok(ValueWrapper(value).into_pyobject(py)?.unbind())
                }
            }
            Err(errors) => Err(QSharpError::new_err(format_errors(errors))),
        }
    }

    #[pyo3(signature=(callable, args=None, callback=None))]
    fn invoke(
        &mut self,
        py: Python,
        callable: GlobalCallable,
        args: Option<PyObject>,
        callback: Option<PyObject>,
    ) -> PyResult<PyObject> {
        let mut receiver = OptionalCallbackReceiver { callback, py };
        let (input_ty, output_ty) = self
            .interpreter
            .global_callable_ty(&callable.0)
            .ok_or(QSharpError::new_err("callable not found"))?;

        let args = args_to_values(&self.interpreter, py, args, &input_ty, &output_ty)?;

        match self.interpreter.invoke(&mut receiver, callable.0, args) {
            Ok(value) => {
                typed_value_to_value_ir(&self.interpreter, &value, &output_ty)?.into_py_any(py)
            }
            Err(errors) => Err(QSharpError::new_err(format_errors(errors))),
        }
    }

    #[pyo3(signature=(entry_expr=None, callable=None, args=None))]
    fn qir(
        &mut self,
        py: Python,
        entry_expr: Option<&str>,
        callable: Option<GlobalCallable>,
        args: Option<PyObject>,
    ) -> PyResult<String> {
        if let Some(entry_expr) = entry_expr {
            match self.interpreter.qirgen(entry_expr) {
                Ok(qir) => Ok(qir),
                Err(errors) => Err(QSharpError::new_err(format_errors(errors))),
            }
        } else {
            let callable = callable.ok_or_else(|| {
                QSharpError::new_err("either entry_expr or callable must be specified")
            })?;
            let (input_ty, output_ty) = self
                .interpreter
                .global_callable_ty(&callable.0)
                .ok_or(QSharpError::new_err("callable not found"))?;

            let args = args_to_values(&self.interpreter, py, args, &input_ty, &output_ty)?;
            match self.interpreter.qirgen_from_callable(&callable.0, args) {
                Ok(qir) => Ok(qir),
                Err(errors) => Err(QSharpError::new_err(format_errors(errors))),
            }
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
    /// :param callable: A callable to synthesize.
    ///
    /// :param args: The arguments to pass to the callable.
    ///
    /// :raises QSharpError: If there is an error synthesizing the circuit.
    #[pyo3(signature=(entry_expr=None, operation=None, callable=None, args=None))]
    fn circuit(
        &mut self,
        py: Python,
        entry_expr: Option<String>,
        operation: Option<String>,
        callable: Option<GlobalCallable>,
        args: Option<PyObject>,
    ) -> PyResult<PyObject> {
        let entrypoint = match (entry_expr, operation, callable) {
            (Some(entry_expr), None, None) => CircuitEntryPoint::EntryExpr(entry_expr),
            (None, Some(operation), None) => CircuitEntryPoint::Operation(operation),
            (None, None, Some(callable)) => {
                let (input_ty, output_ty) = self
                    .interpreter
                    .global_callable_ty(&callable.0)
                    .ok_or(QSharpError::new_err("callable not found"))?;
                let args = args_to_values(&self.interpreter, py, args, &input_ty, &output_ty)?;
                CircuitEntryPoint::Callable(callable.0, args)
            }
            _ => {
                return Err(PyException::new_err(
                    "either entry_expr or operation must be specified",
                ));
            }
        };

        match self.interpreter.circuit(entrypoint, false) {
            Ok(circuit) => Circuit(circuit).into_py_any(py),
            Err(errors) => Err(QSharpError::new_err(format_errors(errors))),
        }
    }

    #[pyo3(signature=(job_params, entry_expr=None, callable=None, args=None))]
    fn estimate(
        &mut self,
        py: Python,
        job_params: &str,
        entry_expr: Option<&str>,
        callable: Option<GlobalCallable>,
        args: Option<PyObject>,
    ) -> PyResult<String> {
        let results = if let Some(entry_expr) = entry_expr {
            estimate_expr(&mut self.interpreter, entry_expr, job_params)
        } else {
            let callable = callable.ok_or_else(|| {
                QSharpError::new_err("either entry_expr or callable must be specified")
            })?;
            let (input_ty, output_ty) = self
                .interpreter
                .global_callable_ty(&callable.0)
                .ok_or(QSharpError::new_err("callable not found"))?;
            let args = args_to_values(&self.interpreter, py, args, &input_ty, &output_ty)?;
            estimate_call(&mut self.interpreter, callable.0, args, job_params)
        };
        match results {
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

#[pyclass]
#[derive(Clone)]
enum TypeIR {
    Primitive(PrimitiveKind),
    Tuple(Vec<TypeIR>),
    Array(Vec<TypeIR>),
    Udt(UdtIR),
}

#[pymethods]
impl TypeIR {
    fn kind(&self) -> TypeKind {
        match self {
            Self::Primitive(_) => TypeKind::Primitive,
            Self::Tuple(_) => TypeKind::Tuple,
            Self::Array(_) => TypeKind::Array,
            Self::Udt(_) => TypeKind::Udt,
        }
    }

    fn primitive(&self) -> PyResult<PrimitiveKind> {
        if let Self::Primitive(ty) = self {
            Ok(*ty)
        } else {
            Err(QSharpError::new_err(
                "ValueError: type is not a primitive".to_string(),
            ))
        }
    }

    fn tuple(&self) -> PyResult<Vec<TypeIR>> {
        if let Self::Tuple(ty) = self {
            Ok(ty.clone())
        } else {
            Err(QSharpError::new_err(
                "ValueError: type is not a tuple".to_string(),
            ))
        }
    }

    fn array(&self) -> PyResult<Vec<TypeIR>> {
        if let Self::Tuple(ty) = self {
            Ok(ty.clone())
        } else {
            Err(QSharpError::new_err(
                "ValueError: type is not an array".to_string(),
            ))
        }
    }

    fn udt(&self) -> PyResult<UdtIR> {
        if let Self::Udt(ty) = self {
            Ok(ty.clone())
        } else {
            Err(QSharpError::new_err(
                "ValueError: type is not a UDT".to_string(),
            ))
        }
    }
}

fn type_ir_from_qsharp_ty(ctx: &interpret::Interpreter, ty: &Ty) -> PyResult<TypeIR> {
    match ty {
        Ty::Prim(prim) => {
            let prim = match prim {
                Prim::Bool => PrimitiveKind::Bool,
                Prim::Int | Prim::BigInt => PrimitiveKind::Int,
                Prim::Double => PrimitiveKind::Double,
                Prim::String => PrimitiveKind::String,
                Prim::Pauli => PrimitiveKind::Pauli,
                Prim::Result => PrimitiveKind::Result,

                Prim::Qubit | Prim::Range | Prim::RangeTo | Prim::RangeFrom | Prim::RangeFull => {
                    return Err(QSharpError::new_err(format!(
                        "unsupported interop type: `{ty}`"
                    )));
                }
            };
            Ok(TypeIR::Primitive(prim))
        }
        Ty::Array(ty) => Ok(TypeIR::Array(vec![type_ir_from_qsharp_ty(ctx, ty)?])),
        Ty::Tuple(items) => {
            let mut tuple = Vec::new();
            for item in items {
                tuple.push(type_ir_from_qsharp_ty(ctx, item)?);
            }
            Ok(TypeIR::Tuple(tuple))
        }
        Ty::Udt(name, res) => {
            let qsc::hir::Res::Item(item_id) = res else {
                panic!("Udt should be an item");
            };
            let Some(udt) = ctx.udt_ty(item_id) else {
                unreachable!(
                    "we verified that the udt is defined in `first_unsupported_interop_ty`"
                );
            };

            // Handle `Complex` special case.
            if is_complex_udt(udt) {
                return Ok(TypeIR::Primitive(PrimitiveKind::Complex));
            }

            let udt_fields = collect_udt_fields(udt)?;
            let mut fields = Vec::new();

            for (name, ty) in udt_fields {
                fields.push((name.to_string(), type_ir_from_qsharp_ty(ctx, ty)?));
            }

            Ok(TypeIR::Udt(UdtIR {
                name: name.to_string(),
                fields,
            }))
        }
        Ty::Param { .. } | Ty::Infer(..) | Ty::Arrow(..) | Ty::Err => Err(QSharpError::new_err(
            format!("unsupported interop type: `{ty}`"),
        )),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[pyclass(eq, eq_int, ord)]
enum TypeKind {
    Primitive,
    Tuple,
    Array,
    Udt,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[pyclass(eq, eq_int, ord)]
enum PrimitiveKind {
    Bool,
    Int,
    Double,
    Complex,
    String,
    Pauli,
    Result,
}

#[pyclass]
#[derive(Clone)]
struct UdtIR {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    fields: Vec<(String, TypeIR)>,
}

#[pyclass]
#[derive(Clone)]
enum ValueIR {
    Primitive(PrimitiveValue),
    Tuple(Vec<ValueIR>),
    Array(Vec<ValueIR>),
    Udt(UdtValue),
}

impl ValueIR {
    fn ty_name(&self) -> String {
        fn ty_name_rec(val: &ValueIR, f: &mut String) -> std::fmt::Result {
            match val {
                ValueIR::Primitive(primitive) => match primitive {
                    PrimitiveValue::Bool(_) => write!(f, "Bool"),
                    PrimitiveValue::Int(_) => write!(f, "Int"),
                    PrimitiveValue::BigInt(_) => write!(f, "BigInt"),
                    PrimitiveValue::Double(_) => write!(f, "Double"),
                    PrimitiveValue::Complex(_) => write!(f, "Complex"),
                    PrimitiveValue::String(_) => write!(f, "String"),
                    PrimitiveValue::Result(_) => write!(f, "Result"),
                    PrimitiveValue::Pauli(_) => write!(f, "Pauli"),
                },
                ValueIR::Tuple(tuple) => {
                    write!(f, "(")?;
                    for value in tuple {
                        ty_name_rec(value, f)?;
                    }
                    write!(f, ")")
                }
                ValueIR::Array(array) => {
                    write!(f, "[")?;
                    for value in array {
                        ty_name_rec(value, f)?;
                    }
                    write!(f, "]")
                }
                ValueIR::Udt(_) => write!(f, "Udt"),
            }
        }
        let mut buffer = String::new();
        ty_name_rec(self, &mut buffer).expect("writing to String should succeed");
        buffer
    }
}

#[pymethods]
impl ValueIR {
    fn kind(&self) -> TypeKind {
        match self {
            Self::Primitive(_) => TypeKind::Primitive,
            Self::Tuple(_) => TypeKind::Tuple,
            Self::Array(_) => TypeKind::Array,
            Self::Udt(_) => TypeKind::Udt,
        }
    }

    fn unwrap_primitive(&self, py: Python) -> PyResult<PyObject> {
        if let Self::Primitive(prim) = self {
            match prim {
                PrimitiveValue::Bool(val) => val.into_py_any(py),
                PrimitiveValue::Int(val) => val.into_py_any(py),
                PrimitiveValue::BigInt(val) => val.into_py_any(py),
                PrimitiveValue::Double(val) => val.into_py_any(py),
                PrimitiveValue::Complex(val) => val.into_py_any(py),
                PrimitiveValue::String(val) => val.into_py_any(py),
                PrimitiveValue::Result(val) => val.into_py_any(py),
                PrimitiveValue::Pauli(val) => val.into_py_any(py),
            }
        } else {
            Err(QSharpError::new_err(
                "ValueError: value is not a primitive".to_string(),
            ))
        }
    }

    fn unwrap_tuple(&self, py: Python) -> PyResult<PyObject> {
        if let Self::Tuple(tuple) = self {
            tuple.clone().into_py_any(py)
        } else {
            Err(QSharpError::new_err(
                "ValueError: value is not a tuple".to_string(),
            ))
        }
    }

    fn unwrap_array(&self, py: Python) -> PyResult<PyObject> {
        if let Self::Array(tuple) = self {
            tuple.clone().into_py_any(py)
        } else {
            Err(QSharpError::new_err(
                "ValueError: value is not an array".to_string(),
            ))
        }
    }

    fn unwrap_udt(&self) -> PyResult<UdtValue> {
        if let Self::Udt(udt) = self {
            Ok(udt.clone())
        } else {
            Err(QSharpError::new_err(
                "ValueError: value is not a UDT".to_string(),
            ))
        }
    }

    #[staticmethod]
    fn udt(fields: FxHashMap<String, Self>) -> Self {
        Self::Udt(UdtValue { name: None, fields })
    }

    #[staticmethod]
    fn tuple(values: Vec<ValueIR>) -> Self {
        Self::Tuple(values)
    }

    #[staticmethod]
    fn array(values: Vec<ValueIR>) -> Self {
        Self::Array(values)
    }

    #[staticmethod]
    fn bool(value: bool) -> Self {
        Self::Primitive(PrimitiveValue::Bool(value))
    }

    #[staticmethod]
    fn int(value: i64) -> Self {
        Self::Primitive(PrimitiveValue::Int(value))
    }

    #[staticmethod]
    fn bigint(value: BigInt) -> Self {
        Self::Primitive(PrimitiveValue::BigInt(value))
    }

    #[staticmethod]
    fn double(value: f64) -> Self {
        Self::Primitive(PrimitiveValue::Double(value))
    }

    #[staticmethod]
    fn complex(value: num_complex::Complex64) -> Self {
        Self::Primitive(PrimitiveValue::Complex(value))
    }

    #[staticmethod]
    fn str(value: String) -> Self {
        Self::Primitive(PrimitiveValue::String(value))
    }

    #[staticmethod]
    fn result(value: Result) -> Self {
        Self::Primitive(PrimitiveValue::Result(value))
    }

    #[staticmethod]
    fn pauli(value: Pauli) -> Self {
        Self::Primitive(PrimitiveValue::Pauli(value))
    }
}

impl ValueIR {
    fn is_complex(&self) -> bool {
        matches!(self, Self::Primitive(PrimitiveValue::Complex(_)))
    }
}

fn is_complex_udt(udt: &qsc::hir::ty::Udt) -> bool {
    if let qsc::hir::ty::UdtDefKind::Tuple(fields) = &udt.definition.kind {
        if fields.len() != 2 {
            return false;
        }
        let qsc::hir::ty::UdtDefKind::Field(real) = &fields[0].kind else {
            return false;
        };
        let qsc::hir::ty::UdtDefKind::Field(imag) = &fields[1].kind else {
            return false;
        };
        return matches!(real.ty, Ty::Prim(Prim::Double))
            && matches!(imag.ty, Ty::Prim(Prim::Double))
            && &*udt.name == "Complex";
    }
    false
}

fn typed_value_to_value_ir(
    ctx: &interpret::Interpreter,
    value: &Value,
    ty: &Ty,
) -> PyResult<ValueIR> {
    match value {
        Value::Int(val) => Ok(ValueIR::int(*val)),
        Value::BigInt(val) => Ok(ValueIR::bigint(val.clone())),
        Value::Double(val) => Ok(ValueIR::double(*val)),
        Value::Bool(val) => Ok(ValueIR::bool(*val)),
        Value::String(val) => Ok(ValueIR::str(val.to_string())),
        Value::Result(val) => match val {
            qsc::interpret::Result::Id(_) => {
                panic!("unexpected Result::Id in typed_value_to_value_ir")
            }
            qsc::interpret::Result::Val(true) => Ok(ValueIR::result(Result::One)),
            qsc::interpret::Result::Val(false) => Ok(ValueIR::result(Result::Zero)),
            qsc::interpret::Result::Loss => Ok(ValueIR::result(Result::Loss)),
        },
        Value::Pauli(val) => Ok(match val {
            fir::Pauli::I => ValueIR::pauli(Pauli::I),
            fir::Pauli::X => ValueIR::pauli(Pauli::X),
            fir::Pauli::Y => ValueIR::pauli(Pauli::Y),
            fir::Pauli::Z => ValueIR::pauli(Pauli::Z),
        }),
        Value::Tuple(values) => match ty {
            Ty::Tuple(items) => {
                let mut tuple = Vec::new();
                for (val, ty) in values.iter().zip(items) {
                    tuple.push(typed_value_to_value_ir(ctx, val, ty)?);
                }
                Ok(ValueIR::tuple(tuple))
            }
            Ty::Udt(_, res) => {
                let qsc::hir::Res::Item(item_id) = res else {
                    panic!("Udt should be an item");
                };
                let Some(udt) = ctx.udt_ty(item_id) else {
                    unreachable!("output type should be defined");
                };
                if is_complex_udt(udt) {
                    let re = values[0].clone().unwrap_double();
                    let im = values[1].clone().unwrap_double();
                    return Ok(ValueIR::complex(num_complex::Complex { re, im }));
                }
                let ty_fields = collect_udt_fields(udt)?;
                let mut fields = Vec::new();
                for (value, (name, ty)) in values.iter().zip(ty_fields) {
                    fields.push((name.to_string(), typed_value_to_value_ir(ctx, value, ty)?));
                }
                let fields = fields.into_iter().collect();
                Ok(ValueIR::Udt(UdtValue {
                    name: Some(udt.name.to_string()),
                    fields,
                }))
            }
            _ => unreachable!(),
        },
        Value::Array(values) => {
            let Ty::Array(ty) = ty else {
                unreachable!();
            };
            let mut array = Vec::new();
            for val in values.iter() {
                array.push(typed_value_to_value_ir(ctx, val, ty)?);
            }
            Ok(ValueIR::array(array))
        }
        _ => Err(QSharpError::new_err(format!(
            "unsupported interop type: `{}`",
            value.type_name(),
        ))),
    }
}

#[pyclass]
#[derive(Clone)]
struct UdtValue {
    #[pyo3(get)]
    name: Option<String>,
    #[pyo3(get)]
    fields: FxHashMap<String, ValueIR>,
}

#[pyclass]
#[derive(Clone)]
enum PrimitiveValue {
    Bool(bool),
    Int(i64),
    BigInt(BigInt),
    Double(f64),
    Complex(num_complex::Complex64),
    String(String),
    Result(Result),
    Pauli(Pauli),
}

fn args_to_values(
    ctx: &interpret::Interpreter,
    py: Python,
    args: Option<PyObject>,
    input_ty: &Ty,
    output_ty: &Ty,
) -> PyResult<Value> {
    // If the types are not supported, we can't convert the arguments or return value.
    // Check this before trying to convert the arguments, and return an error if the types are not supported.
    if let Some(ty) = first_unsupported_interop_ty(ctx, input_ty) {
        return Err(QSharpError::new_err(format!(
            "unsupported input type: `{ty}`"
        )));
    }
    if let Some(ty) = first_unsupported_interop_ty(ctx, output_ty) {
        return Err(QSharpError::new_err(format!(
            "unsupported output type: `{ty}`"
        )));
    }

    // Conver the Python arguments to Q# values, treating None as an empty tuple aka `Unit`.
    if matches!(&input_ty, Ty::Tuple(tup) if tup.is_empty()) {
        // Special case for unit, where args should be None
        if args.is_some() {
            return Err(QSharpError::new_err("expected no arguments"));
        }
        Ok(Value::unit())
    } else {
        let Some(args) = args else {
            return Err(QSharpError::new_err(format!(
                "expected arguments of type `{input_ty}`"
            )));
        };
        // This conversion will produce errors if the types don't match or can't be converted.
        let value_ir = args.extract::<ValueIR>(py)?;
        Ok(convert_value_ir_with_ty(ctx, value_ir, input_ty)?)
    }
}

/// Finds any Q# type recursively that does not support interop with Python, meaning our code cannot convert it back and forth
/// across the interop boundary.
fn first_unsupported_interop_ty<'ctx, 'ty>(
    ctx: &'ctx interpret::Interpreter,
    ty: &'ty Ty,
) -> Option<&'ctx Ty>
where
    'ty: 'ctx,
{
    match ty {
        Ty::Prim(prim_ty) => match prim_ty {
            Prim::Pauli
            | Prim::BigInt
            | Prim::Bool
            | Prim::Double
            | Prim::Int
            | Prim::String
            | Prim::Result => None,
            Prim::Qubit | Prim::Range | Prim::RangeTo | Prim::RangeFrom | Prim::RangeFull => {
                Some(ty)
            }
        },
        Ty::Tuple(tup) => tup
            .iter()
            .find(|t| first_unsupported_interop_ty(ctx, t).is_some()),
        Ty::Array(ty) => first_unsupported_interop_ty(ctx, ty),
        Ty::Udt(_, res) => {
            let qsc::hir::Res::Item(item_id) = res else {
                panic!("Udt should be an item");
            };
            let Some(udt) = ctx.udt_ty(item_id) else {
                return Some(ty);
            };

            let Ok(fields) = collect_udt_fields(udt) else {
                return Some(ty);
            };

            for field in fields {
                if let Some(ty) = first_unsupported_interop_ty(ctx, field.1) {
                    return Some(ty);
                }
            }

            None
        }
        _ => Some(ty),
    }
}

fn collect_udt_fields<'ctx, 'udt_def>(
    udt: &'udt_def qsc::hir::ty::Udt,
) -> PyResult<Vec<(Rc<str>, &'ctx Ty)>>
where
    'udt_def: 'ctx,
{
    let mut fields = Vec::new();
    collect_udt_fields_rec(&udt.name, &udt.definition, &mut fields)?;
    Ok(fields)
}

fn collect_udt_fields_rec<'ctx, 'udt_def>(
    udt_name: &str,
    udt_def: &'udt_def qsc::hir::ty::UdtDef,
    buffer: &mut Vec<(Rc<str>, &'ctx Ty)>,
) -> PyResult<()>
where
    'udt_def: 'ctx,
{
    match &udt_def.kind {
        qsc::hir::ty::UdtDefKind::Field(udt_field) => {
            if let Some(name) = udt_field.name.as_ref() {
                buffer.push((name.clone(), &udt_field.ty));
                Ok(())
            } else {
                Err(QSharpError::new_err(format!(
                    "structs with anonymous fields are not supported: {udt_name}"
                )))
            }
        }
        qsc::hir::ty::UdtDefKind::Tuple(udt_defs) => {
            for udt_def in udt_defs {
                collect_udt_fields_rec(udt_name, udt_def, buffer)?;
            }
            Ok(())
        }
    }
}

/// A helper macro for converting a primitive `ValueIR` to a primitive `Value`
/// returning an error if the convertion fails.
macro_rules! convert_prim {
    ($val:expr, $prim:ident) => {
        if let ValueIR::Primitive(PrimitiveValue::$prim(val)) = $val {
            Ok(Value::$prim(val.into()))
        } else {
            return Err(QSharpError::new_err(format!(
                "mismatched types: expected {}, found {}",
                stringify!($prim),
                $val.ty_name()
            )));
        }
    };
}

/// Given a type, convert a Python object into a Q# value of that type. This will recur through tuples and arrays,
/// and will return an error if the type is not supported or the object cannot be converted.
fn convert_value_ir_with_ty(
    ctx: &interpret::Interpreter,
    value_ir: ValueIR,
    ty: &Ty,
) -> PyResult<Value> {
    match ty {
        Ty::Prim(prim_ty) => match prim_ty {
            Prim::Bool => convert_prim!(value_ir, Bool),
            Prim::Int => convert_prim!(value_ir, Int),
            Prim::BigInt => convert_prim!(value_ir, BigInt),
            Prim::Double => convert_prim!(value_ir, Double),
            Prim::Result => convert_prim!(value_ir, Result),
            Prim::Pauli => convert_prim!(value_ir, Pauli),
            Prim::String => convert_prim!(value_ir, String),
            Prim::Qubit | Prim::Range | Prim::RangeTo | Prim::RangeFrom | Prim::RangeFull => {
                unimplemented!("primitive input type: {prim_ty:?}")
            }
        },
        Ty::Tuple(tup) => {
            if let ValueIR::Tuple(values) = value_ir {
                if tup.len() != values.len() {
                    return Err(QSharpError::new_err(format!(
                        "mismatched tuple arity: expected {}, got {}",
                        tup.len(),
                        values.len()
                    )));
                }

                if values.len() == 1 {
                    let val = values
                        .into_iter()
                        .next()
                        .expect("there is exactly one element");
                    convert_value_ir_with_ty(ctx, val, &tup[0])
                } else {
                    let mut tuple = Vec::new();
                    for (val, ty) in values.into_iter().zip(tup) {
                        tuple.push(convert_value_ir_with_ty(ctx, val, ty)?);
                    }
                    Ok(Value::Tuple(tuple.into()))
                }
            } else {
                Err(QSharpError::new_err(format!(
                    "mismatched types: expected {}, found {}",
                    ty,
                    value_ir.ty_name()
                )))
            }
        }
        Ty::Array(ty) => {
            if let ValueIR::Array(values) = value_ir {
                let ty = &**ty;
                let mut array = Vec::new();
                for val in values {
                    array.push(convert_value_ir_with_ty(ctx, val, ty)?);
                }
                Ok(Value::Array(array.into()))
            } else {
                Err(QSharpError::new_err(format!(
                    "mismatched types: expected {}, found {}",
                    ty,
                    value_ir.ty_name()
                )))
            }
        }
        Ty::Udt(_, res) => {
            let ValueIR::Udt(udt_value) = &value_ir else {
                // Handle `Complex` special case.
                if value_ir.is_complex() {
                    let ValueIR::Primitive(PrimitiveValue::Complex(v)) = value_ir else {
                        unreachable!("we checked the value is complex");
                    };
                    let tuple = Value::Tuple(Rc::new([Value::Double(v.re), Value::Double(v.im)]));
                    return Ok(tuple);
                }

                return Err(QSharpError::new_err(format!(
                    "mismatched types: expected {}, found {}",
                    ty,
                    value_ir.ty_name()
                )));
            };

            let qsc::hir::Res::Item(item_id) = res else {
                panic!("Udt should be an item");
            };
            let Some(udt) = ctx.udt_ty(item_id) else {
                unreachable!(
                    "we verified that the udt is defined in `first_unsupported_interop_ty`"
                );
            };

            let mut tuple = Vec::new();
            for (name, ty) in collect_udt_fields(udt)? {
                let Some(value) = udt_value.fields.get(&*name) else {
                    return Err(QSharpError::new_err(format!(
                        "mismatched types: missing field {} in {}",
                        name, udt.name,
                    )));
                };
                verify_that_field_type_matches_field_value(ctx, ty, value)?;
                tuple.push(convert_value_ir_with_ty(ctx, value.clone(), ty)?);
            }
            Ok(Value::Tuple(tuple.into()))
        }
        _ => unimplemented!("input type: {ty}"),
    }
}

fn verify_that_udt_matches_value(
    ctx: &interpret::Interpreter,
    udt: &qsc::hir::ty::Udt,
    value: &UdtValue,
) -> PyResult<()> {
    verify_that_udt_def_matches_fields(ctx, &udt.name, &udt.definition, &value.fields)
}

fn verify_that_udt_def_matches_fields(
    ctx: &interpret::Interpreter,
    udt_name: &str,
    udt_def: &qsc::hir::ty::UdtDef,
    fields: &FxHashMap<String, ValueIR>,
) -> PyResult<()> {
    match &udt_def.kind {
        qsc::hir::ty::UdtDefKind::Field(udt_field) => {
            let Some(udt_field_name) = udt_field.name.clone() else {
                return Err(QSharpError::new_err(format!(
                    "unsupported: {udt_name} has anonymous fields",
                )));
            };

            let Some(field_value) = fields.get(&*udt_field_name) else {
                return Err(QSharpError::new_err(format!(
                    "mismatched types: missing field {udt_field_name} in {udt_name}",
                )));
            };

            verify_that_field_type_matches_field_value(ctx, &udt_field.ty, field_value)?;
        }
        qsc::hir::ty::UdtDefKind::Tuple(udt_defs) => {
            for udt_def in udt_defs {
                verify_that_udt_def_matches_fields(ctx, udt_name, udt_def, fields)?;
            }
        }
    }

    Ok(())
}

fn verify_that_field_type_matches_field_value(
    ctx: &interpret::Interpreter,
    ty: &Ty,
    value: &ValueIR,
) -> PyResult<()> {
    match ty {
        Ty::Arrow(..) | Ty::Infer(..) | Ty::Param { .. } | Ty::Err => {
            unreachable!("we verified unsupported types in `first_unsupported_interop_ty`");
        }
        Ty::Prim(prim) => match (prim, value) {
            (Prim::Pauli, ValueIR::Primitive(PrimitiveValue::Pauli(..)))
            | (Prim::Bool, ValueIR::Primitive(PrimitiveValue::Bool(..)))
            | (Prim::Double, ValueIR::Primitive(PrimitiveValue::Double(..)))
            | (Prim::Int, ValueIR::Primitive(PrimitiveValue::Int(..)))
            | (Prim::BigInt, ValueIR::Primitive(PrimitiveValue::BigInt(..)))
            | (Prim::String, ValueIR::Primitive(PrimitiveValue::String(..)))
            | (Prim::Result, ValueIR::Primitive(PrimitiveValue::Result(..))) => (),
            _ => {
                return Err(QSharpError::new_err(format!(
                    "mismatched types: expected {}, found {}",
                    ty,
                    value.ty_name()
                )));
            }
        },
        Ty::Array(ty) => {
            if let ValueIR::Array(values) = value {
                for value in values {
                    verify_that_field_type_matches_field_value(ctx, ty, value)?;
                }
            } else {
                return Err(QSharpError::new_err(format!(
                    "mismatched types: expected {}, found {}",
                    ty,
                    value.ty_name()
                )));
            }
        }
        Ty::Tuple(items) => {
            if let ValueIR::Tuple(values) = value
                && items.len() == values.len()
            {
                for (ty, value) in items.iter().zip(values) {
                    verify_that_field_type_matches_field_value(ctx, ty, value)?;
                }
            } else {
                return Err(QSharpError::new_err(format!(
                    "mismatched types: expected {}, found {}",
                    ty,
                    value.ty_name()
                )));
            }
        }
        Ty::Udt(_, res) => {
            let ValueIR::Udt(udt_value) = &value else {
                // Handle `Complex` special case.
                if value.is_complex() {
                    return Ok(());
                }

                return Err(QSharpError::new_err(format!(
                    "mismatched types: expected {}, found {}",
                    ty,
                    value.ty_name()
                )));
            };
            let qsc::hir::Res::Item(item_id) = res else {
                panic!("Udt should be an item");
            };
            let Some(udt) = ctx.udt_ty(item_id) else {
                unreachable!(
                    "we verified that the udt is defined in `first_unsupported_interop_ty`"
                );
            };
            verify_that_udt_matches_value(ctx, udt, udt_value)?;
        }
    }

    Ok(())
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
        .collect::<Vec<_>>()
        .join("\n")
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
            DisplayableOutput::Matrix(matrix) => matrix.to_plain(),
            DisplayableOutput::Message(msg) => msg.clone(),
        }
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    fn _repr_markdown_(&self) -> Option<String> {
        match &self.0 {
            DisplayableOutput::State(state) => {
                let latex = if let Some(latex) = state.to_latex() {
                    format!("\n\n{latex}")
                } else {
                    String::default()
                };
                Some(format!("{}{latex}", state.to_html()))
            }
            DisplayableOutput::Message(_) => None,
            DisplayableOutput::Matrix(matrix) => Some(matrix.to_latex()),
        }
    }

    fn state_dump(&self) -> Option<StateDumpData> {
        match &self.0 {
            DisplayableOutput::State(state) => Some(StateDumpData(state.clone())),
            DisplayableOutput::Matrix(_) | DisplayableOutput::Message(_) => None,
        }
    }

    fn is_state_dump(&self) -> bool {
        matches!(&self.0, DisplayableOutput::State(_))
    }

    fn is_matrix(&self) -> bool {
        matches!(&self.0, DisplayableOutput::Matrix(_))
    }

    fn is_message(&self) -> bool {
        matches!(&self.0, DisplayableOutput::Message(_))
    }
}

#[pyclass]
/// Captured simlation state dump.
pub(crate) struct StateDumpData(pub(crate) DisplayableState);

#[pymethods]
impl StateDumpData {
    fn get_dict<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyDict>> {
        let dict = rustc_hash::FxHashMap::from_iter(self.0.0.clone());
        dict.into_pyobject(py)
    }

    #[getter]
    fn get_qubit_count(&self) -> usize {
        self.0.1
    }

    fn __len__(&self) -> usize {
        self.0.0.len()
    }

    fn __repr__(&self) -> String {
        self.0.to_plain()
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    fn _repr_markdown_(&self) -> String {
        let latex = if let Some(latex) = self.0.to_latex() {
            format!("\n\n{latex}")
        } else {
            String::default()
        };
        format!("{}{latex}", self.0.to_html())
    }

    fn _repr_latex_(&self) -> Option<String> {
        self.0.to_latex()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[pyclass(eq, eq_int, ord)]
/// A Q# measurement result.
pub(crate) enum Result {
    Zero,
    One,
    Loss,
}

impl From<Result> for qsc::interpret::Result {
    fn from(value: Result) -> Self {
        qsc::interpret::Result::Val(value == Result::One)
    }
}

#[pymethods]
impl Result {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn __repr__(&self) -> String {
        match self {
            Result::Zero => "Zero".to_owned(),
            Result::One => "One".to_owned(),
            Result::Loss => "Loss".to_owned(),
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
            Result::Loss => 2,
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

impl From<Pauli> for fir::Pauli {
    fn from(value: Pauli) -> Self {
        match value {
            Pauli::I => fir::Pauli::I,
            Pauli::X => fir::Pauli::X,
            Pauli::Y => fir::Pauli::Y,
            Pauli::Z => fir::Pauli::Z,
        }
    }
}

// Mapping of Q# value types to Python value types.
pub(crate) struct ValueWrapper(pub(crate) Value);

impl<'py> IntoPyObject<'py> for ValueWrapper {
    type Target = PyAny;

    type Output = Bound<'py, Self::Target>;

    type Error = pyo3::PyErr;

    fn into_pyobject(self, py: Python<'py>) -> std::result::Result<Self::Output, Self::Error> {
        match self.0 {
            Value::Int(val) => val.into_bound_py_any(py),
            Value::BigInt(val) => val.into_bound_py_any(py),
            Value::Double(val) => val.into_bound_py_any(py),
            Value::Bool(val) => val.into_bound_py_any(py),
            Value::String(val) => val.into_bound_py_any(py),
            Value::Result(val) => match val {
                qsc::interpret::Result::Id(_) => panic!("unexpected Result::Id in ValueWrapper"),
                qsc::interpret::Result::Val(true) => Result::One,
                qsc::interpret::Result::Val(false) => Result::Zero,
                qsc::interpret::Result::Loss => Result::Loss,
            }
            .into_bound_py_any(py),
            Value::Pauli(val) => match val {
                fir::Pauli::I => Pauli::I.into_bound_py_any(py),
                fir::Pauli::X => Pauli::X.into_bound_py_any(py),
                fir::Pauli::Y => Pauli::Y.into_bound_py_any(py),
                fir::Pauli::Z => Pauli::Z.into_bound_py_any(py),
            },
            Value::Tuple(val) => {
                if val.is_empty() {
                    // Special case Value::unit as None
                    Ok(py.None().into_bound(py))
                } else {
                    PyTuple::new(py, val.iter().map(|v| ValueWrapper(v.clone())))?
                        .into_bound_py_any(py)
                }
            }
            Value::Array(val) => {
                PyList::new(py, val.iter().map(|v| ValueWrapper(v.clone())))?.into_bound_py_any(py)
            }
            _ => format!("<{}> {}", Value::type_name(&self.0), &self.0).into_bound_py_any(py),
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
                    PyTuple::new(
                        self.py,
                        &[Py::new(self.py, Output(out)).expect("should be able to create output")],
                    )
                    .map_err(|_| Error)?,
                )
                .map_err(|_| Error)?;
        }
        Ok(())
    }

    fn matrix(&mut self, matrix: Vec<Vec<Complex64>>) -> std::result::Result<(), Error> {
        if let Some(callback) = &self.callback {
            let out = DisplayableOutput::Matrix(DisplayableMatrix(matrix));
            callback
                .call1(
                    self.py,
                    PyTuple::new(
                        self.py,
                        &[Py::new(self.py, Output(out)).expect("should be able to create output")],
                    )
                    .map_err(|_| Error)?,
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
                    )
                    .map_err(|_| Error)?,
                )
                .map_err(|_| Error)?;
        }
        Ok(())
    }
}

#[pyclass]
pub(crate) struct Circuit(pub qsc::circuit::Circuit);

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

#[pyclass(unsendable)]
#[derive(Clone)]
struct GlobalCallable(Value);

impl From<Value> for GlobalCallable {
    fn from(val: Value) -> Self {
        match val {
            val @ Value::Global(..) => GlobalCallable(val),
            _ => panic!("expected global callable"),
        }
    }
}

impl From<GlobalCallable> for Value {
    fn from(val: GlobalCallable) -> Self {
        val.0
    }
}

/// Create a Python callable from a Q# callable and adds it to the given environment.
fn create_py_callable(
    py: Python,
    make_callable: &PyObject,
    namespace: &[Rc<str>],
    name: &str,
    val: Value,
) -> PyResult<()> {
    if namespace.is_empty() && name == "<lambda>" {
        // We don't want to bind auto-generated lambda callables.
        return Ok(());
    }

    let args = (
        Py::new(py, GlobalCallable::from(val)).expect("should be able to create callable"), // callable id
        PyList::new(py, namespace.iter().map(ToString::to_string))?, // namespace as string array
        PyString::new(py, name),                                     // name of callable
    );

    // Call into the Python layer to create the function wrapping the callable invocation.
    make_callable.call1(py, args)?;

    Ok(())
}

/// Create a Python class from a Q# type and adds it to the given environment.
fn create_py_class(
    ctx: &interpret::Interpreter,
    py: Python,
    make_class: &PyObject,
    namespace: &[Rc<str>],
    name: &str,
    ty: &Ty,
) -> PyResult<()> {
    let args = (
        Py::new(py, type_ir_from_qsharp_ty(ctx, ty)?).expect("should be able to create callable"), // callable id
        PyList::new(py, namespace.iter().map(ToString::to_string))?, // namespace as string array
        PyString::new(py, name),                                     // name of callable
    );

    // Call into the Python layer to create the function wrapping the callable invocation.
    make_class.call1(py, args)?;

    Ok(())
}
