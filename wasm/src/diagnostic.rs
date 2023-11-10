// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::serializable_type;
use miette::{Diagnostic, LabeledSpan, Severity};
use qsc::{self, error::WithSource, interpret::stateful, SourceName};
use serde::{Deserialize, Serialize};
use std::{fmt::Write, iter};
use wasm_bindgen::prelude::*;

serializable_type! {
    VSDiagnostic,
    {
        pub start_pos: u32,
        pub end_pos: u32,
        pub message: String,
        pub severity: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub code: Option<String>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub related: Vec<Related>
    },
    r#"export interface VSDiagnostic {
        start_pos: number;
        end_pos: number;
        message: string;
        severity: "error" | "warning" | "info"
        code?: string;
        related?: IRelatedInformation[];
    }"#
}

serializable_type! {
    Related,
    {
        pub source: String,
        pub start_pos: u32,
        pub end_pos: u32,
        pub message: String,
    },
    r#"export interface IRelatedInformation {
        source: string;
        start_pos: number;
        end_pos: number;
        message: string;
    }"#
}

impl VSDiagnostic {
    pub fn json(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("serializing VSDiagnostic should succeed")
    }

    /// Creates a [VSDiagnostic] from an interpreter error. See `VSDiagnostic::new()` for details.
    pub(crate) fn from_interpret_error(
        source_name: &str,
        err: &qsc::interpret::stateful::Error,
    ) -> Self {
        let labels = match err {
            stateful::Error::Compile(e) => error_labels(e),
            stateful::Error::Pass(e) => error_labels(e),
            stateful::Error::Eval(e) => error_labels(e.error()),
            stateful::Error::NoEntryPoint => Vec::new(),
            stateful::Error::TargetMismatch => Vec::new(),
        };

        Self::new(labels, source_name, err)
    }

    /// Creates a [VSDiagnostic] from a compiler error. See `VSDiagnostic::new()` for details.
    pub(crate) fn from_compile_error(source_name: &str, err: &qsc::compile::Error) -> Self {
        let labels = error_labels(err);

        Self::new(labels, source_name, err)
    }

    /// Creates a [VSDiagnostic] using the information from a [miette::Diagnostic].
    /// The error message, code and severity are straightforwardly generated,
    /// while mapping label spans is a little trickier.
    ///
    /// While a [miette::Diagnostic] can be associated with zero or more spans,
    /// a [VSDiagnostic] is intended to be shown as a squiggle in a specific document,
    /// and therefore needs to have at least one span that falls in the current document.
    ///
    /// If the first label's span falls in the current document, that span will be
    /// used for the squiggle. Some errors are not associated with a span
    /// at all, e.g. "entry point not found". Some other errors, e.g. some runtime errors,
    /// originate outside the current file. In those cases, a default span is
    /// used just to make the squiggle visible in the current document.
    ///
    /// Any labels with associated messages are included as "related information"
    /// objects in the [VSDiagnostic], whether they fall in the current document or not.
    /// Editors can display these as links to other source locations.
    fn new<T>(labels: Vec<Label>, source_name: &str, err: &T) -> VSDiagnostic
    where
        T: Diagnostic,
    {
        let mut labels = labels.into_iter().peekable();

        let default = (0, 1);
        let (start_pos, end_pos) = labels
            .peek()
            .filter(|l| l.source_name.as_ref() == source_name)
            .map_or(default, |l| (l.start, l.end));

        let related: Vec<Related> = labels
            .filter_map(|label| match label.message {
                Some(message) => Some(Related {
                    // Here, the stdlib/core files could be mapped to
                    // "qsharp-library-source" uris to allow for navigation
                    // in VS Code, but currently only the file path is returned.
                    source: label.source_name.to_string(),
                    start_pos: label.start,
                    end_pos: label.end,
                    message,
                }),
                None => None,
            })
            .collect();

        // e.g. "runtime error"
        let mut message = err.to_string();
        for source in iter::successors(std::error::Error::source(&err), |e| e.source()) {
            // e.g. " Qubit0 released while not in |0⟩ state"
            write!(message, ": {source}").expect("message should be writable");
        }
        if let Some(help) = err.help() {
            // e.g. "qubits should be returned to the |0⟩ state before being released to satisfy the assumption that allocated qubits start in the |0⟩ state"
            write!(message, "\n\nhelp: {help}").expect("message should be writable");
        }

        // e.g. Qsc.Eval.ReleasedQubitNotZero
        let code = err.code().map(|c| c.to_string());

        Self {
            start_pos,
            end_pos,
            severity: (match err.severity().unwrap_or(Severity::Error) {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Advice => "info",
            })
            .to_string(),
            message,
            code,
            related,
        }
    }
}

struct Label {
    pub source_name: SourceName,
    pub start: u32,
    pub end: u32,
    pub message: Option<String>,
}

fn error_labels<T>(e: &WithSource<T>) -> Vec<Label>
where
    T: Diagnostic + Send + Sync,
{
    e.labels()
        .into_iter()
        .flatten()
        .map(|l| resolve_label(e, &l))
        .collect()
}

fn resolve_label<T>(e: &WithSource<T>, labeled_span: &LabeledSpan) -> Label
where
    T: Diagnostic + Send + Sync,
{
    let (source, span) = e.resolve_span(labeled_span.inner());
    let start = u32::try_from(span.offset()).expect("offset should fit in u32");
    let len = u32::try_from(span.len()).expect("length should fit in u32");

    Label {
        source_name: source.name.clone(),
        start,
        end: start + len,
        message: labeled_span.label().map(ToString::to_string),
    }
}
