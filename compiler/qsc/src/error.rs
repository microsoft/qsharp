// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::{
    Diagnostic, MietteError, MietteSpanContents, Report, SourceCode, SourceSpan, SpanContents,
};
use qsc_frontend::compile::SourceMap;

pub fn report(sources: &SourceMap, error: impl Diagnostic + Send + Sync + 'static) -> Report {
    let Some(first_label) = error.labels().and_then(|mut ls| ls.next()) else {
            return Report::new(error);
        };

    let source = sources.find_by_offset(first_label.offset());
    Report::new(error).with_source_code(OffsetSource {
        name: source.name.to_string_lossy().to_string(),
        source: source.content.clone(),
        offset: source.offset,
    })
}

struct OffsetSource {
    name: String,
    source: String,
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

fn with_offset(span: &SourceSpan, f: impl FnOnce(usize) -> usize) -> SourceSpan {
    SourceSpan::new(f(span.offset()).into(), span.len().into())
}
