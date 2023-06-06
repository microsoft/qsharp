// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::language_service::CompilationState;
use crate::ls_utils::span_contains;
use qsc_hir::hir::{CallableKind, Ty};
use qsc_hir::visit::Visitor;
use std::fmt::Write;

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
    compilation_state: &CompilationState,
    _uri: &str,
    offset: u32,
) -> Option<Hover> {
    let package = &compilation_state
        .compile_unit
        .as_ref()
        .expect(
            "a compilation unit should exist for the current file - has update_code been called?",
        )
        .package;

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
                            let mut s = String::new();
                            s.push('(');
                            if let Some((first, rest)) = items.split_first() {
                                let _ = write!(s, "{}", first);
                                if rest.is_empty() {
                                    s.push_str(", ");
                                } else {
                                    for item in rest {
                                        s.push_str(", ");
                                        let _ = write!(s, "{}", item);
                                    }
                                }
                            }
                            s.push(')');
                            s
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
