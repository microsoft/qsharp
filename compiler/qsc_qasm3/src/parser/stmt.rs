// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
pub(crate) mod tests;

use qsc_data_structures::span::Span;
use std::rc::Rc;

use super::{
    completion::WordKinds,
    error::{Error, ErrorKind},
    expr::{self, designator, gate_operand},
    prim::{self, barrier, many, opt, recovering, recovering_semi, recovering_token, seq, shorten},
    Result,
};
use crate::{
    ast::{
        list_from_iter, AccessControl, AliasDeclStmt, AngleType, Annotation, ArrayBaseTypeKind,
        ArrayReferenceType, ArrayType, BitType, Block, BreakStmt, ClassicalDeclarationStmt,
        ComplexType, ConstantDeclaration, ContinueStmt, DefStmt, EndStmt, EnumerableSet, Expr,
        ExprKind, ExprStmt, ExternDecl, ExternParameter, FloatType, ForStmt, FunctionCall,
        GateCall, GateModifierKind, IODeclaration, IOKeyword, Ident, Identifier, IfStmt,
        IncludeStmt, IntType, List, LiteralKind, Pragma, QuantumGateDefinition,
        QuantumGateModifier, QuantumStmt, QubitDeclaration, RangeDefinition, ReturnStmt,
        ScalarType, ScalarTypeKind, Stmt, StmtKind, SwitchStmt, TypeDef, TypedParameter, UIntType,
        WhileLoop,
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
    } else if let Some(decl) = opt(s, parse_gatedef)? {
        Box::new(decl)
    } else if let Some(decl) = opt(s, parse_def)? {
        Box::new(decl)
    } else if let Some(include) = opt(s, parse_include)? {
        Box::new(include)
    } else if let Some(decl) = opt(s, parse_local)? {
        Box::new(decl)
    } else if let Some(decl) = opt(s, parse_extern)? {
        Box::new(decl)
    } else if let Some(switch) = opt(s, parse_switch_stmt)? {
        Box::new(StmtKind::Switch(switch))
    } else if let Some(stmt) = opt(s, parse_if_stmt)? {
        Box::new(StmtKind::If(stmt))
    } else if let Some(stmt) = opt(s, parse_for_loop)? {
        Box::new(StmtKind::For(stmt))
    } else if let Some(stmt) = opt(s, parse_while_loop)? {
        Box::new(StmtKind::WhileLoop(stmt))
    } else if let Some(stmt) = opt(s, parse_return)? {
        Box::new(stmt)
    } else if let Some(stmt) = opt(s, parse_continue_stmt)? {
        Box::new(StmtKind::Continue(stmt))
    } else if let Some(stmt) = opt(s, parse_break_stmt)? {
        Box::new(StmtKind::Break(stmt))
    } else if let Some(stmt) = opt(s, parse_end_stmt)? {
        Box::new(StmtKind::End(stmt))
    } else if let Some(stmt_kind) = opt(s, parse_gate_call_stmt)? {
        Box::new(stmt_kind)
    } else if let Some(stmt) = opt(s, parse_expression_stmt)? {
        Box::new(StmtKind::ExprStmt(stmt))
    } else if let Some(alias) = opt(s, parse_alias_stmt)? {
        Box::new(StmtKind::Alias(alias))
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

// qreg and creg are separate from classical and quantum declarations
// simply for performance reasons. The latter are more common and old
// style declarations should be rare.
fn parse_local(s: &mut ParserContext) -> Result<StmtKind> {
    if let Some(decl) = opt(s, parse_classical_decl)? {
        Ok(decl)
    } else if let Some(decl) = opt(s, parse_quantum_decl)? {
        Ok(decl)
    } else if let Some(decl) = opt(s, parse_io_decl)? {
        Ok(decl)
    } else if let Some(decl) = opt(s, qreg_decl)? {
        Ok(decl)
    } else if let Some(decl) = opt(s, creg_decl)? {
        Ok(decl)
    } else {
        Err(Error::new(ErrorKind::Rule(
            "local declaration",
            s.peek().kind,
            s.peek().span,
        )))
    }
}

fn parse_extern(s: &mut ParserContext) -> Result<StmtKind> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Keyword(crate::keyword::Keyword::Extern))?;
    let ident = Box::new(prim::ident(s)?);
    token(s, TokenKind::Open(Delim::Paren))?;
    let (params, _) = seq(s, extern_arg_def)?;
    token(s, TokenKind::Close(Delim::Paren))?;
    let return_type = opt(s, return_sig)?;
    recovering_semi(s);
    let kind = StmtKind::ExternDecl(ExternDecl {
        span: s.span(lo),
        ident,
        params: list_from_iter(params),
        return_type,
    });
    Ok(kind)
}

