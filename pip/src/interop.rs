// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::path::{Path, PathBuf};

use std::fmt::Write;

use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use qsc::compile::ErrorKind;
use qsc::interpret::into_errors;
use qsc::interpret::output::Receiver;
use qsc::target::Profile;
use qsc::{
    ast::Package, error::WithSource, interpret, project::FileSystem, LanguageFeatures,
    PackageStore, SourceMap, TargetCapabilityFlags,
};
use qsc::{Backend, PackageType, SparseSim};
use qsc_qasm3::io::SourceResolver;
use qsc_qasm3::{OutputSemantics, ProgramType};

use crate::fs::file_system;
use crate::interpreter::{
    format_errors, OptionalCallbackReceiver, QSharpError, QasmError, TargetProfile, ValueWrapper,
};

use resource_estimator::{self as re};

struct ImportResolver<T>
where
    T: FileSystem,
{
    fs: T,
    path: PathBuf,
}

impl<T> ImportResolver<T>
where
    T: FileSystem,
{
    fn new<P: AsRef<Path>>(fs: T, path: P) -> Self {
        Self {
            fs,
            path: PathBuf::from(path.as_ref()),
        }
    }
}

impl<T> SourceResolver for ImportResolver<T>
where
    T: FileSystem,
{
    fn resolve<P>(&self, path: P) -> miette::Result<(PathBuf, String)>
    where
        P: AsRef<Path>,
    {
        let path = self.path.join(path);
        let (path, source) = self.fs.read_file(path.as_ref())?;
        Ok((
            PathBuf::from(path.as_ref().to_owned()),
            source.as_ref().to_owned(),
        ))
    }
}

/// This call while exported is not intended to be used directly by the user.
/// It is intended to be used by the Python wrapper which will handle the
/// callbacks and other Python specific details.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
#[pyo3(
    signature = (source, callback, read_file, list_directory, resolve_path, fetch_github, **kwargs)
)]
pub fn run_qasm3(
    py: Python,
    source: &str,
    callback: Option<PyObject>,
    read_file: Option<PyObject>,
    list_directory: Option<PyObject>,
    resolve_path: Option<PyObject>,
    fetch_github: Option<PyObject>,
    kwargs: Option<&PyDict>,
) -> PyResult<PyObject> {
    let kwargs = kwargs.unwrap_or_else(|| PyDict::new(py));
    let target: TargetProfile = kwargs.get_item("target_profile")?.map_or_else(
        || Ok(TargetProfile::Unrestricted),
        pyo3::PyAny::extract::<TargetProfile>,
    )?;
    let name = kwargs
        .get_item("name")?
        .map_or_else(|| Ok("program"), pyo3::PyAny::extract::<&str>)?;

    // sanitize the name to ensure it is a valid identifier
    // When creating operation, we'll throw an error if the name is not a valid identifier
    // so that the user gets the exact name they expect, but here it's better to sanitize.
    let name = sanitize_name(name);

    let seed = kwargs
        .get_item("seed")?
        .map_or_else(|| None::<u64>, |x| x.extract::<u64>().ok());
    let shots = kwargs.get_item("shots")?.map_or_else(
        || Err(PyException::new_err("Could not parse shots".to_string())),
        pyo3::PyAny::extract::<usize>,
    )?;
    let search_path = kwargs.get_item("search_path")?.map_or_else(
        || {
            Err(PyException::new_err(
                "Could not parse search path".to_string(),
            ))
        },
        pyo3::PyAny::extract::<&str>,
    )?;

    let target = target.into();
    let language_features = LanguageFeatures::default();
    let fs = create_filesystem_from_py(py, read_file, list_directory, resolve_path, fetch_github);
    let resolver = ImportResolver::new(fs, PathBuf::from(search_path));

    let unit = qsc_qasm3::compile_qasm_to_program(
        source,
        format!("{name}.qasm"),
        &resolver,
        ProgramType::File(name.to_string()),
        OutputSemantics::Qiskit,
    )
    .map_err(|e| QSharpError::new_err(e.to_string()))?;

    if unit.has_errors() {
        return Err(QasmError::new_err(format_qasm_errors(unit.errors())));
    }
    let (source_map, _, package) = unit.into_tuple();
    let Some(package) = package else {
        return Err(QasmError::new_err("package should have had value"));
    };
    let mut receiver = OptionalCallbackReceiver { callback, py };
    match run_ast(
        package,
        source_map,
        &mut receiver,
        target,
        language_features,
        shots,
        seed,
    ) {
        Ok(result) => Ok(PyList::new(
            py,
            result.iter().map(|v| ValueWrapper(v.clone()).into_py(py)),
        )
        .into_py(py)),
        Err(errors) => Err(QSharpError::new_err(format_errors(errors))),
    }
}

