// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::path::Path;

use qsc_qasm::io::SourceResolver;
pub use qsc_qasm::{
    CompilerConfig, OperationSignature, OutputSemantics, ProgramType, QasmCompileUnit,
    QubitSemantics,
};
pub mod io {
    pub use qsc_qasm::io::*;
}
pub mod parser {
    pub use qsc_qasm::parser::*;
}
pub mod error {
    pub use qsc_qasm::Error;
    pub use qsc_qasm::ErrorKind;
}
pub mod completion {
    pub use qsc_qasm::parser::completion::*;
}
pub use qsc_qasm::compile_to_qsharp_ast_with_config;
pub use qsc_qasm::package_store_with_qasm;

#[must_use]
pub fn parse_raw_qasm_as_fragments<S, P, R>(
    source: S,
    path: P,
    resolver: Option<&mut R>,
) -> QasmCompileUnit
where
    S: AsRef<str>,
    P: AsRef<Path>,
    R: SourceResolver,
{
    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::Fragments,
        None,
        None,
    );
    compile_to_qsharp_ast_with_config(source, path, resolver, config)
}

#[must_use]
pub fn parse_raw_qasm_as_operation<S, P, R>(
    source: S,
    name: S,
    path: P,
    resolver: Option<&mut R>,
) -> QasmCompileUnit
where
    S: AsRef<str>,
    P: AsRef<Path>,
    R: SourceResolver,
{
    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::Operation,
        Some(name.as_ref().into()),
        None,
    );
    compile_to_qsharp_ast_with_config(source, path, resolver, config)
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
    qsc_qasm3::compile_to_qsharp_ast_with_config(
        source,
        path,
        None::<&mut InMemorySourceResolver>,
        config,
    )
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