fn parse_def(s: &mut ParserContext) -> Result<StmtKind> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Keyword(crate::keyword::Keyword::Def))?;
    let name = Box::new(prim::ident(s)?);
    token(s, TokenKind::Open(Delim::Paren))?;
    let (exprs, _) = seq(s, arg_def)?;
    token(s, TokenKind::Close(Delim::Paren))?;
    let return_type = opt(s, return_sig)?;
    let body = parse_block(s)?;
    let kind = StmtKind::Def(DefStmt {
        span: s.span(lo),
        name,
        params: list_from_iter(exprs),
        body,
        return_type,
    });
    Ok(kind)
}

fn extern_arg_def(s: &mut ParserContext) -> Result<ExternParameter> {
    let lo = s.peek().span.lo;

    let kind = if let Ok(ty) = scalar_type(s) {
        ExternParameter::Scalar(ty, s.span(lo))
    } else if let Ok(ty) = array_reference_ty(s) {
        ExternParameter::ArrayReference(ty, s.span(lo))
    } else if let Ok(size) = extern_creg_type(s) {
        let ty = ScalarType {
            span: s.span(lo),
            kind: ScalarTypeKind::Bit(BitType {
                size,
                span: s.span(lo),
            }),
        };
        ExternParameter::Scalar(ty, s.span(lo))
    } else {
        return Err(Error::new(ErrorKind::Rule(
            "extern argument definition",
            s.peek().kind,
            s.peek().span,
        )));
    };
    Ok(kind)
}

fn arg_def(s: &mut ParserContext) -> Result<TypedParameter> {
    let lo = s.peek().span.lo;

    let kind = if let Ok(ty) = scalar_type(s) {
        let ident = prim::ident(s)?;
        TypedParameter::Scalar(ty, Box::new(ident), s.span(lo))
    } else if let Ok(size) = qubit_type(s) {
        let ident = prim::ident(s)?;
        TypedParameter::Quantum(size, Box::new(ident), s.span(lo))
    } else if let Ok((ident, size)) = creg_type(s) {
        let ty = ScalarType {
            span: s.span(lo),
            kind: ScalarTypeKind::Bit(BitType {
                size,
                span: s.span(lo),
            }),
        };
        TypedParameter::Scalar(ty, ident, s.span(lo))
    } else if let Ok((ident, size)) = qreg_type(s) {
        TypedParameter::Quantum(size, ident, s.span(lo))
    } else if let Ok(ty) = array_reference_ty(s) {
        let ident = prim::ident(s)?;
        TypedParameter::ArrayReference(ty, Box::new(ident), s.span(lo))
    } else {
        return Err(Error::new(ErrorKind::Rule(
            "argument definition",
            s.peek().kind,
            s.peek().span,
        )));
    };
    Ok(kind)
}

