// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::ops::ShlAssign;
use std::path::PathBuf;

use super::types::binop_requires_int_conversion_for_type;
use super::types::binop_requires_symmetric_int_conversion;
use super::types::is_complex_binop_supported;
use super::types::promote_to_uint_ty;
use super::types::requires_symmetric_conversion;
use super::types::try_promote_with_casting;
use super::types::types_equal_except_const;
use super::types::unary_op_can_be_applied_to_type;
use super::types::Type;
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use num_traits::Num;
use qsc_data_structures::span::Span;
use qsc_frontend::{compile::SourceMap, error::WithSource};

use super::symbols::{IOKind, Symbol, SymbolTable};

use crate::oqasm_helpers::safe_i64_to_f64;
use crate::parser::QasmSource;
use crate::semantic::types::can_cast_literal;
use crate::semantic::types::can_cast_literal_with_value_knowledge;
use crate::semantic::types::ArrayDimensions;

use crate::parser::ast as syntax;

use super::{
    ast::{Stmt, Version},
    SemanticErrorKind,
};

pub(super) struct Lowerer {
    /// The root QASM source to compile.
    pub source: QasmSource,
    /// The source map of QASM sources for error reporting.
    pub source_map: SourceMap,
    pub errors: Vec<WithSource<crate::Error>>,
    /// The file stack is used to track the current file for error reporting.
    /// When we include a file, we push the file path to the stack and pop it
    /// when we are done with the file.
    /// This allows us to report errors with the correct file path.
    pub file_stack: Vec<PathBuf>,
    pub symbols: SymbolTable,
    pub version: Option<Version>,
    pub stmts: Vec<Stmt>,
}
impl Lowerer {
    pub fn lower(mut self) -> crate::semantic::QasmSemanticParseResult {
        // Should we fail if we see a version in included files?
        let source = &self.source.clone();
        self.version = self.lower_version(source.program().version);

        self.lower_source(source);

        let program = super::ast::Program {
            version: self.version,
            statements: syntax::list_from_iter(self.stmts),
        };

        super::QasmSemanticParseResult {
            source: self.source,
            source_map: self.source_map,
            symbols: self.symbols,
            program,
            errors: self.errors,
        }
    }

    fn lower_version(&mut self, version: Option<syntax::Version>) -> Option<Version> {
        if let Some(version) = version {
            if version.major != 3 {
                self.push_semantic_error(SemanticErrorKind::UnsupportedVersion(
                    format!("{version}"),
                    version.span,
                ));
            } else if let Some(minor) = version.minor {
                if minor != 0 && minor != 1 {
                    self.push_semantic_error(SemanticErrorKind::UnsupportedVersion(
                        format!("{version}"),
                        version.span,
                    ));
                }
            }
            return Some(crate::semantic::ast::Version {
                span: version.span,
                major: version.major,
                minor: version.minor,
            });
        }
        None
    }

    /// Root recursive function for lowering the source.
    fn lower_source(&mut self, source: &QasmSource) {
        // we push the file path to the stack so we can track the current file
        // for reporting errors. This saves us from having to pass around
        // the current QasmSource value.
        self.file_stack.push(source.path());

        // we keep an iterator of the includes so we can match them with the
        // source includes. The include statements only have the path, but
        // we have already loaded all of source files in the
        // `source.includes()`
        let mut includes = source.includes().iter();

        for stmt in &source.program().statements {
            match &*stmt.kind {
                syntax::StmtKind::Include(include) => {
                    // if we are not in the root  we should not be able to include
                    // as this is a limitation of the QASM3 language
                    if !self.symbols.is_current_scope_global() {
                        let kind = SemanticErrorKind::IncludeNotInGlobalScope(
                            include.filename.to_string(),
                            include.span,
                        );
                        self.push_semantic_error(kind);
                        continue;
                    }

                    // special case for stdgates.inc
                    // it won't be in the includes list
                    if include.filename.to_lowercase() == "stdgates.inc" {
                        self.define_stdgates(include);
                        continue;
                    }

                    let include = includes.next().expect("missing include");
                    self.lower_source(include);
                }
                _ => {
                    if let Some(stmt) = self.lower_stmt(stmt) {
                        self.stmts.push(stmt);
                    }
                }
            }
        }

        // Finally we pop the file path from the stack so that we
        // can return to the previous file for error handling.
        self.file_stack.pop();
    }

    #[allow(clippy::too_many_lines)]
    fn lower_stmt(&mut self, stmt: &syntax::Stmt) -> Option<super::ast::Stmt> {
        let kind = match &*stmt.kind {
            syntax::StmtKind::Alias(stmt) => super::ast::StmtKind::Alias(self.lower_alias(stmt)?),
            syntax::StmtKind::Assign(stmt) => self.lower_assign(stmt)?,
            syntax::StmtKind::AssignOp(stmt) => {
                super::ast::StmtKind::AssignOp(self.lower_assign_op(stmt)?)
            }
            syntax::StmtKind::Barrier(stmt) => {
                super::ast::StmtKind::Barrier(self.lower_barrier(stmt)?)
            }
            syntax::StmtKind::Box(stmt) => super::ast::StmtKind::Box(self.lower_box(stmt)?),
            syntax::StmtKind::Break(stmt) => self.lower_break(stmt)?,
            syntax::StmtKind::Block(stmt) => {
                super::ast::StmtKind::Block(Box::new(self.lower_block(stmt)?))
            }
            syntax::StmtKind::Cal(stmt) => self.lower_calibration(stmt)?,
            syntax::StmtKind::CalibrationGrammar(stmt) => {
                super::ast::StmtKind::CalibrationGrammar(self.lower_calibration_grammar(stmt)?)
            }
            syntax::StmtKind::ClassicalDecl(stmt) => {
                super::ast::StmtKind::ClassicalDecl(self.lower_classical_decl(stmt)?)
            }
            syntax::StmtKind::ConstDecl(stmt) => {
                super::ast::StmtKind::ClassicalDecl(self.lower_const_decl(stmt)?)
            }
            syntax::StmtKind::Continue(stmt) => self.lower_continue_stmt(stmt)?,
            syntax::StmtKind::Def(stmt) => super::ast::StmtKind::Def(self.lower_def(stmt)?),
            syntax::StmtKind::DefCal(stmt) => {
                super::ast::StmtKind::DefCal(self.lower_def_cal(stmt)?)
            }
            syntax::StmtKind::Delay(stmt) => super::ast::StmtKind::Delay(self.lower_delay(stmt)?),
            syntax::StmtKind::Empty => {
                // we ignore empty statements
                None?
            }
            syntax::StmtKind::End(stmt) => super::ast::StmtKind::End(self.lower_end_stmt(stmt)?),
            syntax::StmtKind::ExprStmt(stmt) => {
                super::ast::StmtKind::ExprStmt(self.lower_expr_stmt(stmt)?)
            }
            syntax::StmtKind::ExternDecl(extern_decl) => {
                super::ast::StmtKind::ExternDecl(self.lower_extern(extern_decl)?)
            }
            syntax::StmtKind::For(stmt) => super::ast::StmtKind::For(self.lower_for_stmt(stmt)?),
            syntax::StmtKind::If(stmt) => super::ast::StmtKind::If(self.lower_if_stmt(stmt)?),
            syntax::StmtKind::GateCall(stmt) => {
                super::ast::StmtKind::GateCall(self.lower_gate_call(stmt)?)
            }
            syntax::StmtKind::GPhase(stmt) => {
                super::ast::StmtKind::GPhase(self.lower_gphase(stmt)?)
            }
            syntax::StmtKind::Include(stmt) => {
                super::ast::StmtKind::Include(self.lower_include(stmt)?)
            }
            syntax::StmtKind::IODeclaration(stmt) => {
                super::ast::StmtKind::IODeclaration(self.lower_io_decl(stmt)?)
            }
            syntax::StmtKind::Measure(stmt) => {
                super::ast::StmtKind::Measure(self.lower_measure(stmt)?)
            }
            syntax::StmtKind::Pragma(stmt) => {
                super::ast::StmtKind::Pragma(self.lower_pragma(stmt)?)
            }
            syntax::StmtKind::QuantumGateDefinition(stmt) => {
                super::ast::StmtKind::QuantumGateDefinition(self.lower_gate_def(stmt)?)
            }
            syntax::StmtKind::QuantumDecl(stmt) => {
                super::ast::StmtKind::QuantumDecl(self.lower_quantum_decl(stmt)?)
            }
            syntax::StmtKind::Reset(stmt) => super::ast::StmtKind::Reset(self.lower_reset(stmt)?),
            syntax::StmtKind::Return(stmt) => {
                super::ast::StmtKind::Return(self.lower_return(stmt)?)
            }
            syntax::StmtKind::Switch(stmt) => {
                super::ast::StmtKind::Switch(self.lower_switch(stmt)?)
            }
            syntax::StmtKind::WhileLoop(stmt) => {
                super::ast::StmtKind::WhileLoop(self.lower_while_loop(stmt)?)
            }
            syntax::StmtKind::Err => {
                self.push_semantic_error(SemanticErrorKind::UnexpectedParserError(
                    "Unexpected error".to_string(),
                    stmt.span,
                ));
                return None;
            }
        };
        let annotations = self.lower_annotations(&stmt.annotations, &stmt.kind);
        Some(super::ast::Stmt {
            span: stmt.span,
            annotations: syntax::list_from_iter(annotations),
            kind: Box::new(kind),
        })
    }

    /// Define the standard gates in the symbol table.
    /// The sdg, tdg, crx, cry, crz, and ch are defined
    /// as their bare gates, and modifiers are applied
    /// when calling them.
    fn define_stdgates(&mut self, include: &syntax::IncludeStmt) {
        fn gate_symbol(name: &str, cargs: u32, qargs: u32) -> Symbol {
            Symbol {
                name: name.to_string(),
                ty: Type::Gate(cargs, qargs),
                ..Default::default()
            }
        }
        let gates = vec![
            gate_symbol("X", 0, 1),
            gate_symbol("Y", 0, 1),
            gate_symbol("Z", 0, 1),
            gate_symbol("H", 0, 1),
            gate_symbol("S", 0, 1),
            gate_symbol("T", 0, 1),
            gate_symbol("Rx", 1, 1),
            gate_symbol("Rxx", 1, 2),
            gate_symbol("Ry", 1, 1),
            gate_symbol("Ryy", 1, 2),
            gate_symbol("Rz", 1, 1),
            gate_symbol("Rzz", 1, 2),
            gate_symbol("CNOT", 0, 2),
            gate_symbol("CY", 0, 2),
            gate_symbol("CZ", 0, 2),
            gate_symbol("I", 0, 1),
            gate_symbol("SWAP", 0, 2),
            gate_symbol("CCNOT", 0, 3),
        ];
        for gate in gates {
            let name = gate.name.clone();
            if self.symbols.insert_symbol(gate).is_err() {
                self.push_redefined_symbol_error(name.as_str(), include.span);
            }
        }
    }