fn run_ast(
    ast_package: Package,
    source_map: SourceMap,
    receiver: &mut impl Receiver,
    profile: Profile,
    language_features: LanguageFeatures,
    shots: usize,
    seed: Option<u64>,
) -> Result<Vec<qsc::interpret::Value>, Vec<interpret::Error>> {
    let mut store = PackageStore::new(qsc::compile::core());
    let mut dependencies = Vec::new();

    let (package_type, capabilities) = (PackageType::Exe, profile.into());

    dependencies.push((store.insert(qsc::compile::std(&store, capabilities)), None));
    let (mut unit, errors) = qsc::compile::compile_ast(
        &store,
        &dependencies,
        ast_package,
        source_map,
        package_type,
        capabilities,
    );
    unit.expose();
    if !errors.is_empty() {
        return Err(into_errors(errors));
    }

    let source_package_id = store.insert(unit);

    let mut interpreter = interpret::Interpreter::from(
        store,
        source_package_id,
        capabilities,
        language_features,
        &dependencies,
    )?;

    let mut results = Vec::with_capacity(shots);
    for i in 0..shots {
        let mut sim = SparseSim::new();
        // If seed is provided, we want to use a different seed for each shot
        // so that the results are different for each shot, but still deterministic
        sim.set_seed(seed.map(|s| s + i as u64));
        results.push(interpreter.eval_entry_with_sim(&mut sim, receiver)?);
    }

    Ok(results)
}

