// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::ops::ShlAssign;
use std::rc::Rc;

use super::symbols::ScopeKind;
use super::types::binop_requires_asymmetric_angle_op;
use super::types::binop_requires_int_conversion_for_type;
use super::types::binop_requires_symmetric_int_conversion;
use super::types::is_complex_binop_supported;
use super::types::promote_to_uint_ty;
use super::types::promote_width;
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
use crate::parser::ast::list_from_iter;
use crate::parser::QasmSource;
use crate::semantic::types::can_cast_literal;
use crate::semantic::types::can_cast_literal_with_value_knowledge;
use crate::semantic::types::ArrayDimensions;
use crate::stdlib::angle::Angle;

use super::ast as semantic;
use crate::parser::ast as syntax;

use super::{
    ast::{Stmt, Version},
    SemanticErrorKind,
};

/// Macro to create an error expression. Used when we fail to
/// lower an expression. It is assumed that an error was
/// already reported.
macro_rules! err_expr {
    ($ty:expr) => {
        semantic::Expr {
            span: Span::default(),
            kind: Box::new(semantic::ExprKind::Err),
            ty: $ty,
        }
    };

    ($ty:expr, $span:expr) => {
        semantic::Expr {
            span: $span,
            kind: Box::new(semantic::ExprKind::Err),
            ty: $ty,
        }
    };
}

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
    pub symbols: SymbolTable,
    pub version: Option<Version>,
    pub stmts: Vec<Stmt>,
}
impl Lowerer {
    pub fn new(source: QasmSource, source_map: SourceMap) -> Self {
        let symbols = SymbolTable::default();
        let version = None;
        let stmts = Vec::new();
        let errors = Vec::new();
        Self {
            source,
            source_map,
            errors,
            symbols,
            version,
            stmts,
        }
    }