    /// Pushes a missing symbol error with the given name
    /// This is a convenience method for pushing a `SemanticErrorKind::UndefinedSymbol` error.
    pub fn push_missing_symbol_error<S: AsRef<str>>(&mut self, name: S, span: Span) {
        let kind = SemanticErrorKind::UndefinedSymbol(name.as_ref().to_string(), span);
        self.push_semantic_error(kind);
    }

    /// Pushes a redefined symbol error with the given name and span.
    /// This is a convenience method for pushing a `SemanticErrorKind::RedefinedSymbol` error.
    pub fn push_redefined_symbol_error<S: AsRef<str>>(&mut self, name: S, span: Span) {
        let kind = SemanticErrorKind::RedefinedSymbol(name.as_ref().to_string(), span);
        self.push_semantic_error(kind);
    }

    /// Pushes an unsupported error with the supplied message.
    pub fn push_unsupported_error_message<S: AsRef<str>>(&mut self, message: S, span: Span) {
        let kind = SemanticErrorKind::NotSupported(message.as_ref().to_string(), span);
        self.push_semantic_error(kind);
    }

    /// Pushes an unimplemented error with the supplied message.
    pub fn push_unimplemented_error_message<S: AsRef<str>>(&mut self, message: S, span: Span) {
        let kind = SemanticErrorKind::Unimplemented(message.as_ref().to_string(), span);
        self.push_semantic_error(kind);
    }

    /// Pushes a semantic error with the given kind.
    pub fn push_semantic_error(&mut self, kind: SemanticErrorKind) {
        let kind = crate::ErrorKind::Semantic(crate::semantic::Error(kind));
        let error = self.create_err(kind);
        self.errors.push(error);
    }

    /// Creates an error from the given kind with the current source mapping.
    fn create_err(&self, kind: crate::ErrorKind) -> WithSource<crate::Error> {
        let error = crate::Error(kind);
        let path = self.file_stack.last().map_or("<compiler>", |p| {
            p.to_str().expect("expected source mapping to exist.")
        });
        let source = self.source_map.find_by_name(path);
        let offset = source.map_or(0, |x| x.offset);
        let offset_error = error.with_offset(offset);
        WithSource::from_map(&self.source_map, offset_error)
    }

    fn lower_alias(&mut self, alias: &syntax::AliasDeclStmt) -> Option<super::ast::AliasDeclStmt> {
        let name = get_identifier_name(&alias.ident);
        // alias statements do their types backwards, you read the right side
        // and assign it to the left side.
        // the types of the rhs should be in the symbol table.
        let rhs = alias
            .exprs
            .iter()
            .filter_map(|expr| self.lower_expr(expr))
            .collect::<Vec<_>>();
        let first = rhs.first().expect("missing rhs");

        let symbol = Symbol {
            name: name.to_string(),
            ty: first.ty.clone(),
            qsharp_ty: self.convert_semantic_type_to_qsharp_type(&first.ty, alias.ident.span())?,
            span: alias.ident.span(),
            io_kind: IOKind::Default,
        };

        let symbol_id = self.symbols.insert_symbol(symbol);
        if symbol_id.is_err() {
            self.push_redefined_symbol_error(name, alias.ident.span());
        }

        if rhs.iter().any(|expr| expr.ty != first.ty) {
            let tys = rhs
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            let kind = SemanticErrorKind::InconsistentTypesInAlias(tys, alias.span);
            self.push_semantic_error(kind);
            return None;
        }

        let symbol_id = symbol_id.ok()?;

        if rhs.len() != alias.exprs.len() {
            // we failed
            return None;
        }
        Some(super::ast::AliasDeclStmt {
            span: alias.span,
            symbol_id,
            exprs: syntax::list_from_iter(rhs),
        })
    }

    fn lower_assign(&mut self, stmt: &syntax::AssignStmt) -> Option<super::ast::StmtKind> {
        if stmt.lhs.indices.is_empty() {
            Some(super::ast::StmtKind::Assign(
                self.lower_simple_assign_expr(&stmt.lhs.name, &stmt.rhs, stmt.span)?,
            ))
        } else {
            Some(super::ast::StmtKind::IndexedAssign(
                self.lower_indexed_assign_expr(&stmt.lhs, &stmt.rhs, stmt.span)?,
            ))
        }
    }

    fn lower_simple_assign_expr(
        &mut self,
        ident: &syntax::Ident,
        rhs: &syntax::Expr,
        span: Span,
    ) -> Option<super::ast::AssignStmt> {
        let (lhs_symbol_id, lhs_symbol) = self.symbols.get_symbol_by_name(&ident.name)?;
        let ty = lhs_symbol.ty.clone();
        if ty.is_const() {
            let kind =
                SemanticErrorKind::CannotUpdateConstVariable(ident.name.to_string(), ident.span);
            self.push_semantic_error(kind);
            // usually we'd return None here, but we'll continue to compile the rhs
            // looking for more errors. There is nothing in this type of error that
            // would prevent us from compiling the rhs.
        }

        let rhs = self.lower_expr_with_target_type(Some(rhs), &ty, span)?;
        Some(super::ast::AssignStmt {
            symbold_id: lhs_symbol_id,
            rhs,
            span,
        })
    }

    fn lower_indexed_assign_expr(
        &mut self,
        index_expr: &syntax::IndexedIdent,
        rhs: &syntax::Expr,
        span: Span,
    ) -> Option<super::ast::IndexedAssignStmt> {
        let ident = index_expr.name.clone();
        let (lhs_symbol_id, lhs_symbol) = self.symbols.get_symbol_by_name(&ident.name)?;
        let ty = lhs_symbol.ty.clone();
        if ty.is_const() {
            let kind =
                SemanticErrorKind::CannotUpdateConstVariable(ident.name.to_string(), ident.span);
            self.push_semantic_error(kind);
            // usually we'd return None here, but we'll continue to compile the rhs
            // looking for more errors. There is nothing in this type of error that
            // would prevent us from compiling the rhs.
            return None;
        }

        let lhs = self.lower_indexed_ident_expr(index_expr);
        let rhs = self.lower_expr_with_target_type(Some(rhs), &ty, span);
        let (lhs, rhs) = (lhs?, rhs?);

        Some(super::ast::IndexedAssignStmt {
            symbold_id: lhs_symbol_id,
            lhs,
            rhs,
            span,
        })
    }

    fn lower_assign_op(&mut self, stmt: &syntax::AssignOpStmt) -> Option<super::ast::AssignOpStmt> {
        let op = stmt.op;
        let lhs = &stmt.lhs;
        let rhs = &stmt.rhs;

        let ident = lhs.name.clone();

        let (lhs_symbol_id, lhs_symbol) = self.symbols.get_symbol_by_name(&ident.name)?;
        let ty = lhs_symbol.ty.clone();
        if ty.is_const() {
            let kind =
                SemanticErrorKind::CannotUpdateConstVariable(ident.name.to_string(), ident.span);
            self.push_semantic_error(kind);
            // usually we'd return None here, but we'll continue to compile the rhs
            // looking for more errors. There is nothing in this type of error that
            // would prevent us from compiling the rhs.
            return None;
        }

        let lhs = self.lower_indexed_ident_expr(lhs);

        let rhs = self.lower_expr(rhs);
        let (lhs, rhs) = (lhs?, rhs?);
        let rhs = self.lower_binary_op_expr(op, lhs.clone(), rhs, stmt.span)?;
        let rhs = self.cast_expr_to_type(&ty, &rhs, stmt.span)?;
        Some(super::ast::AssignOpStmt {
            span: stmt.span,
            symbold_id: lhs_symbol_id,
            lhs,
            rhs,
        })
    }

    fn lower_expr(&mut self, expr: &syntax::Expr) -> Option<super::ast::Expr> {
        match &*expr.kind {
            syntax::ExprKind::BinaryOp(bin_op_expr) => {
                let lhs = self.lower_expr(&bin_op_expr.lhs);
                let rhs = self.lower_expr(&bin_op_expr.rhs);
                // once we've compiled both sides, we can check if they failed
                // and return None if they did so that the error is propagated
                let (lhs, rhs) = (lhs?, rhs?);
                self.lower_binary_op_expr(bin_op_expr.op, lhs, rhs, expr.span)
            }
            syntax::ExprKind::Cast(_) => {
                self.push_unimplemented_error_message("cast expr", expr.span);
                None
            }
            syntax::ExprKind::Err => {
                unreachable!("Err expr should not be lowered");
            }
            syntax::ExprKind::FunctionCall(_) => {
                self.push_unimplemented_error_message("function call expr", expr.span);
                None
            }
            syntax::ExprKind::Ident(ident) => self.lower_ident_expr(ident),
            syntax::ExprKind::IndexExpr(expr) => self.lower_index_expr(expr),

            syntax::ExprKind::Lit(lit) => self.lower_lit_expr(lit),

            syntax::ExprKind::Paren(expr) => self.lower_paren_expr(expr),
            syntax::ExprKind::UnaryOp(expr) => self.lower_unary_op_expr(expr),
        }
    }

    fn lower_ident_expr(&mut self, ident: &syntax::Ident) -> Option<super::ast::Expr> {
        let name = ident.name.clone();
        let Some((symbol_id, symbol)) = self.symbols.get_symbol_by_name(&name) else {
            self.push_missing_symbol_error(&name, ident.span);
            return None;
        };

        let kind = super::ast::ExprKind::Ident(symbol_id);
        Some(super::ast::Expr {
            span: ident.span,
            kind: Box::new(kind),
            ty: symbol.ty.clone(),
        })
    }