fn array_reference_ty(s: &mut ParserContext) -> Result<ArrayReferenceType> {
    let lo = s.peek().span.lo;

    let mutability = if token(s, TokenKind::Keyword(crate::keyword::Keyword::ReadOnly)).is_ok() {
        AccessControl::ReadOnly
    } else if token(s, TokenKind::Keyword(crate::keyword::Keyword::Mutable)).is_ok() {
        AccessControl::Mutable
    } else {
        let token = s.peek();
        return Err(Error::new(ErrorKind::Rule(
            "array reference declaration",
            token.kind,
            token.span,
        )));
    };
    token(s, TokenKind::Type(Type::Array))?;
    token(s, TokenKind::Open(Delim::Bracket))?;
    let base_type = array_base_type(s)?;
    token(s, TokenKind::Comma)?;

    let dimensions = if token(s, TokenKind::Dim).is_ok() {
        token(s, TokenKind::Eq)?;
        vec![expr::expr(s)?]
    } else {
        expr::expr_list(s)?
    };

    token(s, TokenKind::Close(Delim::Bracket))?;
    Ok(ArrayReferenceType {
        span: s.span(lo),
        mutability,
        base_type,
        dimensions: list_from_iter(dimensions),
    })
}

fn return_sig(s: &mut ParserContext) -> Result<ScalarType> {
    token(s, TokenKind::Arrow)?;
    scalar_type(s)
}

fn parse_gatedef(s: &mut ParserContext) -> Result<StmtKind> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Keyword(crate::keyword::Keyword::Gate))?;
    let ident = Box::new(prim::ident(s)?);
    let params = opt(s, gate_params)?.unwrap_or_else(Vec::new);
    let (qubits, _) = seq(s, prim::ident)?;
    let body = parse_block(s)?;
    Ok(StmtKind::QuantumGateDefinition(QuantumGateDefinition {
        span: s.span(lo),
        ident,
        params: list_from_iter(params),
        qubits: list_from_iter(qubits),
        body,
    }))
}

fn gate_params(s: &mut ParserContext<'_>) -> Result<Vec<Ident>> {
    token(s, TokenKind::Open(Delim::Paren))?;
    let (params, _) = seq(s, prim::ident)?;
    token(s, TokenKind::Close(Delim::Paren))?;
    Ok(params)
}

fn parse_return(s: &mut ParserContext) -> Result<StmtKind> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Keyword(crate::keyword::Keyword::Return))?;
    let expr = opt(s, expr::value_expr)?;
    recovering_semi(s);
    Ok(StmtKind::Return(ReturnStmt {
        span: s.span(lo),
        expr,
    }))
}

fn parse_quantum_decl(s: &mut ParserContext) -> Result<StmtKind> {
    let lo = s.peek().span.lo;
    let size = qubit_type(s)?;
    let ident = prim::ident(s)?;

    recovering_semi(s);
    Ok(StmtKind::QuantumDecl(QubitDeclaration {
        span: s.span(lo),
        qubit: Box::new(ident),
        size,
    }))
}

