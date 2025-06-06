// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod ast;
use crate::io::SourceResolver;
use ast::{Program, StmtKind};
use mut_visit::MutVisitor;
use qsc_data_structures::span::Span;
use qsc_frontend::compile::SourceMap;
use qsc_frontend::error::WithSource;
use scan::ParserContext;
use std::sync::Arc;

#[cfg(test)]
pub(crate) mod tests;

pub mod completion;
mod error;
pub use error::Error;
mod expr;
mod mut_visit;
mod prgm;
mod prim;
mod scan;
mod stmt;

type QasmSourceResult = std::result::Result<QasmSource, crate::parser::Error>;
type QasmSourceResultVec = Vec<std::result::Result<QasmSource, crate::parser::Error>>;

struct Offsetter(pub(super) u32);

impl MutVisitor for Offsetter {
    fn visit_span(&mut self, span: &mut Span) {
        span.lo += self.0;
        span.hi += self.0;
    }
}

pub struct QasmParseResult {
    pub source: QasmSource,
    pub source_map: SourceMap,
}

impl QasmParseResult {
    #[must_use]
    pub fn new(source: QasmSource) -> QasmParseResult {
        let source_map = create_source_map(&source);
        let mut source = source;
        update_offsets(&source_map, &mut source);
        QasmParseResult { source, source_map }
    }

    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.source.has_errors()
    }

    #[must_use]
    pub fn all_errors(&self) -> Vec<WithSource<crate::Error>> {
        let mut self_errors = self.errors();
        let include_errors = self
            .source
            .includes()
            .iter()
            .flat_map(|res| match res {
                Ok(qasm_source) => qasm_source.all_errors(),
                // If the source failed to resolve we don't push an error here.
                // We will push an error later during lowering, so that we can
                // construct the error with the right span.
                Err(_) => vec![],
            })
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
        WithSource::from_map(
            &self.source_map,
            crate::Error(crate::ErrorKind::Parser(error)),
        )
    }
}

/// all spans and errors spans are relative to the start of the file
/// We need to update the spans based on the offset of the file in the source map.
/// We have to do this after a full parse as we don't know what files will be loaded
/// until we have parsed all the includes.
fn update_offsets(source_map: &SourceMap, source: &mut QasmSource) {
    let source_file = source_map.find_by_name(&source.path());
    let offset = source_file.map_or(0, |source| source.offset);
    // Update the errors' offset
    source
        .errors
        .iter_mut()
        .for_each(|e| *e = e.clone().with_offset(offset));
    // Update the program's spans with the offset
    let mut offsetter = Offsetter(offset);
    offsetter.visit_program(&mut source.program);

    // Recursively update the includes, their programs, and errors
    for include in source.includes_mut().iter_mut().flatten() {
        update_offsets(source_map, include);
    }
}

/// Parse a QASM file and return the parse result.
/// This function will resolve includes using the provided resolver.
/// If an include file cannot be resolved, an error will be returned.
/// If a file is included recursively, a stack overflow occurs.
pub fn parse_source<R: SourceResolver, S: Into<Arc<str>>, P: Into<Arc<str>>>(
    source: S,
    path: P,
    resolver: &mut R,
) -> QasmParseResult {
    let res = parse_qasm_source(source.into(), path.into(), resolver);
    QasmParseResult::new(res)
}

/// Creates a Q# source map from a QASM parse output. The `QasmSource`
/// has all of the recursive includes resolved with their own source
/// and parse results.
fn create_source_map(source: &QasmSource) -> SourceMap {
    let mut files: Vec<(Arc<str>, Arc<str>)> = Vec::new();
    collect_source_files(source, &mut files);
    SourceMap::new(files, None)
}

/// Recursively collect all source files from the includes
fn collect_source_files(source: &QasmSource, files: &mut Vec<(Arc<str>, Arc<str>)>) {
    files.push((source.path(), source.source()));
    // Collect all source files from the includes, this
    // begins the recursive process of collecting all source files.
    for include in source.includes().iter().flatten() {
        collect_source_files(include, files);
    }
}

