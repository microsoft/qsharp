// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::path::Path;
use std::vec;

use qsc_data_structures::target::TargetCapabilityFlags;
use qsc_frontend::compile::PackageStore;
use qsc_frontend::error::WithSource;
use qsc_hir::hir::PackageId;
use qsc_passes::PackageType;
pub use qsc_qasm3::CompilerConfig;
pub use qsc_qasm3::OperationSignature;
pub use qsc_qasm3::OutputSemantics;
pub use qsc_qasm3::ProgramType;
pub use qsc_qasm3::QasmCompileUnit;
pub use qsc_qasm3::QubitSemantics;
pub mod io {
    pub use qsc_qasm3::io::*;
}
pub mod parser {
    pub use qsc_qasm3::parser::*;
}
pub mod error {
    pub use qsc_qasm3::Error;
    pub use qsc_qasm3::ErrorKind;
}
pub mod completion {
    pub use qsc_qasm3::parser::completion::*;
}
pub use qsc_qasm3::package_store_with_qasm;
pub mod old {
    pub use qsc_qasm3::parse::parse_source;
    pub use qsc_qasm3::qasm_to_program;
}

#[must_use]
pub fn parse_raw_qasm_as_fragments<S, P>(source: S, path: P) -> QasmCompileUnit
where
    S: AsRef<str>,
    P: AsRef<Path>,
{
    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::Fragments,
        None,
        None,
    );
    qsc_qasm3::compile_with_config(source, path, config)
}

#[must_use]
pub fn parse_raw_qasm_as_operation<S, P>(source: S, name: S, path: P) -> QasmCompileUnit
where
    S: AsRef<str>,
    P: AsRef<Path>,
{
    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::Operation,
        Some(name.as_ref().into()),
        None,
    );
    qsc_qasm3::compile_with_config(source, path, config)
}

#[must_use]
pub fn compile_raw_qasm<S, P>(
    source: S,
    path: P,
    package_type: PackageType,
    capabilities: TargetCapabilityFlags,
) -> (
    PackageStore,
    PackageId,
    Vec<(PackageId, Option<std::sync::Arc<str>>)>,
    Option<OperationSignature>,
    Vec<crate::compile::Error>,
)
where
    S: AsRef<str>,
    P: AsRef<Path>,
{
    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::File,
        Some("Main".into()),
        None,
    );
    compile_with_config(source, path, config, package_type, capabilities)
}

#[must_use]
pub fn compile_qiskit_qasm<S, P>(
    source: S,
    path: P,
    package_type: PackageType,
    capabilities: TargetCapabilityFlags,
) -> (
    PackageStore,
    PackageId,
    Vec<(PackageId, Option<std::sync::Arc<str>>)>,
    Option<OperationSignature>,
    Vec<crate::compile::Error>,
)
where
    S: AsRef<str>,
    P: AsRef<Path>,
{
    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::Qiskit,
        ProgramType::File,
        Some("Main".into()),
        None,
    );
    compile_with_config(source, path, config, package_type, capabilities)
}

#[must_use]
pub fn convert_with_config<S, P>(source: S, path: P, config: CompilerConfig) -> QasmCompileUnit
where
    S: AsRef<str>,
    P: AsRef<Path>,
{
    qsc_qasm3::compile_with_config(source, path, config)
}

#[must_use]
pub fn compile_with_config<S, P>(
    source: S,
    path: P,
    config: CompilerConfig,
    package_type: PackageType,
    capabilities: TargetCapabilityFlags,
) -> (
    PackageStore,
    PackageId,
    Vec<(PackageId, Option<std::sync::Arc<str>>)>,
    Option<OperationSignature>,
    Vec<crate::compile::Error>,
)
where
    S: AsRef<str>,
    P: AsRef<Path>,
{
    let unit = convert_with_config(source, path, config);

    let (source_map, errors, package, sig) = unit.into_tuple();

    let (stdid, qasmid, mut store) = qsc_qasm3::package_store_with_qasm(capabilities);
    let dependencies = vec![
        (PackageId::CORE, None),
        (stdid, None),
        (qasmid, Some("QasmStd".into())),
    ];

    let (mut unit, compile_errors) = crate::compile::compile_ast(
        &store,
        &dependencies,
        package.expect("Should have a package"),
        source_map.clone(),
        package_type,
        capabilities,
    );
    unit.expose();
    let source_package_id = store.insert(unit);

    let mut compile_errors = compile_errors;
    for error in errors {
        let err = WithSource::from_map(
            &source_map,
            crate::compile::ErrorKind::OpenQasm(error.into_error()),
        );
        compile_errors.push(err);
    }

    (store, source_package_id, dependencies, sig, compile_errors)
}