fn qubit_type(s: &mut ParserContext<'_>) -> Result<Option<Expr>> {
    token(s, TokenKind::Keyword(crate::keyword::Keyword::Qubit))?;
    let size = opt(s, designator)?;
    Ok(size)
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

    let ident = Box::new(prim::ident(s)?);
    recovering_semi(s);
    let decl = IODeclaration {
        span: s.span(lo),
        io_identifier: kind,
        r#type: ty,
        ident,
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

    let identifier = Box::new(prim::ident(s)?);

    let stmt = if is_const {
        token(s, TokenKind::Eq)?;
        let init_expr = expr::expr(s)?;
        recovering_semi(s);
        let decl = ConstantDeclaration {
            span: s.span(lo),
            r#type: ty,
            identifier,
            init_expr,
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
        dimensions: list_from_iter(expr_list),
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

fn creg_decl(s: &mut ParserContext) -> Result<StmtKind> {
    let lo: u32 = s.peek().span.lo;
    let (identifier, size) = creg_type(s)?;
    recovering_semi(s);
    Ok(StmtKind::ClassicalDecl(ClassicalDeclarationStmt {
        span: s.span(lo),
        r#type: TypeDef::Scalar(ScalarType {
            span: s.span(lo),
            kind: ScalarTypeKind::Bit(BitType {
                size,
                span: s.span(lo),
            }),
        }),
        identifier,
        init_expr: None,
    }))
}

fn qreg_decl(s: &mut ParserContext) -> Result<StmtKind> {
    let lo = s.peek().span.lo;
    let (identifier, size) = qreg_type(s)?;
    recovering_semi(s);
    Ok(StmtKind::QuantumDecl(QubitDeclaration {
        span: s.span(lo),
        qubit: identifier,
        size,
    }))
}

fn extern_creg_type(s: &mut ParserContext) -> Result<Option<Expr>> {
    token(s, TokenKind::Keyword(crate::keyword::Keyword::CReg))?;
    let size = opt(s, designator)?;
    Ok(size)
}

fn creg_type(s: &mut ParserContext) -> Result<(Box<Ident>, Option<Expr>)> {
    token(s, TokenKind::Keyword(crate::keyword::Keyword::CReg))?;
    let name = Box::new(prim::ident(s)?);
    let size = opt(s, designator)?;
    Ok((name, size))
}

fn qreg_type(s: &mut ParserContext) -> Result<(Box<Ident>, Option<Expr>)> {
    token(s, TokenKind::Keyword(crate::keyword::Keyword::QReg))?;
    let name = Box::new(prim::ident(s)?);
    let size = opt(s, designator)?;
    Ok((name, size))
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
/// We followed the Spec when writing the parser.
/// Reference: <https://openqasm.com/language/classical.html#the-switch-statement>.
pub fn parse_switch_stmt(s: &mut ParserContext) -> Result<SwitchStmt> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Keyword(Keyword::Switch))?;

    // Controlling expression.
    token(s, TokenKind::Open(Delim::Paren))?;
    let controlling_expr = expr::paren_expr(s, lo)?;

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
    recovering_token(s, TokenKind::Close(Delim::Brace));

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

/// Parses a block or a statement. This is a helper function
/// to be used in loops and if stmts, in which their bodies
/// can be a block expr or a single statement.
fn parse_block_or_stmt(s: &mut ParserContext) -> Result<List<Stmt>> {
    if let Some(block) = opt(s, parse_block)? {
        Ok(block.stmts)
    } else {
        Ok(Box::new([parse(s)?]))
    }
}

/// Grammar `IF LPAREN expression RPAREN if_body=statementOrScope (ELSE else_body=statementOrScope)?`.
/// Source: <https://openqasm.com/language/classical.html#if-else-statements>.
pub fn parse_if_stmt(s: &mut ParserContext) -> Result<IfStmt> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Keyword(Keyword::If))?;
    let paren_expr_lo = s.peek().span.lo;
    token(s, TokenKind::Open(Delim::Paren))?;
    let condition = expr::paren_expr(s, paren_expr_lo)?;
    let if_block = parse_block_or_stmt(s)?;
    let else_block = if opt(s, |s| token(s, TokenKind::Keyword(Keyword::Else)))?.is_some() {
        Some(parse_block_or_stmt(s)?)
    } else {
        None
    };

    Ok(IfStmt {
        span: s.span(lo),
        condition,
        if_block,
        else_block,
    })
}

/// Ranges in for loops are a bit different. They must have explicit start and end.
/// Grammar `LBRACKET start=expression COLON (step=expression COLON)? stop=expression]`.
/// Reference: <https://openqasm.com/language/classical.html#for-loops>.
fn for_loop_range_expr(s: &mut ParserContext) -> Result<RangeDefinition> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Open(Delim::Bracket))?;
    let start = Some(expr::expr(s)?);
    token(s, TokenKind::Colon)?;

    // QASM ranges have the pattern [start : (step :)? end]
    // We assume the second expr is the `end`.
    let mut end = Some(expr::expr(s)?);
    let mut step = None;

    // If we find a third expr, then the second expr was the `step`.
    // and this third expr is the actual `end`.
    if token(s, TokenKind::Colon).is_ok() {
        step = end;
        end = Some(expr::expr(s)?);
    }

    recovering_token(s, TokenKind::Close(Delim::Bracket));

    Ok(RangeDefinition {
        span: s.span(lo),
        start,
        end,
        step,
    })
}

