// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
pub(crate) mod tests;

use std::rc::Rc;

use qsc_data_structures::span::Span;

use super::{
    completion::WordKinds,
    error::{Error, ErrorKind},
    expr::{self, designator},
    prim::{self, barrier, many, opt, recovering, recovering_semi, recovering_token, shorten},
    Result,
};
use crate::{
    ast::{
        list_from_iter, AngleType, Annotation, ArrayBaseTypeKind, ArrayType, BitType, Block,
        ClassicalDeclarationStmt, ComplexType, ConstantDeclaration, Expr, ExprStmt, FloatType,
        IODeclaration, IOKeyword, IncludeStmt, IntType, List, LiteralKind, Pragma,
        QubitDeclaration, ScalarType, ScalarTypeKind, Stmt, StmtKind, SwitchStmt, TypeDef,
        UIntType,
    },
    keyword::Keyword,
    lex::{
        cooked::{Literal, Type},
        Delim, TokenKind,
    },
};

use super::{prim::token, ParserContext};

pub(super) fn parse(s: &mut ParserContext) -> Result<Box<Stmt>> {
    let lo = s.peek().span.lo;
    if let Some(pragma) = opt(s, parse_pragma)? {
        return Ok(Box::new(Stmt {
            span: s.span(lo),
            annotations: [].into(),
            kind: Box::new(StmtKind::Pragma(pragma)),
        }));
    }
    let attrs = many(s, parse_annotation)?;
    let kind = if token(s, TokenKind::Semicolon).is_ok() {
        if attrs.is_empty() {
            Box::new(StmtKind::Empty)
        } else {
            let err_item = default(s.span(lo));
            s.push_error(Error::new(ErrorKind::FloatingAnnotation(err_item.span)));
            return Ok(err_item);
        }
    } else if let Some(v) = opt(s, parse_include)? {
        Box::new(v)
    } else if let Some(decl) = opt(s, parse_local)? {
        Box::new(decl)
    } else if let Some(switch) = opt(s, parse_switch_stmt)? {
        Box::new(StmtKind::Switch(switch))
    } else {
        return Err(Error::new(ErrorKind::Rule(
            "statement",
            s.peek().kind,
            s.peek().span,
        )));
    };

    Ok(Box::new(Stmt {
        span: s.span(lo),
        annotations: attrs.into_boxed_slice(),
        kind,
    }))
}

#[allow(clippy::vec_box)]
pub(super) fn parse_many(s: &mut ParserContext) -> Result<Vec<Box<Stmt>>> {
    many(s, |s| {
        recovering(s, default, &[TokenKind::Semicolon], parse)
    })
}

pub(super) fn parse_block(s: &mut ParserContext) -> Result<Box<Block>> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Open(Delim::Brace))?;
    let stmts = barrier(s, &[TokenKind::Close(Delim::Brace)], parse_many)?;
    recovering_token(s, TokenKind::Close(Delim::Brace));
    Ok(Box::new(Block {
        span: s.span(lo),
        stmts: stmts.into_boxed_slice(),
    }))
}

#[allow(clippy::unnecessary_box_returns)]
fn default(span: Span) -> Box<Stmt> {
    Box::new(Stmt {
        span,
        annotations: Vec::new().into_boxed_slice(),
        kind: Box::new(StmtKind::Err),
    })
}

