// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::Error;
use crate::{
    lex::{Lexer, Token, TokenKind},
    predict::CursorAwareLexer,
    ErrorKind,
};
use qsc_data_structures::{language_features::LanguageFeatures, span::Span};

#[derive(Debug)]
pub(super) struct NoBarrierError;

#[derive(Clone, Debug, PartialEq)]
pub enum Prediction {
    Path,
    Field,
    Attr,
    Namespace,
    Qubit,
    Ty,
    TyParam,
    Keyword(&'static str),
}

enum ScannerKind<'a> {
    Normal(Lexer<'a>),
    Predict(CursorAwareLexer<'a>, Vec<Prediction>),
}

pub(super) struct ParserContext<'a> {
    scanner: Scanner<'a>,
    language_features: LanguageFeatures,
}

impl ParserContext<'_> {
    pub fn push_prediction(&mut self, expectations: Vec<Prediction>) {
        self.scanner.push_prediction(expectations);
    }

    pub fn into_predictions(self) -> Vec<Prediction> {
        self.scanner.into_predictions()
    }
}

/// Scans over the token stream. Notably enforces LL(1) parser behavior via
/// its lack of a [Clone] implementation and limited peek functionality.
/// This struct should never be clonable, and it should never be able to
/// peek more than one token ahead, to maintain LL(1) enforcement.
pub(super) struct Scanner<'a> {
    input: &'a str,
    kind: ScannerKind<'a>,
    barriers: Vec<&'a [TokenKind]>,
    errors: Vec<Error>,
    recovered_eof: bool,
    peek: Token,
    pub(super) offset: u32,
}

impl<'a> ParserContext<'a> {
    pub fn new(input: &'a str, language_features: LanguageFeatures) -> Self {
        Self {
            scanner: Scanner::new(input),
            language_features,
        }
    }

    pub fn predict_mode(input: &'a str, cursor_offset: u32) -> Self {
        Self {
            scanner: Scanner::predict_mode(input, cursor_offset),
            language_features: LanguageFeatures::default(),
        }
    }

    pub(super) fn peek(&self) -> Token {
        self.scanner.peek()
    }

    pub(super) fn read(&self) -> &'a str {
        self.scanner.read()
    }

    pub(super) fn span(&self, from: u32) -> Span {
        self.scanner.span(from)
    }

    pub(super) fn advance(&mut self) {
        self.scanner.advance();
    }

    /// Pushes a recovery barrier. While the barrier is active, recovery will never advance past any
    /// of the barrier tokens, unless it is explicitly listed as a recovery token.
    pub(super) fn push_barrier(&mut self, tokens: &'a [TokenKind]) {
        self.scanner.push_barrier(tokens);
    }

    /// Pops the most recently pushed active barrier.
    pub(super) fn pop_barrier(&mut self) -> Result<(), NoBarrierError> {
        self.scanner.pop_barrier()
    }

    /// Tries to recover from a parse error by advancing tokens until any of the given recovery
    /// tokens, or a barrier token, is found. If a recovery token is found, it is consumed. If a
    /// barrier token is found first, it is not consumed.
    pub(super) fn recover(&mut self, tokens: &[TokenKind]) {
        self.scanner.recover(tokens);
    }

    pub(super) fn push_error(&mut self, error: Error) {
        self.scanner.push_error(error);
    }

    pub(super) fn into_errors(self) -> Vec<Error> {
        self.scanner.into_errors()
    }

    pub(crate) fn contains_language_feature(&self, feat: LanguageFeatures) -> bool {
        self.language_features.contains(feat)
    }
}

impl<'a> Scanner<'a> {
    pub(super) fn new(input: &'a str) -> Self {
        let mut tokens = Lexer::new(input);
        let (peek, errors) = next_ok(&mut tokens);
        Self {
            input,
            kind: ScannerKind::Normal(tokens),
            peek: peek.unwrap_or_else(|| eof(input.len())),
            errors: errors
                .into_iter()
                .map(|e| Error(ErrorKind::Lex(e)))
                .collect(),
            barriers: Vec::new(),
            offset: 0,
            recovered_eof: false,
        }
    }

    pub(super) fn predict_mode(input: &'a str, cursor_offset: u32) -> Self {
        let mut tokens = CursorAwareLexer::new(input, cursor_offset);
        let (peek, errors) = next_ok(&mut tokens);
        Self {
            input,
            kind: ScannerKind::Predict(tokens, Vec::new()),
            recovered_eof: false,
            peek: peek.unwrap_or_else(|| eof(input.len())),
            errors: errors
                .into_iter()
                .map(|e| Error(ErrorKind::Lex(e)))
                .collect(),
            barriers: Vec::new(),
            offset: 0,
        }
    }