/// Represents a QASM source file that has been parsed.
#[derive(Clone, Debug)]
pub struct QasmSource {
    /// The path to the source file. This is used for error reporting.
    /// This path is just a name, it does not have to exist on disk.
    path: Arc<str>,
    /// The source code of the file.
    source: Arc<str>,
    /// The parsed AST of the source file or any parse errors.
    program: Program,
    /// Any parse errors that occurred.
    errors: Vec<Error>,
    /// Any included files that were resolved.
    /// Note that this is a recursive structure.
    included: QasmSourceResultVec,
}

impl QasmSource {
    #[must_use]
    pub fn new(
        source: Arc<str>,
        path: Arc<str>,
        program: Program,
        errors: Vec<Error>,
        included: QasmSourceResultVec,
    ) -> QasmSource {
        QasmSource {
            path,
            source,
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
        self.includes().iter().any(|res| match res {
            Ok(qasm_source) => qasm_source.has_errors(),
            Err(_) => true,
        })
    }

    #[must_use]
    pub fn all_errors(&self) -> Vec<crate::parser::Error> {
        let mut self_errors = self.errors();
        let include_errors = self.includes().iter().flat_map(|res| match res {
            Ok(qasm_source) => qasm_source.all_errors(),
            // If the source failed to resolve we don't push an error here.
            // We will push an error later during lowering, so that we can
            // construct the error with the right span.
            Err(_) => vec![],
        });
        self_errors.extend(include_errors);
        self_errors
    }

    #[must_use]
    pub fn includes(&self) -> &QasmSourceResultVec {
        self.included.as_ref()
    }

    #[must_use]
    pub fn includes_mut(&mut self) -> &mut QasmSourceResultVec {
        self.included.as_mut()
    }

    #[must_use]
    pub fn program(&self) -> &Program {
        &self.program
    }

    #[must_use]
    pub fn path(&self) -> Arc<str> {
        self.path.clone()
    }

    #[must_use]
    pub fn errors(&self) -> Vec<crate::parser::Error> {
        self.errors.clone()
    }

    #[must_use]
    pub fn source(&self) -> Arc<str> {
        self.source.clone()
    }
}

/// Parse a QASM file and return the parse result using the provided resolver.
/// Returns `Err` if the resolver cannot resolve the file.
/// Returns `Ok` otherwise. Any parse errors will be included in the result.
///
/// This function is the start of a recursive process that will resolve all
/// includes in the QASM file. Any includes are parsed as if their contents
/// were defined where the include statement is.
fn parse_qasm_file<R>(path: &Arc<str>, resolver: &mut R) -> QasmSourceResult
where
    R: SourceResolver,
{
    match resolver.resolve(path) {
        Ok((path, source)) => {
            let parse_result = parse_qasm_source(source, path.clone(), resolver);

            // Once we finish parsing the source, we pop the file from the
            // resolver. This is needed to keep track of multiple includes
            // and cyclic includes.
            resolver.ctx().pop_current_file();

            Ok(parse_result)
        }
        Err(e) => {
            let error = crate::parser::error::ErrorKind::IO(e);
            Err(crate::parser::Error(error, None))
        }
    }
}

fn parse_qasm_source<R>(source: Arc<str>, path: Arc<str>, resolver: &mut R) -> QasmSource
where
    R: SourceResolver,
{
    let (program, errors, includes) = parse_source_and_includes(source.as_ref(), resolver);
    QasmSource::new(source, path, program, errors, includes)
}

fn parse_source_and_includes<P: AsRef<str>, R>(
    source: P,
    resolver: &mut R,
) -> (Program, Vec<Error>, QasmSourceResultVec)
where
    R: SourceResolver,
{
    let (program, errors) = parse(source.as_ref());
    let included = parse_includes(&program, resolver);
    (program, errors, included)
}

fn parse_includes<R>(program: &Program, resolver: &mut R) -> QasmSourceResultVec
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
            let source = parse_qasm_file(file_path, resolver);
            includes.push(source);
        }
    }

    includes
}

pub(crate) type Result<T> = std::result::Result<T, crate::parser::error::Error>;

pub(crate) trait Parser<T>: FnMut(&mut ParserContext) -> Result<T> {}

impl<T, F: FnMut(&mut ParserContext) -> Result<T>> Parser<T> for F {}

#[must_use]
pub fn parse(input: &str) -> (Program, Vec<Error>) {
    let mut scanner = ParserContext::new(input);
    let program = prgm::parse(&mut scanner);
    (program, scanner.into_errors())
}