    pub fn lower(mut self) -> crate::semantic::QasmSemanticParseResult {
        // Should we fail if we see a version in included files?
        let source = &self.source.clone();
        self.version = self.lower_version(source.program().version);

        self.lower_source(source);

        assert!(
            self.symbols.is_current_scope_global(),
            "Scope stack was non popped correctly."
        );

        let program = semantic::Program {
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
        // we keep an iterator of the includes so we can match them with the
        // source includes. The include statements only have the path, but
        // we have already loaded all of source files in the
        // `source.includes()`
        let mut includes = source.includes().iter();

        for stmt in &source.program().statements {
            if let syntax::StmtKind::Include(include) = &*stmt.kind {
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
            } else {
                let stmt = self.lower_stmt(stmt);
                self.stmts.push(stmt);
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn lower_stmt(&mut self, stmt: &syntax::Stmt) -> semantic::Stmt {
        let kind = match &*stmt.kind {
            syntax::StmtKind::Alias(stmt) => self.lower_alias(stmt),
            syntax::StmtKind::Assign(stmt) => self.lower_assign(stmt),
            syntax::StmtKind::AssignOp(stmt) => self.lower_assign_op(stmt),
            syntax::StmtKind::Barrier(stmt) => self.lower_barrier_stmt(stmt),
            syntax::StmtKind::Box(stmt) => self.lower_box(stmt),
            syntax::StmtKind::Break(stmt) => self.lower_break(stmt),
            syntax::StmtKind::Block(stmt) => {
                semantic::StmtKind::Block(Box::new(self.lower_block(stmt)))
            }
            syntax::StmtKind::Cal(stmt) => self.lower_calibration(stmt),
            syntax::StmtKind::CalibrationGrammar(stmt) => self.lower_calibration_grammar(stmt),
            syntax::StmtKind::ClassicalDecl(stmt) => self.lower_classical_decl(stmt),
            syntax::StmtKind::ConstDecl(stmt) => self.lower_const_decl(stmt),
            syntax::StmtKind::Continue(stmt) => self.lower_continue_stmt(stmt),
            syntax::StmtKind::Def(stmt) => self.lower_def(stmt),
            syntax::StmtKind::DefCal(stmt) => self.lower_def_cal(stmt),
            syntax::StmtKind::Delay(stmt) => self.lower_delay(stmt),
            syntax::StmtKind::End(stmt) => Self::lower_end_stmt(stmt),
            syntax::StmtKind::ExprStmt(stmt) => self.lower_expr_stmt(stmt),
            syntax::StmtKind::ExternDecl(extern_decl) => self.lower_extern(extern_decl),
            syntax::StmtKind::For(stmt) => self.lower_for_stmt(stmt),
            syntax::StmtKind::If(stmt) => self.lower_if_stmt(stmt),
            syntax::StmtKind::GateCall(stmt) => self.lower_gate_call_stmt(stmt),
            syntax::StmtKind::GPhase(stmt) => self.lower_gphase_stmt(stmt),
            syntax::StmtKind::Include(stmt) => self.lower_include(stmt),
            syntax::StmtKind::IODeclaration(stmt) => self.lower_io_decl(stmt),
            syntax::StmtKind::Measure(stmt) => self.lower_measure(stmt),
            syntax::StmtKind::Pragma(stmt) => self.lower_pragma(stmt),
            syntax::StmtKind::QuantumGateDefinition(stmt) => self.lower_gate_def(stmt),
            syntax::StmtKind::QuantumDecl(stmt) => self.lower_quantum_decl(stmt),
            syntax::StmtKind::Reset(stmt) => self.lower_reset(stmt),
            syntax::StmtKind::Return(stmt) => self.lower_return(stmt),
            syntax::StmtKind::Switch(stmt) => self.lower_switch(stmt),
            syntax::StmtKind::WhileLoop(stmt) => self.lower_while_stmt(stmt),
            syntax::StmtKind::Err => semantic::StmtKind::Err,
        };
        let annotations = Self::lower_annotations(&stmt.annotations);
        semantic::Stmt {
            span: stmt.span,
            annotations: syntax::list_from_iter(annotations),
            kind: Box::new(kind),
        }
    }

    /// Define the standard gates in the symbol table.
    /// The sdg, tdg, crx, cry, crz, and ch are defined
    /// as their bare gates, and modifiers are applied
    /// when calling them.
    fn define_stdgates(&mut self, include: &syntax::IncludeStmt) {
        fn gate_symbol(name: &str, cargs: u32, qargs: u32) -> Symbol {
            Symbol::new(
                name,
                Span::default(),
                Type::Gate(cargs, qargs),
                Default::default(),
                Default::default(),
            )
        }
        let gates = vec![
            gate_symbol("p", 1, 1),
            gate_symbol("x", 0, 1),
            gate_symbol("y", 0, 1),
            gate_symbol("z", 0, 1),
            gate_symbol("h", 0, 1),
            gate_symbol("s", 0, 1),
            gate_symbol("t", 0, 1),
            gate_symbol("sx", 0, 1),
            gate_symbol("rx", 1, 1),
            gate_symbol("rxx", 1, 2),
            gate_symbol("ry", 1, 1),
            gate_symbol("ryy", 1, 2),
            gate_symbol("rz", 1, 1),
            gate_symbol("rzz", 1, 2),
            gate_symbol("cx", 0, 2),
            gate_symbol("cy", 0, 2),
            gate_symbol("cz", 0, 2),
            gate_symbol("cp", 1, 2),
            gate_symbol("swap", 0, 2),
            gate_symbol("ccx", 0, 3),
            gate_symbol("cu", 4, 2),
            gate_symbol("CX", 0, 2),
            gate_symbol("phase", 1, 1),
            gate_symbol("id", 0, 1),
            gate_symbol("u1", 1, 1),
            gate_symbol("u2", 2, 1),
            gate_symbol("u3", 3, 1),
        ];
        for gate in gates {
            let name = gate.name.clone();
            if self.symbols.insert_symbol(gate).is_err() {
                self.push_redefined_symbol_error(name.as_str(), include.span);
            }
        }
    }

    fn try_insert_or_get_existing_symbol_id<S>(
        &mut self,
        name: S,
        symbol: Symbol,
    ) -> super::symbols::SymbolId
    where
        S: AsRef<str>,
    {
        let symbol_span = symbol.span;
        let symbol_id = match self.symbols.try_insert_or_get_existing(symbol) {
            Ok(symbol_id) => symbol_id,
            Err(symbol_id) => {
                self.push_redefined_symbol_error(name.as_ref(), symbol_span);
                symbol_id
            }
        };
        symbol_id
    }

    fn try_get_existing_or_insert_err_symbol<S>(
        &mut self,
        name: S,
        span: Span,
    ) -> (super::symbols::SymbolId, std::rc::Rc<Symbol>)
    where
        S: AsRef<str>,
    {
        let (symbol_id, symbol) = match self
            .symbols
            .try_get_existing_or_insert_err_symbol(name.as_ref(), span)
        {
            Ok((symbol_id, symbol)) => (symbol_id, symbol),
            Err((symbol_id, symbol)) => {
                self.push_missing_symbol_error(name, span);
                (symbol_id, symbol)
            }
        };
        (symbol_id, symbol)
    }

    fn lower_alias(&mut self, alias: &syntax::AliasDeclStmt) -> semantic::StmtKind {
        let name = get_identifier_name(&alias.ident);
        // alias statements do their types backwards, you read the right side
        // and assign it to the left side.
        // the types of the rhs should be in the symbol table.
        let rhs = alias
            .exprs
            .iter()
            .map(|expr| self.lower_expr(expr))
            .collect::<Vec<_>>();
        let first = rhs.first().expect("missing rhs");

        let symbol = Symbol::new(
            &name,
            alias.ident.span(),
            first.ty.clone(),
            self.convert_semantic_type_to_qsharp_type(&first.ty, alias.ident.span()),
            IOKind::Default,
        );

        let symbol_id = self.try_insert_or_get_existing_symbol_id(name, symbol);

        if rhs.iter().any(|expr| expr.ty != first.ty) {
            let tys = rhs
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            let kind = SemanticErrorKind::InconsistentTypesInAlias(tys, alias.span);
            self.push_semantic_error(kind);
        }

        semantic::StmtKind::Alias(semantic::AliasDeclStmt {
            span: alias.span,
            symbol_id,
            exprs: syntax::list_from_iter(rhs),
        })
    }

    fn lower_assign(&mut self, stmt: &syntax::AssignStmt) -> semantic::StmtKind {
        if stmt.lhs.indices.is_empty() {
            self.lower_simple_assign_expr(&stmt.lhs.name, &stmt.rhs, stmt.span)
        } else {
            self.lower_indexed_assign_expr(&stmt.lhs, &stmt.rhs, stmt.span)
        }
    }

    fn lower_simple_assign_expr(
        &mut self,
        ident: &syntax::Ident,
        rhs: &syntax::ValueExpr,
        span: Span,
    ) -> semantic::StmtKind {
        let (symbol_id, symbol) =
            self.try_get_existing_or_insert_err_symbol(&ident.name, ident.span);

        let ty = symbol.ty.clone();
        let rhs = match rhs {
            syntax::ValueExpr::Expr(expr) => {
                let expr = self.lower_expr(expr);
                self.cast_expr_with_target_type_or_default(Some(expr), &ty, span)
            }
            syntax::ValueExpr::Measurement(measure_expr) => {
                let expr = self.lower_measure_expr(measure_expr);
                self.cast_expr_to_type(&ty, &expr)
            }
        };

        if ty.is_const() {
            let kind =
                SemanticErrorKind::CannotUpdateConstVariable(ident.name.to_string(), ident.span);
            self.push_semantic_error(kind);
            return semantic::StmtKind::Err;
        }

        semantic::StmtKind::Assign(semantic::AssignStmt {
            symbol_id,
            lhs_span: ident.span,
            rhs,
            span,
        })
    }

    fn lower_indexed_assign_expr(
        &mut self,
        index_expr: &syntax::IndexedIdent,
        rhs: &syntax::ValueExpr,
        span: Span,
    ) -> semantic::StmtKind {
        let ident = index_expr.name.clone();
        let (symbol_id, symbol) =
            self.try_get_existing_or_insert_err_symbol(&ident.name, ident.span);

        let indexed_ty =
            &self.get_indexed_type(&symbol.ty, index_expr.name.span, index_expr.indices.len());

        let indices = list_from_iter(
            index_expr
                .indices
                .iter()
                .map(|index| self.lower_index_element(index)),
        );

        let rhs = match rhs {
            syntax::ValueExpr::Expr(expr) => {
                let expr = self.lower_expr(expr);
                self.cast_expr_with_target_type_or_default(Some(expr), indexed_ty, span)
            }
            syntax::ValueExpr::Measurement(measure_expr) => {
                let expr = self.lower_measure_expr(measure_expr);
                self.cast_expr_to_type(indexed_ty, &expr)
            }
        };

        if symbol.ty.is_const() {
            let kind =
                SemanticErrorKind::CannotUpdateConstVariable(ident.name.to_string(), ident.span);
            self.push_semantic_error(kind);
        }

        semantic::StmtKind::IndexedAssign(semantic::IndexedAssignStmt {
            span,
            symbol_id,
            indices,
            rhs,
        })
    }

    fn lower_assign_op(&mut self, stmt: &syntax::AssignOpStmt) -> semantic::StmtKind {
        let op = stmt.op.into();
        let lhs = &stmt.lhs;
        let rhs = &stmt.rhs;
        let ident = lhs.name.clone();

        let (symbol_id, symbol) =
            self.try_get_existing_or_insert_err_symbol(&ident.name, ident.span);

        let ty = if lhs.indices.len() == 0 {
            &symbol.ty
        } else {
            &self.get_indexed_type(&symbol.ty, lhs.name.span, lhs.indices.len())
        };
        let indices = list_from_iter(
            lhs.indices
                .iter()
                .map(|index| self.lower_index_element(index)),
        );

        if ty.is_const() {
            let kind =
                SemanticErrorKind::CannotUpdateConstVariable(ident.name.to_string(), ident.span);
            self.push_semantic_error(kind);
        }

        let lhs = self.lower_indexed_ident_expr(lhs);
        let rhs = match rhs {
            syntax::ValueExpr::Expr(expr) => {
                let expr = self.lower_expr(expr);
                self.cast_expr_with_target_type_or_default(Some(expr), ty, stmt.span)
            }
            syntax::ValueExpr::Measurement(measure_expr) => {
                let expr = self.lower_measure_expr(measure_expr);
                self.cast_expr_to_type(ty, &expr)
            }
        };

        semantic::StmtKind::AssignOp(semantic::AssignOpStmt {
            span: stmt.span,
            symbol_id,
            indices,
            op,
            lhs,
            rhs,
        })
    }

    fn lower_expr(&mut self, expr: &syntax::Expr) -> semantic::Expr {
        match &*expr.kind {
            syntax::ExprKind::BinaryOp(bin_op_expr) => {
                let lhs = self.lower_expr(&bin_op_expr.lhs);
                let rhs = self.lower_expr(&bin_op_expr.rhs);
                self.lower_binary_op_expr(bin_op_expr.op, lhs, rhs, expr.span)
            }
            syntax::ExprKind::Cast(_) => {
                self.push_unimplemented_error_message("cast expr", expr.span);
                err_expr!(Type::Err, expr.span)
            }
            syntax::ExprKind::Err => err_expr!(Type::Err, expr.span),
            syntax::ExprKind::FunctionCall(expr) => self.lower_function_call_expr(expr),
            syntax::ExprKind::Ident(ident) => self.lower_ident_expr(ident),
            syntax::ExprKind::IndexExpr(expr) => self.lower_index_expr(expr),

            syntax::ExprKind::Lit(lit) => self.lower_lit_expr(lit),

            syntax::ExprKind::Paren(pexpr) => self.lower_paren_expr(pexpr, expr.span),
            syntax::ExprKind::UnaryOp(expr) => self.lower_unary_op_expr(expr),
        }
    }

    fn lower_ident_expr(&mut self, ident: &syntax::Ident) -> semantic::Expr {
        let name = ident.name.clone();

        let (symbol_id, symbol) = self.try_get_existing_or_insert_err_symbol(&name, ident.span);

        // Design Note: The end goal of this const evaluation is to be able to compile qasm
        //              annotations as Q# attributes like `@SimulatableIntrinsic()`.
        //
        //              QASM3 subroutines and gates can be recursive and capture const symbols
        //              outside their scope. In Q#, only lambdas can capture symbols, but only
        //              proper functions and operations can be recursive or have attributes on
        //              them. To get both, annotations & recursive gates/functions and the
        //              ability to capture const symbols outside the gate/function scope, we
        //              decided to compile the gates/functions as proper Q# operations/functions
        //              and evaluate at lowering-time all references to const symbols outside
        //              the current gate/function scope.

        // This is true if we are inside any gate or function scope.
        let is_symbol_inside_gate_or_function_scope =
            self.symbols.is_scope_rooted_in_gate_or_subroutine();

        // This is true if the symbol is outside the most inner gate or function scope.
        let is_symbol_outside_most_inner_gate_or_function_scope = self
            .symbols
            .is_symbol_outside_most_inner_gate_or_function_scope(symbol_id);

        let is_const_evaluation_necessary = symbol.is_const()
            && is_symbol_inside_gate_or_function_scope
            && is_symbol_outside_most_inner_gate_or_function_scope;

        let kind = if is_const_evaluation_necessary {
            if let Some(val) = symbol.get_const_expr().const_eval(&self.symbols) {
                semantic::ExprKind::Lit(val)
            } else {
                self.push_semantic_error(SemanticErrorKind::ExprMustBeConst(
                    ident.name.to_string(),
                    ident.span,
                ));
                semantic::ExprKind::Err
            }
        } else {
            semantic::ExprKind::Ident(symbol_id)
        };

        semantic::Expr {
            span: ident.span,
            kind: Box::new(kind),
            ty: symbol.ty.clone(),
        }
    }

    fn lower_lit_expr(&mut self, expr: &syntax::Lit) -> semantic::Expr {
        let (kind, ty) = match &expr.kind {
            syntax::LiteralKind::BigInt(value) => {
                // this case is only valid when there is an integer literal
                // that requires more than 64 bits to represent. We should probably
                // introduce a new type for this as openqasm promotion rules don't
                // cover this case as far as I know.
                (
                    semantic::ExprKind::Lit(semantic::LiteralKind::BigInt(value.clone())),
                    Type::Err,
                )
            }
            syntax::LiteralKind::Bitstring(value, size) => (
                semantic::ExprKind::Lit(semantic::LiteralKind::Bitstring(value.clone(), *size)),
                Type::BitArray(super::types::ArrayDimensions::from(*size), true),
            ),
            syntax::LiteralKind::Bool(value) => (
                semantic::ExprKind::Lit(semantic::LiteralKind::Bool(*value)),
                Type::Bool(true),
            ),
            syntax::LiteralKind::Int(value) => (
                semantic::ExprKind::Lit(semantic::LiteralKind::Int(*value)),
                Type::Int(None, true),
            ),
            syntax::LiteralKind::Float(value) => (
                semantic::ExprKind::Lit(semantic::LiteralKind::Float(*value)),
                Type::Float(None, true),
            ),
            syntax::LiteralKind::Imaginary(value) => (
                semantic::ExprKind::Lit(semantic::LiteralKind::Complex(0.0, *value)),
                Type::Complex(None, true),
            ),
            syntax::LiteralKind::String(_) => {
                self.push_unsupported_error_message("String literals", expr.span);
                (semantic::ExprKind::Err, Type::Err)
            }
            syntax::LiteralKind::Duration(value, time_unit) => (
                semantic::ExprKind::Lit(semantic::LiteralKind::Duration(
                    *value,
                    (*time_unit).into(),
                )),
                Type::Duration(true),
            ),
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
                    .map(|expr| self.lower_expr(expr))
                    .collect::<Vec<_>>();

                (
                    semantic::ExprKind::Lit(semantic::LiteralKind::Array(syntax::list_from_iter(
                        texprs,
                    ))),
                    Type::Err,
                )
            }
        };
        semantic::Expr {
            span: expr.span,
            kind: Box::new(kind),
            ty,
        }
    }

    fn lower_paren_expr(&mut self, expr: &syntax::Expr, span: Span) -> semantic::Expr {
        let expr = self.lower_expr(expr);
        let ty = expr.ty.clone();
        let kind = semantic::ExprKind::Paren(expr);
        semantic::Expr {
            span,
            kind: Box::new(kind),
            ty,
        }
    }

    fn lower_unary_op_expr(&mut self, expr: &syntax::UnaryOpExpr) -> semantic::Expr {
        match expr.op {
            syntax::UnaryOp::Neg => {
                let op = expr.op;
                let expr = self.lower_expr(&expr.expr);
                let ty = expr.ty.clone();
                if !unary_op_can_be_applied_to_type(op, &ty) {
                    let kind = SemanticErrorKind::TypeDoesNotSupportedUnaryNegation(
                        expr.ty.to_string(),
                        expr.span,
                    );
                    self.push_semantic_error(kind);
                }
                let span = expr.span;
                let unary = semantic::UnaryOpExpr {
                    span,
                    op: semantic::UnaryOp::Neg,
                    expr,
                };
                semantic::Expr {
                    span,
                    kind: Box::new(semantic::ExprKind::UnaryOp(unary)),
                    ty,
                }
            }
            syntax::UnaryOp::NotB => {
                let op = expr.op;
                let expr = self.lower_expr(&expr.expr);
                let ty = expr.ty.clone();
                if !unary_op_can_be_applied_to_type(op, &ty) {
                    let kind = SemanticErrorKind::TypeDoesNotSupportedUnaryNegation(
                        expr.ty.to_string(),
                        expr.span,
                    );
                    self.push_semantic_error(kind);
                }
                let span = expr.span;
                let unary = semantic::UnaryOpExpr {
                    span,
                    op: semantic::UnaryOp::NotB,
                    expr,
                };
                semantic::Expr {
                    span,
                    kind: Box::new(semantic::ExprKind::UnaryOp(unary)),
                    ty,
                }
            }
            syntax::UnaryOp::NotL => {
                // this is the  only unary operator that tries to coerce the type
                // I can't find it in the spec, but when looking at existing code
                // it seems that the ! operator coerces to a bool if possible
                let expr = self.lower_expr(&expr.expr);
                let expr_span = expr.span;
                let target_ty = Type::Bool(expr.ty.is_const());

                let expr =
                    self.cast_expr_with_target_type_or_default(Some(expr), &target_ty, expr_span);

                let ty = expr.ty.clone();

                semantic::Expr {
                    span: expr.span,
                    kind: Box::new(semantic::ExprKind::UnaryOp(semantic::UnaryOpExpr {
                        span: expr.span,
                        op: semantic::UnaryOp::NotL,
                        expr,
                    })),
                    ty,
                }
            }
        }
    }

    fn lower_annotations(annotations: &[Box<syntax::Annotation>]) -> Vec<semantic::Annotation> {
        annotations
            .iter()
            .map(|annotation| Self::lower_annotation(annotation))
            .collect::<Vec<_>>()
    }

    fn lower_annotation(annotation: &syntax::Annotation) -> semantic::Annotation {
        semantic::Annotation {
            span: annotation.span,
            identifier: annotation.identifier.clone(),
            value: annotation.value.as_ref().map(Clone::clone),
        }
    }

    fn convert_semantic_type_to_qsharp_type(
        &mut self,
        ty: &super::types::Type,
        span: Span,
    ) -> crate::types::Type {
        let is_const = ty.is_const();
        match ty {
            Type::Bit(_) => crate::types::Type::Result(is_const),
            Type::Qubit => crate::types::Type::Qubit,
            Type::HardwareQubit => {
                let message = "HardwareQubit to Q# type";
                self.push_unsupported_error_message(message, span);
                crate::types::Type::Err
            }
            Type::Int(width, _) | Type::UInt(width, _) => {
                if let Some(width) = width {
                    if *width > 64 {
                        crate::types::Type::BigInt(is_const)
                    } else {
                        crate::types::Type::Int(is_const)
                    }
                } else {
                    crate::types::Type::Int(is_const)
                }
            }
            Type::Float(_, _) => crate::types::Type::Double(is_const),
            Type::Angle(_, _) => crate::types::Type::Angle(is_const),
            Type::Complex(_, _) => crate::types::Type::Complex(is_const),
            Type::Bool(_) => crate::types::Type::Bool(is_const),
            Type::Duration(_) => {
                self.push_unsupported_error_message("Duration type values", span);
                crate::types::Type::Err
            }
            Type::Stretch(_) => {
                self.push_unsupported_error_message("Stretch type values", span);
                crate::types::Type::Err
            }
            Type::BitArray(dims, _) => crate::types::Type::ResultArray(dims.into(), is_const),
            Type::QubitArray(dims) => crate::types::Type::QubitArray(dims.into()),
            Type::IntArray(size, dims) | Type::UIntArray(size, dims) => {
                if let Some(size) = size {
                    if *size > 64 {
                        crate::types::Type::BigIntArray(dims.into(), is_const)
                    } else {
                        crate::types::Type::IntArray(dims.into(), is_const)
                    }
                } else {
                    crate::types::Type::IntArray(dims.into(), is_const)
                }
            }
            Type::FloatArray(_, dims) => crate::types::Type::DoubleArray(dims.into()),
            Type::BoolArray(dims) => crate::types::Type::BoolArray(dims.into(), is_const),
            Type::Gate(cargs, qargs) => {
                crate::types::Type::Callable(crate::types::CallableKind::Operation, *cargs, *qargs)
            }
            Type::Range => crate::types::Type::Range,
            Type::Void => crate::types::Type::Tuple(vec![]),
            Type::Err => crate::types::Type::Err,
            _ => {
                let msg = format!("Converting {ty:?} to Q# type");
                self.push_unimplemented_error_message(msg, span);
                crate::types::Type::Err
            }
        }
    }

    fn lower_barrier_stmt(&mut self, stmt: &syntax::BarrierStmt) -> semantic::StmtKind {
        let qubits = stmt.qubits.iter().map(|q| self.lower_gate_operand(q));
        let qubits = list_from_iter(qubits);
        semantic::StmtKind::Barrier(semantic::BarrierStmt {
            span: stmt.span,
            qubits,
        })
    }

    /// The "boxable" stmts were taken from the reference parser at
    /// <https://github.com/openqasm/openqasm/blob/main/source/openqasm/openqasm3/ast.py>.
    /// Search for the definition of `Box` there, and then for all the classes
    /// inhereting from `QuantumStatement`.
    fn lower_box(&mut self, stmt: &syntax::BoxStmt) -> semantic::StmtKind {
        let _stmts = stmt
            .body
            .iter()
            .map(|stmt| self.lower_stmt(stmt))
            .collect::<Vec<_>>();

        let mut _has_invalid_stmt_kinds = false;
        for stmt in &stmt.body {
            match &*stmt.kind {
                syntax::StmtKind::Barrier(_)
                | syntax::StmtKind::Delay(_)
                | syntax::StmtKind::Reset(_)
                | syntax::StmtKind::GateCall(_)
                | syntax::StmtKind::GPhase(_)
                | syntax::StmtKind::Box(_) => {
                    // valid statements
                }
                _ => {
                    self.push_semantic_error(SemanticErrorKind::ClassicalStmtInBox(stmt.span));
                    _has_invalid_stmt_kinds = true;
                }
            }
        }

        if let Some(duration) = &stmt.duration {
            self.push_unsupported_error_message("Box with duration", duration.span);
        }

        // we semantically checked the stmts, but we still need to lower them
        // with the correct behavior based on any pragmas that might be present
        self.push_unimplemented_error_message("box stmt", stmt.span);
        semantic::StmtKind::Err
    }

    fn lower_break(&mut self, stmt: &syntax::BreakStmt) -> semantic::StmtKind {
        if self.symbols.is_scope_rooted_in_loop_scope() {
            semantic::StmtKind::Break(semantic::BreakStmt { span: stmt.span })
        } else {
            self.push_semantic_error(SemanticErrorKind::InvalidScope(
                "break".into(),
                "loop".into(),
                stmt.span,
            ));
            semantic::StmtKind::Err
        }
    }

    fn lower_block(&mut self, stmt: &syntax::Block) -> semantic::Block {
        self.symbols.push_scope(ScopeKind::Block);
        let stmts = stmt.stmts.iter().map(|stmt| self.lower_stmt(stmt));
        let stmts = list_from_iter(stmts);
        self.symbols.pop_scope();

        semantic::Block {
            span: stmt.span,
            stmts,
        }
    }

    fn lower_calibration(&mut self, stmt: &syntax::CalibrationStmt) -> semantic::StmtKind {
        self.push_unimplemented_error_message("calibration stmt", stmt.span);
        semantic::StmtKind::Err
    }

    fn lower_calibration_grammar(
        &mut self,
        stmt: &syntax::CalibrationGrammarStmt,
    ) -> semantic::StmtKind {
        self.push_unimplemented_error_message("calibration grammar stmt", stmt.span);
        semantic::StmtKind::Err
    }

    fn lower_classical_decl(
        &mut self,
        stmt: &syntax::ClassicalDeclarationStmt,
    ) -> semantic::StmtKind {
        let is_const = false; // const decls are handled separately
        let ty = self.get_semantic_type_from_tydef(&stmt.ty, is_const);

        let init_expr = stmt.init_expr.as_deref();
        let ty_span = stmt.ty.span();
        let stmt_span = stmt.span;
        let name = stmt.identifier.name.clone();
        let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty.clone(), ty_span);
        let symbol = Symbol::new(
            &name,
            stmt.identifier.span,
            ty.clone(),
            qsharp_ty,
            IOKind::Default,
        );

        // process the symbol and init_expr gathering any errors
        let init_expr = match init_expr {
            Some(expr) => match expr {
                syntax::ValueExpr::Expr(expr) => {
                    let expr = self.lower_expr(expr);
                    self.cast_expr_with_target_type_or_default(Some(expr), &ty, stmt_span)
                }
                syntax::ValueExpr::Measurement(measure_expr) => {
                    let expr = self.lower_measure_expr(measure_expr);
                    self.cast_expr_to_type(&ty, &expr)
                }
            },
            None => self.cast_expr_with_target_type_or_default(None, &ty, stmt_span),
        };

        let symbol_id = self.try_insert_or_get_existing_symbol_id(name, symbol);

        semantic::StmtKind::ClassicalDecl(semantic::ClassicalDeclarationStmt {
            span: stmt_span,
            ty_span,
            symbol_id,
            init_expr: Box::new(init_expr),
        })
    }

    fn lower_const_decl(&mut self, stmt: &syntax::ConstantDeclStmt) -> semantic::StmtKind {
        let is_const = true;
        let ty = self.get_semantic_type_from_tydef(&stmt.ty, is_const);
        let ty_span = stmt.ty.span();
        let name = stmt.identifier.name.clone();
        let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty.clone(), stmt.ty.span());
        let init_expr = match &stmt.init_expr {
            syntax::ValueExpr::Expr(expr) => {
                let expr = self.lower_expr(expr);
                self.cast_expr_with_target_type_or_default(Some(expr), &ty, stmt.span)
            }
            syntax::ValueExpr::Measurement(measure_expr) => self.lower_measure_expr(measure_expr),
        };

        let mut symbol = Symbol::new(
            &name,
            stmt.identifier.span,
            ty.clone(),
            qsharp_ty,
            IOKind::Default,
        );

        if init_expr.ty.is_const() {
            symbol = symbol.with_const_expr(Rc::new(init_expr.clone()));
        }

        let symbol_id = self.try_insert_or_get_existing_symbol_id(name, symbol);

        if !init_expr.ty.is_const() {
            self.push_semantic_error(SemanticErrorKind::ExprMustBeConst(
                "const decl init expr".to_string(),
                init_expr.span,
            ));
        }

        semantic::StmtKind::ClassicalDecl(semantic::ClassicalDeclarationStmt {
            span: stmt.span,
            ty_span,
            symbol_id,
            init_expr: Box::new(init_expr),
        })
    }

    fn lower_continue_stmt(&mut self, stmt: &syntax::ContinueStmt) -> semantic::StmtKind {
        if self.symbols.is_scope_rooted_in_loop_scope() {
            semantic::StmtKind::Continue(semantic::ContinueStmt { span: stmt.span })
        } else {
            self.push_semantic_error(SemanticErrorKind::InvalidScope(
                "continue".into(),
                "loop".into(),
                stmt.span,
            ));
            semantic::StmtKind::Err
        }
    }

    fn lower_def(&mut self, stmt: &syntax::DefStmt) -> semantic::StmtKind {
        // 1. Check that we are in the global scope. QASM3 semantics
        //    only allow def declarations in the global scope.
        if !self.symbols.is_current_scope_global() {
            let kind = SemanticErrorKind::DefDeclarationInNonGlobalScope(stmt.span);
            self.push_semantic_error(kind);
        }

        // 2. Build the parameter's type.
        let mut param_types = Vec::with_capacity(stmt.params.len());
        let mut param_symbols = Vec::with_capacity(stmt.params.len());

        for param in &stmt.params {
            let symbol = self.lower_typed_parameter(param);
            param_types.push(symbol.ty.clone());
            param_symbols.push(symbol);
        }

        // 3. Build the return type.
        let (return_ty, qsharp_return_ty) = if let Some(ty) = &stmt.return_type {
            let ty_span = ty.span;
            let tydef = syntax::TypeDef::Scalar(*ty.clone());
            let ty = self.get_semantic_type_from_tydef(&tydef, false);
            let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty, ty_span);
            (ty, qsharp_ty)
        } else {
            (
                crate::semantic::types::Type::Void,
                crate::types::Type::Tuple(Default::default()),
            )
        };

        // 2. Push the function symbol to the symbol table.
        #[allow(clippy::cast_possible_truncation)]
        let arity = stmt.params.len() as u32;
        let name = stmt.name.name.clone();
        let name_span = stmt.name.span;
        let ty = crate::semantic::types::Type::Function(param_types.into(), Rc::new(return_ty));

        let has_qubit_params = stmt
            .params
            .iter()
            .any(|arg| matches!(&**arg, syntax::TypedParameter::Quantum(..)));

        let kind = if has_qubit_params {
            crate::types::CallableKind::Operation
        } else {
            crate::types::CallableKind::Function
        };

        let qsharp_ty = crate::types::Type::Callable(kind, arity, 0);

        let symbol = Symbol::new(&name, name_span, ty, qsharp_ty, IOKind::Default);
        let symbol_id = self.try_insert_or_get_existing_symbol_id(name, symbol);

        // Push the scope where the def lives.
        self.symbols.push_scope(ScopeKind::Function);

        let params = param_symbols
            .into_iter()
            .map(|symbol| {
                let name = symbol.name.clone();
                self.try_insert_or_get_existing_symbol_id(name, symbol)
            })
            .collect();

        let body = semantic::Block {
            span: stmt.span,
            stmts: list_from_iter(stmt.body.stmts.iter().map(|stmt| self.lower_stmt(stmt))),
        };

        // Pop the scope where the def lives.
        self.symbols.pop_scope();

        semantic::StmtKind::Def(semantic::DefStmt {
            span: stmt.span,
            symbol_id,
            has_qubit_params,
            params,
            body,
            return_type: qsharp_return_ty,
        })
    }