pub fn parse_annotation(s: &mut ParserContext) -> Result<Box<Annotation>> {
    let lo = s.peek().span.lo;
    s.expect(WordKinds::Annotation);

    let token = s.peek();
    let pat = &['\t', ' '];
    let parts: Vec<&str> = if token.kind == TokenKind::Annotation {
        let lexeme = s.read();
        s.advance();
        // remove @
        // split lexeme at first space/tab collecting each side
        shorten(1, 0, lexeme).splitn(2, pat).collect()
    } else {
        return Err(Error::new(ErrorKind::Rule(
            "annotation",
            token.kind,
            token.span,
        )));
    };

    let identifier = parts.first().map_or_else(
        || {
            Err(Error::new(ErrorKind::Rule(
                "annotation",
                token.kind,
                token.span,
            )))
        },
        |s| Ok(Into::<Rc<str>>::into(*s)),
    )?;

    if identifier.is_empty() {
        s.push_error(Error::new(ErrorKind::Rule(
            "annotation missing identifier",
            token.kind,
            token.span,
        )));
    }

    // remove any leading whitespace from the value side
    let value = parts
        .get(1)
        .map(|s| Into::<Rc<str>>::into(s.trim_start_matches(pat)));

    Ok(Box::new(Annotation {
        span: s.span(lo),
        identifier,
        value,
    }))
}

fn parse_include(s: &mut ParserContext) -> Result<StmtKind> {
    let lo = s.peek().span.lo;
    s.expect(WordKinds::Include);
    token(s, TokenKind::Keyword(crate::keyword::Keyword::Include))?;
    let next = s.peek();

    let v = expr::lit(s)?;
    if let Some(v) = v {
        if let LiteralKind::String(v) = v.kind {
            let r = IncludeStmt {
                span: s.span(lo),
                filename: v.to_string(),
            };
            token(s, TokenKind::Semicolon)?;
            return Ok(StmtKind::Include(r));
        }
    };
    Err(Error::new(ErrorKind::Rule(
        "include statement",
        TokenKind::Literal(Literal::String),
        next.span,
    )))
}

fn parse_pragma(s: &mut ParserContext) -> Result<Pragma> {
    let lo = s.peek().span.lo;
    s.expect(WordKinds::Pragma);

    let token = s.peek();

    let parts: Vec<&str> = if token.kind == TokenKind::Pragma {
        let lexeme = s.read();
        s.advance();
        // remove pragma keyword and any leading whitespace
        // split lexeme at first space/tab collecting each side
        let pat = &['\t', ' '];
        shorten(6, 0, lexeme)
            .trim_start_matches(pat)
            .splitn(2, pat)
            .collect()
    } else {
        return Err(Error::new(ErrorKind::Rule(
            "pragma", token.kind, token.span,
        )));
    };

    let identifier = parts.first().map_or_else(
        || {
            Err(Error::new(ErrorKind::Rule(
                "pragma", token.kind, token.span,
            )))
        },
        |s| Ok(Into::<Rc<str>>::into(*s)),
    )?;

    if identifier.is_empty() {
        s.push_error(Error::new(ErrorKind::Rule(
            "pragma missing identifier",
            token.kind,
            token.span,
        )));
    }
    let value = parts.get(1).map(|s| Into::<Rc<str>>::into(*s));

    Ok(Pragma {
        span: s.span(lo),
        identifier,
        value,
    })
}

fn parse_local(s: &mut ParserContext) -> Result<StmtKind> {
    if let Some(decl) = opt(s, parse_classical_decl)? {
        Ok(decl)
    } else if let Some(decl) = opt(s, parse_quantum_decl)? {
        Ok(decl)
    } else if let Some(decl) = opt(s, parse_io_decl)? {
        Ok(decl)
    } else {
        Err(Error::new(ErrorKind::Rule(
            "local declaration",
            s.peek().kind,
            s.peek().span,
        )))
    }
}

fn parse_quantum_decl(s: &mut ParserContext) -> Result<StmtKind> {
    let lo = s.peek().span.lo;
    s.expect(WordKinds::Qubit);
    token(s, TokenKind::Keyword(crate::keyword::Keyword::Qubit))?;
    let size = opt(s, designator)?;
    let name = prim::ident(s)?;

    recovering_semi(s);
    Ok(StmtKind::QuantumDecl(QubitDeclaration {
        span: s.span(lo),
        qubit: *name,
        size,
    }))
}

