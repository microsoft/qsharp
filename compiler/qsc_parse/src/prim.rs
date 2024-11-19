// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use super::{keyword::Keyword, scan::ParserContext, ty::recovering_ty, Error, Parser, Result};
use crate::{
    completion::WordKinds,
    item::throw_away_doc,
    lex::{Delim, TokenKind},
    ErrorKind,
};
use qsc_ast::ast::{Ident, IncompletePath, NodeId, Pat, PatKind, Path, PathKind};
use qsc_data_structures::span::{Span, WithSpan};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FinalSep {
    Present,
    Missing,
}

impl FinalSep {
    pub(super) fn reify<T, U>(
        self,
        mut xs: Vec<T>,
        mut as_paren: impl FnMut(T) -> U,
        mut as_seq: impl FnMut(Box<[T]>) -> U,
    ) -> U {
        if self == Self::Missing && xs.len() == 1 {
            as_paren(xs.pop().expect("vector should have exactly one item"))
        } else {
            as_seq(xs.into_boxed_slice())
        }
    }
}

pub(super) fn token(s: &mut ParserContext, t: TokenKind) -> Result<()> {
    if let TokenKind::Keyword(k) = t {
        s.expect(k.into());
    }

    if s.peek().kind == t {
        s.advance();
        Ok(())
    } else {
        Err(Error::new(ErrorKind::Token(
            t,
            s.peek().kind,
            s.peek().span,
        )))
    }
}

pub(super) fn apos_ident(s: &mut ParserContext) -> Result<Box<Ident>> {
    s.expect(WordKinds::TyParam);
    let peek = s.peek();
    if peek.kind == TokenKind::AposIdent {
        let name = s.read().into();
        s.advance();
        Ok(Box::new(Ident {
            id: NodeId::default(),
            span: peek.span,
            name,
        }))
    } else {
        Err(Error::new(ErrorKind::Rule(
            "generic parameter",
            peek.kind,
            peek.span,
        )))
    }
}

pub(super) fn ident(s: &mut ParserContext) -> Result<Box<Ident>> {
    let peek = s.peek();
    if peek.kind == TokenKind::Ident {
        let name = s.read().into();
        s.advance();
        Ok(Box::new(Ident {
            id: NodeId::default(),
            span: peek.span,
            name,
        }))
    } else {
        Err(Error::new(ErrorKind::Rule(
            "identifier",
            peek.kind,
            peek.span,
        )))
    }
}

/// A `path` is a dot-separated list of idents like "Foo.Bar.Baz"
/// this can be a namespace name (in an open statement or namespace declaration),
/// a reference to an item, like `Microsoft.Quantum.Diagnostics.DumpMachine`,
/// or a field access.
///
/// Path parser. If parsing fails, also returns any valid segments
/// that were parsed up to the final `.` token.
pub(super) fn path(
    s: &mut ParserContext,
    kind: WordKinds,
) -> std::result::Result<Box<Path>, (Error, Option<Box<IncompletePath>>)> {
    s.expect(kind);

    let lo = s.peek().span.lo;
    let i = ident(s).map_err(|e| (e, None))?;

    let mut parts = vec![*i];
    while token(s, TokenKind::Dot).is_ok() {
        s.expect(WordKinds::PathSegment);
        match ident(s) {
            Ok(ident) => parts.push(*ident),
            Err(error) => {
                let trivia_span = s.skip_trivia();
                let keyword = trivia_span.hi == trivia_span.lo
                    && matches!(s.peek().kind, TokenKind::Keyword(_));
                if keyword {
                    // Consume any keyword that comes immediately after the final
                    // dot, assuming it was intended to be part of the path.
                    s.advance();
                }

                return Err((
                    error,
                    Some(Box::new(IncompletePath {
                        span: s.span(lo),
                        segments: parts.into(),
                        keyword,
                    })),
                ));
            }
        }
    }

    let name = parts.pop().expect("path should have at least one part");
    let namespace = if parts.is_empty() {
        None
    } else {
        Some(parts.into())
    };

    Ok(Box::new(Path {
        id: NodeId::default(),
        span: s.span(lo),
        segments: namespace,
        name: name.into(),
    }))
}