/// This call while exported is not intended to be used directly by the user.
/// It is intended to be used by the Python wrapper which will handle the
/// callbacks and other Python specific details.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
#[pyo3(
    signature = (source, job_params, read_file, list_directory, resolve_path, fetch_github, **kwargs)
)]
pub(crate) fn resource_estimate_qasm3(
    py: Python,
    source: &str,
    job_params: &str,
    read_file: Option<PyObject>,
    list_directory: Option<PyObject>,
    resolve_path: Option<PyObject>,
    fetch_github: Option<PyObject>,
    kwargs: Option<&PyDict>,
) -> PyResult<String> {
    let kwargs = kwargs.unwrap_or_else(|| PyDict::new(py));
    let name = kwargs
        .get_item("name")?
        .map_or_else(|| Ok("program"), pyo3::PyAny::extract::<&str>)?;

    // sanitize the name to ensure it is a valid identifier
    // When creating operation, we'll throw an error if the name is not a valid identifier
    // so that the user gets the exact name they expect, but here it's better to sanitize.
    let name = sanitize_name(name);

    let search_path = kwargs.get_item("search_path")?.map_or_else(
        || {
            Err(PyException::new_err(
                "Could not parse search path".to_string(),
            ))
        },
        pyo3::PyAny::extract::<&str>,
    )?;

    let fs = create_filesystem_from_py(py, read_file, list_directory, resolve_path, fetch_github);
    let resolver = ImportResolver::new(fs, PathBuf::from(search_path));

    // The output semantics are set to OpenQasm because we are only interested in the resource
    // estimation and not the QIR output. So there is no need to do extra work
    // for the simulator to coerce the output.
    let unit = qsc_qasm3::compile_qasm_to_program(
        source,
        format!("{name}.qasm"),
        &resolver,
        ProgramType::File(name),
        OutputSemantics::ResourceEstimation,
    )
    .map_err(|e| QSharpError::new_err(e.to_string()))?;
    if unit.has_errors() {
        return Err(QasmError::new_err(format_qasm_errors(unit.errors())));
    }
    let (source_map, _, package) = unit.into_tuple();
    match crate::interop::estimate_qasm3(
        package.expect("Package must exist when there are no errors"),
        source_map,
        job_params,
    ) {
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

/// This call while exported is not intended to be used directly by the user.
/// It is intended to be used by the Python wrapper which will handle the
/// callbacks and other Python specific details.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
#[pyo3(
    signature = (source, read_file, list_directory, resolve_path, fetch_github, **kwargs)
)]
pub(crate) fn compile_qasm3_to_qir(
    py: Python,
    source: &str,
    read_file: Option<PyObject>,
    list_directory: Option<PyObject>,
    resolve_path: Option<PyObject>,
    fetch_github: Option<PyObject>,
    kwargs: Option<&PyDict>,
) -> PyResult<String> {
    let kwargs = kwargs.unwrap_or_else(|| PyDict::new(py));
    let target: TargetProfile = kwargs.get_item("target_profile")?.map_or_else(
        || Ok(TargetProfile::Unrestricted),
        pyo3::PyAny::extract::<TargetProfile>,
    )?;
    let name = kwargs
        .get_item("name")?
        .map_or_else(|| Ok("program"), pyo3::PyAny::extract::<&str>)?;

    // sanitize the name to ensure it is a valid identifier
    // When creating operation, we'll throw an error if the name is not a valid identifier
    // so that the user gets the exact name they expect, but here it's better to sanitize.
    let name = sanitize_name(name);
    let entry_expr = kwargs.get_item("entry_expr")?.map_or_else(
        || {
            Err(PyException::new_err(
                "Could not parse entry expr".to_string(),
            ))
        },
        |expr| {
            Ok(expr
                .extract::<&str>()
                .map(ToString::to_string)
                .unwrap_or(format!("qasm3_import.{name}()")))
        },
    )?;

    let search_path = kwargs.get_item("search_path")?.map_or_else(
        || {
            Err(PyException::new_err(
                "Could not parse search path".to_string(),
            ))
        },
        pyo3::PyAny::extract::<&str>,
    )?;

    let target = target.into();
    let language_features = LanguageFeatures::default();
    let fs = create_filesystem_from_py(py, read_file, list_directory, resolve_path, fetch_github);
    let resolver = ImportResolver::new(fs, PathBuf::from(search_path));

    let unit = qsc_qasm3::compile_qasm_to_program(
        source,
        format!("{}.qasm", &name),
        &resolver,
        ProgramType::File(name.clone()),
        OutputSemantics::Qiskit,
    )
    .map_err(|e| QSharpError::new_err(e.to_string()))?;
    if unit.has_errors() {
        return Err(QasmError::new_err(format_qasm_errors(unit.errors())));
    }
    let (source_map, _, package) = unit.into_tuple();
    let Some(package) = package else {
        return Err(QasmError::new_err("package should have had value"));
    };

    generate_qir_from_ast(entry_expr, package, source_map, target, language_features)
        .map_err(|errors| QSharpError::new_err(format_errors(errors)))
}