    fn lower_lit_expr(&mut self, expr: &syntax::Lit) -> Option<super::ast::Expr> {
        let (kind, ty) = match &expr.kind {
            syntax::LiteralKind::BigInt(value) => {
                // todo: this case is only valid when there is an integer literal
                // that requires more than 64 bits to represent. We should probably
                // introduce a new type for this as openqasm promotion rules don't
                // cover this case as far as I know.
                (super::ast::LiteralKind::BigInt(value.clone()), Type::Err)
            }
            syntax::LiteralKind::Bitstring(value, size) => (
                super::ast::LiteralKind::Bitstring(value.clone(), *size),
                Type::BitArray(super::types::ArrayDimensions::One(*size), true),
            ),
            syntax::LiteralKind::Bool(value) => {
                (super::ast::LiteralKind::Bool(*value), Type::Bool(true))
            }
            syntax::LiteralKind::Int(value) => {
                (super::ast::LiteralKind::Int(*value), Type::Int(None, true))
            }
            syntax::LiteralKind::Float(value) => (
                super::ast::LiteralKind::Float(*value),
                Type::Float(None, true),
            ),
            syntax::LiteralKind::Imaginary(value) => (
                super::ast::LiteralKind::Complex(0.0, *value),
                Type::Complex(None, true),
            ),
            syntax::LiteralKind::String(_) => {
                self.push_unsupported_error_message("String literals", expr.span);
                return None;
            }
            syntax::LiteralKind::Duration(_, _) => {
                self.push_unsupported_error_message("Duration literals", expr.span);
                return None;
            }
            syntax::LiteralKind::Array(exprs) => {
                // array literals are only valid in classical decals (const and mut)
                // and we have to know the expected type of the array in order to lower it
                // So we can't lower array literals in general.
                self.push_semantic_error(SemanticErrorKind::ArrayLiteralInNonClassicalDecl(
                    expr.span,
                ));
                // place holder for now, this code will need to move to the correct place when we
                // add support for classical decls
                let texprs = exprs
                    .iter()
                    .filter_map(|expr| self.lower_expr(expr))
                    .collect::<Vec<_>>();
                if texprs.len() != exprs.len() {
                    // we failed to lower all the entries and an error was pushed
                    return None;
                }

                (
                    super::ast::LiteralKind::Array(syntax::list_from_iter(texprs)),
                    Type::Err,
                )
            }
        };
        Some(super::ast::Expr {
            span: expr.span,
            kind: Box::new(super::ast::ExprKind::Lit(kind)),
            ty,
        })
    }

    fn lower_paren_expr(&mut self, expr: &syntax::Expr) -> Option<super::ast::Expr> {
        let expr = self.lower_expr(expr)?;
        let span = expr.span;
        let ty = expr.ty.clone();
        let kind = super::ast::ExprKind::Paren(expr);
        Some(super::ast::Expr {
            span,
            kind: Box::new(kind),
            ty,
        })
    }

    fn lower_unary_op_expr(&mut self, expr: &syntax::UnaryOpExpr) -> Option<super::ast::Expr> {
        match expr.op {
            syntax::UnaryOp::Neg => {
                if let syntax::ExprKind::Lit(lit) = expr.expr.kind.as_ref() {
                    self.lower_negated_literal_as_ty(lit, None, expr.expr.span)
                } else {
                    let expr = self.lower_expr(&expr.expr)?;
                    let ty = expr.ty.clone();
                    if unary_op_can_be_applied_to_type(syntax::UnaryOp::Neg, &ty) {
                        let span = expr.span;
                        let unary = super::ast::UnaryOpExpr {
                            op: super::ast::UnaryOp::Neg,
                            expr,
                        };
                        Some(super::ast::Expr {
                            span,
                            kind: Box::new(super::ast::ExprKind::UnaryOp(unary)),
                            ty,
                        })
                    } else {
                        let kind = SemanticErrorKind::TypeDoesNotSupportedUnaryNegation(
                            expr.ty.to_string(),
                            expr.span,
                        );
                        self.push_semantic_error(kind);
                        None
                    }
                }
            }
            syntax::UnaryOp::NotB => {
                let expr = self.lower_expr(&expr.expr)?;
                let ty = expr.ty.clone();
                if unary_op_can_be_applied_to_type(syntax::UnaryOp::NotB, &ty) {
                    let span = expr.span;
                    let unary = super::ast::UnaryOpExpr {
                        op: super::ast::UnaryOp::NotB,
                        expr,
                    };
                    Some(super::ast::Expr {
                        span,
                        kind: Box::new(super::ast::ExprKind::UnaryOp(unary)),
                        ty,
                    })
                } else {
                    let kind = SemanticErrorKind::TypeDoesNotSupportedUnaryNegation(
                        expr.ty.to_string(),
                        expr.span,
                    );
                    self.push_semantic_error(kind);
                    None
                }
            }
            syntax::UnaryOp::NotL => {
                // this is the  only unary operator that tries to coerce the type
                // I can't find it in the spec, but when looking at existing code
                // it seems that the ! operator coerces to a bool if possible
                let target_ty = Type::Bool(false);
                let expr =
                    self.lower_expr_with_target_type(Some(&expr.expr), &target_ty, expr.expr.span)?;

                let ty = expr.ty.clone();

                Some(super::ast::Expr {
                    span: expr.span,
                    kind: Box::new(super::ast::ExprKind::UnaryOp(super::ast::UnaryOpExpr {
                        op: super::ast::UnaryOp::NotL,
                        expr,
                    })),
                    ty,
                })
            }
        }
    }

    fn lower_annotations(
        &mut self,
        annotations: &[Box<syntax::Annotation>],
        kind: &syntax::StmtKind,
    ) -> Vec<super::ast::Annotation> {
        annotations
            .iter()
            .map(|annotation| self.lower_annotation(annotation, kind))
            .collect::<Vec<_>>()
    }

    fn lower_annotation(
        &mut self,
        annotation: &syntax::Annotation,
        kind: &syntax::StmtKind,
    ) -> super::ast::Annotation {
        if !matches!(
            annotation.identifier.to_string().as_str(),
            "SimulatableIntrinsic" | "Config"
        ) {
            self.push_unsupported_error_message(
                format!("Annotation {}.", annotation.identifier),
                annotation.span,
            );
        }

        if let syntax::StmtKind::GateCall(_) = &kind {
            self.push_unsupported_error_message(
                format!(
                    "Annotation {} is only allowed on gate definitions.",
                    annotation.identifier
                ),
                annotation.span,
            );
        }

        super::ast::Annotation {
            span: annotation.span,
            identifier: annotation.identifier.clone(),
            value: annotation.value.as_ref().map(Clone::clone),
        }
    }

    fn convert_semantic_type_to_qsharp_type(
        &mut self,
        ty: &super::types::Type,
        span: Span,
    ) -> Option<crate::types::Type> {
        let is_const = ty.is_const();
        match ty {
            Type::Bit(_) => Some(crate::types::Type::Result(is_const)),
            Type::Qubit => Some(crate::types::Type::Qubit),
            Type::HardwareQubit => {
                let message = "HardwareQubit to Q# type";
                self.push_unsupported_error_message(message, span);
                None
            }
            Type::Int(width, _) | Type::UInt(width, _) => {
                if let Some(width) = width {
                    if *width > 64 {
                        Some(crate::types::Type::BigInt(is_const))
                    } else {
                        Some(crate::types::Type::Int(is_const))
                    }
                } else {
                    Some(crate::types::Type::Int(is_const))
                }
            }
            Type::Float(_, _) | Type::Angle(_, _) => Some(crate::types::Type::Double(is_const)),
            Type::Complex(_, _) => Some(crate::types::Type::Complex(is_const)),
            Type::Bool(_) => Some(crate::types::Type::Bool(is_const)),
            Type::Duration(_) => {
                self.push_unsupported_error_message("Duration type values", span);
                None
            }
            Type::Stretch(_) => {
                self.push_unsupported_error_message("Stretch type values", span);
                None
            }
            Type::BitArray(dims, _) => Some(crate::types::Type::ResultArray(dims.into(), is_const)),
            Type::QubitArray(dims) => Some(crate::types::Type::QubitArray(dims.into())),
            Type::IntArray(size, dims) | Type::UIntArray(size, dims) => {
                if let Some(size) = size {
                    if *size > 64 {
                        Some(crate::types::Type::BigIntArray(dims.into(), is_const))
                    } else {
                        Some(crate::types::Type::IntArray(dims.into(), is_const))
                    }
                } else {
                    Some(crate::types::Type::IntArray(dims.into(), is_const))
                }
            }
            Type::FloatArray(_, dims) => Some(crate::types::Type::DoubleArray(dims.into())),
            Type::AngleArray(_, _) => todo!("AngleArray to Q# type"),
            Type::ComplexArray(_, _) => todo!("ComplexArray to Q# type"),
            Type::BoolArray(dims) => Some(crate::types::Type::BoolArray(dims.into(), is_const)),
            Type::Gate(cargs, qargs) => Some(crate::types::Type::Callable(
                crate::types::CallableKind::Operation,
                *cargs,
                *qargs,
            )),
            Type::Range => Some(crate::types::Type::Range),
            Type::Set => todo!("Set to Q# type"),
            Type::Void => Some(crate::types::Type::Tuple(vec![])),
            _ => {
                let msg = format!("Converting {ty:?} to Q# type");
                self.push_unimplemented_error_message(msg, span);
                None
            }
        }
    }

    fn lower_barrier(&mut self, stmt: &syntax::BarrierStmt) -> Option<super::ast::BarrierStmt> {
        self.push_unimplemented_error_message("barrier stmt", stmt.span);
        None
    }

    fn lower_box(&mut self, stmt: &syntax::BoxStmt) -> Option<super::ast::BoxStmt> {
        self.push_unimplemented_error_message("box stmt", stmt.span);
        None
    }

    fn lower_break(&mut self, stmt: &syntax::BreakStmt) -> Option<super::ast::StmtKind> {
        self.push_unimplemented_error_message("break stmt", stmt.span);
        None
    }

    fn lower_block(&mut self, stmt: &syntax::Block) -> Option<super::ast::Block> {
        self.push_unimplemented_error_message("block stmt", stmt.span);
        None
    }

    fn lower_calibration(
        &mut self,
        stmt: &syntax::CalibrationStmt,
    ) -> Option<super::ast::StmtKind> {
        self.push_unimplemented_error_message("calibration stmt", stmt.span);
        None
    }

    fn lower_calibration_grammar(
        &mut self,
        stmt: &syntax::CalibrationGrammarStmt,
    ) -> Option<super::ast::CalibrationGrammarStmt> {
        self.push_unimplemented_error_message("calibration stmt", stmt.span);
        None
    }

    fn lower_classical_decl(
        &mut self,
        stmt: &syntax::ClassicalDeclarationStmt,
    ) -> Option<super::ast::ClassicalDeclarationStmt> {
        let is_const = false; // const decls are handled separately
        let ty = self.get_semantic_type_from_tydef(&stmt.ty, is_const)?;

        let init_expr = stmt.init_expr.as_deref();
        let ty_span = stmt.ty.span();
        let stmt_span = stmt.span;
        let name = stmt.identifier.name.clone();
        let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty.clone(), ty_span)?;
        let symbol = Symbol {
            name: name.to_string(),
            ty: ty.clone(),
            qsharp_ty,
            span: stmt.identifier.span,
            io_kind: IOKind::Default,
        };