/// Recovering [`Path`] parser. Parsing only fails if no segments
/// were successfully parsed. If any segments were successfully parsed,
/// returns a [`PathKind::Err`] containing the segments that were
/// successfully parsed up to the final `.` token.
pub(super) fn recovering_path(s: &mut ParserContext, kind: WordKinds) -> Result<PathKind> {
    match path(s, kind) {
        Ok(path) => Ok(PathKind::Ok(path)),
        Err((error, Some(incomplete_path))) => {
            s.push_error(error);
            Ok(PathKind::Err(Some(incomplete_path)))
        }
        Err((error, None)) => Err(error),
    }
}

pub(super) fn pat(s: &mut ParserContext) -> Result<Box<Pat>> {
    throw_away_doc(s);
    let lo = s.peek().span.lo;
    let kind = if token(s, TokenKind::Keyword(Keyword::Underscore)).is_ok() {
        let ty = if token(s, TokenKind::Colon).is_ok() {
            Some(Box::new(recovering_ty(s)?))
        } else {
            None
        };
        Ok(PatKind::Discard(ty))
    } else if token(s, TokenKind::DotDotDot).is_ok() {
        Ok(PatKind::Elided)
    } else if token(s, TokenKind::Open(Delim::Paren)).is_ok() {
        let (pats, final_sep) = seq(s, pat)?;
        token(s, TokenKind::Close(Delim::Paren))?;
        Ok(final_sep.reify(pats, PatKind::Paren, PatKind::Tuple))
    } else {
        let name = ident(s).map_err(|e| map_rule_name("pattern", e))?;
        let ty = if token(s, TokenKind::Colon).is_ok() {
            Some(Box::new(recovering_ty(s)?))
        } else {
            None
        };
        Ok(PatKind::Bind(name, ty))
    }?;

    Ok(Box::new(Pat {
        id: NodeId::default(),
        span: s.span(lo),
        kind: Box::new(kind),
    }))
}

/// Optionally parse with the given parser.
/// Returns Ok(Some(value)) if the parser succeeded,
/// Ok(None) if the parser failed on the first token,
/// Err(error) if the parser failed after consuming some tokens.
pub(super) fn opt<T>(s: &mut ParserContext, mut p: impl Parser<T>) -> Result<Option<T>> {
    let offset = s.peek().span.lo;
    match p(s) {
        Ok(x) => Ok(Some(x)),
        Err(error) if advanced(s, offset) => Err(error),
        Err(_) => Ok(None),
    }
}

pub(super) fn many<T>(s: &mut ParserContext, mut p: impl Parser<T>) -> Result<Vec<T>> {
    let mut xs = Vec::new();
    while let Some(x) = opt(s, &mut p)? {
        xs.push(x);
    }
    Ok(xs)
}

/// Parses a sequence of items separated by commas.
/// Supports recovering on missing items.
pub(super) fn seq<T>(s: &mut ParserContext, mut p: impl Parser<T>) -> Result<(Vec<T>, FinalSep)>
where
    T: Default + WithSpan,
{
    let mut xs = Vec::new();
    let mut final_sep = FinalSep::Missing;
    while s.peek().kind == TokenKind::Comma {
        let mut span = s.peek().span;
        span.hi = span.lo;
        s.push_error(Error::new(ErrorKind::MissingSeqEntry(span)));
        xs.push(T::default().with_span(span));
        s.advance();
    }
    while let Some(x) = opt(s, &mut p)? {
        xs.push(x);
        if token(s, TokenKind::Comma).is_ok() {
            while s.peek().kind == TokenKind::Comma {
                let mut span = s.peek().span;
                span.hi = span.lo;
                s.push_error(Error::new(ErrorKind::MissingSeqEntry(span)));
                xs.push(T::default().with_span(span));
                s.advance();
            }
            final_sep = FinalSep::Present;
        } else {
            final_sep = FinalSep::Missing;
            break;
        }
    }
    Ok((xs, final_sep))
}

