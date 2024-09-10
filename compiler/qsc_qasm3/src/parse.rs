// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::io::SourceResolver;
use crate::oqasm_helpers::text_range_to_span;
use oq3_syntax::SyntaxNode;
use oq3_syntax::{ast::Stmt, ParseOrErrors, SourceFile};
use qsc::{error::WithSource, SourceMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(test)]
pub(crate) mod tests;

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
            .map(|e| self.map_error(e));

        self_errors.extend(include_errors);
        self_errors
    }

    #[must_use]
    pub fn errors(&self) -> Vec<WithSource<crate::Error>> {
        self.source
            .errors()
            .iter()
            .map(|e| self.map_error(e.clone()))
            .collect()
    }

    fn map_error(&self, error: crate::Error) -> WithSource<crate::Error> {
        let path = self.source.path().display().to_string();
        let source = self.source_map.find_by_name(&path);
        let offset = source.map_or(0, |source| source.offset);

        let offset_error = error.with_offset(offset);

        WithSource::from_map(&self.source_map, offset_error)
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
    // Map the main source file to the entry point expression
    // This may be incorrect, but it's the best we can do for now.
    SourceMap::new(files, Some(Arc::from(source.source())))
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
    ast: ParseOrErrors<oq3_syntax::SourceFile>,
    /// Any included files that were resolved.
    /// Note that this is a recursive structure.
    included: Vec<QasmSource>,
}

impl QasmSource {
    pub fn new<T: AsRef<str>, P: AsRef<Path>>(
        source: T,
        file_path: P,
        ast: ParseOrErrors<oq3_syntax::SourceFile>,
        included: Vec<QasmSource>,
    ) -> QasmSource {
        QasmSource {
            source: source.as_ref().into(),
            path: file_path.as_ref().to_owned(),
            ast,
            included,
        }
    }

    #[must_use]
    pub fn has_errors(&self) -> bool {
        if !self.parse_result().errors().is_empty() {
            return true;
        }
        self.includes().iter().any(QasmSource::has_errors)
    }

    #[must_use]
    pub fn all_errors(&self) -> Vec<crate::Error> {
        let mut self_errors = self.errors();
        let include_errors = self.includes().iter().flat_map(QasmSource::all_errors);
        self_errors.extend(include_errors);
        self_errors
    }

    #[must_use]
    pub fn tree(&self) -> oq3_syntax::SourceFile {
        self.parse_result().tree()
    }

    #[must_use]
    pub fn syntax_node(&self) -> SyntaxNode {
        self.parse_result().syntax_node()
    }

    #[must_use]
    pub fn includes(&self) -> &Vec<QasmSource> {
        self.included.as_ref()
    }

    #[must_use]
    pub fn parse_result(&self) -> &ParseOrErrors<oq3_syntax::SourceFile> {
        &self.ast
    }

    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    #[must_use]
    pub fn errors(&self) -> Vec<crate::Error> {
        self.parse_result()
            .errors()
            .iter()
            .map(|e| {
                crate::Error(crate::ErrorKind::Parse(
                    e.message().to_string(),
                    text_range_to_span(e.range()),
                ))
            })
            .collect()
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
    let (parse_result, includes) = parse_source_and_includes(source.as_ref(), resolver)?;
    Ok(QasmSource::new(source, path, parse_result, includes))
}

fn parse_source_and_includes<P: AsRef<str>, R>(
    source: P,
    resolver: &R,
) -> miette::Result<(ParseOrErrors<SourceFile>, Vec<QasmSource>)>
where
    R: SourceResolver,
{
    let parse_result = oq3_syntax::SourceFile::parse_check_lex(source.as_ref());
    if parse_result.have_parse() {
        let included = parse_includes(&parse_result, resolver)?;
        Ok((parse_result, included))
    } else {
        Ok((parse_result, vec![]))
    }
}

fn parse_includes<R>(
    syntax_ast: &ParseOrErrors<oq3_syntax::SourceFile>,
    resolver: &R,
) -> miette::Result<Vec<QasmSource>>
where
    R: SourceResolver,
{
    let mut includes = vec![];
    for stmt in syntax_ast.tree().statements() {
        if let Stmt::Include(include) = stmt {
            if let Some(file) = include.file() {
                if let Some(file_path) = file.to_string() {
                    // Skip the standard gates include file.
                    // Handling of this file is done by the compiler.
                    if file_path.to_lowercase() == "stdgates.inc" {
                        continue;
                    }
                    let source = parse_qasm_file(file_path, resolver)?;
                    includes.push(source);
                }
            }
        }
    }

    Ok(includes)
}