/// Parses the `(setExpression | LBRACKET rangeExpression RBRACKET | expression)`
/// part of a for loop statement.
/// Reference: <https://openqasm.com/language/classical.html#for-loops>.
fn for_loop_iterable_expr(s: &mut ParserContext) -> Result<EnumerableSet> {
    if let Some(range) = opt(s, for_loop_range_expr)? {
        Ok(EnumerableSet::RangeDefinition(range))
    } else if let Some(set) = opt(s, expr::set_expr)? {
        Ok(EnumerableSet::DiscreteSet(set))
    } else {
        Ok(EnumerableSet::Expr(expr::expr(s)?))
    }
}

/// Grammar: `FOR scalarType Identifier IN (setExpression | LBRACKET rangeExpression RBRACKET | expression) body=statementOrScope`.
/// Reference: <https://openqasm.com/language/classical.html#for-loops>.
pub fn parse_for_loop(s: &mut ParserContext) -> Result<ForStmt> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Keyword(Keyword::For))?;
    let r#type = scalar_type(s)?;
    let identifier = Identifier::Ident(Box::new(prim::ident(s)?));
    token(s, TokenKind::Keyword(Keyword::In))?;
    let set_declaration = Box::new(for_loop_iterable_expr(s)?);
    let block = parse_block_or_stmt(s)?;

    Ok(ForStmt {
        span: s.span(lo),
        r#type,
        identifier,
        set_declaration,
        block,
    })
}

/// Grammar: `WHILE LPAREN expression RPAREN body=statementOrScope`.
/// Reference: <https://openqasm.com/language/classical.html#while-loops>.
pub fn parse_while_loop(s: &mut ParserContext) -> Result<WhileLoop> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Keyword(Keyword::While))?;
    let paren_expr_lo = s.peek().span.lo;
    token(s, TokenKind::Open(Delim::Paren))?;
    let while_condition = expr::paren_expr(s, paren_expr_lo)?;
    let block = parse_block_or_stmt(s)?;

    Ok(WhileLoop {
        span: s.span(lo),
        while_condition,
        block,
    })
}

/// Grammar: `CONTINUE SEMICOLON`.
fn parse_continue_stmt(s: &mut ParserContext) -> Result<ContinueStmt> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Keyword(Keyword::Continue))?;
    recovering_semi(s);
    Ok(ContinueStmt { span: s.span(lo) })
}

/// Grammar: `BREAK SEMICOLON`.
fn parse_break_stmt(s: &mut ParserContext) -> Result<BreakStmt> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Keyword(Keyword::Break))?;
    recovering_semi(s);
    Ok(BreakStmt { span: s.span(lo) })
}

/// Grammar: `END SEMICOLON`.
fn parse_end_stmt(s: &mut ParserContext) -> Result<EndStmt> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Keyword(Keyword::End))?;
    recovering_semi(s);
    Ok(EndStmt { span: s.span(lo) })
}

/// Grammar: `expression SEMICOLON`.
fn parse_expression_stmt(s: &mut ParserContext) -> Result<ExprStmt> {
    let lo = s.peek().span.lo;
    let expr = expr::expr(s)?;
    recovering_semi(s);
    Ok(ExprStmt {
        span: s.span(lo),
        expr,
    })
}

/// Grammar: `LET Identifier EQUALS aliasExpression SEMICOLON`.
fn parse_alias_stmt(s: &mut ParserContext) -> Result<AliasDeclStmt> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Keyword(Keyword::Let))?;
    let ident = Identifier::Ident(Box::new(prim::ident(s)?));
    token(s, TokenKind::Eq)?;
    let exprs = expr::alias_expr(s)?;
    recovering_semi(s);

    Ok(AliasDeclStmt {
        ident,
        exprs,
        span: s.span(lo),
    })
}

