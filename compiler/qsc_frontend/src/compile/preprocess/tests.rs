// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::ast::{Attr, Expr, ExprKind, Ident, NodeId, Path, PathKind};
use qsc_data_structures::span::Span;

use crate::compile::{preprocess::matches_config, TargetCapabilityFlags};

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
                kind: Box::new(ExprKind::Path(PathKind::Ok(Box::new(Path {
                    id: NodeId::default(),
                    span: Span::default(),
                    segments: None,
                    name: Box::new(Ident {
                        name: value.into(),
                        span: Span::default(),
                        id: NodeId::default(),
                    }),
                })))),
            }))),
        }),
        span: Span::default(),
        id: NodeId::default(),
    }
}

#[test]
fn no_attrs_matches() {
    assert!(matches_config(&[], TargetCapabilityFlags::empty()));
}

#[test]
fn unknown_attrs_matches() {
    assert!(matches_config(
        &[Box::new(named_attr("unknown"))],
        TargetCapabilityFlags::empty()
    ));
}

#[test]
fn none_attrs_matches_empty() {
    assert!(matches_config(
        &[Box::new(name_value_attr("Config", "Base"))],
        TargetCapabilityFlags::empty()
    ));
}

#[test]
fn none_attrs_does_not_match_all() {
    assert!(!matches_config(
        &[Box::new(name_value_attr("Config", "Base"))],
        TargetCapabilityFlags::all()
    ));
}

#[test]
fn none_attrs_does_not_match_adaptive() {
    assert!(!matches_config(
        &[Box::new(name_value_attr("Config", "Base"))],
        TargetCapabilityFlags::Adaptive
    ));
}

#[test]
fn adaptive_attrs_does_not_match_empty() {
    assert!(!matches_config(
        &[Box::new(name_value_attr("Config", "Adaptive"))],
        TargetCapabilityFlags::empty()
    ));
}

#[test]
fn integercomputations_attrs_does_not_match_empty() {
    assert!(!matches_config(
        &[Box::new(name_value_attr("Config", "IntegerComputations"))],
        TargetCapabilityFlags::empty()
    ));
}

#[test]
fn floatingpointcomputations_attrs_does_not_match_empty() {
    assert!(!matches_config(
        &[Box::new(name_value_attr(
            "Config",
            "FloatingPointComputations"
        ))],
        TargetCapabilityFlags::empty()
    ));
}

#[test]
fn unrestricted_attrs_does_not_match_empty() {
    assert!(!matches_config(
        &[Box::new(name_value_attr("Config", "Unrestricted"))],
        TargetCapabilityFlags::empty()
    ));
}

#[test]
fn unrestricted_attrs_matches_all() {
    assert!(matches_config(
        &[Box::new(name_value_attr("Config", "Unrestricted"))],
        TargetCapabilityFlags::all()
    ));
}
