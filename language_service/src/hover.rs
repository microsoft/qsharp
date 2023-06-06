// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::qsc_utils::{span_contains, Compilation};
use qsc_hir::hir::{CallableKind, Ty};
use qsc_hir::visit::Visitor;

#[derive(Debug)]
pub struct Hover {
    pub contents: String,
    pub span: Span,
}

#[derive(Debug)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

pub(crate) fn get_hover(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Option<Hover> {
    let package = &compilation.compile_unit.package;
    // Map the file offset into a SourceMap offset
    let offset = compilation
        .compile_unit
        .sources
        .map_offset(source_name, offset);

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
            start: offset,
            end: offset + 1,
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
    fn visit_callable_decl(&mut self, decl: &qsc_hir::hir::CallableDecl) {
        if span_contains(decl.name.span, self.offset) {
            let kind = match decl.kind {
                CallableKind::Function => "function",
                CallableKind::Operation => "operation",
            };

            let chars = &['/', ' '];

            let docs = decl
                .doc_comments
                .iter()
                .map(|c| c.trim_start_matches(chars))
                .collect::<Vec<_>>();
            let mut doc_paragraph = String::new();
            for line in docs {
                doc_paragraph.push_str(line);
                doc_paragraph.push('\n');
            }

            let header = format!(
                "```qsharp
{} {}{} : {}
```

{}

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
                decl.output,
                doc_paragraph
            );

            self.header = Some(header);
            self.start = decl.span.lo;
            self.end = decl.span.hi;
        }
    }
}
