// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::{
    Diagnostic, MietteError, MietteSpanContents, Report, SourceCode, SourceSpan, SpanContents,
};
use qsc_frontend::compile::{SourceIndex, SourceMap};
use std::{path::PathBuf, sync::Arc};

pub struct Reporter<'a> {
    offsets: &'a SourceMap,
    paths: Vec<PathBuf>,
    sources: Vec<Arc<String>>,
    entry: Arc<String>,
}

impl<'a> Reporter<'a> {
    pub fn new(
        offsets: &'a SourceMap,
        paths: Vec<PathBuf>,
        sources: Vec<String>,
        entry: Option<String>,
    ) -> Self {
        Self {
            offsets,
            paths,
            sources: sources.into_iter().map(Arc::new).collect(),
            entry: Arc::new(entry.unwrap_or_default()),
        }
    }

    pub fn report(&self, error: impl Diagnostic + Send + Sync + 'static) -> Report {
        let Some(first_label) = error.labels().and_then(|mut ls| ls.next()) else {
            return Report::new(error);
        };

        // Use the offset of the first labeled span to find which source code to include in the report.
        let (index, offset) = self.offsets.offset(first_label.offset());
        let name = source_name(&self.paths, index);
        let source = self.sources.get(index.0).unwrap_or(&self.entry).clone();

        // Adjust all spans in the error to be relative to the start of this source.
        Report::new(error).with_source_code(OffsetSource {
            name: name.to_string(),
            source,
            offset,
        })
    }
}

struct OffsetSource {
    name: String,
    source: Arc<String>,
    offset: usize,
}

impl SourceCode for OffsetSource {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        let contents = self.source.read_span(
            &with_offset(span, |o| o - self.offset),
            context_lines_before,
            context_lines_after,
        )?;

        Ok(Box::new(MietteSpanContents::new_named(
            self.name.clone(),
            contents.data(),
            with_offset(contents.span(), |o| o + self.offset),
            contents.line(),
            contents.column(),
            contents.line_count(),
        )))
    }
}

fn source_name(paths: &[PathBuf], index: SourceIndex) -> &str {
    paths
        .get(index.0)
        .map_or("<unknown>", |p| match p.to_str() {
            Some("-") => "<stdin>",
            Some(name) => name,
            None => "<unknown>",
        })
}

fn with_offset(span: &SourceSpan, f: impl FnOnce(usize) -> usize) -> SourceSpan {
    SourceSpan::new(f(span.offset()).into(), span.len().into())
}