fn parse_io_decl(s: &mut ParserContext) -> Result<StmtKind> {
    let lo = s.peek().span.lo;

    let kind = if token(s, TokenKind::Keyword(crate::keyword::Keyword::Input)).is_ok() {
        IOKeyword::Input
    } else if token(s, TokenKind::Keyword(crate::keyword::Keyword::Output)).is_ok() {
        IOKeyword::Output
    } else {
        let token = s.peek();
        return Err(Error::new(ErrorKind::Rule(
            "io declaration",
            token.kind,
            token.span,
        )));
    };

    let ty = scalar_or_array_type(s)?;

    let identifier = prim::ident(s)?;
    recovering_semi(s);
    let decl = IODeclaration {
        span: s.span(lo),
        io_identifier: kind,
        r#type: ty,
        identifier,
    };
    Ok(StmtKind::IODeclaration(decl))
}

pub fn scalar_or_array_type(s: &mut ParserContext) -> Result<TypeDef> {
    if let Ok(v) = scalar_type(s) {
        return Ok(TypeDef::Scalar(v));
    }
    if let Ok(v) = array_type(s) {
        return Ok(TypeDef::Array(v));
    }
    Err(Error::new(ErrorKind::Rule(
        "scalar or array type",
        s.peek().kind,
        s.peek().span,
    )))
}

fn parse_classical_decl(s: &mut ParserContext) -> Result<StmtKind> {
    let lo = s.peek().span.lo;
    let is_const = if s.peek().kind == TokenKind::Keyword(crate::keyword::Keyword::Const) {
        s.advance();
        true
    } else {
        false
    };
    let ty = scalar_or_array_type(s)?;

    let identifier = prim::ident(s)?;

    let stmt = if is_const {
        token(s, TokenKind::Eq)?;
        let init_expr = expr::expr(s)?;
        recovering_semi(s);
        let decl = ConstantDeclaration {
            span: s.span(lo),
            r#type: ty,
            identifier,
            init_expr: Box::new(ExprStmt {
                span: init_expr.span,
                expr: init_expr,
            }),
        };
        StmtKind::ConstDecl(decl)
    } else {
        let init_expr = if s.peek().kind == TokenKind::Eq {
            s.advance();
            Some(expr::value_expr(s)?)
        } else {
            None
        };
        recovering_semi(s);
        let decl = ClassicalDeclarationStmt {
            span: s.span(lo),
            r#type: ty,
            identifier,
            init_expr,
        };
        StmtKind::ClassicalDecl(decl)
    };

    Ok(stmt)
}

pub(super) fn array_type(s: &mut ParserContext) -> Result<ArrayType> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Type(Type::Array))?;
    token(s, TokenKind::Open(Delim::Bracket))?;
    let kind = array_base_type(s)?;
    token(s, TokenKind::Comma)?;
    let expr_list = expr::expr_list(s)?;
    token(s, TokenKind::Close(Delim::Bracket))?;

    Ok(ArrayType {
        base_type: kind,
        span: s.span(lo),
        dimensions: expr_list
            .into_iter()
            .map(Box::new)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    })
}

pub(super) fn array_base_type(s: &mut ParserContext) -> Result<ArrayBaseTypeKind> {
    if let Ok(v) = array_angle_type(s) {
        return Ok(v);
    }
    if let Ok(v) = array_bool_type(s) {
        return Ok(v);
    }
    if let Ok(v) = array_int_type(s) {
        return Ok(v);
    }
    if let Ok(v) = array_uint_type(s) {
        return Ok(v);
    }
    if let Ok(v) = array_float_type(s) {
        return Ok(v);
    }
    if let Ok(v) = array_complex_type(s) {
        return Ok(v);
    }
    if let Ok(v) = array_duration_type(s) {
        return Ok(v);
    }

    Err(Error::new(ErrorKind::Rule(
        "array type",
        s.peek().kind,
        s.peek().span,
    )))
}