        // process the symbol and init_expr gathering any errors
        let init_expr = match init_expr {
            Some(expr) => match expr {
                syntax::ValueExpression::Expr(expr) => self
                    .lower_expr_with_target_type(Some(expr), &ty, stmt_span)
                    .map(super::ast::ValueExpression::Expr),
                syntax::ValueExpression::Measurement(measure_expr) => self
                    .lower_measure_expr_with_target_type(measure_expr, &ty, stmt_span)
                    .map(super::ast::ValueExpression::Measurement),
            },
            None => self
                .lower_expr_with_target_type(None, &ty, stmt_span)
                .map(super::ast::ValueExpression::Expr),
        };

        let Ok(symbol_id) = self.symbols.insert_symbol(symbol) else {
            self.push_redefined_symbol_error(&name, stmt.identifier.span);
            return None;
        };

        // even if init_expr was None, Q# semantics require that we have an initial value
        // for classical declarations. So if None is returned we hit an error with the expression.
        let init_expr = init_expr?;

        Some(super::ast::ClassicalDeclarationStmt {
            span: stmt_span,
            ty_span,
            symbol_id,
            init_expr: Box::new(init_expr),
        })
    }

    fn lower_const_decl(
        &mut self,
        stmt: &syntax::ConstantDeclStmt,
    ) -> Option<super::ast::ClassicalDeclarationStmt> {
        let is_const = true;
        let ty = self.get_semantic_type_from_tydef(&stmt.ty, is_const)?;
        let ty_span = stmt.ty.span();
        let name = stmt.identifier.name.clone();
        let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty.clone(), stmt.ty.span())?;
        let symbol = Symbol {
            name: name.to_string(),
            ty: ty.clone(),
            qsharp_ty,
            span: stmt.identifier.span,
            io_kind: IOKind::Default,
        };

        // process the symbol and init_expr gathering any errors
        let init_expr = self.lower_expr_with_target_type(Some(&stmt.init_expr), &ty, stmt.span);

        let Ok(symbol_id) = self.symbols.insert_symbol(symbol) else {
            self.push_redefined_symbol_error(&name, stmt.identifier.span);
            return None;
        };

        // even if init_expr was None, Q# semantics require that we have an initial value
        // for classical declarations. So if None is returned we hit an error with the expression.
        let init_expr = init_expr?;

        Some(super::ast::ClassicalDeclarationStmt {
            span: stmt.span,
            ty_span,
            symbol_id,
            init_expr: Box::new(super::ast::ValueExpression::Expr(init_expr)),
        })
    }

    fn lower_continue_stmt(&mut self, stmt: &syntax::ContinueStmt) -> Option<super::ast::StmtKind> {
        self.push_unimplemented_error_message("continue stmt", stmt.span);
        None
    }

    fn lower_def(&mut self, stmt: &syntax::DefStmt) -> Option<super::ast::DefStmt> {
        self.push_unimplemented_error_message("def stmt", stmt.span);
        None
    }

    fn lower_def_cal(&mut self, stmt: &syntax::DefCalStmt) -> Option<super::ast::DefCalStmt> {
        self.push_unimplemented_error_message("def cal stmt", stmt.span);
        None
    }

    fn lower_delay(&mut self, stmt: &syntax::DelayStmt) -> Option<super::ast::DelayStmt> {
        self.push_unimplemented_error_message("delay stmt", stmt.span);
        None
    }

    fn lower_end_stmt(&mut self, stmt: &syntax::EndStmt) -> Option<super::ast::EndStmt> {
        self.push_unimplemented_error_message("end stmt", stmt.span);
        None
    }

    fn lower_expr_stmt(&mut self, stmt: &syntax::ExprStmt) -> Option<super::ast::ExprStmt> {
        let expr = self.lower_expr(&stmt.expr)?;
        Some(super::ast::ExprStmt {
            span: stmt.span,
            expr,
        })
    }

    fn lower_extern(&mut self, stmt: &syntax::ExternDecl) -> Option<super::ast::ExternDecl> {
        self.push_unimplemented_error_message("extern stmt", stmt.span);
        None
    }

    fn lower_for_stmt(&mut self, stmt: &syntax::ForStmt) -> Option<super::ast::ForStmt> {
        self.push_unimplemented_error_message("for stmt", stmt.span);
        None
    }

    fn lower_if_stmt(&mut self, stmt: &syntax::IfStmt) -> Option<super::ast::IfStmt> {
        self.push_unimplemented_error_message("if stmt", stmt.span);
        None
    }

    fn lower_gate_call(&mut self, stmt: &syntax::GateCall) -> Option<super::ast::GateCall> {
        self.push_unimplemented_error_message("gate call stmt", stmt.span);
        None
    }

    fn lower_gphase(&mut self, stmt: &syntax::GPhase) -> Option<super::ast::GPhase> {
        self.push_unimplemented_error_message("gphase stmt", stmt.span);
        None
    }

    /// This function is always a indication of a error. Either the
    /// program is declaring include in a non-global scope or the
    /// include is not handled in `self.lower_source` properly.
    fn lower_include(&mut self, stmt: &syntax::IncludeStmt) -> Option<super::ast::IncludeStmt> {
        // if we are not in the root we should not be able to include
        if !self.symbols.is_current_scope_global() {
            let name = stmt.filename.to_string();
            let kind = SemanticErrorKind::IncludeNotInGlobalScope(name, stmt.span);
            self.push_semantic_error(kind);
            return None;
        }
        // if we are at the root and we have an include, we should have
        // already handled it and we are in an invalid state
        panic!("Include should have been handled in lower_source")
    }

    fn lower_io_decl(&mut self, stmt: &syntax::IODeclaration) -> Option<super::ast::IODeclaration> {
        let is_const = false;
        let ty = self.get_semantic_type_from_tydef(&stmt.ty, is_const)?;
        let io_kind = stmt.io_identifier.into();

        let ty_span = stmt.ty.span();
        let stmt_span = stmt.span;
        let name = stmt.ident.name.clone();
        let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty.clone(), ty_span)?;
        let symbol = Symbol {
            name: name.to_string(),
            ty: ty.clone(),
            qsharp_ty,
            span: stmt.ident.span,
            io_kind,
        };

        let Ok(symbol_id) = self.symbols.insert_symbol(symbol) else {
            self.push_redefined_symbol_error(&name, stmt.ident.span);
            return None;
        };

        // if we have output, we need to assign a default value to declare the variable
        // if we have input, we can keep return none as we would promote the variable
        // to a parameter in the function signature once we generate the function
        if io_kind == IOKind::Output {
            let init_expr = self.get_default_value(&ty, stmt_span)?;
            Some(super::ast::IODeclaration {
                span: stmt_span,
                symbol_id,
                init_expr: Box::new(init_expr),
            })
        } else {
            None
        }
    }

    fn lower_measure(&mut self, stmt: &syntax::MeasureStmt) -> Option<super::ast::MeasureStmt> {
        self.push_unimplemented_error_message("measure stmt", stmt.span);
        None
    }

    fn lower_pragma(&mut self, stmt: &syntax::Pragma) -> Option<super::ast::Pragma> {
        self.push_unimplemented_error_message("pragma stmt", stmt.span);
        None
    }

    fn lower_gate_def(
        &mut self,
        stmt: &syntax::QuantumGateDefinition,
    ) -> Option<super::ast::QuantumGateDefinition> {
        self.push_unimplemented_error_message("gate def stmt", stmt.span);
        None
    }

    fn lower_quantum_decl(
        &mut self,
        stmt: &syntax::QubitDeclaration,
    ) -> Option<super::ast::QubitDeclaration> {
        self.push_unimplemented_error_message("qubit decl stmt", stmt.span);
        None
    }

    fn lower_reset(&mut self, stmt: &syntax::ResetStmt) -> Option<super::ast::ResetStmt> {
        self.push_unimplemented_error_message("reset stmt", stmt.span);
        None
    }

    fn lower_return(&mut self, stmt: &syntax::ReturnStmt) -> Option<super::ast::ReturnStmt> {
        self.push_unimplemented_error_message("return stmt", stmt.span);
        None
    }

    fn lower_switch(&mut self, stmt: &syntax::SwitchStmt) -> Option<super::ast::SwitchStmt> {
        self.push_unimplemented_error_message("switch stmt", stmt.span);
        None
    }

    fn lower_while_loop(&mut self, stmt: &syntax::WhileLoop) -> Option<super::ast::WhileLoop> {
        self.push_unimplemented_error_message("while loop stmt", stmt.span);
        None
    }

    fn get_semantic_type_from_tydef(
        &mut self,
        scalar_ty: &syntax::TypeDef,
        is_const: bool,
    ) -> Option<crate::semantic::types::Type> {
        match scalar_ty {
            syntax::TypeDef::Scalar(scalar_type) => {
                self.get_semantic_type_from_scalar_ty(scalar_type, is_const)
            }
            syntax::TypeDef::Array(array_type) => {
                self.get_semantic_type_from_array_ty(array_type, is_const)
            }
            syntax::TypeDef::ArrayReference(array_reference_type) => {
                self.get_semantic_type_from_array_reference_ty(array_reference_type, is_const)
            }
        }
    }

    /// designators are positive integer literals when used
    /// in the context of a type definition.
    fn get_size_designator_from_expr(&mut self, expr: &syntax::Expr) -> Option<u32> {
        if let syntax::ExprKind::Lit(lit) = expr.kind.as_ref() {
            if let syntax::LiteralKind::Int(value) = lit.kind {
                if value > 0 {
                    if let Ok(value) = u32::try_from(value) {
                        Some(value)
                    } else {
                        self.push_semantic_error(SemanticErrorKind::DesignatorTooLarge(lit.span));
                        None
                    }
                } else {
                    self.push_semantic_error(
                        SemanticErrorKind::DesignatorMustBePositiveIntLiteral(lit.span),
                    );
                    None
                }
            } else {
                self.push_semantic_error(SemanticErrorKind::DesignatorMustBePositiveIntLiteral(
                    lit.span,
                ));
                None
            }
        } else {
            self.push_semantic_error(SemanticErrorKind::DesignatorMustBePositiveIntLiteral(
                expr.span,
            ));
            None
        }
    }

    fn get_semantic_type_from_scalar_ty(
        &mut self,
        scalar_ty: &syntax::ScalarType,
        is_const: bool,
    ) -> Option<crate::semantic::types::Type> {
        match &scalar_ty.kind {
            syntax::ScalarTypeKind::Bit(bit_type) => match &bit_type.size {
                Some(size) => Some(crate::semantic::types::Type::BitArray(
                    super::types::ArrayDimensions::One(self.get_size_designator_from_expr(size)?),
                    is_const,
                )),
                None => Some(crate::semantic::types::Type::Bit(is_const)),
            },
            syntax::ScalarTypeKind::Int(int_type) => match &int_type.size {
                Some(size) => Some(crate::semantic::types::Type::Int(
                    Some(self.get_size_designator_from_expr(size)?),
                    is_const,
                )),
                None => Some(crate::semantic::types::Type::Int(None, is_const)),
            },
            syntax::ScalarTypeKind::UInt(uint_type) => match &uint_type.size {
                Some(size) => Some(crate::semantic::types::Type::UInt(
                    Some(self.get_size_designator_from_expr(size)?),
                    is_const,
                )),
                None => Some(crate::semantic::types::Type::UInt(None, is_const)),
            },
            syntax::ScalarTypeKind::Float(float_type) => match &float_type.size {
                Some(size) => Some(crate::semantic::types::Type::Float(
                    Some(self.get_size_designator_from_expr(size)?),
                    is_const,
                )),
                None => Some(crate::semantic::types::Type::Float(None, is_const)),
            },
            syntax::ScalarTypeKind::Complex(complex_type) => match &complex_type.base_size {
                Some(float_type) => match &float_type.size {
                    Some(size) => Some(crate::semantic::types::Type::Complex(
                        Some(self.get_size_designator_from_expr(size)?),
                        is_const,
                    )),
                    None => Some(crate::semantic::types::Type::Complex(None, is_const)),
                },
                None => Some(crate::semantic::types::Type::Complex(None, is_const)),
            },
            syntax::ScalarTypeKind::Angle(angle_type) => match &angle_type.size {
                Some(size) => Some(crate::semantic::types::Type::Angle(
                    Some(self.get_size_designator_from_expr(size)?),
                    is_const,
                )),
                None => Some(crate::semantic::types::Type::Angle(None, is_const)),
            },
            syntax::ScalarTypeKind::BoolType => Some(crate::semantic::types::Type::Bool(is_const)),
            syntax::ScalarTypeKind::Duration => {
                Some(crate::semantic::types::Type::Duration(is_const))
            }
            syntax::ScalarTypeKind::Stretch => {
                Some(crate::semantic::types::Type::Stretch(is_const))
            }
            syntax::ScalarTypeKind::Err => Some(crate::semantic::types::Type::Err),
        }
    }

    fn get_semantic_type_from_array_ty(
        &mut self,
        array_ty: &syntax::ArrayType,
        _is_const: bool,
    ) -> Option<crate::semantic::types::Type> {
        self.push_unimplemented_error_message("semantic type from array type", array_ty.span);
        None
    }

    fn get_semantic_type_from_array_reference_ty(
        &mut self,
        array_ref_ty: &syntax::ArrayReferenceType,
        _is_const: bool,
    ) -> Option<crate::semantic::types::Type> {
        self.push_unimplemented_error_message(
            "semantic type from array refence type",
            array_ref_ty.span,
        );
        None
    }
    fn lower_expr_with_target_type(
        &mut self,
        expr: Option<&syntax::Expr>,
        ty: &Type,
        span: Span,
    ) -> Option<super::ast::Expr> {
        let Some(expr) = expr else {
            // In OpenQASM, classical variables may be uninitialized, but in Q#,
            // they must be initialized. We will use the default value for the type
            // to initialize the variable.
            return self.get_default_value(ty, span);
        };
        let rhs = self.lower_expr(expr)?;
        let rhs_ty = rhs.ty.clone();
        // if we have an exact type match, we can use the rhs as is
        if types_equal_except_const(ty, &rhs_ty) {
            return Some(rhs);
        }

        // if the rhs is a literal, we can try to cast it to the target type
        // if they share the same base type.
        if let super::ast::ExprKind::Lit(lit) = &*rhs.kind {
            // if the rhs is a literal, we can try to coerce it to the lhs type
            // we can do better than just types given we have a literal value
            if can_cast_literal(ty, &rhs_ty) || can_cast_literal_with_value_knowledge(ty, lit) {
                return self.coerce_literal_expr_to_type(ty, &rhs, lit);
            }
            // if we can't cast the literal, we can't proceed
            // create a semantic error and return
            let kind = SemanticErrorKind::CannotAssignToType(
                format!("{:?}", rhs.ty),
                format!("{ty:?}"),
                span,
            );
            self.push_semantic_error(kind);
            return None;
        }
        // the lhs has a type, but the rhs may be of a different type with
        // implicit and explicit conversions. We need to cast the rhs to the
        // lhs type, but if that cast fails, we will have already pushed an error
        // and we can't proceed
        self.cast_expr_to_type(ty, &rhs, span)
    }

    fn lower_measure_expr_with_target_type(
        &mut self,
        _expr: &syntax::MeasureExpr,
        _ty: &Type,
        span: Span,
    ) -> Option<super::ast::MeasureExpr> {
        self.push_unimplemented_error_message("measure expr with target type", span);
        None
    }

    fn get_default_value(&mut self, ty: &Type, span: Span) -> Option<super::ast::Expr> {
        use super::ast::Expr;
        use super::ast::ExprKind;
        use super::ast::LiteralKind;
        let from_lit_kind = |kind| -> Expr {
            Expr {
                span: Span::default(),
                kind: Box::new(ExprKind::Lit(kind)),
                ty: ty.as_const(),
            }
        };
        match ty {
            Type::Bit(_) | Type::Int(_, _) | Type::UInt(_, _) => {
                Some(from_lit_kind(LiteralKind::Int(0)))
            }
            Type::Bool(_) => Some(from_lit_kind(LiteralKind::Bool(false))),
            Type::Angle(_, _) | Type::Float(_, _) => Some(from_lit_kind(LiteralKind::Float(0.0))),
            Type::Complex(_, _) => Some(from_lit_kind(LiteralKind::Complex(0.0, 0.0))),
            Type::Stretch(_) => {
                let message = "Stretch default values";
                self.push_unsupported_error_message(message, span);
                None
            }
            Type::Qubit => {
                let message = "Qubit default values";
                self.push_unsupported_error_message(message, span);
                None
            }
            Type::HardwareQubit => {
                let message = "HardwareQubit default values";
                self.push_unsupported_error_message(message, span);
                None
            }
            Type::QubitArray(_) => {
                let message = "QubitArray default values";
                self.push_unsupported_error_message(message, span);
                None
            }
            Type::BitArray(_, _) => {
                self.push_unimplemented_error_message("bit array default value", span);
                None
            }
            Type::BoolArray(_) => {
                self.push_unimplemented_error_message("bool array default value", span);
                None
            }
            Type::DurationArray(_) => {
                self.push_unimplemented_error_message("duration array default value", span);
                None
            }
            Type::AngleArray(_, _) => {
                self.push_unimplemented_error_message("angle array default value", span);
                None
            }
            Type::ComplexArray(_, _) => {
                self.push_unimplemented_error_message("complex array default value", span);
                None
            }
            Type::FloatArray(_, _) => {
                self.push_unimplemented_error_message("float array default value", span);
                None
            }
            Type::IntArray(_, _) => {
                self.push_unimplemented_error_message("int array default value", span);
                None
            }
            Type::UIntArray(_, _) => {
                self.push_unimplemented_error_message("uint array default value", span);
                None
            }
            Type::Duration(_)
            | Type::Err
            | Type::Gate(_, _)
            | Type::Range
            | Type::Set
            | Type::Void => {
                let message = format!("Default values for {ty:?} are unsupported.");
                self.push_unsupported_error_message(message, span);
                None
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn coerce_literal_expr_to_type(
        &mut self,
        ty: &Type,
        rhs: &super::ast::Expr,
        kind: &super::ast::LiteralKind,
    ) -> Option<super::ast::Expr> {
        if *ty == rhs.ty {
            // Base case, we shouldn't have gotten here
            // but if we did, we can just return the rhs
            return Some(rhs.clone());
        }
        if types_equal_except_const(ty, &rhs.ty) {
            // literals are always const, so we can safely return
            // the const ty
            if rhs.ty.is_const() {
                return Some(rhs.clone());
            }
            // the lsh is supposed to be const but is being initialized
            // to a non-const value, we can't allow this
            return None;
        }
        assert!(can_cast_literal(ty, &rhs.ty) || can_cast_literal_with_value_knowledge(ty, kind));
        let lhs_ty = ty.clone();
        let rhs_ty = rhs.ty.clone();
        let span = rhs.span;

        if matches!(lhs_ty, Type::Bit(..)) {
            if let super::ast::LiteralKind::Int(value) = kind {
                // can_cast_literal_with_value_knowledge guarantees that value is 0 or 1
                return Some(super::ast::Expr {
                    span,
                    kind: Box::new(super::ast::ExprKind::Lit(super::ast::LiteralKind::Int(
                        *value,
                    ))),
                    ty: lhs_ty.as_const(),
                });
            } else if let super::ast::LiteralKind::Bool(value) = kind {
                return Some(super::ast::Expr {
                    span,
                    kind: Box::new(super::ast::ExprKind::Lit(super::ast::LiteralKind::Int(
                        i64::from(*value),
                    ))),
                    ty: lhs_ty.as_const(),
                });
            }
        }
        // if lhs_ty is 1 dim bitarray and rhs is int/uint, we can cast
        let (is_int_to_bit_array, size) = match &lhs_ty {
            Type::BitArray(dims, _) => {
                if matches!(rhs.ty, Type::Int(..) | Type::UInt(..)) {
                    match dims {
                        &ArrayDimensions::One(size) => (true, size),
                        _ => (false, 0),
                    }
                } else {
                    (false, 0)
                }
            }
            _ => (false, 0),
        };
        if is_int_to_bit_array {
            if let super::ast::LiteralKind::Int(value) = kind {
                if *value < 0 || *value >= (1 << size) {
                    // todo: error message
                    return None;
                }

                let u_size = size as usize;
                let bitstring = format!("{value:0u_size$b}");
                let Ok(value) = BigInt::from_str_radix(&bitstring, 2) else {
                    // todo: error message
                    return None;
                };

                return Some(super::ast::Expr {
                    span,
                    kind: Box::new(super::ast::ExprKind::Lit(
                        super::ast::LiteralKind::Bitstring(value, size),
                    )),
                    ty: lhs_ty.as_const(),
                });
            }
        }
        if matches!(lhs_ty, Type::UInt(..)) {
            if let super::ast::LiteralKind::Int(value) = kind {
                // this should have been validated by can_cast_literal_with_value_knowledge
                return Some(super::ast::Expr {
                    span,
                    kind: Box::new(super::ast::ExprKind::Lit(super::ast::LiteralKind::Int(
                        *value,
                    ))),
                    ty: lhs_ty.as_const(),
                });
            }
        }
        let result = match (&lhs_ty, &rhs_ty) {
            (Type::Float(..), Type::Int(..) | Type::UInt(..)) => {
                if let super::ast::LiteralKind::Int(value) = kind {
                    if let Some(value) = safe_i64_to_f64(*value) {
                        return Some(super::ast::Expr {
                            span,
                            kind: Box::new(super::ast::ExprKind::Lit(
                                super::ast::LiteralKind::Float(value),
                            )),
                            ty: lhs_ty.as_const(),
                        });
                    }
                    self.push_semantic_error(SemanticErrorKind::InvalidCastValueRange(
                        rhs_ty.to_string(),
                        lhs_ty.to_string(),
                        span,
                    ));
                    return None?;
                }
                None
            }
            (Type::Angle(..) | Type::Float(..), Type::Float(..)) => {
                if let super::ast::LiteralKind::Float(value) = kind {
                    return Some(super::ast::Expr {
                        span,
                        kind: Box::new(super::ast::ExprKind::Lit(super::ast::LiteralKind::Float(
                            *value,
                        ))),
                        ty: lhs_ty.as_const(),
                    });
                }
                None
            }
            (Type::Complex(..), Type::Complex(..)) => {
                if let super::ast::LiteralKind::Complex(real, imag) = kind {
                    return Some(super::ast::Expr {
                        span,
                        kind: Box::new(super::ast::ExprKind::Lit(
                            super::ast::LiteralKind::Complex(*real, *imag),
                        )),
                        ty: lhs_ty.as_const(),
                    });
                }
                None
            }
            (Type::Complex(..), Type::Float(..)) => {
                if let super::ast::LiteralKind::Float(value) = kind {
                    return Some(super::ast::Expr {
                        span,
                        kind: Box::new(super::ast::ExprKind::Lit(
                            super::ast::LiteralKind::Complex(*value, 0.0),
                        )),
                        ty: lhs_ty.as_const(),
                    });
                }
                None
            }
            (Type::Complex(..), Type::Int(..) | Type::UInt(..)) => {
                // complex requires a double as input, so we need to
                // convert the int to a double, then create the complex
                if let super::ast::LiteralKind::Int(value) = kind {
                    if let Some(value) = safe_i64_to_f64(*value) {
                        return Some(super::ast::Expr {
                            span,
                            kind: Box::new(super::ast::ExprKind::Lit(
                                super::ast::LiteralKind::Complex(value, 0.0),
                            )),
                            ty: lhs_ty.as_const(),
                        });
                    }
                    let kind = SemanticErrorKind::InvalidCastValueRange(
                        "Integer".to_string(),
                        "Double".to_string(),
                        span,
                    );
                    self.push_semantic_error(kind);
                    return None?;
                }
                None
            }
            (Type::Bit(..), Type::Int(..) | Type::UInt(..)) => {
                // we've already checked that the value is 0 or 1
                if let super::ast::LiteralKind::Int(value) = kind {
                    if *value == 0 || *value == 1 {
                        return Some(super::ast::Expr {
                            span,
                            kind: Box::new(super::ast::ExprKind::Lit(
                                super::ast::LiteralKind::Int(*value),
                            )),
                            ty: lhs_ty.as_const(),
                        });
                    }
                    panic!("Value must be 0 or 1");
                } else {
                    panic!("Literal must be an IntNumber");
                }
            }
            (Type::Int(width, _), Type::Int(_, _) | Type::UInt(_, _)) => {
                // we've already checked that this conversion can happen from a signed to unsigned int
                match kind {
                    super::ast::LiteralKind::Int(value) => {
                        return Some(super::ast::Expr {
                            span,
                            kind: Box::new(super::ast::ExprKind::Lit(
                                super::ast::LiteralKind::Int(*value),
                            )),
                            ty: lhs_ty.as_const(),
                        });
                    }
                    super::ast::LiteralKind::BigInt(value) => {
                        if let Some(width) = width {
                            let mut cap = BigInt::from_i64(1).expect("1 is a valid i64");
                            BigInt::shl_assign(&mut cap, width);
                            if *value >= cap {
                                self.push_semantic_error(SemanticErrorKind::InvalidCastValueRange(
                                    "BigInt".to_string(),
                                    "Int".to_string(),
                                    span,
                                ));
                                return None;
                            }
                        }
                        return Some(super::ast::Expr {
                            span,
                            kind: Box::new(super::ast::ExprKind::Lit(
                                super::ast::LiteralKind::BigInt(value.clone()),
                            )),
                            ty: lhs_ty.as_const(),
                        });
                    }
                    _ => panic!("Literal must be an IntNumber or BigInt"),
                }
            }
            _ => None,
        };
        if result.is_none() {
            // we assert that the type can be casted
            // but we didn't cast it, so this is a bug
            panic!("Literal type cast failed lhs: {:?}, rhs: {:?}", ty, rhs.ty);
        } else {
            result
        }
    }

    // Rules for negating literals are different than that of expressions
    // What those rules are is not clear from the spec, so this is a best guess
    // based on other qasm implementations.
    fn lower_negated_literal_as_ty(
        &mut self,
        lit: &syntax::Lit,
        target_ty: Option<Type>,
        span: Span,
    ) -> Option<super::ast::Expr> {
        let (kind, ty) = (match &lit.kind {
            syntax::LiteralKind::Float(value) => Some((
                super::ast::LiteralKind::Float(-value),
                Type::Float(None, true),
            )),
            syntax::LiteralKind::Imaginary(value) => Some((
                super::ast::LiteralKind::Complex(0.0, -value),
                Type::Complex(None, true),
            )),
            syntax::LiteralKind::Int(value) => {
                Some((super::ast::LiteralKind::Int(-value), Type::Int(None, true)))
            }
            syntax::LiteralKind::BigInt(value) => {
                let value = BigInt::from(-1) * value;
                Some((
                    super::ast::LiteralKind::BigInt(value),
                    Type::Int(None, true),
                ))
            }
            syntax::LiteralKind::Duration(value, time_unit) => {
                let unit = match time_unit {
                    syntax::TimeUnit::Dt => super::ast::TimeUnit::Dt,
                    syntax::TimeUnit::Ms => super::ast::TimeUnit::Ms,
                    syntax::TimeUnit::Ns => super::ast::TimeUnit::Ns,
                    syntax::TimeUnit::S => super::ast::TimeUnit::S,
                    syntax::TimeUnit::Us => super::ast::TimeUnit::Us,
                };
                Some((
                    super::ast::LiteralKind::Duration(-value, unit),
                    Type::Duration(true),
                ))
            }
            syntax::LiteralKind::Array(_) => {
                self.push_unsupported_error_message("negated array literal expressions", span);
                None
            }
            syntax::LiteralKind::Bitstring(_, _) => {
                self.push_unsupported_error_message("negated bitstring literal expressions", span);
                None
            }
            syntax::LiteralKind::Bool(_) => {
                self.push_unsupported_error_message("negated bool literal expressions", span);
                None
            }
            syntax::LiteralKind::String(_) => {
                self.push_unsupported_error_message("negated string literal expressions", span);
                None
            }
        })?;

        let expr = super::ast::Expr {
            span,
            kind: Box::new(super::ast::ExprKind::Lit(kind.clone())),
            ty,
        };
        if let Some(target_ty) = target_ty {
            return self.coerce_literal_expr_to_type(&target_ty, &expr, &kind);
        }
        Some(expr)
    }

    fn cast_expr_to_type(
        &mut self,
        ty: &Type,
        expr: &super::ast::Expr,
        span: Span,
    ) -> Option<super::ast::Expr> {
        let cast_expr = self.try_cast_expr_to_type(ty, expr, span);
        if cast_expr.is_none() {
            let rhs_ty_name = format!("{:?}", expr.ty);
            let lhs_ty_name = format!("{ty:?}");
            let kind = SemanticErrorKind::CannotCast(rhs_ty_name, lhs_ty_name, span);
            self.push_semantic_error(kind);
        }
        cast_expr
    }

    fn try_cast_expr_to_type(
        &mut self,
        ty: &Type,
        expr: &super::ast::Expr,
        span: Span,
    ) -> Option<super::ast::Expr> {
        if *ty == expr.ty {
            // Base case, we shouldn't have gotten here
            // but if we did, we can just return the rhs
            return Some(expr.clone());
        }
        if types_equal_except_const(ty, &expr.ty) {
            if expr.ty.is_const() {
                // lhs isn't const, but rhs is, we can just return the rhs
                return Some(expr.clone());
            }
            // the lsh is supposed to be const but is being initialized
            // to a non-const value, we can't allow this
            return None;
        }
        // if the target type is wider, we can try to relax the rhs type
        // We only do this for float and complex. Int types
        // require extra handling for BigInts
        match (ty, &expr.ty) {
            (Type::Angle(w1, _), Type::Angle(w2, _))
            | (Type::Float(w1, _), Type::Float(w2, _))
            | (Type::Complex(w1, _), Type::Complex(w2, _)) => {
                if w1.is_none() && w2.is_some() {
                    return Some(super::ast::Expr {
                        span: expr.span,
                        kind: expr.kind.clone(),
                        ty: ty.clone(),
                    });
                }

                if *w1 >= *w2 {
                    return Some(super::ast::Expr {
                        span: expr.span,
                        kind: expr.kind.clone(),
                        ty: ty.clone(),
                    });
                }
            }
            _ => {}
        }
        // Casting of literals is handled elsewhere. This is for casting expressions
        // which cannot be bypassed and must be handled by built-in Q# casts from
        // the standard library.
        match &expr.ty {
            Type::Angle(_, _) => Self::cast_angle_expr_to_type(ty, expr),
            Type::Bit(_) => self.cast_bit_expr_to_type(ty, expr, span),
            Type::Bool(_) => Self::cast_bool_expr_to_type(ty, expr),
            Type::Complex(_, _) => cast_complex_expr_to_type(ty, expr),
            Type::Float(_, _) => Self::cast_float_expr_to_type(ty, expr),
            Type::Int(_, _) | Type::UInt(_, _) => Self::cast_int_expr_to_type(ty, expr),
            Type::BitArray(dims, _) => Self::cast_bitarray_expr_to_type(dims, ty, expr),
            _ => None,
        }
    }

    /// +----------------+-------------------------------------------------------------+
    /// | Allowed casts  | Casting To                                                  |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | Casting From   | bool  | int | uint | float | angle | bit | duration | qubit |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | angle          | Yes   | No  | No   | No    | -     | Yes | No       | No    |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    fn cast_angle_expr_to_type(ty: &Type, rhs: &super::ast::Expr) -> Option<super::ast::Expr> {
        assert!(matches!(rhs.ty, Type::Angle(..)));
        match ty {
            Type::Bit(..) | Type::Bool(..) => {
                Some(wrap_expr_in_implicit_cast_expr(ty.clone(), rhs.clone()))
            }
            _ => None,
        }
    }

    /// +----------------+-------------------------------------------------------------+
    /// | Allowed casts  | Casting To                                                  |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | Casting From   | bool  | int | uint | float | angle | bit | duration | qubit |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | bit            | Yes   | Yes | Yes  | No    | Yes   | -   | No       | No    |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    fn cast_bit_expr_to_type(
        &mut self,
        ty: &Type,
        rhs: &super::ast::Expr,
        span: Span,
    ) -> Option<super::ast::Expr> {
        assert!(matches!(rhs.ty, Type::Bit(..)));
        // There is no operand, choosing the span of the node
        // but we could use the expr span as well.
        match ty {
            &Type::Angle(..) => {
                let msg = "Cast bit to angle";
                self.push_unimplemented_error_message(msg, span);
                None
            }
            &Type::Float(..) => {
                // The spec says that this cast isn't supported, but it
                // casts to other types that case to float. For now, we'll
                // say it is invalid like the spec.
                None
            }
            &Type::Bool(_) | &Type::Int(_, _) | &Type::UInt(_, _) => {
                Some(wrap_expr_in_implicit_cast_expr(ty.clone(), rhs.clone()))
            }

            _ => None,
        }
    }

    /// +----------------+-------------------------------------------------------------+
    /// | Allowed casts  | Casting To                                                  |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | Casting From   | bool  | int | uint | float | angle | bit | duration | qubit |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | float          | Yes   | Yes | Yes  | -     | Yes   | No  | No       | No    |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    ///
    /// Additional cast to complex
    fn cast_float_expr_to_type(ty: &Type, rhs: &super::ast::Expr) -> Option<super::ast::Expr> {
        assert!(matches!(rhs.ty, Type::Float(..)));
        match ty {
            &Type::Angle(..)
            | &Type::Complex(_, _)
            | &Type::Int(_, _)
            | &Type::UInt(_, _)
            | &Type::Bool(_) => {
                // this will eventually be a call into Complex(expr, 0.0)
                Some(wrap_expr_in_implicit_cast_expr(ty.clone(), rhs.clone()))
            }
            _ => None,
        }
    }

    /// +----------------+-------------------------------------------------------------+
    /// | Allowed casts  | Casting To                                                  |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | Casting From   | bool  | int | uint | float | angle | bit | duration | qubit |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | bool           | -     | Yes | Yes  | Yes   | No    | Yes | No       | No    |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    fn cast_bool_expr_to_type(ty: &Type, rhs: &super::ast::Expr) -> Option<super::ast::Expr> {
        assert!(matches!(rhs.ty, Type::Bool(..)));
        match ty {
            &Type::Bit(_) | &Type::Float(_, _) | &Type::Int(_, _) | &Type::UInt(_, _) => {
                Some(wrap_expr_in_implicit_cast_expr(ty.clone(), rhs.clone()))
            }
            _ => None,
        }
    }

    /// +----------------+-------------------------------------------------------------+
    /// | Allowed casts  | Casting To                                                  |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | Casting From   | bool  | int | uint | float | angle | bit | duration | qubit |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | int            | Yes   | -   | Yes  | Yes   | No    | Yes | No       | No    |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | uint           | Yes   | Yes | -    | Yes   | No    | Yes | No       | No    |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    ///
    /// Additional cast to ``BigInt``
    #[allow(clippy::too_many_lines)]
    fn cast_int_expr_to_type(ty: &Type, rhs: &super::ast::Expr) -> Option<super::ast::Expr> {
        assert!(matches!(rhs.ty, Type::Int(..) | Type::UInt(..)));

        match ty {
            Type::BitArray(_, _)
            | Type::Float(_, _)
            | Type::Int(_, _)
            | Type::UInt(_, _)
            | Type::Bool(..)
            | Type::Bit(..)
            | Type::Complex(..) => Some(wrap_expr_in_implicit_cast_expr(ty.clone(), rhs.clone())),
            _ => None,
        }
    }

    fn cast_bitarray_expr_to_type(
        dims: &ArrayDimensions,
        ty: &Type,
        rhs: &super::ast::Expr,
    ) -> Option<super::ast::Expr> {
        let ArrayDimensions::One(array_width) = dims else {
            return None;
        };
        if !matches!(ty, Type::Int(..) | Type::UInt(..)) {
            return None;
        }
        // we know we have a bit array being cast to an int/uint
        // verfiy widths
        let int_width = ty.width();

        if int_width.is_none() || (int_width == Some(*array_width)) {
            Some(wrap_expr_in_implicit_cast_expr(ty.clone(), rhs.clone()))
        } else {
            None
        }
    }

    #[allow(clippy::too_many_lines)]
    fn lower_binary_op_expr(
        &mut self,
        op: syntax::BinOp,
        lhs: super::ast::Expr,
        rhs: super::ast::Expr,
        span: Span,
    ) -> Option<super::ast::Expr> {
        if lhs.ty.is_quantum() {
            let kind = SemanticErrorKind::QuantumTypesInBinaryExpression(lhs.span);
            self.push_semantic_error(kind);
        }

        if rhs.ty.is_quantum() {
            let kind = SemanticErrorKind::QuantumTypesInBinaryExpression(rhs.span);
            self.push_semantic_error(kind);
        }

        let left_type = lhs.ty.clone();
        let right_type = rhs.ty.clone();

        if Self::binop_requires_bitwise_conversion(op, &left_type) {
            // if the operator requires bitwise conversion, we need to determine
            // what size of UInt to promote to. If we can't promote to a UInt, we
            // push an error and return None.
            let (ty, lhs_uint_promotion, rhs_uint_promotion) =
                promote_to_uint_ty(&left_type, &right_type);
            let Some(ty) = ty else {
                let target_ty = Type::UInt(None, false);
                if lhs_uint_promotion.is_none() {
                    let target_str: String = format!("{target_ty:?}");
                    let kind = SemanticErrorKind::CannotCast(
                        format!("{left_type:?}"),
                        target_str,
                        lhs.span,
                    );
                    self.push_semantic_error(kind);
                }
                if rhs_uint_promotion.is_none() {
                    let target_str: String = format!("{target_ty:?}");
                    let kind = SemanticErrorKind::CannotCast(
                        format!("{right_type:?}"),
                        target_str,
                        rhs.span,
                    );
                    self.push_semantic_error(kind);
                }
                return None;
            };
            // Now that we know the effective Uint type, we can cast the lhs and rhs
            // so that operations can be performed on them.
            let new_lhs = self.cast_expr_to_type(&ty, &lhs, lhs.span)?;
            // only cast the rhs if the operator requires symmetric conversion
            let new_rhs = if Self::binop_requires_bitwise_symmetric_conversion(op) {
                self.cast_expr_to_type(&ty, &rhs, rhs.span)?
            } else {
                rhs
            };

            let bin_expr = super::ast::BinaryOpExpr {
                lhs: new_lhs,
                rhs: new_rhs,
                op: op.into(),
            };
            let kind = super::ast::ExprKind::BinaryOp(bin_expr);
            let expr = super::ast::Expr {
                span,
                kind: Box::new(kind),
                ty,
            };

            let final_expr = self.cast_expr_to_type(&left_type, &expr, span)?;
            return Some(final_expr);
        }

        // for int, uint, float, and complex, the lesser of the two types is cast to
        // the greater of the two. Within each level of complex and float, types with
        // greater width are greater than types with lower width.
        // complex > float > int/uint
        // Q# has built-in functions: IntAsDouble, IntAsBigInt to handle two cases.
        // If the width of a float is greater than 64, we can't represent it as a double.

        let (lhs, rhs, ty) = if matches!(op, syntax::BinOp::AndL | syntax::BinOp::OrL) {
            let ty = Type::Bool(false);
            let new_lhs = self.cast_expr_to_type(&ty, &lhs, lhs.span)?;
            let new_rhs = self.cast_expr_to_type(&ty, &rhs, rhs.span)?;
            (new_lhs, new_rhs, ty)
        } else if binop_requires_int_conversion_for_type(op, &left_type, &rhs.ty) {
            let ty = Type::Int(None, false);
            let new_lhs = self.cast_expr_to_type(&ty, &lhs, lhs.span)?;
            let new_rhs = self.cast_expr_to_type(&ty, &rhs, lhs.span)?;
            (new_lhs, new_rhs, ty)
        } else if requires_symmetric_conversion(op) {
            let promoted_type = try_promote_with_casting(&left_type, &right_type);
            let new_left = if promoted_type == left_type {
                lhs
            } else {
                match &lhs.kind.as_ref() {
                    super::ast::ExprKind::Lit(kind) => {
                        if can_cast_literal(&promoted_type, &left_type)
                            || can_cast_literal_with_value_knowledge(&promoted_type, kind)
                        {
                            self.coerce_literal_expr_to_type(&promoted_type, &lhs, kind)?
                        } else {
                            self.cast_expr_to_type(&promoted_type, &lhs, lhs.span)?
                        }
                    }
                    _ => self.cast_expr_to_type(&promoted_type, &lhs, lhs.span)?,
                }
            };
            let new_right = if promoted_type == right_type {
                rhs
            } else {
                match &rhs.kind.as_ref() {
                    super::ast::ExprKind::Lit(kind) => {
                        if can_cast_literal(&promoted_type, &right_type)
                            || can_cast_literal_with_value_knowledge(&promoted_type, kind)
                        {
                            self.coerce_literal_expr_to_type(&promoted_type, &rhs, kind)?
                        } else {
                            self.cast_expr_to_type(&promoted_type, &rhs, rhs.span)?
                        }
                    }
                    _ => self.cast_expr_to_type(&promoted_type, &rhs, rhs.span)?,
                }
            };
            (new_left, new_right, promoted_type)
        } else if binop_requires_symmetric_int_conversion(op) {
            let ty = Type::Int(None, false);
            let new_rhs = self.cast_expr_to_type(&ty, &rhs, rhs.span)?;
            (lhs, new_rhs, left_type)
        } else {
            (lhs, rhs, left_type)
        };

        // now that we have the lhs and rhs expressions, we can create the binary expression
        // but we need to check if the chosen operator is supported by the types after
        // promotion and conversion.

        let expr = if matches!(ty, Type::Complex(..)) {
            if is_complex_binop_supported(op) {
                // TODO: How do we handle complex binary expressions?
                // this is going to be a call to a built-in function
                // that doesn't map to qasm def semantics
                self.push_unimplemented_error_message("complex binary exprs", span);
                None
            } else {
                let kind = SemanticErrorKind::OperatorNotSupportedForTypes(
                    format!("{op:?}"),
                    format!("{ty:?}"),
                    format!("{ty:?}"),
                    span,
                );
                self.push_semantic_error(kind);
                None
            }
        } else {
            let bin_expr = super::ast::BinaryOpExpr {
                op: op.into(),
                lhs,
                rhs,
            };
            let kind = super::ast::ExprKind::BinaryOp(bin_expr);
            let expr = super::ast::Expr {
                span,
                kind: Box::new(kind),
                ty: ty.clone(),
            };
            Some(expr)
        };

        let ty = match op.into() {
            super::ast::BinOp::AndL
            | super::ast::BinOp::Eq
            | super::ast::BinOp::Gt
            | super::ast::BinOp::Gte
            | super::ast::BinOp::Lt
            | super::ast::BinOp::Lte
            | super::ast::BinOp::Neq
            | super::ast::BinOp::OrL => Type::Bool(false),
            _ => ty,
        };
        let mut expr = expr?;
        expr.ty = ty;
        Some(expr)
    }

    fn binop_requires_bitwise_conversion(op: syntax::BinOp, left_type: &Type) -> bool {
        if (matches!(
            op,
            syntax::BinOp::AndB | syntax::BinOp::OrB | syntax::BinOp::XorB
        ) && matches!(
            left_type,
            Type::Bit(..)
                | Type::UInt(..)
                | Type::Angle(..)
                | Type::BitArray(ArrayDimensions::One(_), _)
        )) {
            return true;
        }
        if (matches!(op, syntax::BinOp::Shl | syntax::BinOp::Shr)
            && matches!(
                left_type,
                Type::Bit(..)
                    | Type::UInt(..)
                    | Type::Angle(..)
                    | Type::BitArray(ArrayDimensions::One(_), _)
            ))
        {
            return true;
        }
        false
    }

    fn binop_requires_bitwise_symmetric_conversion(op: syntax::BinOp) -> bool {
        matches!(
            op,
            syntax::BinOp::AndB | syntax::BinOp::OrB | syntax::BinOp::XorB
        )
    }

    // TODO: which these are parsed as different types, they are effectively the same
    fn lower_index_element(
        &mut self,
        index: &syntax::IndexElement,
    ) -> Option<super::ast::IndexElement> {
        match index {
            syntax::IndexElement::DiscreteSet(set) => {
                let items = set
                    .values
                    .iter()
                    .filter_map(|expr| self.lower_expr(expr))
                    .collect::<Vec<_>>();
                if set.values.len() == items.len() {
                    return Some(super::ast::IndexElement::DiscreteSet(
                        super::ast::DiscreteSet {
                            span: set.span,
                            values: syntax::list_from_iter(items),
                        },
                    ));
                }
                let kind = SemanticErrorKind::FailedToCompileExpressionList(set.span);
                self.push_semantic_error(kind);
                None
            }
            syntax::IndexElement::IndexSet(set) => {
                let items = set
                    .values
                    .iter()
                    .filter_map(|expr| self.lower_index_set_item(expr))
                    .collect::<Vec<_>>();
                if set.values.len() == items.len() {
                    return Some(super::ast::IndexElement::IndexSet(super::ast::IndexSet {
                        span: set.span,
                        values: syntax::list_from_iter(items),
                    }));
                }
                let kind = SemanticErrorKind::FailedToCompileExpressionList(set.span);
                self.push_semantic_error(kind);
                None
            }
        }
    }

    fn lower_index_set_item(
        &mut self,
        item: &syntax::IndexSetItem,
    ) -> Option<super::ast::IndexSetItem> {
        let item = match item {
            syntax::IndexSetItem::RangeDefinition(range_definition) => {
                super::ast::IndexSetItem::RangeDefinition(
                    self.lower_range_definition(range_definition)?,
                )
            }
            syntax::IndexSetItem::Expr(expr) => {
                super::ast::IndexSetItem::Expr(self.lower_expr(expr)?)
            }
            syntax::IndexSetItem::Err => {
                unreachable!("IndexSetItem::Err should have been handled")
            }
        };
        Some(item)
    }

    fn lower_range_definition(
        &mut self,
        range_definition: &syntax::RangeDefinition,
    ) -> Option<super::ast::RangeDefinition> {
        let mut failed = false;
        let start = range_definition.start.as_ref().map(|e| {
            let expr = self.lower_expr(e);
            if expr.is_none() {
                failed = true;
            }
            expr
        });
        let step = range_definition.step.as_ref().map(|e| {
            let expr = self.lower_expr(e);
            if expr.is_none() {
                failed = true;
            }
            expr
        });
        let end = range_definition.end.as_ref().map(|e| {
            let expr = self.lower_expr(e);
            if expr.is_none() {
                failed = true;
            }
            expr
        });
        if failed {
            return None;
        }
        Some(super::ast::RangeDefinition {
            span: range_definition.span,
            start: start.map(|s| s.expect("start should be Some")),
            step: step.map(|s| s.expect("step should be Some")),
            end: end.map(|e| e.expect("end should be Some")),
        })
    }

    fn lower_index_expr(&mut self, expr: &syntax::IndexExpr) -> Option<super::ast::Expr> {
        let collection = self.lower_expr(&expr.collection);
        let index = self.lower_index_element(&expr.index);
        let collection = collection?;
        let index = index?;

        let indexed_ty = self.get_indexed_type(&collection.ty, expr.span, 1)?;

        Some(super::ast::Expr {
            span: expr.span,
            kind: Box::new(super::ast::ExprKind::IndexExpr(super::ast::IndexExpr {
                span: expr.span,
                collection,
                index,
            })),
            ty: indexed_ty,
        })
    }

    fn get_indexed_type(
        &mut self,
        ty: &Type,
        span: Span,
        num_indices: usize,
    ) -> Option<super::types::Type> {
        if !ty.is_array() {
            let kind = SemanticErrorKind::CannotIndexType(format!("{ty:?}"), span);
            self.push_semantic_error(kind);
            return None;
        }

        if num_indices > ty.num_dims() {
            let kind = SemanticErrorKind::TooManyIndices(span);
            self.push_semantic_error(kind);
            return None;
        }

        let mut indexed_ty = ty.clone();
        for _ in 0..num_indices {
            let Some(ty) = indexed_ty.get_indexed_type() else {
                // we failed to get the indexed type, unknown error
                // we should have caught this earlier with the two checks above
                let kind = SemanticErrorKind::CannotIndexType(format!("{ty:?}"), span);
                self.push_semantic_error(kind);
                return None;
            };
            indexed_ty = ty;
        }
        Some(indexed_ty)
    }

    fn lower_indexed_ident_expr(
        &mut self,
        indexed_ident: &syntax::IndexedIdent,
    ) -> Option<super::ast::Expr> {
        let ident = indexed_ident.name.clone();

        let indices = indexed_ident
            .indices
            .iter()
            .filter_map(|index| self.lower_index_element(index))
            .collect::<Vec<_>>();

        let (symbol_id, lhs_symbol) = self.symbols.get_symbol_by_name(&ident.name)?;
        let ty = lhs_symbol.ty.clone();
        // use the supplied number of indicies rathar than the number of indicies we lowered
        let ty = self.get_indexed_type(&ty, indexed_ident.span, indexed_ident.indices.len())?;

        if indices.len() != indexed_ident.indices.len() {
            // we failed to lower all the indices, error was already pushed
            return None;
        }

        Some(super::ast::Expr {
            span: indexed_ident.span,
            kind: Box::new(super::ast::ExprKind::IndexedIdentifier(
                super::ast::IndexedIdent {
                    span: indexed_ident.span,
                    symbol_id,
                    indices: syntax::list_from_iter(indices),
                },
            )),
            ty,
        })
    }
}

