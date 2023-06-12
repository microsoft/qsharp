// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::qsc_utils::{map_offset, span_contains, Compilation};
use qsc::hir::{ty::Ty, visit::Visitor, CallableDecl, CallableKind};

#[derive(Debug, PartialEq)]
pub struct Hover {
    pub contents: String,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

pub(crate) fn get_hover(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Option<Hover> {
    // Map the file offset into a SourceMap offset
    let offset = map_offset(&compilation.source_map, source_name, offset);
    let package = &compilation.package;

    let mut callable_finder = CallableFinder {
        offset,
        header: None,
        start: 0,
        end: 0,
    };
    callable_finder.visit_package(package);

    callable_finder.header.map(|header| Hover {
        contents: header,
        span: Span {
            start: callable_finder.start,
            end: callable_finder.end,
        },
    })
}

struct CallableFinder {
    offset: u32,
    header: Option<String>,
    start: u32,
    end: u32,
}

impl Visitor<'_> for CallableFinder {
    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        if span_contains(decl.name.span, self.offset) {
            let kind = match decl.kind {
                CallableKind::Function => "function",
                CallableKind::Operation => "operation",
            };

            // Doc comments would be formatted as markdown into this
            // string once we're able to parse them out.
            let header = format!(
                "```qsharp
{} {}{} : {}
```
",
                kind,
                decl.name.name,
                match &decl.input.ty {
                    Ty::Tuple(items) => {
                        // I'm just doing this so I can format Unit as ()
                        if items.is_empty() {
                            "()".to_string()
                        } else {
                            format!("{}", &decl.input.ty)
                        }
                    }
                    x => x.to_string(),
                },
                decl.output
            );

            self.header = Some(header);
            self.start = decl.name.span.lo;
            self.end = decl.name.span.hi;
        }
    }
}