pub(super) fn scalar_type(s: &mut ParserContext) -> Result<ScalarType> {
    if let Ok(v) = scalar_bit_type(s) {
        return Ok(v);
    }
    if let Ok(v) = scalar_angle_type(s) {
        return Ok(v);
    }
    if let Ok(v) = scalar_bool_type(s) {
        return Ok(v);
    }
    if let Ok(v) = scalar_int_type(s) {
        return Ok(v);
    }
    if let Ok(v) = scalar_uint_type(s) {
        return Ok(v);
    }
    if let Ok(v) = scalar_float_type(s) {
        return Ok(v);
    }
    if let Ok(v) = scalar_complex_type(s) {
        return Ok(v);
    }
    if let Ok(v) = scalar_duration_type(s) {
        return Ok(v);
    }
    if let Ok(v) = scalar_stretch_type(s) {
        return Ok(v);
    }
    Err(Error::new(ErrorKind::Rule(
        "scalar type",
        s.peek().kind,
        s.peek().span,
    )))
}

fn scalar_bit_type(s: &mut ParserContext) -> Result<ScalarType> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Type(Type::Bit))?;
    let size = opt(s, designator)?;
    Ok(ScalarType {
        span: s.span(lo),
        kind: ScalarTypeKind::Bit(BitType {
            size,
            span: s.span(lo),
        }),
    })
}

fn scalar_int_type(s: &mut ParserContext) -> Result<ScalarType> {
    let lo = s.peek().span.lo;
    let ty = int_type(s)?;
    Ok(ScalarType {
        span: s.span(lo),
        kind: ScalarTypeKind::Int(ty),
    })
}

fn array_int_type(s: &mut ParserContext) -> Result<ArrayBaseTypeKind> {
    let ty = int_type(s)?;
    Ok(ArrayBaseTypeKind::Int(ty))
}

fn int_type(s: &mut ParserContext) -> Result<IntType> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Type(Type::Int))?;
    let size = opt(s, designator)?;
    Ok(IntType {
        size,
        span: s.span(lo),
    })
}
fn scalar_uint_type(s: &mut ParserContext) -> Result<ScalarType> {
    let lo = s.peek().span.lo;
    let ty = uint_type(s)?;
    Ok(ScalarType {
        span: s.span(lo),
        kind: ScalarTypeKind::UInt(ty),
    })
}

fn array_uint_type(s: &mut ParserContext) -> Result<ArrayBaseTypeKind> {
    let ty = uint_type(s)?;
    Ok(ArrayBaseTypeKind::UInt(ty))
}

fn uint_type(s: &mut ParserContext) -> Result<UIntType> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Type(Type::UInt))?;
    let size = opt(s, designator)?;
    Ok(UIntType {
        size,
        span: s.span(lo),
    })
}

fn scalar_float_type(s: &mut ParserContext) -> Result<ScalarType> {
    let lo = s.peek().span.lo;
    let ty = float_type(s)?;
    Ok(ScalarType {
        span: s.span(lo),
        kind: ScalarTypeKind::Float(ty),
    })
}

fn array_float_type(s: &mut ParserContext) -> Result<ArrayBaseTypeKind> {
    let ty = float_type(s)?;
    Ok(ArrayBaseTypeKind::Float(ty))
}

fn float_type(s: &mut ParserContext) -> Result<FloatType> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Type(Type::Float))?;
    let size = opt(s, designator)?;
    Ok(FloatType {
        span: s.span(lo),
        size,
    })
}

fn scalar_angle_type(s: &mut ParserContext) -> Result<ScalarType> {
    let lo = s.peek().span.lo;
    let ty = angle_type(s)?;
    Ok(ScalarType {
        span: s.span(lo),
        kind: ScalarTypeKind::Angle(ty),
    })
}

fn array_angle_type(s: &mut ParserContext) -> Result<ArrayBaseTypeKind> {
    let ty = angle_type(s)?;
    Ok(ArrayBaseTypeKind::Angle(ty))
}

fn angle_type(s: &mut ParserContext) -> Result<AngleType> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Type(Type::Angle))?;
    let size = opt(s, designator)?;
    Ok(AngleType {
        size,
        span: s.span(lo),
    })
}