/// In QASM3, it is hard to disambiguate between a quantum-gate-call missing modifiers
/// and expression statements. Consider the following expressions:
///  1. `Rx(2, 3) q0;`
///  2. `Rx(2, 3) + q0;`
///  3. `Rx(2, 3);`
///  4. `Rx;`
///
/// (1) is a quantum-gate-call, (2) is a binary operation, (3) is a function call, and
/// (4) is an identifer. We don't know for sure until we see the what is beyond the gate
/// name and its potential classical parameters.
///
/// Therefore, we parse the gate name and its potential parameters using the expr parser.
/// If the expr is a function call or an identifier and it is followed by qubit arguments,
/// we reinterpret the expression as a quantum gate.
///
/// Grammar:
///  `gateModifier* Identifier (LPAREN expressionList? RPAREN)? designator? gateOperandList  SEMICOLON
/// | gateModifier* GPHASE     (LPAREN expressionList? RPAREN)? designator? gateOperandList? SEMICOLON`.
fn parse_gate_call_stmt(s: &mut ParserContext) -> Result<StmtKind> {
    let lo = s.peek().span.lo;
    let modifiers = list_from_iter(many(s, gate_modifier)?);

    // As explained in the docstring, we parse the gate using the expr parser.
    let gate_or_expr = expr::expr(s)?;

    let duration = opt(s, designator)?;
    let qubits = list_from_iter(many(s, gate_operand)?);
    recovering_semi(s);

    // If didn't parse modifiers, a duration, nor qubit args then this is an expr, not a gate call.
    if modifiers.is_empty() && duration.is_none() && qubits.is_empty() {
        return Ok(StmtKind::ExprStmt(ExprStmt {
            span: s.span(lo),
            expr: gate_or_expr,
        }));
    }

    let (name, args) = match *gate_or_expr.kind {
        ExprKind::FunctionCall(FunctionCall { name, args, .. }) => (name, args),
        ExprKind::Ident(ident) => (Identifier::Ident(Box::new(ident)), Default::default()),
        _ => {
            return Err(Error::new(ErrorKind::ExpectedItem(
                TokenKind::Identifier,
                gate_or_expr.span,
            )))
        }
    };

    let quantum_stmt = QuantumStmt {
        span: s.span(lo),
        kind: crate::ast::QuantumStmtKind::Gate(GateCall {
            span: s.span(lo),
            modifiers,
            name,
            args,
            qubits,
            duration,
        }),
    };

    Ok(StmtKind::Quantum(quantum_stmt))
}

/// Grammar:
/// `(
///     INV
///     | POW LPAREN expression RPAREN
///     | (CTRL | NEGCTRL) (LPAREN expression RPAREN)?
/// ) AT`.
fn gate_modifier(s: &mut ParserContext) -> Result<QuantumGateModifier> {
    let lo = s.peek().span.lo;

    let kind = if opt(s, |s| token(s, TokenKind::Inv))?.is_some() {
        GateModifierKind::Inv
    } else if opt(s, |s| token(s, TokenKind::Pow))?.is_some() {
        token(s, TokenKind::Open(Delim::Paren))?;
        let expr = expr::expr(s)?;
        recovering_token(s, TokenKind::Close(Delim::Paren));
        GateModifierKind::Pow(expr)
    } else if opt(s, |s| token(s, TokenKind::Ctrl))?.is_some() {
        let expr = opt(s, |s| {
            token(s, TokenKind::Open(Delim::Paren))?;
            let expr = expr::expr(s)?;
            recovering_token(s, TokenKind::Close(Delim::Paren));
            Ok(expr)
        })?;
        GateModifierKind::Ctrl(expr)
    } else {
        token(s, TokenKind::NegCtrl)?;
        let expr = opt(s, |s| {
            token(s, TokenKind::Open(Delim::Paren))?;
            let expr = expr::expr(s)?;
            recovering_token(s, TokenKind::Close(Delim::Paren));
            Ok(expr)
        })?;
        GateModifierKind::NegCtrl(expr)
    };

    recovering_token(s, TokenKind::At);

    Ok(QuantumGateModifier {
        span: s.span(lo),
        kind,
    })
}
