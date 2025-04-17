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
