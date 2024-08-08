// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{rc::Rc, result};

use item::parse_doc;
use qsc_cst::cst::{Expr, Namespace, TopLevelNode};
use qsc_data_structures::language_features::LanguageFeatures;

use crate::{keyword::Keyword, lex::TokenKind, scan::ParserContext, Error};

mod expr;
mod item;
mod prim;
mod stmt;
mod ty;

type Result<T> = result::Result<T, Error>;

#[must_use]
pub fn namespaces(
    input: &str,
    source_name: Option<&str>,
    language_features: LanguageFeatures,
) -> (Vec<Namespace>, Vec<Error>) {
    let mut scanner = ParserContext::new(input, language_features);
    let doc = parse_doc(&mut scanner);
    let doc = Rc::from(doc.unwrap_or_default());
    #[allow(clippy::unnecessary_unwrap)]
    let result: Result<_> = (|| {
        if source_name.is_some() && scanner.peek().kind != TokenKind::Keyword(Keyword::Namespace) {
            let mut ns = item::parse_implicit_namespace(
                source_name.expect("invariant checked above via `.is_some()`"),
                &mut scanner,
            )
            .map(|x| vec![x])?;
            if let Some(ref mut ns) = ns.get_mut(0) {
                if let Some(x) = ns.items.get_mut(0) {
                    x.span.lo = 0;
                    x.doc = doc;
                };
            }
            Ok(ns)
        } else {
            let mut ns = item::parse_namespaces(&mut scanner)?;
            if let Some(x) = ns.get_mut(0) {
                x.span.lo = 0;
                x.doc = doc;
            };
            Ok(ns)
        }
    })();

    match result {
        Ok(namespaces) => (namespaces, scanner.into_errors()),
        Err(error) => {
            let mut errors = scanner.into_errors();
            errors.push(error);
            (Vec::new(), errors)
        }
    }
}

#[must_use]
pub fn top_level_nodes(
    input: &str,
    language_features: LanguageFeatures,
) -> (Vec<TopLevelNode>, Vec<Error>) {
    let mut scanner = ParserContext::new(input, language_features);
    match item::parse_top_level_nodes(&mut scanner) {
        Ok(nodes) => (nodes, scanner.into_errors()),
        Err(error) => {
            let mut errors = scanner.into_errors();
            errors.push(error);
            (Vec::new(), errors)
        }
    }
}

#[must_use]
pub fn expr(input: &str, language_features: LanguageFeatures) -> (Box<Expr>, Vec<Error>) {
    let mut scanner = ParserContext::new(input, language_features);
    match expr::expr_eof(&mut scanner) {
        Ok(expr) => (expr, scanner.into_errors()),
        Err(error) => {
            let mut errors = scanner.into_errors();
            errors.push(error);
            (Box::default(), errors)
        }
    }
}
