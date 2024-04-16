// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::ast::{Attr, Expr, ExprKind, Ident, NodeId, Path};
use qsc_data_structures::span::Span;

use crate::compile::{preprocess::matches_config, RuntimeCapabilityFlags};

fn named_attr(name: &str) -> Attr {
    Attr {
        name: Box::new(Ident {
            name: name.into(),
            span: Span::default(),
            id: NodeId::default(),
        }),
        arg: Box::new(Expr {
            id: NodeId::default(),
            span: Span::default(),
            kind: Box::new(ExprKind::Tuple(Box::new([]))),
        }),
        span: Span::default(),
        id: NodeId::default(),
    }
}

fn name_value_attr(name: &str, value: &str) -> Attr {
    Attr {
        name: Box::new(Ident {
            name: name.into(),
            span: Span::default(),
            id: NodeId::default(),
        }),
        arg: Box::new(Expr {
            id: NodeId::default(),
            span: Span::default(),
            kind: Box::new(ExprKind::Paren(Box::new(Expr {
                id: NodeId::default(),
                span: Span::default(),
                kind: Box::new(ExprKind::Path(Box::new(Path {
                    id: NodeId::default(),
                    span: Span::default(),
                    namespace: None,
                    name: Box::new(Ident {
                        name: value.into(),
                        span: Span::default(),
                        id: NodeId::default(),
                    }),
                }))),
            }))),
        }),
        span: Span::default(),
        id: NodeId::default(),
    }
}

#[test]
fn no_attrs_matches() {
    assert!(matches_config(&[], RuntimeCapabilityFlags::empty()));
}

#[test]
fn unknown_attrs_matches() {
    assert!(matches_config(
        &[Box::new(named_attr("unknown"))],
        RuntimeCapabilityFlags::empty()
    ));
}

#[test]
fn none_attrs_matches_empty() {
    assert!(matches_config(
        &[Box::new(name_value_attr("Config", "Base"))],
        RuntimeCapabilityFlags::empty()
    ));
}

#[test]
fn none_attrs_does_not_match_all() {
    assert!(!matches_config(
        &[Box::new(name_value_attr("Config", "Base"))],
        RuntimeCapabilityFlags::all()
    ));
}

#[test]
fn none_attrs_does_not_match_adaptive() {
    assert!(!matches_config(
        &[Box::new(name_value_attr("Config", "Base"))],
        RuntimeCapabilityFlags::Adaptive
    ));
}

#[test]
fn adaptive_attrs_does_not_match_empty() {
    assert!(!matches_config(
        &[Box::new(name_value_attr("Config", "Adaptive"))],
        RuntimeCapabilityFlags::empty()
    ));
}

#[test]
fn integercomputations_attrs_does_not_match_empty() {
    assert!(!matches_config(
        &[Box::new(name_value_attr("Config", "IntegerComputations"))],
        RuntimeCapabilityFlags::empty()
    ));
}

#[test]
fn floatingpointcomputations_attrs_does_not_match_empty() {
    assert!(!matches_config(
        &[Box::new(name_value_attr(
            "Config",
            "FloatingPointComputations"
        ))],
        RuntimeCapabilityFlags::empty()
    ));
}

#[test]
fn unrestricted_attrs_does_not_match_empty() {
    assert!(!matches_config(
        &[Box::new(name_value_attr("Config", "Unrestricted"))],
        RuntimeCapabilityFlags::empty()
    ));
}

#[test]
fn unrestricted_attrs_matches_all() {
    assert!(matches_config(
        &[Box::new(name_value_attr("Config", "Unrestricted"))],
        RuntimeCapabilityFlags::all()
    ));
}
