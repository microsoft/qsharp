// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::path::Path;

use qsc_qasm3::io::SourceResolver;
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
    qsc_qasm3::compile_to_ast_with_config(source, path, resolver, config)
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
    qsc_qasm3::compile_to_ast_with_config(source, path, resolver, config)
}

#[must_use]
pub fn compile_to_ast_with_config<S, P, R>(
    source: S,
    path: P,
    resolver: Option<&mut R>,
    config: CompilerConfig,
) -> QasmCompileUnit
where
    S: AsRef<str>,
    P: AsRef<Path>,
    R: SourceResolver,
{
    qsc_qasm3::compile_to_ast_with_config(source, path, resolver, config)
}