    fn lower_typed_parameter(&mut self, typed_param: &syntax::TypedParameter) -> Symbol {
        match typed_param {
            syntax::TypedParameter::ArrayReference(param) => {
                self.lower_array_reference_parameter(param)
            }
            syntax::TypedParameter::Quantum(param) => self.lower_quantum_parameter(param),
            syntax::TypedParameter::Scalar(param) => self.lower_scalar_parameter(param),
        }
    }

    fn lower_array_reference_parameter(
        &mut self,
        typed_param: &syntax::ArrayTypedParameter,
    ) -> Symbol {
        let tydef = syntax::TypeDef::ArrayReference(*typed_param.ty.clone());
        let ty = self.get_semantic_type_from_tydef(&tydef, false);
        let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty, typed_param.ty.span);

        Symbol::new(
            &typed_param.ident.name,
            typed_param.ident.span,
            ty,
            qsharp_ty,
            IOKind::Default,
        )
    }

    fn lower_quantum_parameter(&mut self, typed_param: &syntax::QuantumTypedParameter) -> Symbol {
        let (ty, qsharp_ty) = if let Some(size) = &typed_param.size {
            if let Some(size) = self.const_eval_array_size_designator_from_expr(size) {
                let ty = crate::semantic::types::Type::QubitArray(ArrayDimensions::One(size));
                let qsharp_ty = crate::types::Type::QubitArray(crate::types::ArrayDimensions::One(
                    size as usize,
                ));
                (ty, qsharp_ty)
            } else {
                (crate::semantic::types::Type::Err, crate::types::Type::Err)
            }
        } else {
            (
                crate::semantic::types::Type::Qubit,
                crate::types::Type::Qubit,
            )
        };

        Symbol::new(
            &typed_param.ident.name,
            typed_param.ident.span,
            ty,
            qsharp_ty,
            IOKind::Default,
        )
    }

    fn lower_scalar_parameter(&mut self, typed_param: &syntax::ScalarTypedParameter) -> Symbol {
        let tydef = syntax::TypeDef::Scalar(*typed_param.ty.clone());
        let ty = self.get_semantic_type_from_tydef(&tydef, false);
        let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty, typed_param.ty.span);

        Symbol::new(
            &typed_param.ident.name,
            typed_param.ident.span,
            ty,
            qsharp_ty,
            IOKind::Default,
        )
    }

    fn lower_def_cal(&mut self, stmt: &syntax::DefCalStmt) -> semantic::StmtKind {
        self.push_unimplemented_error_message("def cal stmt", stmt.span);
        semantic::StmtKind::Err
    }

    fn lower_delay(&mut self, stmt: &syntax::DelayStmt) -> semantic::StmtKind {
        self.push_unimplemented_error_message("delay stmt", stmt.span);
        semantic::StmtKind::Err
    }

    fn lower_end_stmt(stmt: &syntax::EndStmt) -> semantic::StmtKind {
        semantic::StmtKind::End(semantic::EndStmt { span: stmt.span })
    }

    fn lower_expr_stmt(&mut self, stmt: &syntax::ExprStmt) -> semantic::StmtKind {
        let expr = self.lower_expr(&stmt.expr);
        semantic::StmtKind::ExprStmt(semantic::ExprStmt {
            span: stmt.span,
            expr,
        })
    }

    fn lower_extern(&mut self, stmt: &syntax::ExternDecl) -> semantic::StmtKind {
        self.push_unimplemented_error_message("extern stmt", stmt.span);
        semantic::StmtKind::Err
    }

    fn lower_for_stmt(&mut self, stmt: &syntax::ForStmt) -> semantic::StmtKind {
        let set_declaration = self.lower_enumerable_set(&stmt.set_declaration);

        // Push scope where the loop variable lives.
        self.symbols.push_scope(ScopeKind::Loop);

        let ty = self.get_semantic_type_from_scalar_ty(&stmt.ty, false);
        let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty.clone(), stmt.ty.span);
        let symbol = Symbol::new(
            &stmt.ident.name,
            stmt.ident.span,
            ty.clone(),
            qsharp_ty,
            IOKind::Default,
        );

        // This is the first variable in this scope, so
        // we don't need to check for redefined symbols.
        let symbol_id = self
            .symbols
            .insert_symbol(symbol)
            .expect("this should be the first variable in this scope");

        // We lower the body after registering the loop variable symbol_id.
        // The body of the for loop could be a single statement redefining
        // the loop variable, in which case we need to push a redefined
        // symbol error.
        let body = self.lower_stmt(&stmt.body);

        // Pop the scope where the loop variable lives.
        self.symbols.pop_scope();

        semantic::StmtKind::For(semantic::ForStmt {
            span: stmt.span,
            loop_variable: symbol_id,
            set_declaration: Box::new(set_declaration),
            body,
        })
    }

    fn lower_if_stmt(&mut self, stmt: &syntax::IfStmt) -> semantic::StmtKind {
        let condition = self.lower_expr(&stmt.condition);
        let if_body = self.lower_stmt(&stmt.if_body);
        let else_body = stmt.else_body.as_ref().map(|body| self.lower_stmt(body));

        // The semantics of a if statement is that the condition must be
        // of type bool, so we try to cast it, inserting a cast if necessary.
        let cond_ty = Type::Bool(condition.ty.is_const());
        let condition = self.cast_expr_to_type(&cond_ty, &condition);

        semantic::StmtKind::If(semantic::IfStmt {
            span: stmt.span,
            condition,
            if_body,
            else_body,
        })
    }

    fn lower_function_call_expr(&mut self, expr: &syntax::FunctionCall) -> semantic::Expr {
        // 1. Check that the function name actually refers to a function
        //    in the symbol table and get its symbol_id & symbol.
        let name = expr.name.name.clone();
        let name_span = expr.name.span;
        let (symbol_id, symbol) = self.try_get_existing_or_insert_err_symbol(name, name_span);

        let (params_ty, return_ty) = if let Type::Function(params_ty, return_ty) = &symbol.ty {
            let arity = params_ty.len();

            // 2. Check that function classical arity matches the number of classical args.
            if arity != expr.args.len() {
                self.push_semantic_error(SemanticErrorKind::InvalidNumberOfClassicalArgs(
                    arity,
                    expr.args.len(),
                    expr.span,
                ));
            }

            (params_ty.clone(), (**return_ty).clone())
        } else {
            self.push_semantic_error(SemanticErrorKind::CannotCallNonFunction(symbol.span));
            (Rc::default(), crate::semantic::types::Type::Err)
        };

        // 3. Lower the args. There are three cases.
        // 3.1 If there are fewer args than the arity of the function.

        // 3.2 If the number of args and the arity match.

        // 3.3 If there are more args than the arity of the function.
        let mut params_ty_iter = params_ty.iter();
        let args = expr.args.iter().map(|arg| {
            let arg = self.lower_expr(arg);
            if let Some(ty) = params_ty_iter.next() {
                self.cast_expr_to_type(ty, &arg)
            } else {
                arg
            }
        });
        let args = list_from_iter(args);

        let kind = Box::new(semantic::ExprKind::FunctionCall(semantic::FunctionCall {
            span: expr.span,
            symbol_id,
            args,
        }));

        semantic::Expr {
            span: expr.span,
            kind,
            ty: return_ty,
        }
    }

    fn lower_gate_call_stmt(&mut self, stmt: &syntax::GateCall) -> semantic::StmtKind {
        // 1. Lower all the fields:
        //   1.1. Lower the modifiers.
        let mut modifiers = stmt
            .modifiers
            .iter()
            .filter_map(|modifier| self.lower_modifier(modifier))
            .collect::<Vec<_>>();
        // If we couldn't compute the modifiers there is no way to compile the gates
        // correctly, since we can't check its arity. In this case we return an Err.
        if modifiers.len() != stmt.modifiers.len() {
            return semantic::StmtKind::Err;
        }

        //   1.3. Lower the args.
        let args = stmt.args.iter().map(|arg| {
            let arg = self.lower_expr(arg);
            self.cast_expr_to_type(&crate::semantic::types::Type::Angle(None, false), &arg)
        });
        let args = list_from_iter(args);
        //   1.4. Lower the qubits.
        let qubits = stmt.qubits.iter().map(|q| self.lower_gate_operand(q));
        let qubits = list_from_iter(qubits);
        //   1.5. Lower the duration.
        let duration = stmt.duration.as_ref().map(|d| self.lower_expr(d));

        if let Some(duration) = &duration {
            self.push_unsupported_error_message("gate call duration", duration.span);
        }

        let mut name = stmt.name.name.to_string();
        if let Some((qsharp_name, implicit_modifier)) =
            try_get_qsharp_name_and_implicit_modifiers(&name, stmt.name.span)
        {
            // Override the gate name if Q# name is another.
            name = qsharp_name;

            // 2. Get implicit modifiers and make them explicit.
            //    Q: Do we need this during lowering?
            //    A: Yes, we need it to check the gate_call arity.
            modifiers.push(implicit_modifier);
        }

        // 3. Check that the gate_name actually refers to a gate in the symbol table
        //    and get its symbol_id & symbol. Make sure to use the name that could've
        //    been overriden by the Q# name and the span of the original name.
        let (symbol_id, symbol) = self.try_get_existing_or_insert_err_symbol(name, stmt.name.span);

        let (classical_arity, quantum_arity) =
            if let Type::Gate(classical_arity, quantum_arity) = &symbol.ty {
                (*classical_arity, *quantum_arity)
            } else {
                self.push_semantic_error(SemanticErrorKind::CannotCallNonGate(symbol.span));
                (0, 0)
            };

        // 4. Check that gate_call classical arity matches the number of classical args.
        if classical_arity as usize != args.len() {
            self.push_semantic_error(SemanticErrorKind::InvalidNumberOfClassicalArgs(
                classical_arity as usize,
                args.len(),
                stmt.span,
            ));
        }

        // 5. Check that gate_call quantum arity with modifiers matches the
        //    number of qubit args.
        let mut quantum_arity_with_modifiers = quantum_arity;
        for modifier in &modifiers {
            match &modifier.kind {
                semantic::GateModifierKind::Inv | semantic::GateModifierKind::Pow(_) => (),
                semantic::GateModifierKind::Ctrl(n) | semantic::GateModifierKind::NegCtrl(n) => {
                    quantum_arity_with_modifiers += n;
                }
            }
        }

        if quantum_arity_with_modifiers as usize != qubits.len() {
            self.push_semantic_error(SemanticErrorKind::InvalidNumberOfQubitArgs(
                quantum_arity_with_modifiers as usize,
                qubits.len(),
                stmt.span,
            ));
        }

        // 6. Return:
        //   6.1. Gate symbol_id.
        //   6.2. All controls made explicit.
        //   6.3. Classical args.
        //   6.4. Quantum args in the order expected by the compiler.
        modifiers.reverse();
        let modifiers = list_from_iter(modifiers);
        semantic::StmtKind::GateCall(semantic::GateCall {
            span: stmt.span,
            modifiers,
            symbol_id,
            args,
            qubits,
            duration,
            classical_arity,
            quantum_arity,
        })

        // The compiler will be left to do all things that need explicit Q# knowledge.
        // But it won't need to check arities, know about implicit modifiers, or do
        // any casting of classical args. There is still some inherit complexity to
        // building a Q# gate call with this information, but it won't be cluttered
        // by all the semantic analysis.
    }

    /// This is just syntax sugar around a gate call.
    fn lower_gphase_stmt(&mut self, stmt: &syntax::GPhase) -> semantic::StmtKind {
        let name = syntax::Ident {
            span: stmt.gphase_token_span,
            name: "gphase".into(),
        };
        let gate_call_stmt = syntax::GateCall {
            span: stmt.span,
            modifiers: stmt.modifiers.clone(),
            name,
            args: stmt.args.clone(),
            qubits: stmt.qubits.clone(),
            duration: stmt.duration.clone(),
        };
        self.lower_gate_call_stmt(&gate_call_stmt)
    }

    fn lower_modifier(
        &mut self,
        modifier: &syntax::QuantumGateModifier,
    ) -> Option<semantic::QuantumGateModifier> {
        let kind = match &modifier.kind {
            syntax::GateModifierKind::Inv => semantic::GateModifierKind::Inv,
            syntax::GateModifierKind::Pow(expr) => {
                semantic::GateModifierKind::Pow(self.lower_expr(expr))
            }
            syntax::GateModifierKind::Ctrl(expr) => {
                let ctrl_args = self.lower_modifier_ctrl_args(expr.as_ref())?;
                semantic::GateModifierKind::Ctrl(ctrl_args)
            }
            syntax::GateModifierKind::NegCtrl(expr) => {
                let ctrl_args = self.lower_modifier_ctrl_args(expr.as_ref())?;
                semantic::GateModifierKind::NegCtrl(ctrl_args)
            }
        };

        Some(semantic::QuantumGateModifier {
            span: modifier.span,
            kind,
        })
    }

    fn lower_modifier_ctrl_args(&mut self, expr: Option<&syntax::Expr>) -> Option<u32> {
        let Some(expr) = expr else {
            return Some(1);
        };

        let expr = self.lower_expr(expr);

        let target_ty = &Type::UInt(None, true);
        let Some(expr) = Self::try_cast_expr_to_type(target_ty, &expr) else {
            self.push_invalid_cast_error(target_ty, &expr.ty, expr.span);
            return None;
        };
        let Some(lit) = expr.const_eval(&self.symbols) else {
            self.push_semantic_error(SemanticErrorKind::ExprMustBeConst(
                "ctrl modifier argument".into(),
                expr.span,
            ));
            return None;
        };

        let semantic::LiteralKind::Int(n) = lit else {
            unreachable!("we casted the expr to UInt before const evaluating it")
        };

        let Ok(n) = u32::try_from(n) else {
            self.push_semantic_error(SemanticErrorKind::ExprMustFitInU32(
                "ctrl modifier argument".into(),
                expr.span,
            ));
            return None;
        };

        Some(n)
    }

    /// This function is always a indication of a error. Either the
    /// program is declaring include in a non-global scope or the
    /// include is not handled in `self.lower_source` properly.
    fn lower_include(&mut self, stmt: &syntax::IncludeStmt) -> semantic::StmtKind {
        // if we are not in the root we should not be able to include
        if !self.symbols.is_current_scope_global() {
            let name = stmt.filename.to_string();
            let kind = SemanticErrorKind::IncludeNotInGlobalScope(name, stmt.span);
            self.push_semantic_error(kind);
            return semantic::StmtKind::Err;
        }
        // if we are at the root and we have an include, we should have
        // already handled it and we are in an invalid state
        panic!("Include should have been handled in lower_source")
    }

    fn lower_io_decl(&mut self, stmt: &syntax::IODeclaration) -> semantic::StmtKind {
        let is_const = false;
        let ty = self.get_semantic_type_from_tydef(&stmt.ty, is_const);
        let io_kind = stmt.io_identifier.into();

        assert!(
            io_kind == IOKind::Input || io_kind == IOKind::Output,
            "IOKind should be Input or Output"
        );

        let ty_span = stmt.ty.span();
        let stmt_span = stmt.span;
        let name = stmt.ident.name.clone();
        let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty, ty_span);
        let symbol = Symbol::new(&name, stmt.ident.span, ty.clone(), qsharp_ty, io_kind);

        let symbol_id = self.try_insert_or_get_existing_symbol_id(name, symbol);

        if io_kind == IOKind::Input {
            return semantic::StmtKind::InputDeclaration(semantic::InputDeclaration {
                span: stmt_span,
                symbol_id,
            });
        }

        // if we have output, we need to assign a default value to declare the variable
        // if we have input, we can keep return none as we would promote the variable
        // to a parameter in the function signature once we generate the function
        let init_expr = self.get_default_value(&ty, stmt_span);
        semantic::StmtKind::OutputDeclaration(semantic::OutputDeclaration {
            span: stmt_span,
            ty_span,
            symbol_id,
            init_expr: Box::new(init_expr),
        })
    }

    fn lower_measure(&mut self, stmt: &syntax::MeasureArrowStmt) -> semantic::StmtKind {
        // `measure q -> c;` is syntax sugar for `c = measure q;`
        if let Some(target) = &stmt.target {
            self.lower_assign(&syntax::AssignStmt {
                span: stmt.span,
                lhs: *target.clone(),
                rhs: syntax::ValueExpr::Measurement(stmt.measurement.clone()),
            })
        } else {
            let measure = self.lower_measure_expr(&stmt.measurement);
            semantic::StmtKind::ExprStmt(semantic::ExprStmt {
                span: stmt.span,
                expr: measure,
            })
        }
    }

    fn lower_pragma(&mut self, stmt: &syntax::Pragma) -> semantic::StmtKind {
        self.push_unimplemented_error_message("pragma stmt", stmt.span);
        semantic::StmtKind::Err
    }

    fn lower_gate_def(&mut self, stmt: &syntax::QuantumGateDefinition) -> semantic::StmtKind {
        // 1. Check that we are in the global scope. QASM3 semantics
        //    only allow gate definitions in the global scope.
        if !self.symbols.is_current_scope_global() {
            let kind = SemanticErrorKind::QuantumDeclarationInNonGlobalScope(stmt.span);
            self.push_semantic_error(kind);
        }

        // 2. Push the gate symbol to the symbol table.
        #[allow(clippy::cast_possible_truncation)]
        let classical_arity = stmt.params.len() as u32;
        #[allow(clippy::cast_possible_truncation)]
        let quantum_arity = stmt.qubits.len() as u32;
        let name = stmt.ident.name.clone();
        let ty = crate::semantic::types::Type::Gate(classical_arity, quantum_arity);
        let qsharp_ty = crate::types::Type::Callable(
            crate::types::CallableKind::Operation,
            classical_arity,
            quantum_arity,
        );
        let symbol = Symbol::new(&name, stmt.ident.span, ty, qsharp_ty, IOKind::Default);
        let symbol_id = self.try_insert_or_get_existing_symbol_id(name, symbol);

        // Push the scope where the gate definition lives.
        self.symbols.push_scope(ScopeKind::Gate);

        let params = stmt
            .params
            .iter()
            .map(|arg| {
                let ty = crate::semantic::types::Type::Angle(None, false);
                let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty, Span::default());
                let symbol = Symbol::new(&arg.name, arg.span, ty, qsharp_ty, IOKind::Default);
                self.try_insert_or_get_existing_symbol_id(&arg.name, symbol)
            })
            .collect();

        let qubits = stmt
            .qubits
            .iter()
            .map(|arg| {
                let ty = crate::semantic::types::Type::Qubit;
                let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty, Span::default());
                let symbol =
                    Symbol::new(&arg.name, stmt.ident.span, ty, qsharp_ty, IOKind::Default);
                self.try_insert_or_get_existing_symbol_id(&arg.name, symbol)
            })
            .collect();

        let body = semantic::Block {
            span: stmt.span,
            stmts: list_from_iter(stmt.body.stmts.iter().map(|stmt| self.lower_stmt(stmt))),
        };

        // Pop the scope where the gate definition lives.
        self.symbols.pop_scope();

        semantic::StmtKind::QuantumGateDefinition(semantic::QuantumGateDefinition {
            span: stmt.span,
            symbol_id,
            params,
            qubits,
            body,
        })
    }

    fn lower_quantum_decl(&mut self, stmt: &syntax::QubitDeclaration) -> semantic::StmtKind {
        // If there wasn't an explicit size, infer the size to be 1.
        let (ty, size_and_span) = if let Some(size_expr) = &stmt.size {
            let size_expr = self.lower_expr(size_expr);
            let span = size_expr.span;
            let size_expr = Self::try_cast_expr_to_type(&Type::UInt(None, true), &size_expr);

            if let Some(Some(semantic::LiteralKind::Int(val))) =
                size_expr.map(|expr| expr.const_eval(&self.symbols))
            {
                if let Ok(size) = u32::try_from(val) {
                    (
                        Type::QubitArray(ArrayDimensions::One(size)),
                        Some((size, span)),
                    )
                } else {
                    let message = "quantum register size".into();
                    self.push_semantic_error(SemanticErrorKind::ExprMustFitInU32(message, span));
                    return semantic::StmtKind::Err;
                }
            } else {
                let message = "quantum register size".into();
                self.push_semantic_error(SemanticErrorKind::ExprMustBeConst(message, span));
                return semantic::StmtKind::Err;
            }
        } else {
            (Type::Qubit, None)
        };

        let name = stmt.qubit.name.clone();
        let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty.clone(), stmt.ty_span);

        let symbol = Symbol::new(
            &name,
            stmt.qubit.span,
            ty.clone(),
            qsharp_ty,
            IOKind::Default,
        );

        let symbol_id = self.try_insert_or_get_existing_symbol_id(name, symbol);

        if let Some((size, size_span)) = size_and_span {
            semantic::StmtKind::QubitArrayDecl(semantic::QubitArrayDeclaration {
                span: stmt.span,
                symbol_id,
                size,
                size_span,
            })
        } else {
            semantic::StmtKind::QubitDecl(semantic::QubitDeclaration {
                span: stmt.span,
                symbol_id,
            })
        }
    }

    fn lower_reset(&mut self, stmt: &syntax::ResetStmt) -> semantic::StmtKind {
        let operand = self.lower_gate_operand(&stmt.operand);
        semantic::StmtKind::Reset(semantic::ResetStmt {
            span: stmt.span,
            reset_token_span: stmt.reset_token_span,
            operand: Box::new(operand),
        })
    }

    fn lower_return(&mut self, stmt: &syntax::ReturnStmt) -> semantic::StmtKind {
        let expr = stmt
            .expr
            .as_ref()
            .map(|expr| match &**expr {
                syntax::ValueExpr::Expr(expr) => self.lower_expr(expr),
                syntax::ValueExpr::Measurement(expr) => self.lower_measure_expr(expr),
            })
            .map(Box::new);

        semantic::StmtKind::Return(semantic::ReturnStmt {
            span: stmt.span,
            expr,
        })
    }

    fn lower_switch(&mut self, stmt: &syntax::SwitchStmt) -> semantic::StmtKind {
        // Semantics of switch case is that the outer block doesn't introduce
        // a new scope but each case rhs does.

        // Can we add a new scope anyway to hold a temporary variable?
        // if we do that, we can refer to a new variable instead of the control
        // expr this would allow us to avoid the need to resolve the control
        // expr multiple times in the case where we have to coerce the control
        // expr to the correct type. Introducing a new variable without a new
        // scope would effect output semantics.
        let cases = stmt
            .cases
            .iter()
            .map(|case| self.lower_switch_case(case))
            .collect::<Vec<_>>();
        let default = stmt.default.as_ref().map(|d| self.lower_block(d));
        let target = self.lower_expr(&stmt.target);

        // The condition for the switch statement must be an integer type
        // so we use `cast_expr_to_type`, forcing the type to be an integer
        // type with implicit casts if necessary.
        let target_ty = Type::Int(None, target.ty.is_const());
        let target = self.cast_expr_to_type(&target_ty, &target);

        // We push a semantic error on switch statements if version is less than 3.1,
        // as they were introduced in 3.1.
        if let Some(ref version) = self.version {
            const SWITCH_MINIMUM_SUPPORTED_VERSION: semantic::Version = semantic::Version {
                major: 3,
                minor: Some(1),
                span: Span { lo: 0, hi: 0 },
            };
            if version < &SWITCH_MINIMUM_SUPPORTED_VERSION {
                self.push_unsuported_in_this_version_error_message(
                    "switch statements",
                    &SWITCH_MINIMUM_SUPPORTED_VERSION,
                    stmt.span,
                );
            }
        }

        semantic::StmtKind::Switch(semantic::SwitchStmt {
            span: stmt.span,
            target,
            cases: list_from_iter(cases),
            default,
        })
    }

    fn lower_switch_case(&mut self, switch_case: &syntax::SwitchCase) -> semantic::SwitchCase {
        let label_ty = Type::Int(None, true);
        let labels = switch_case
            .labels
            .iter()
            .map(|label| {
                // The labels for each switch case must be of integer type
                // so we use `cast_expr_to_type`, forcing the type to be an integer
                // type with implicit casts if necessary.
                let label = self.lower_expr(label);
                self.cast_expr_to_type(&label_ty, &label)
            })
            .collect::<Vec<_>>();

        let block = self.lower_block(&switch_case.block);

        semantic::SwitchCase {
            span: switch_case.span,
            labels: list_from_iter(labels),
            block,
        }
    }

    fn lower_while_stmt(&mut self, stmt: &syntax::WhileLoop) -> semantic::StmtKind {
        // Push scope where the while loop lives. The while loop needs its own scope
        // so that break and continue know if they are inside a valid scope.
        self.symbols.push_scope(ScopeKind::Loop);

        let condition = self.lower_expr(&stmt.while_condition);
        let body = self.lower_stmt(&stmt.body);

        // The semantics of a while statement is that the condition must be
        // of type bool, so we try to cast it, inserting a cast if necessary.
        let cond_ty = Type::Bool(condition.ty.is_const());
        let while_condition = self.cast_expr_to_type(&cond_ty, &condition);

        // Pop scope where the while loop lives.
        self.symbols.pop_scope();

        semantic::StmtKind::WhileLoop(semantic::WhileLoop {
            span: stmt.span,
            condition: while_condition,
            body,
        })
    }

    fn get_semantic_type_from_tydef(
        &mut self,
        ty: &syntax::TypeDef,
        is_const: bool,
    ) -> crate::semantic::types::Type {
        match ty {
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

    /// Helper function for const evaluating array sizes, type widths, and durations.
    fn const_eval_designator(&mut self, expr: &syntax::Expr) -> Option<semantic::LiteralKind> {
        let expr = self.lower_expr(expr);
        let expr_span = expr.span;
        let expr = self.cast_expr_with_target_type_or_default(
            Some(expr),
            &Type::UInt(None, true),
            expr_span,
        );

        if let Some(lit) = expr.const_eval(&self.symbols) {
            Some(lit)
        } else {
            self.push_semantic_error(SemanticErrorKind::ExprMustBeConst(
                "designator".to_string(),
                expr.span,
            ));
            None
        }
    }

    fn const_eval_array_size_designator_from_expr(&mut self, expr: &syntax::Expr) -> Option<u32> {
        let semantic::LiteralKind::Int(val) = self.const_eval_designator(expr)? else {
            self.push_semantic_error(SemanticErrorKind::ArraySizeMustBeNonNegativeConstExpr(
                expr.span,
            ));
            return None;
        };

        if val < 0 {
            self.push_semantic_error(SemanticErrorKind::ArraySizeMustBeNonNegativeConstExpr(
                expr.span,
            ));
            return None;
        };

        let Ok(val) = u32::try_from(val) else {
            self.push_semantic_error(SemanticErrorKind::DesignatorTooLarge(expr.span));
            return None;
        };

        Some(val)
    }

    fn const_eval_type_width_designator_from_expr(&mut self, expr: &syntax::Expr) -> Option<u32> {
        let semantic::LiteralKind::Int(val) = self.const_eval_designator(expr)? else {
            self.push_semantic_error(SemanticErrorKind::TypeWidthMustBePositiveIntConstExpr(
                expr.span,
            ));
            return None;
        };

        if val < 1 {
            self.push_semantic_error(SemanticErrorKind::TypeWidthMustBePositiveIntConstExpr(
                expr.span,
            ));
            return None;
        };

        let Ok(val) = u32::try_from(val) else {
            self.push_semantic_error(SemanticErrorKind::DesignatorTooLarge(expr.span));
            return None;
        };

        Some(val)
    }

    fn get_semantic_type_from_scalar_ty(
        &mut self,
        scalar_ty: &syntax::ScalarType,
        is_const: bool,
    ) -> crate::semantic::types::Type {
        match &scalar_ty.kind {
            syntax::ScalarTypeKind::Bit(bit_type) => match &bit_type.size {
                Some(size) => {
                    let Some(size) = self.const_eval_array_size_designator_from_expr(size) else {
                        return crate::semantic::types::Type::Err;
                    };
                    crate::semantic::types::Type::BitArray(
                        super::types::ArrayDimensions::from(size),
                        is_const,
                    )
                }
                None => crate::semantic::types::Type::Bit(is_const),
            },
            syntax::ScalarTypeKind::Int(int_type) => match &int_type.size {
                Some(size) => {
                    let Some(size) = self.const_eval_type_width_designator_from_expr(size) else {
                        return crate::semantic::types::Type::Err;
                    };
                    crate::semantic::types::Type::Int(Some(size), is_const)
                }
                None => crate::semantic::types::Type::Int(None, is_const),
            },
            syntax::ScalarTypeKind::UInt(uint_type) => match &uint_type.size {
                Some(size) => {
                    let Some(size) = self.const_eval_type_width_designator_from_expr(size) else {
                        return crate::semantic::types::Type::Err;
                    };
                    crate::semantic::types::Type::UInt(Some(size), is_const)
                }
                None => crate::semantic::types::Type::UInt(None, is_const),
            },
            syntax::ScalarTypeKind::Float(float_type) => match &float_type.size {
                Some(size) => {
                    let Some(size) = self.const_eval_type_width_designator_from_expr(size) else {
                        return crate::semantic::types::Type::Err;
                    };
                    if size > 64 {
                        self.push_semantic_error(SemanticErrorKind::TypeMaxWidthExceeded(
                            "float".to_string(),
                            64,
                            size as usize,
                            float_type.span,
                        ));
                        crate::semantic::types::Type::Err
                    } else {
                        crate::semantic::types::Type::Float(Some(size), is_const)
                    }
                }
                None => crate::semantic::types::Type::Float(None, is_const),
            },
            syntax::ScalarTypeKind::Complex(complex_type) => match &complex_type.base_size {
                Some(float_type) => match &float_type.size {
                    Some(size) => {
                        let Some(size) = self.const_eval_type_width_designator_from_expr(size)
                        else {
                            return crate::semantic::types::Type::Err;
                        };
                        crate::semantic::types::Type::Complex(Some(size), is_const)
                    }
                    None => crate::semantic::types::Type::Complex(None, is_const),
                },
                None => crate::semantic::types::Type::Complex(None, is_const),
            },
            syntax::ScalarTypeKind::Angle(angle_type) => match &angle_type.size {
                Some(size) => {
                    let Some(size) = self.const_eval_type_width_designator_from_expr(size) else {
                        return crate::semantic::types::Type::Err;
                    };

                    if size > 64 {
                        self.push_semantic_error(SemanticErrorKind::TypeMaxWidthExceeded(
                            "angle".to_string(),
                            64,
                            size as usize,
                            angle_type.span,
                        ));
                        crate::semantic::types::Type::Err
                    } else {
                        crate::semantic::types::Type::Angle(Some(size), is_const)
                    }
                }
                None => crate::semantic::types::Type::Angle(None, is_const),
            },
            syntax::ScalarTypeKind::BoolType => crate::semantic::types::Type::Bool(is_const),
            syntax::ScalarTypeKind::Duration => crate::semantic::types::Type::Duration(is_const),
            syntax::ScalarTypeKind::Stretch => crate::semantic::types::Type::Stretch(is_const),
            syntax::ScalarTypeKind::Err => crate::semantic::types::Type::Err,
        }
    }

    fn get_semantic_type_from_array_ty(
        &mut self,
        array_ty: &syntax::ArrayType,
        _is_const: bool,
    ) -> crate::semantic::types::Type {
        self.push_unimplemented_error_message("semantic type from array type", array_ty.span);
        crate::semantic::types::Type::Err
    }

    fn get_semantic_type_from_array_reference_ty(
        &mut self,
        array_ref_ty: &syntax::ArrayReferenceType,
        _is_const: bool,
    ) -> crate::semantic::types::Type {
        self.push_unimplemented_error_message(
            "semantic type from array refence type",
            array_ref_ty.span,
        );
        crate::semantic::types::Type::Err
    }

    fn cast_expr_with_target_type_or_default(
        &mut self,
        expr: Option<semantic::Expr>,
        ty: &Type,
        span: Span,
    ) -> semantic::Expr {
        let Some(mut rhs) = expr else {
            // In OpenQASM, classical variables may be uninitialized, but in Q#,
            // they must be initialized. We will use the default value for the type
            // to initialize the variable.
            return self.get_default_value(ty, span);
        };

        let rhs_ty = rhs.ty.clone();

        // avoid the cast
        if *ty == rhs_ty {
            // if the types are the same, we can use the rhs as is
            return rhs;
        }

        // if we have an exact type match, we can use the rhs as is
        if types_equal_except_const(ty, &rhs_ty) {
            // Since one the two exprs is non-const, we need to relax
            // the constness in the returned expr.
            rhs.ty = rhs.ty.as_non_const();
            return rhs;
        }

        // if the rhs is a literal, we can try to cast it to the target type
        // if they share the same base type.
        if let semantic::ExprKind::Lit(lit) = &*rhs.kind {
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
            return rhs;
        }
        // the lhs has a type, but the rhs may be of a different type with
        // implicit and explicit conversions. We need to cast the rhs to the
        // lhs type, but if that cast fails, we will have already pushed an error
        // and we can't proceed
        self.cast_expr_to_type(ty, &rhs)
    }

    fn lower_measure_expr(&mut self, expr: &syntax::MeasureExpr) -> semantic::Expr {
        let measurement = semantic::MeasureExpr {
            span: expr.span,
            measure_token_span: expr.measure_token_span,
            operand: self.lower_gate_operand(&expr.operand),
        };
        semantic::Expr {
            span: expr.span,
            kind: Box::new(semantic::ExprKind::Measure(measurement)),
            ty: Type::Bit(false),
        }
    }

    fn get_default_value(&mut self, ty: &Type, span: Span) -> semantic::Expr {
        use semantic::Expr;
        use semantic::ExprKind;
        use semantic::LiteralKind;
        let from_lit_kind = |kind| -> Expr {
            Expr {
                span: Span::default(),
                kind: Box::new(ExprKind::Lit(kind)),
                ty: ty.as_const(),
            }
        };
        let expr = match ty {
            Type::Angle(_, _) => Some(from_lit_kind(LiteralKind::Angle(Default::default()))),
            Type::Bit(_) => Some(from_lit_kind(LiteralKind::Bit(false))),
            Type::Int(_, _) | Type::UInt(_, _) => Some(from_lit_kind(LiteralKind::Int(0))),
            Type::Bool(_) => Some(from_lit_kind(LiteralKind::Bool(false))),
            Type::Float(_, _) => Some(from_lit_kind(LiteralKind::Float(0.0))),
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
            Type::BitArray(dims, _) => match dims {
                ArrayDimensions::One(size) => Some(from_lit_kind(
                    semantic::LiteralKind::Bitstring(BigInt::ZERO, *size),
                )),
                ArrayDimensions::Err => None,
                _ => {
                    self.push_unimplemented_error_message(
                        "multidimensional bit array default value",
                        span,
                    );
                    None
                }
            },
            Type::Duration(_) => Some(from_lit_kind(LiteralKind::Duration(
                0.0,
                semantic::TimeUnit::Ns,
            ))),
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
            Type::Gate(_, _) | Type::Function(..) | Type::Range | Type::Set | Type::Void => {
                let message = format!("Default values for {ty:?}");
                self.push_unsupported_error_message(message, span);
                None
            }
            Type::Err => None,
        };
        let Some(expr) = expr else {
            return err_expr!(ty.as_const());
        };
        expr
    }

    fn coerce_literal_expr_to_type(
        &mut self,
        ty: &Type,
        expr: &semantic::Expr,
        kind: &semantic::LiteralKind,
    ) -> semantic::Expr {
        let Some(expr) = self.try_coerce_literal_expr_to_type(ty, expr, kind) else {
            self.push_invalid_literal_cast_error(ty, &expr.ty, expr.span);
            return expr.clone();
        };
        expr
    }

    #[allow(clippy::too_many_lines)]
    fn try_coerce_literal_expr_to_type(
        &mut self,
        ty: &Type,
        rhs: &semantic::Expr,
        kind: &semantic::LiteralKind,
    ) -> Option<semantic::Expr> {
        assert!(matches!(*rhs.kind, semantic::ExprKind::Lit(..)));
        assert!(rhs.ty.is_const(), "Literals must have const types");

        if *ty == rhs.ty {
            // Base case, we shouldn't have gotten here
            // but if we did, we can just return the rhs
            return Some(rhs.clone());
        }

        if types_equal_except_const(ty, &rhs.ty) {
            // lhs isn't const, but rhs is, this is allowed
            return Some(rhs.clone());
        }
        assert!(can_cast_literal(ty, &rhs.ty) || can_cast_literal_with_value_knowledge(ty, kind));
        let lhs_ty = ty.clone();
        let rhs_ty = rhs.ty.clone();
        let span = rhs.span;

        if matches!(lhs_ty, Type::Bit(..)) {
            if let semantic::LiteralKind::Int(value) = kind {
                // can_cast_literal_with_value_knowledge guarantees that value is 0 or 1
                return Some(semantic::Expr {
                    span,
                    kind: Box::new(semantic::ExprKind::Lit(semantic::LiteralKind::Bit(
                        *value != 0,
                    ))),
                    ty: lhs_ty.as_const(),
                });
            } else if let semantic::LiteralKind::Bool(value) = kind {
                return Some(semantic::Expr {
                    span,
                    kind: Box::new(semantic::ExprKind::Lit(semantic::LiteralKind::Bit(*value))),
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
            if let semantic::LiteralKind::Int(value) = kind {
                if *value < 0 || *value >= (1 << size) {
                    return None;
                }

                let u_size = size as usize;
                let bitstring = format!("{value:0u_size$b}");
                let Ok(value) = BigInt::from_str_radix(&bitstring, 2) else {
                    return None;
                };

                return Some(semantic::Expr {
                    span,
                    kind: Box::new(semantic::ExprKind::Lit(semantic::LiteralKind::Bitstring(
                        value, size,
                    ))),
                    ty: lhs_ty.as_const(),
                });
            }
        }
        if matches!(lhs_ty, Type::UInt(..)) {
            if let semantic::LiteralKind::Int(value) = kind {
                // this should have been validated by can_cast_literal_with_value_knowledge
                return Some(semantic::Expr {
                    span,
                    kind: Box::new(semantic::ExprKind::Lit(semantic::LiteralKind::Int(*value))),
                    ty: lhs_ty.as_const(),
                });
            }
        }
        let result = match (&lhs_ty, &rhs_ty) {
            (Type::Float(..), Type::Int(..) | Type::UInt(..)) => {
                if let semantic::LiteralKind::Int(value) = kind {
                    if let Some(value) = safe_i64_to_f64(*value) {
                        return Some(semantic::Expr {
                            span,
                            kind: Box::new(semantic::ExprKind::Lit(semantic::LiteralKind::Float(
                                value,
                            ))),
                            ty: lhs_ty.as_const(),
                        });
                    }
                    self.push_semantic_error(SemanticErrorKind::InvalidCastValueRange(
                        rhs_ty.to_string(),
                        lhs_ty.to_string(),
                        span,
                    ));
                    return None;
                }
                None
            }
            (Type::Angle(width, _), Type::Float(..)) => {
                if let semantic::LiteralKind::Float(value) = kind {
                    return Some(semantic::Expr {
                        span,
                        kind: Box::new(semantic::ExprKind::Lit(semantic::LiteralKind::Angle(
                            Angle::from_f64_maybe_sized(*value, *width),
                        ))),
                        ty: lhs_ty.as_const(),
                    });
                }
                None
            }
            (Type::Float(..), Type::Float(..)) => {
                if let semantic::LiteralKind::Float(value) = kind {
                    return Some(semantic::Expr {
                        span,
                        kind: Box::new(semantic::ExprKind::Lit(semantic::LiteralKind::Float(
                            *value,
                        ))),
                        ty: lhs_ty.as_const(),
                    });
                }
                None
            }
            (Type::Complex(..), Type::Complex(..)) => {
                if let semantic::LiteralKind::Complex(real, imag) = kind {
                    return Some(semantic::Expr {
                        span,
                        kind: Box::new(semantic::ExprKind::Lit(semantic::LiteralKind::Complex(
                            *real, *imag,
                        ))),
                        ty: lhs_ty.as_const(),
                    });
                }
                None
            }
            (Type::Complex(..), Type::Float(..)) => {
                if let semantic::LiteralKind::Float(value) = kind {
                    return Some(semantic::Expr {
                        span,
                        kind: Box::new(semantic::ExprKind::Lit(semantic::LiteralKind::Complex(
                            *value, 0.0,
                        ))),
                        ty: lhs_ty.as_const(),
                    });
                }
                None
            }
            (Type::Complex(..), Type::Int(..) | Type::UInt(..)) => {
                // complex requires a double as input, so we need to
                // convert the int to a double, then create the complex
                if let semantic::LiteralKind::Int(value) = kind {
                    if let Some(value) = safe_i64_to_f64(*value) {
                        return Some(semantic::Expr {
                            span,
                            kind: Box::new(semantic::ExprKind::Lit(
                                semantic::LiteralKind::Complex(value, 0.0),
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
                    return None;
                }
                None
            }
            (Type::Bit(..), Type::Int(..) | Type::UInt(..)) => {
                // we've already checked that the value is 0 or 1
                if let semantic::LiteralKind::Int(value) = kind {
                    if *value == 0 || *value == 1 {
                        return Some(semantic::Expr {
                            span,
                            kind: Box::new(semantic::ExprKind::Lit(semantic::LiteralKind::Int(
                                *value,
                            ))),
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
                    semantic::LiteralKind::Int(value) => {
                        return Some(semantic::Expr {
                            span,
                            kind: Box::new(semantic::ExprKind::Lit(semantic::LiteralKind::Int(
                                *value,
                            ))),
                            ty: lhs_ty.as_const(),
                        });
                    }
                    semantic::LiteralKind::BigInt(value) => {
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
                        return Some(semantic::Expr {
                            span,
                            kind: Box::new(semantic::ExprKind::Lit(semantic::LiteralKind::BigInt(
                                value.clone(),
                            ))),
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

    fn cast_expr_to_type(&mut self, ty: &Type, expr: &semantic::Expr) -> semantic::Expr {
        let Some(cast_expr) = Self::try_cast_expr_to_type(ty, expr) else {
            self.push_invalid_cast_error(ty, &expr.ty, expr.span);
            return expr.clone();
        };
        cast_expr
    }

    fn try_cast_expr_to_type(ty: &Type, expr: &semantic::Expr) -> Option<semantic::Expr> {
        if *ty == expr.ty {
            // Base case, we shouldn't have gotten here
            // but if we did, we can just return the rhs
            return Some(expr.clone());
        }
        if types_equal_except_const(ty, &expr.ty) {
            if expr.ty.is_const() {
                // lhs isn't const, but rhs is, we can just return the rhs
                let mut expr = expr.clone();
                // relax constness
                expr.ty = expr.ty.as_non_const();
                return Some(expr);
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
                    return Some(wrap_expr_in_implicit_cast_expr(ty.clone(), expr.clone()));
                }

                if *w1 >= *w2 {
                    return Some(wrap_expr_in_implicit_cast_expr(ty.clone(), expr.clone()));
                }
            }
            _ => {}
        }
        // Casting of literals is handled elsewhere. This is for casting expressions
        // which cannot be bypassed and must be handled by built-in Q# casts from
        // the standard library.
        match &expr.ty {
            Type::Angle(_, _) => Self::cast_angle_expr_to_type(ty, expr),
            Type::Bit(_) => Self::cast_bit_expr_to_type(ty, expr),
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
    fn cast_angle_expr_to_type(ty: &Type, rhs: &semantic::Expr) -> Option<semantic::Expr> {
        assert!(matches!(rhs.ty, Type::Angle(..)));
        match ty {
            Type::Angle(..) | Type::Bit(..) | Type::BitArray(..) | Type::Bool(..) => {
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
    fn cast_bit_expr_to_type(ty: &Type, rhs: &semantic::Expr) -> Option<semantic::Expr> {
        assert!(matches!(rhs.ty, Type::Bit(..)));
        // There is no operand, choosing the span of the node
        // but we could use the expr span as well.
        match ty {
            &Type::Float(..) => {
                // The spec says that this cast isn't supported, but it
                // casts to other types that cast to float. For now, we'll
                // say it is invalid like the spec.
                None
            }
            &Type::Angle(..) | &Type::Bool(_) | &Type::Int(_, _) | &Type::UInt(_, _) => {
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
    fn cast_float_expr_to_type(ty: &Type, rhs: &semantic::Expr) -> Option<semantic::Expr> {
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
    fn cast_bool_expr_to_type(ty: &Type, rhs: &semantic::Expr) -> Option<semantic::Expr> {
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
    fn cast_int_expr_to_type(ty: &Type, rhs: &semantic::Expr) -> Option<semantic::Expr> {
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
        rhs: &semantic::Expr,
    ) -> Option<semantic::Expr> {
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
        lhs: semantic::Expr,
        rhs: semantic::Expr,
        span: Span,
    ) -> semantic::Expr {
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
                let target_ty = Type::UInt(None, left_type.is_const() && right_type.is_const());
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
                let bin_expr = semantic::BinaryOpExpr {
                    op: op.into(),
                    lhs,
                    rhs,
                };
                let kind = semantic::ExprKind::BinaryOp(bin_expr);
                let expr = semantic::Expr {
                    span,
                    kind: Box::new(kind),
                    ty: target_ty,
                };
                return expr;
            };
            // Now that we know the effective Uint type, we can cast the lhs and rhs
            // so that operations can be performed on them.
            let new_lhs = self.cast_expr_to_type(&ty, &lhs);
            // only cast the rhs if the operator requires symmetric conversion
            let new_rhs = if Self::binop_requires_bitwise_symmetric_conversion(op) {
                self.cast_expr_to_type(&ty, &rhs)
            } else {
                rhs
            };

            let bin_expr = semantic::BinaryOpExpr {
                lhs: new_lhs,
                rhs: new_rhs,
                op: op.into(),
            };
            let kind = semantic::ExprKind::BinaryOp(bin_expr);
            let expr = semantic::Expr {
                span,
                kind: Box::new(kind),
                ty,
            };

            let final_expr = self.cast_expr_to_type(&left_type, &expr);
            return final_expr;
        }

        // for int, uint, float, and complex, the lesser of the two types is cast to
        // the greater of the two. Within each level of complex and float, types with
        // greater width are greater than types with lower width.
        // complex > float > int/uint
        // Q# has built-in functions: IntAsDouble, IntAsBigInt to handle two cases.
        // If the width of a float is greater than 64, we can't represent it as a double.

        let ty_constness = lhs.ty.is_const() && rhs.ty.is_const();

        let (lhs, rhs, ty) = if matches!(op, syntax::BinOp::AndL | syntax::BinOp::OrL) {
            let ty = Type::Bool(ty_constness);
            let new_lhs = self.cast_expr_to_type(&ty, &lhs);
            let new_rhs = self.cast_expr_to_type(&ty, &rhs);
            (new_lhs, new_rhs, ty)
        } else if binop_requires_asymmetric_angle_op(op, &left_type, &rhs.ty) {
            if matches!(op, syntax::BinOp::Div)
                && matches!(left_type, Type::Angle(..))
                && matches!(right_type, Type::Angle(..))
            {
                // result is uint, we need to promote both sides to match width.
                let angle_ty = Type::Angle(promote_width(&left_type, &right_type), ty_constness);
                let new_lhs = self.cast_expr_to_type(&angle_ty, &lhs);
                let new_rhs = self.cast_expr_to_type(&angle_ty, &rhs);
                let int_ty = Type::UInt(angle_ty.width(), ty_constness);
                (new_lhs, new_rhs, int_ty)
            } else if matches!(left_type, Type::Angle(..)) {
                let ty = Type::Angle(left_type.width(), ty_constness);
                let new_lhs = self.cast_expr_to_type(&ty, &lhs);
                let rhs_ty = Type::UInt(ty.width(), ty_constness);
                let new_rhs = self.cast_expr_to_type(&rhs_ty, &rhs);
                (new_lhs, new_rhs, ty)
            } else {
                let lhs_ty = Type::UInt(rhs.ty.width(), ty_constness);
                let new_lhs = self.cast_expr_to_type(&lhs_ty, &lhs);
                let ty = Type::Angle(rhs.ty.width(), ty_constness);
                let new_rhs = self.cast_expr_to_type(&ty, &rhs);
                (new_lhs, new_rhs, ty)
            }
        } else if binop_requires_int_conversion_for_type(op, &left_type, &rhs.ty) {
            let ty = Type::Int(None, ty_constness);
            let new_lhs = self.cast_expr_to_type(&ty, &lhs);
            let new_rhs = self.cast_expr_to_type(&ty, &rhs);
            (new_lhs, new_rhs, ty)
        } else if requires_symmetric_conversion(op) {
            let promoted_type = try_promote_with_casting(&left_type, &right_type);
            let new_left = if promoted_type == left_type {
                lhs
            } else {
                match &lhs.kind.as_ref() {
                    semantic::ExprKind::Lit(kind) => {
                        if can_cast_literal(&promoted_type, &left_type)
                            || can_cast_literal_with_value_knowledge(&promoted_type, kind)
                        {
                            self.coerce_literal_expr_to_type(&promoted_type, &lhs, kind)
                        } else {
                            self.cast_expr_to_type(&promoted_type, &lhs)
                        }
                    }
                    _ => self.cast_expr_to_type(&promoted_type, &lhs),
                }
            };
            let new_right = if promoted_type == right_type {
                rhs
            } else {
                match &rhs.kind.as_ref() {
                    semantic::ExprKind::Lit(kind) => {
                        if can_cast_literal(&promoted_type, &right_type)
                            || can_cast_literal_with_value_knowledge(&promoted_type, kind)
                        {
                            self.coerce_literal_expr_to_type(&promoted_type, &rhs, kind)
                        } else {
                            self.cast_expr_to_type(&promoted_type, &rhs)
                        }
                    }
                    _ => self.cast_expr_to_type(&promoted_type, &rhs),
                }
            };
            (new_left, new_right, promoted_type)
        } else if binop_requires_symmetric_int_conversion(op) {
            let ty = Type::Int(None, ty_constness);
            let new_rhs = self.cast_expr_to_type(&ty, &rhs);
            (lhs, new_rhs, left_type)
        } else {
            (lhs, rhs, left_type)
        };

        // now that we have the lhs and rhs expressions, we can create the binary expression
        // but we need to check if the chosen operator is supported by the types after
        // promotion and conversion.

        let expr = if matches!(ty, Type::Complex(..)) {
            if is_complex_binop_supported(op) {
                let bin_expr = semantic::BinaryOpExpr {
                    op: op.into(),
                    lhs,
                    rhs,
                };
                let kind = semantic::ExprKind::BinaryOp(bin_expr);
                semantic::Expr {
                    span,
                    kind: Box::new(kind),
                    ty: ty.clone(),
                }
            } else {
                let kind = SemanticErrorKind::OperatorNotSupportedForTypes(
                    format!("{op:?}"),
                    format!("{ty:?}"),
                    format!("{ty:?}"),
                    span,
                );
                self.push_semantic_error(kind);
                err_expr!(ty.clone())
            }
        } else {
            let bin_expr = semantic::BinaryOpExpr {
                op: op.into(),
                lhs,
                rhs,
            };
            let kind = semantic::ExprKind::BinaryOp(bin_expr);
            semantic::Expr {
                span,
                kind: Box::new(kind),
                ty: ty.clone(),
            }
        };

        let ty = match op.into() {
            semantic::BinOp::AndL
            | semantic::BinOp::Eq
            | semantic::BinOp::Gt
            | semantic::BinOp::Gte
            | semantic::BinOp::Lt
            | semantic::BinOp::Lte
            | semantic::BinOp::Neq
            | semantic::BinOp::OrL => Type::Bool(ty_constness),
            _ => ty,
        };
        let mut expr = expr;
        expr.ty = ty;
        expr
    }

    fn binop_requires_bitwise_conversion(op: syntax::BinOp, left_type: &Type) -> bool {
        if (matches!(
            op,
            syntax::BinOp::AndB | syntax::BinOp::OrB | syntax::BinOp::XorB
        ) && matches!(
            left_type,
            Type::Bit(..) | Type::UInt(..) | Type::BitArray(ArrayDimensions::One(_), _)
        )) {
            return true;
        }
        if (matches!(op, syntax::BinOp::Shl | syntax::BinOp::Shr)
            && matches!(
                left_type,
                Type::Bit(..) | Type::UInt(..) | Type::BitArray(ArrayDimensions::One(_), _)
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
    fn lower_index_element(&mut self, index: &syntax::IndexElement) -> semantic::IndexElement {
        match index {
            syntax::IndexElement::DiscreteSet(set) => {
                semantic::IndexElement::DiscreteSet(self.lower_discrete_set(set))
            }
            syntax::IndexElement::IndexSet(set) => {
                semantic::IndexElement::IndexSet(self.lower_index_set(set))
            }
        }
    }

    fn lower_index_set_item(&mut self, item: &syntax::IndexSetItem) -> semantic::IndexSetItem {
        match item {
            syntax::IndexSetItem::RangeDefinition(range_definition) => {
                semantic::IndexSetItem::RangeDefinition(
                    self.lower_range_definition(range_definition),
                )
            }
            syntax::IndexSetItem::Expr(expr) => semantic::IndexSetItem::Expr(self.lower_expr(expr)),
            syntax::IndexSetItem::Err => semantic::IndexSetItem::Err,
        }
    }

    fn lower_enumerable_set(&mut self, set: &syntax::EnumerableSet) -> semantic::EnumerableSet {
        match set {
            syntax::EnumerableSet::DiscreteSet(set) => {
                semantic::EnumerableSet::DiscreteSet(self.lower_discrete_set(set))
            }
            syntax::EnumerableSet::RangeDefinition(range_definition) => {
                semantic::EnumerableSet::RangeDefinition(
                    self.lower_range_definition(range_definition),
                )
            }
            syntax::EnumerableSet::Expr(expr) => {
                semantic::EnumerableSet::Expr(self.lower_expr(expr))
            }
        }
    }

    fn lower_index_set(&mut self, set: &syntax::IndexSet) -> semantic::IndexSet {
        let items = set
            .values
            .iter()
            .map(|expr| self.lower_index_set_item(expr))
            .collect::<Vec<_>>();

        semantic::IndexSet {
            span: set.span,
            values: syntax::list_from_iter(items),
        }
    }

    fn lower_discrete_set(&mut self, set: &syntax::DiscreteSet) -> semantic::DiscreteSet {
        let items = set
            .values
            .iter()
            .map(|expr| self.lower_expr(expr))
            .collect::<Vec<_>>();

        semantic::DiscreteSet {
            span: set.span,
            values: list_from_iter(items),
        }
    }

    fn lower_range_definition(
        &mut self,
        range_definition: &syntax::RangeDefinition,
    ) -> semantic::RangeDefinition {
        let start = range_definition.start.as_ref().map(|e| self.lower_expr(e));
        let step = range_definition.step.as_ref().map(|e| self.lower_expr(e));
        let end = range_definition.end.as_ref().map(|e| self.lower_expr(e));

        semantic::RangeDefinition {
            span: range_definition.span,
            start,
            step,
            end,
        }
    }

    fn lower_index_expr(&mut self, expr: &syntax::IndexExpr) -> semantic::Expr {
        let collection = self.lower_expr(&expr.collection);
        let index = self.lower_index_element(&expr.index);
        let indexed_ty = self.get_indexed_type(&collection.ty, expr.span, 1);

        semantic::Expr {
            span: expr.span,
            kind: Box::new(semantic::ExprKind::IndexExpr(semantic::IndexExpr {
                span: expr.span,
                collection,
                index,
            })),
            ty: indexed_ty,
        }
    }

    fn get_indexed_type(
        &mut self,
        ty: &Type,
        span: Span,
        num_indices: usize,
    ) -> super::types::Type {
        if !ty.is_array() {
            let kind = SemanticErrorKind::CannotIndexType(format!("{ty:?}"), span);
            self.push_semantic_error(kind);
            return super::types::Type::Err;
        }

        if num_indices > ty.num_dims() {
            let kind = SemanticErrorKind::TooManyIndices(span);
            self.push_semantic_error(kind);
            return super::types::Type::Err;
        }

        let mut indexed_ty = ty.clone();
        for _ in 0..num_indices {
            let Some(ty) = indexed_ty.get_indexed_type() else {
                // we failed to get the indexed type, unknown error
                // we should have caught this earlier with the two checks above
                let kind = SemanticErrorKind::CannotIndexType(format!("{ty:?}"), span);
                self.push_semantic_error(kind);
                return super::types::Type::Err;
            };
            indexed_ty = ty;
        }
        indexed_ty
    }

    /// Lower an indexed identifier expression
    /// This is an identifier with *zero* or more indices
    /// we tranform this into two different cases:
    ///   1. An identifier with zero indices
    ///   2. An identifier with one or more index
    ///
    /// This changes the type of expression we return to simplify downstream compilation
    fn lower_indexed_ident_expr(&mut self, indexed_ident: &syntax::IndexedIdent) -> semantic::Expr {
        let ident = indexed_ident.name.clone();

        // if we have no indices, we can just lower the identifier
        if indexed_ident.indices.is_empty() {
            return self.lower_ident_expr(&ident);
        }

        let indices = indexed_ident
            .indices
            .iter()
            .map(|index| self.lower_index_element(index));
        let indices = list_from_iter(indices);

        let Some((symbol_id, lhs_symbol)) = self.symbols.get_symbol_by_name(&ident.name) else {
            self.push_missing_symbol_error(ident.name, ident.span);
            return err_expr!(Type::Err, indexed_ident.span);
        };

        let ty = lhs_symbol.ty.clone();
        // use the supplied number of indicies rathar than the number of indicies we lowered
        let ty = self.get_indexed_type(&ty, indexed_ident.span, indexed_ident.indices.len());

        semantic::Expr {
            span: indexed_ident.span,
            kind: Box::new(semantic::ExprKind::IndexedIdentifier(
                semantic::IndexedIdent {
                    span: indexed_ident.span,
                    name_span: ident.span,
                    index_span: indexed_ident.index_span,
                    symbol_id,
                    indices,
                },
            )),
            ty,
        }
    }

    #[allow(clippy::unused_self)]
    fn lower_gate_operand(&mut self, operand: &syntax::GateOperand) -> semantic::GateOperand {
        let kind = match &operand.kind {
            syntax::GateOperandKind::IndexedIdent(indexed_ident) => {
                semantic::GateOperandKind::Expr(Box::new(
                    self.lower_indexed_ident_expr(indexed_ident),
                ))
            }
            syntax::GateOperandKind::HardwareQubit(hw) => {
                semantic::GateOperandKind::HardwareQubit(Self::lower_hardware_qubit(hw))
            }
            syntax::GateOperandKind::Err => semantic::GateOperandKind::Err,
        };
        semantic::GateOperand {
            span: operand.span,
            kind,
        }
    }

    fn lower_hardware_qubit(hw: &syntax::HardwareQubit) -> semantic::HardwareQubit {
        semantic::HardwareQubit {
            span: hw.span,
            name: hw.name.clone(),
        }
    }

    fn push_invalid_cast_error(&mut self, target_ty: &Type, expr_ty: &Type, span: Span) {
        let rhs_ty_name = format!("{expr_ty:?}");
        let lhs_ty_name = format!("{target_ty:?}");
        let kind = SemanticErrorKind::CannotCast(rhs_ty_name, lhs_ty_name, span);
        self.push_semantic_error(kind);
    }

    fn push_invalid_literal_cast_error(&mut self, target_ty: &Type, expr_ty: &Type, span: Span) {
        let rhs_ty_name = format!("{expr_ty:?}");
        let lhs_ty_name = format!("{target_ty:?}");
        let kind = SemanticErrorKind::CannotCastLiteral(rhs_ty_name, lhs_ty_name, span);
        self.push_semantic_error(kind);
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

    pub fn push_unsuported_in_this_version_error_message<S: AsRef<str>>(
        &mut self,
        message: S,
        minimum_supported_version: &Version,
        span: Span,
    ) {
        let message = message.as_ref().to_string();
        let msv = minimum_supported_version.to_string();
        let kind = SemanticErrorKind::NotSupportedInThisVersion(message, msv, span);
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
        WithSource::from_map(&self.source_map, error)
    }
}

fn wrap_expr_in_implicit_cast_expr(ty: Type, rhs: semantic::Expr) -> semantic::Expr {
    semantic::Expr {
        span: rhs.span,
        kind: Box::new(semantic::ExprKind::Cast(semantic::Cast {
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
fn cast_complex_expr_to_type(ty: &Type, rhs: &semantic::Expr) -> Option<semantic::Expr> {
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

fn try_get_qsharp_name_and_implicit_modifiers<S: AsRef<str>>(
    gate_name: S,
    name_span: Span,
) -> Option<(String, semantic::QuantumGateModifier)> {
    use semantic::GateModifierKind::*;

    let make_modifier = |kind| semantic::QuantumGateModifier {
        span: name_span,
        kind,
    };

    match gate_name.as_ref() {
        "cy" => Some(("y".to_string(), make_modifier(Ctrl(1)))),
        "cz" => Some(("z".to_string(), make_modifier(Ctrl(1)))),
        "ch" => Some(("h".to_string(), make_modifier(Ctrl(1)))),
        "crx" => Some(("rx".to_string(), make_modifier(Ctrl(1)))),
        "cry" => Some(("ry".to_string(), make_modifier(Ctrl(1)))),
        "crz" => Some(("rz".to_string(), make_modifier(Ctrl(1)))),
        "cswap" => Some(("swap".to_string(), make_modifier(Ctrl(1)))),
        "sdg" => Some(("s".to_string(), make_modifier(Inv))),
        "tdg" => Some(("t".to_string(), make_modifier(Inv))),
        // Gates for OpenQASM 2 backwards compatibility
        "CX" => Some(("x".to_string(), make_modifier(Ctrl(1)))),
        "cphase" => Some(("phase".to_string(), make_modifier(Ctrl(1)))),
        _ => None,
    }
}