fn scalar_bool_type(s: &mut ParserContext) -> Result<ScalarType> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Type(Type::Bool))?;
    Ok(ScalarType {
        span: s.span(lo),
        kind: ScalarTypeKind::BoolType,
    })
}

fn array_bool_type(s: &mut ParserContext) -> Result<ArrayBaseTypeKind> {
    token(s, TokenKind::Type(Type::Bool))?;
    Ok(ArrayBaseTypeKind::BoolType)
}

fn scalar_duration_type(s: &mut ParserContext) -> Result<ScalarType> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Type(Type::Duration))?;
    Ok(ScalarType {
        span: s.span(lo),
        kind: ScalarTypeKind::Duration,
    })
}

fn array_duration_type(s: &mut ParserContext) -> Result<ArrayBaseTypeKind> {
    token(s, TokenKind::Type(Type::Duration))?;
    Ok(ArrayBaseTypeKind::Duration)
}

fn scalar_stretch_type(s: &mut ParserContext) -> Result<ScalarType> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Type(Type::Stretch))?;
    Ok(ScalarType {
        span: s.span(lo),
        kind: ScalarTypeKind::Stretch,
    })
}

pub(super) fn scalar_complex_type(s: &mut ParserContext) -> Result<ScalarType> {
    let lo = s.peek().span.lo;

    let ty = complex_type(s)?;
    Ok(ScalarType {
        span: s.span(lo),
        kind: ScalarTypeKind::Complex(ty),
    })
}

pub(super) fn array_complex_type(s: &mut ParserContext) -> Result<ArrayBaseTypeKind> {
    let ty = complex_type(s)?;
    Ok(ArrayBaseTypeKind::Complex(ty))
}

pub(super) fn complex_type(s: &mut ParserContext) -> Result<ComplexType> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Type(Type::Complex))?;

    let subty = opt(s, complex_subtype)?;
    Ok(ComplexType {
        base_size: subty,
        span: s.span(lo),
    })
}

pub(super) fn complex_subtype(s: &mut ParserContext) -> Result<FloatType> {
    token(s, TokenKind::Open(Delim::Bracket))?;
    let ty = float_type(s)?;
    token(s, TokenKind::Close(Delim::Bracket))?;
    Ok(ty)
}

/// The Language Spec and the grammar for switch statements disagree.
/// We followed the Spec when writing the parser
/// <https://openqasm.com/language/classical.html#the-switch-statement>.
pub fn parse_switch_stmt(s: &mut ParserContext) -> Result<SwitchStmt> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Keyword(Keyword::Switch))?;

    // Controlling expression.
    token(s, TokenKind::Open(Delim::Paren))?;
    let controlling_expr = expr::expr(s)?;
    token(s, TokenKind::Close(Delim::Paren))?;

    // Open cases bracket.
    token(s, TokenKind::Open(Delim::Brace))?;

    // Cases.
    let lo_cases = s.peek().span.lo;
    let cases = list_from_iter(many(s, case_stmt)?);
    if cases.is_empty() {
        s.push_error(Error::new(ErrorKind::MissingSwitchCases(s.span(lo_cases))));
    }

    // Default case.
    let default = opt(s, default_case_stmt)?;

    // Close cases bracket.
    token(s, TokenKind::Close(Delim::Brace))?;

    Ok(SwitchStmt {
        span: s.span(lo),
        target: controlling_expr,
        cases,
        default,
    })
}

fn case_stmt(s: &mut ParserContext) -> Result<(List<Expr>, Block)> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Keyword(Keyword::Case))?;

    let controlling_label = expr::expr_list(s)?;
    if controlling_label.is_empty() {
        s.push_error(Error::new(ErrorKind::MissingSwitchCaseLabels(s.span(lo))));
    }

    let block = parse_block(s).map(|block| *block)?;
    Ok((list_from_iter(controlling_label), block))
}

fn default_case_stmt(s: &mut ParserContext) -> Result<Block> {
    token(s, TokenKind::Keyword(Keyword::Default))?;
    parse_block(s).map(|block| *block)
}