fn wrap_expr_in_implicit_cast_expr(ty: Type, rhs: super::ast::Expr) -> super::ast::Expr {
    super::ast::Expr {
        span: rhs.span,
        kind: Box::new(super::ast::ExprKind::Cast(super::ast::Cast {
            span: Span::default(),
            expr: rhs,
            ty: ty.clone(),
        })),
        ty,
    }
}

/// +----------------+-------------------------------------------------------------+
/// | Allowed casts  | Casting To                                                  |
/// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
/// | Casting From   | bool  | int | uint | float | angle | bit | duration | qubit |
/// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
/// | complex        | ??    | ??  | ??   | ??    | No    | ??  | No       | No    |
/// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
fn cast_complex_expr_to_type(ty: &Type, rhs: &super::ast::Expr) -> Option<super::ast::Expr> {
    assert!(matches!(rhs.ty, Type::Complex(..)));

    if matches!((ty, &rhs.ty), (Type::Complex(..), Type::Complex(..))) {
        // we are both complex, but our widths are different. If both
        // had implicit widths, we would have already matched for the
        // (None, None). If the rhs width is bigger, we will return
        // None to let the cast fail

        // Here, we can safely cast the rhs to the lhs type if the
        // lhs width can hold the rhs's width
        if ty.width().is_none() && rhs.ty.width().is_some() {
            return Some(wrap_expr_in_implicit_cast_expr(ty.clone(), rhs.clone()));
        }
        if ty.width() >= rhs.ty.width() {
            return Some(wrap_expr_in_implicit_cast_expr(ty.clone(), rhs.clone()));
        }
    }
    None
}

fn get_identifier_name(identifier: &syntax::Identifier) -> std::rc::Rc<str> {
    match identifier {
        syntax::Identifier::Ident(ident) => ident.name.clone(),
        syntax::Identifier::IndexedIdent(ident) => ident.name.name.clone(),
    }
}
