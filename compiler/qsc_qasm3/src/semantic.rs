// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::io::SourceResolver;
use crate::parser::QasmSource;

use qsc_frontend::compile::SourceMap;
use qsc_frontend::error::WithSource;
use symbols::SymbolTable;

use std::path::Path;

mod ast;
pub mod error;
mod lowerer;
pub use error::Error;
pub use error::SemanticErrorKind;
pub mod symbols;
pub mod types;

#[cfg(test)]
pub(crate) mod tests;

pub struct QasmSemanticParseResult {
    pub source: QasmSource,
    pub source_map: SourceMap,
    pub symbols: self::symbols::SymbolTable,
    pub program: self::ast::Program,
    pub errors: Vec<WithSource<crate::Error>>,
}

impl QasmSemanticParseResult {
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.has_syntax_errors() || self.has_semantic_errors()
    }

    #[must_use]
    pub fn has_syntax_errors(&self) -> bool {
        self.source.has_errors()
    }

    #[must_use]
    pub fn has_semantic_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn parse_errors(&self) -> Vec<WithSource<crate::Error>> {
        let mut self_errors = self
            .source
            .errors()
            .iter()
            .map(|e| self.map_parse_error(e.clone()))
            .collect::<Vec<_>>();
        let include_errors = self
            .source
            .includes()
            .iter()
            .flat_map(QasmSource::all_errors)
            .map(|e| self.map_parse_error(e))
            .collect::<Vec<_>>();

        self_errors.extend(include_errors);
        self_errors
    }

    #[must_use]
    pub fn semantic_errors(&self) -> Vec<WithSource<crate::Error>> {
        self.errors().clone()
    }

    #[must_use]
    pub fn all_errors(&self) -> Vec<WithSource<crate::Error>> {
        let mut parse_errors = self.parse_errors();
        let sem_errors = self.semantic_errors();
        parse_errors.extend(sem_errors);
        parse_errors
    }

    #[must_use]
    pub fn errors(&self) -> Vec<WithSource<crate::Error>> {
        self.errors.clone()
    }

    fn map_parse_error(&self, error: crate::parser::Error) -> WithSource<crate::Error> {
        let path = self.source.path().display().to_string();
        let source = self.source_map.find_by_name(&path);
        let offset = source.map_or(0, |source| source.offset);

        let offset_error = error.with_offset(offset);

        WithSource::from_map(
            &self.source_map,
            crate::Error(crate::ErrorKind::Parser(offset_error)),
        )
    }
}

/// Parse a QASM file and return the parse result.
/// This function will resolve includes using the provided resolver.
/// If an include file cannot be resolved, an error will be returned.
/// If a file is included recursively, a stack overflow occurs.
pub fn parse_source<S, P, R>(
    source: S,
    path: P,
    resolver: &R,
) -> miette::Result<QasmSemanticParseResult>
where
    S: AsRef<str>,
    P: AsRef<Path>,
    R: SourceResolver,
{
    let res = crate::parser::parse_source(source, path, resolver)?;
    let analyzer = crate::semantic::lowerer::Lowerer {
        source: res.source,
        source_map: res.source_map,
        errors: vec![],
        file_stack: vec![],
        symbols: SymbolTable::default(),
        version: None,
        stmts: vec![],
    };
    let sem_res = analyzer.lower();
    Ok(QasmSemanticParseResult {
        source: sem_res.source,
        source_map: sem_res.source_map,
        symbols: sem_res.symbols,
        program: sem_res.program,
        errors: sem_res.errors,
    })
}
