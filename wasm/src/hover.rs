// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::ls_utils::{get_compilation, span_contains};
use crate::{Hover, Span};
use qsc_hir::hir::CallableKind;
use qsc_hir::visit::Visitor;
use wasm_bindgen::prelude::*;

pub(crate) fn get_hover(code: &str, offset: u32) -> Result<JsValue, JsValue> {
    let (_, package, _, _) = get_compilation(code);

    let mut callable_finder = CallableFinder {
        offset,
        header: None,
        start: 0,
        end: 0,
    };
    callable_finder.visit_package(&package);

    let hover = Hover {
        contents: match callable_finder.header {
            Some(header) => header,
            None => "not found".to_string(),
        },
        span: Span {
            start: offset,
            end: offset + 1,
        },
    };
    Ok(serde_wasm_bindgen::to_value(&hover)?)
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
{} {}
```

{}

",
                kind, decl.name.name, doc_paragraph
            );

            self.header = Some(header);
            self.start = decl.span.lo;
            self.end = decl.span.hi;
        }
    }
}
