// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::sync::Arc;

use qsc_data_structures::target::TargetCapabilityFlags;
use qsc_frontend::compile::PackageStore;
use qsc_frontend::error::WithSource;
use qsc_hir::hir::PackageId;
use qsc_passes::PackageType;
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

use crate::compile::package_store_with_stdlib;

pub struct CompileRawQasmResult(
    pub PackageStore,
    pub PackageId,
    pub Vec<(PackageId, Option<std::sync::Arc<str>>)>,
    pub Option<OperationSignature>,
    pub Vec<crate::compile::Error>,
);

#[must_use]
pub fn compile_raw_qasm<R: SourceResolver, S: Into<Arc<str>>>(
    source: S,
    path: S,
    resolver: Option<&mut R>,
    package_type: PackageType,
    capabilities: TargetCapabilityFlags,
) -> CompileRawQasmResult {
    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::File,
        Some("program".into()),
        None,
    );
    compile_with_config(source, path, resolver, config, package_type, capabilities)
}

#[must_use]
pub fn compile_with_config<R: SourceResolver, S: Into<Arc<str>>>(
    source: S,
    path: S,
    resolver: Option<&mut R>,
    config: CompilerConfig,
    package_type: PackageType,
    capabilities: TargetCapabilityFlags,
) -> CompileRawQasmResult {
    let unit = compile_to_qsharp_ast_with_config(source, path, resolver, config);

    let (source_map, errors, package, sig) = unit.into_tuple();

    let (stdid, mut store) = package_store_with_stdlib(capabilities);
    let dependencies = vec![(PackageId::CORE, None), (stdid, None)];

    let (mut unit, compile_errors) = crate::compile::compile_ast(
        &store,
        &dependencies,
        package,
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

    CompileRawQasmResult(store, source_package_id, dependencies, sig, compile_errors)
}

#[must_use]
pub fn parse_raw_qasm_as_fragments<R: SourceResolver, S: Into<Arc<str>>>(
    source: S,
    path: S,
    resolver: Option<&mut R>,
) -> QasmCompileUnit {
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
pub fn parse_raw_qasm_as_operation<
    R: SourceResolver,
    S: Into<Arc<str>>,
    N: Into<Arc<str>>,
    P: Into<Arc<str>>,
>(
    source: S,
    name: N,
    path: P,
    resolver: Option<&mut R>,
) -> QasmCompileUnit {
    let config = CompilerConfig::new(
        QubitSemantics::Qiskit,
        OutputSemantics::OpenQasm,
        ProgramType::Operation,
        Some(name.into()),
        None,
    );
    compile_to_qsharp_ast_with_config(source, path, resolver, config)
}