/// Try to parse with the given parser.
///
/// If the parser fails on the first token, returns the default value.
///
/// If the parser fails after consuming some tokens, propagates the error.
pub(super) fn parse_or_else<T>(
    s: &mut ParserContext,
    default: impl FnOnce(Span) -> T,
    mut p: impl Parser<T>,
) -> Result<T> {
    let lo = s.peek().span.lo;
    match p(s) {
        Ok(value) => Ok(value),
        Err(error) if advanced(s, lo) => Err(error),
        Err(error) => {
            s.push_error(error);
            // The whitespace will become part of the error span
            s.skip_trivia();
            Ok(default(s.span(lo)))
        }
    }
}

/// Try to parse with the given parser.
///
/// If the parser fails on the first token, propagates the error.
///
/// If the parser fails after consuming some tokens, performs
/// recovery by advancing until the next token in `tokens` is found.
/// The recovery token is consumed.
pub(super) fn recovering<T>(
    s: &mut ParserContext,
    default: impl FnOnce(Span) -> T,
    tokens: &[TokenKind],
    mut p: impl Parser<T>,
) -> Result<T> {
    let offset = s.peek().span.lo;
    match p(s) {
        Ok(value) => Ok(value),
        Err(error) if advanced(s, offset) => {
            s.push_error(error);
            s.recover(tokens);
            Ok(default(s.span(offset)))
        }
        Err(error) => Err(error),
    }
}

/// Try to parse with the given parser.
///
/// If the parser fails on the first token, returns the default value.
///
/// If the parser fails after consuming some tokens, performs
/// recovery by advancing until the next token in `tokens` is found.
/// The recovery token is consumed.
///
/// This behavior is a combination of [`recovering`] and [`parse_or_else`],
/// and provides the most aggressive error recovery.
pub(super) fn recovering_parse_or_else<T>(
    s: &mut ParserContext,
    default: impl FnOnce(Span) -> T,
    tokens: &[TokenKind],
    mut p: impl Parser<T>,
) -> T {
    let lo = s.peek().span.lo;
    match p(s) {
        Ok(value) => value,
        Err(error) => {
            s.push_error(error);

            if advanced(s, lo) {
                s.recover(tokens);
            } else {
                // The whitespace will become part of the error node span
                s.skip_trivia();
            }
            default(s.span(lo))
        }
    }
}

pub(super) fn recovering_semi(s: &mut ParserContext) {
    if let Err(error) = token(s, TokenKind::Semi) {
        // no recovery, just move on to the next token
        s.push_error(error);
    }
}

pub(super) fn recovering_token(s: &mut ParserContext, t: TokenKind) {
    if let Err(error) = token(s, t) {
        s.push_error(error);
        s.recover(&[t]);
    }
}

pub(super) fn barrier<'a, T>(
    s: &mut ParserContext<'a>,
    tokens: &'a [TokenKind],
    mut p: impl Parser<T>,
) -> Result<T> {
    s.push_barrier(tokens);
    let result = p(s);
    s.pop_barrier().expect("barrier should be popped");
    result
}

pub(super) fn shorten(from_start: usize, from_end: usize, s: &str) -> &str {
    &s[from_start..s.len() - from_end]
}

fn advanced(s: &ParserContext, from: u32) -> bool {
    s.peek().span.lo > from
}

fn map_rule_name(name: &'static str, error: Error) -> Error {
    Error::new(match error.0 {
        ErrorKind::Rule(_, found, span) => ErrorKind::Rule(name, found, span),
        ErrorKind::Convert(_, found, span) => ErrorKind::Convert(name, found, span),
        kind => kind,
    })
}
