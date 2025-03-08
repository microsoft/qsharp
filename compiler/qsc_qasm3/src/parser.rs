// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::ast::{Program, StmtKind};
use crate::io::SourceResolver;
use qsc_frontend::compile::SourceMap;
use qsc_frontend::error::WithSource;
use scan::ParserContext;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(test)]
pub(crate) mod tests;

mod completion;
mod error;
pub use error::Error;
mod expr;
mod prgm;
mod prim;
mod scan;
mod stmt;

pub struct QasmParseResult {
    pub source: QasmSource,
    pub source_map: SourceMap,
}

impl QasmParseResult {
    #[must_use]
    pub fn new(source: QasmSource) -> QasmParseResult {
        let source_map = create_source_map(&source);
        QasmParseResult { source, source_map }
    }

    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.source.has_errors()
    }

    pub fn all_errors(&self) -> Vec<WithSource<crate::Error>> {
        let mut self_errors = self.errors();
        let include_errors = self
            .source
            .includes()
            .iter()
            .flat_map(QasmSource::all_errors)
            .map(|e| self.map_error(e))
            .collect::<Vec<_>>();

        self_errors.extend(include_errors);
        self_errors
    }

    #[must_use]
    pub fn errors(&self) -> Vec<WithSource<crate::Error>> {
        self.source
            .errors()
            .iter()
            .map(|e| self.map_error(e.clone()))
            .collect::<Vec<_>>()
    }

    fn map_error(&self, error: Error) -> WithSource<crate::Error> {
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
pub fn parse_source<S, P, R>(source: S, path: P, resolver: &R) -> miette::Result<QasmParseResult>
where
    S: AsRef<str>,
    P: AsRef<Path>,
    R: SourceResolver,
{
    let res = parse_qasm_source(source, path, resolver)?;
    Ok(QasmParseResult::new(res))
}

/// Creates a Q# source map from a QASM parse output. The `QasmSource`
/// has all of the recursive includes resolved with their own source
/// and parse results.
fn create_source_map(source: &QasmSource) -> SourceMap {
    let mut files: Vec<(Arc<str>, Arc<str>)> = Vec::new();
    files.push((
        Arc::from(source.path().to_string_lossy().to_string()),
        Arc::from(source.source()),
    ));
    // Collect all source files from the includes, this
    // begins the recursive process of collecting all source files.
    for include in source.includes() {
        collect_source_files(include, &mut files);
    }
    SourceMap::new(files, None)
}

/// Recursively collect all source files from the includes
fn collect_source_files(source: &QasmSource, files: &mut Vec<(Arc<str>, Arc<str>)>) {
    files.push((
        Arc::from(source.path().to_string_lossy().to_string()),
        Arc::from(source.source()),
    ));
    for include in source.includes() {
        collect_source_files(include, files);
    }
}

/// Represents a QASM source file that has been parsed.
#[derive(Clone, Debug)]
pub struct QasmSource {
    /// The path to the source file. This is used for error reporting.
    /// This path is just a name, it does not have to exist on disk.
    path: PathBuf,
    /// The source code of the file.
    source: Arc<str>,
    /// The parsed AST of the source file or any parse errors.
    program: Program,
    /// Any parse errors that occurred.
    errors: Vec<Error>,
    /// Any included files that were resolved.
    /// Note that this is a recursive structure.
    included: Vec<QasmSource>,
}

impl QasmSource {
    pub fn new<T: AsRef<str>, P: AsRef<Path>>(
        source: T,
        file_path: P,
        program: Program,
        errors: Vec<Error>,
        included: Vec<QasmSource>,
    ) -> QasmSource {
        QasmSource {
            path: file_path.as_ref().to_owned(),
            source: source.as_ref().into(),
            program,
            errors,
            included,
        }
    }

    #[must_use]
    pub fn has_errors(&self) -> bool {
        if !self.errors().is_empty() {
            return true;
        }
        self.includes().iter().any(QasmSource::has_errors)
    }

    #[must_use]
    pub fn all_errors(&self) -> Vec<crate::parser::Error> {
        let mut self_errors = self.errors();
        let include_errors = self.includes().iter().flat_map(QasmSource::all_errors);
        self_errors.extend(include_errors);
        self_errors
    }

    #[must_use]
    pub fn includes(&self) -> &Vec<QasmSource> {
        self.included.as_ref()
    }

    #[must_use]
    pub fn program(&self) -> &Program {
        &self.program
    }

    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    #[must_use]
    pub fn errors(&self) -> Vec<crate::parser::Error> {
        self.errors.clone()
    }

    #[must_use]
    pub fn source(&self) -> &str {
        self.source.as_ref()
    }
}

/// Parse a QASM file and return the parse result using the provided resolver.
/// Returns `Err` if the resolver cannot resolve the file.
/// Returns `Ok` otherwise. Any parse errors will be included in the result.
///
/// This function is the start of a recursive process that will resolve all
/// includes in the QASM file. Any includes are parsed as if their contents
/// were defined where the include statement is.
fn parse_qasm_file<P, R>(path: P, resolver: &R) -> miette::Result<QasmSource>
where
    P: AsRef<Path>,
    R: SourceResolver,
{
    let (path, source) = resolver.resolve(&path)?;
    parse_qasm_source(source, path, resolver)
}

fn parse_qasm_source<S, P, R>(source: S, path: P, resolver: &R) -> miette::Result<QasmSource>
where
    S: AsRef<str>,
    P: AsRef<Path>,
    R: SourceResolver,
{
    let (program, errors, includes) = parse_source_and_includes(source.as_ref(), resolver)?;
    Ok(QasmSource::new(source, path, program, errors, includes))
}

fn parse_source_and_includes<P: AsRef<str>, R>(
    source: P,
    resolver: &R,
) -> miette::Result<(Program, Vec<Error>, Vec<QasmSource>)>
where
    R: SourceResolver,
{
    let (program, errors) = parse(source.as_ref())?;
    let included = parse_includes(&program, resolver)?;
    Ok((program, errors, included))
}

fn parse_includes<R>(program: &crate::ast::Program, resolver: &R) -> miette::Result<Vec<QasmSource>>
where
    R: SourceResolver,
{
    let mut includes = vec![];
    for stmt in &program.statements {
        if let StmtKind::Include(include) = stmt.kind.as_ref() {
            let file_path = &include.filename;
            // Skip the standard gates include file.
            // Handling of this file is done by the compiler.
            if file_path.to_lowercase() == "stdgates.inc" {
                continue;
            }
            let source = parse_qasm_file(file_path, resolver)?;
            includes.push(source);
        }
    }

    Ok(includes)
}

pub(crate) type Result<T> = std::result::Result<T, crate::parser::error::Error>;

pub(crate) trait Parser<T>: FnMut(&mut ParserContext) -> Result<T> {}

impl<T, F: FnMut(&mut ParserContext) -> Result<T>> Parser<T> for F {}

pub fn parse(input: &str) -> Result<(Program, Vec<Error>)> {
    let mut scanner = ParserContext::new(input);
    let program = prgm::parse(&mut scanner)?;
    Ok((program, scanner.into_errors()))
}