fn generate_qir_from_ast<S: AsRef<str>>(
    entry_expr: S,
    ast_package: Package,
    source_map: SourceMap,
    profile: Profile,
    language_features: LanguageFeatures,
) -> Result<String, Vec<interpret::Error>> {
    let mut store = PackageStore::new(qsc::compile::core());
    let mut dependencies = Vec::new();

    let (package_type, capabilities) = (PackageType::Lib, profile.into());

    dependencies.push((store.insert(qsc::compile::std(&store, capabilities)), None));
    let (mut unit, errors) = qsc::compile::compile_ast(
        &store,
        &dependencies,
        ast_package,
        source_map,
        package_type,
        capabilities,
    );

    if !errors.is_empty() {
        return Err(into_errors(errors));
    }

    unit.expose();
    let source_package_id = store.insert(unit);

    let mut interpreter = interpret::Interpreter::from(
        store,
        source_package_id,
        capabilities,
        language_features,
        &dependencies,
    )?;
    interpreter.qirgen(entry_expr.as_ref())
}

fn estimate_qasm3(
    ast_package: Package,
    source_map: SourceMap,
    params: &str,
) -> Result<String, Vec<resource_estimator::Error>> {
    let mut store = PackageStore::new(qsc::compile::core());
    let mut dependencies = Vec::new();

    let (package_type, capabilities) = (qsc::PackageType::Exe, TargetCapabilityFlags::all());

    dependencies.push((store.insert(qsc::compile::std(&store, capabilities)), None));
    let (mut unit, errors) = qsc::compile::compile_ast(
        &store,
        &dependencies,
        ast_package,
        source_map,
        package_type,
        capabilities,
    );
    unit.expose();
    if !errors.is_empty() {
        return Err(map_compile_errors_to_estimation_errors(errors));
    }

    let package_id = store.insert(unit);

    let mut interpreter = interpret::Interpreter::from(
        store,
        package_id,
        capabilities,
        LanguageFeatures::empty(),
        &dependencies,
    )
    .map_err(into_estimation_errors)?;
    resource_estimator::estimate_entry(&mut interpreter, params)
}

fn map_compile_errors_to_estimation_errors(
    errors: Vec<WithSource<ErrorKind>>,
) -> Vec<resource_estimator::Error> {
    errors
        .into_iter()
        .map(|error| {
            resource_estimator::Error::Interpreter(qsc::interpret::Error::Compile(error.clone()))
        })
        .collect::<Vec<_>>()
}

fn into_estimation_errors(errors: Vec<interpret::Error>) -> Vec<resource_estimator::Error> {
    errors
        .into_iter()
        .map(|error| resource_estimator::Error::Interpreter(error.clone()))
        .collect::<Vec<_>>()
}

pub(crate) fn format_qasm_errors(errors: Vec<WithSource<qsc_qasm3::Error>>) -> String {
    errors
        .into_iter()
        .map(|e| {
            let mut message = String::new();
            let report = miette::Report::new(e);
            write!(message, "{report:?}").unwrap();
            message
        })
        .collect::<String>()
}

fn create_filesystem_from_py(
    py: Python,
    read_file: Option<PyObject>,
    list_directory: Option<PyObject>,
    resolve_path: Option<PyObject>,
    fetch_github: Option<PyObject>,
) -> impl FileSystem + '_ {
    file_system(
        py,
        read_file.expect("file system hooks should have been passed in with a read file callback"),
        list_directory
            .expect("file system hooks should have been passed in with a list directory callback"),
        resolve_path
            .expect("file system hooks should have been passed in with a resolve path callback"),
        fetch_github
            .expect("file system hooks should have been passed in with a fetch github callback"),
    )
}

fn sanitize_name(name: &str) -> String {
    if name.is_empty() {
        return "circuit".to_string();
    }

    let mut output = String::with_capacity(name.len());
    let c = name.chars().next().expect("name should not be empty");
    if c == '_' || c.is_alphabetic() {
        output.push(c);
    } else {
        // invalid first character, replace with '_'
        output.push('_');
    }
    output.extend(name.chars().skip(1).filter_map(|c| {
        if c == '-' {
            Some('_')
        } else if c == '_' || c.is_alphanumeric() {
            Some(c)
        } else {
            None
        }
    }));
    output
}