    pub(super) fn peek(&self) -> Token {
        self.peek
    }

    pub(super) fn read(&self) -> &'a str {
        &self.input[self.peek.span]
    }

    pub(super) fn span(&self, from: u32) -> Span {
        Span {
            lo: from,
            hi: self.offset,
        }
    }

    pub(super) fn advance(&mut self) {
        if self.peek.kind != TokenKind::Eof {
            self.offset = self.peek.span.hi;
            let (peek, errors) = match &mut self.kind {
                ScannerKind::Normal(tokens) => next_ok(tokens),
                ScannerKind::Predict(tokens, _) => next_ok(tokens),
            };

            self.errors
                .extend(errors.into_iter().map(|e| Error(ErrorKind::Lex(e))));
            self.peek = peek.unwrap_or_else(|| eof(self.input.len()));
        }
    }

    /// Pushes a recovery barrier. While the barrier is active, recovery will never advance past any
    /// of the barrier tokens, unless it is explicitly listed as a recovery token.
    pub(super) fn push_barrier(&mut self, tokens: &'a [TokenKind]) {
        self.barriers.push(tokens);
    }

    /// Pops the most recently pushed active barrier.
    pub(super) fn pop_barrier(&mut self) -> Result<(), NoBarrierError> {
        match self.barriers.pop() {
            Some(_) => Ok(()),
            None => Err(NoBarrierError),
        }
    }

    /// Tries to recover from a parse error by advancing tokens until any of the given recovery
    /// tokens, or a barrier token, is found. If a recovery token is found, it is consumed. If a
    /// barrier token is found first, it is not consumed.
    pub(super) fn recover(&mut self, tokens: &[TokenKind]) {
        println!("recovering at {} ", self.peek.span.lo);
        loop {
            let peek = self.peek.kind;
            if contains(peek, tokens) {
                self.advance();
                break;
            } else if peek == TokenKind::Eof || self.barriers.iter().any(|&b| contains(peek, b)) {
                if let ScannerKind::Predict(lexer, _) = &mut self.kind {
                    lexer.at_cursor = false;
                }
                break;
            }
            if peek == TokenKind::Eof || self.barriers.iter().any(|&b| contains(peek, b)) {
                break;
            }
            self.advance();
        }
    }

    pub(super) fn push_error(&mut self, error: Error) {
        let is_eof_err = matches!(
            error.0,
            ErrorKind::Token(_, TokenKind::Eof, _) | ErrorKind::Rule(_, TokenKind::Eof, _)
        );
        if !is_eof_err || !self.recovered_eof {
            self.errors.push(error);
            self.recovered_eof = self.recovered_eof || is_eof_err;
        }
    }

    pub(super) fn into_errors(self) -> Vec<Error> {
        self.errors
    }

    pub fn push_prediction(&mut self, expectations: Vec<Prediction>) {
        if let ScannerKind::Predict(lexer, predictions) = &mut self.kind {
            println!("received predictions: {expectations:?}");
            if lexer.at_cursor {
                println!("at cursor, pushed predictions");
                predictions.extend(expectations);
            }
        }
    }

    pub fn into_predictions(self) -> Vec<Prediction> {
        if let ScannerKind::Predict(_, predictions) = self.kind {
            predictions
        } else {
            panic!("expected prediction scanner")
        }
    }
}

fn eof(offset: usize) -> Token {
    let offset = offset.try_into().expect("eof offset should fit into u32");
    Token {
        kind: TokenKind::Eof,
        span: Span {
            lo: offset,
            hi: offset,
        },
    }
}

/// Advances the iterator by skipping [`Err`] values until the first [`Ok`] value is found. Returns
/// the found value or [`None`] if the iterator is exhausted. All skipped errors are also
/// accumulated into a vector and returned.
fn next_ok<T, E>(iter: impl Iterator<Item = Result<T, E>>) -> (Option<T>, Vec<E>) {
    let mut errors = Vec::new();
    for result in iter {
        match result {
            Ok(v) => return (Some(v), errors),
            Err(e) => errors.push(e),
        }
    }

    (None, errors)
}

fn contains<'a>(token: TokenKind, tokens: impl IntoIterator<Item = &'a TokenKind>) -> bool {
    tokens.into_iter().any(|&t| t == token)
}
