// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::io::SourceResolver;
use crate::parser::QasmSource;

use lowerer::Lowerer;
use qsc_frontend::compile::SourceMap;
use qsc_frontend::error::WithSource;
use symbols::SymbolTable;

use std::path::Path;

pub(crate) mod ast;
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
    pub program: Option<self::ast::Program>,
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
        WithSource::from_map(
            &self.source_map,
            crate::Error(crate::ErrorKind::Parser(error)),
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

    // If there are syntax errors, return early
    if res.source.has_errors() {
        return Ok(QasmSemanticParseResult {
            source: res.source,
            source_map: res.source_map,
            symbols: SymbolTable::default(),
            program: None,
            errors: vec![],
        });
    }

    let analyzer = Lowerer::new(res.source, res.source_map);
    let sem_res = analyzer.lower();
    Ok(QasmSemanticParseResult {
        source: sem_res.source,
        source_map: sem_res.source_map,
        symbols: sem_res.symbols,
        program: sem_res.program,
        errors: sem_res.errors,
    })
}
