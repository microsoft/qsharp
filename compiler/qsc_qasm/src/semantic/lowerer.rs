// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::ops::ShlAssign;
use std::rc::Rc;

use super::const_eval::ConstEvalError;
use super::symbols::ScopeKind;
use super::types::binop_requires_asymmetric_angle_op;
use super::types::binop_requires_int_conversion_for_type;
use super::types::binop_requires_symmetric_uint_conversion;
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
use rustc_hash::FxHashMap;

use super::symbols::{IOKind, Symbol, SymbolTable};

use crate::convert::safe_i64_to_f64;
use crate::parser::ast::list_from_iter;
use crate::parser::ast::List;
use crate::parser::QasmSource;
use crate::semantic::ast::Expr;
use crate::semantic::types::base_types_equal;
use crate::semantic::types::can_cast_literal;
use crate::semantic::types::can_cast_literal_with_value_knowledge;
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
        semantic::Expr::new(Span::default(), semantic::ExprKind::Err, $ty)
    };

    ($ty:expr, $span:expr) => {
        semantic::Expr::new($span, semantic::ExprKind::Err, $ty)
    };
}

pub(crate) struct Lowerer {
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
            "scope stack was non popped correctly"
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
                    self.define_stdgates(include.span);
                    continue;
                }

                let include = includes.next().expect("missing include");
                self.lower_source(include);
            } else {
                let mut stmts = self.lower_stmt(stmt);
                self.stmts.append(&mut stmts);
            }
        }
    }

    fn lower_stmt(&mut self, stmt: &syntax::Stmt) -> Vec<semantic::Stmt> {
        let annotations = list_from_iter(Self::lower_annotations(&stmt.annotations));
        let kind = match &*stmt.kind {
            syntax::StmtKind::Alias(stmt) => self.lower_alias(stmt),
            syntax::StmtKind::Assign(stmt) => self.lower_assign_stmt(stmt),
            syntax::StmtKind::AssignOp(stmt) => self.lower_assign_op_stmt(stmt),
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
            syntax::StmtKind::GateCall(stmt) => {
                return self.lower_gate_call_stmts(stmt, &annotations);
            }
            syntax::StmtKind::GPhase(stmt) => {
                return self.lower_gphase_stmts(stmt, &annotations);
            }
            syntax::StmtKind::Include(stmt) => self.lower_include(stmt),
            syntax::StmtKind::IODeclaration(stmt) => self.lower_io_decl(stmt),
            syntax::StmtKind::Measure(stmt) => self.lower_measure_arrow_stmt(stmt),
            syntax::StmtKind::Pragma(stmt) => self.lower_pragma(stmt),
            syntax::StmtKind::QuantumGateDefinition(stmt) => self.lower_gate_def(stmt),
            syntax::StmtKind::QuantumDecl(stmt) => self.lower_quantum_decl(stmt),
            syntax::StmtKind::Reset(stmt) => self.lower_reset(stmt),
            syntax::StmtKind::Return(stmt) => self.lower_return(stmt),
            syntax::StmtKind::Switch(stmt) => self.lower_switch(stmt),
            syntax::StmtKind::WhileLoop(stmt) => self.lower_while_stmt(stmt),
            syntax::StmtKind::Err => semantic::StmtKind::Err,
        };

        vec![semantic::Stmt {
            span: stmt.span,
            annotations,
            kind: Box::new(kind),
        }]
    }

    /// Define the standard gates in the symbol table.
    /// The sdg, tdg, crx, cry, crz, and ch are defined
    /// as their bare gates, and modifiers are applied
    /// when calling them.
    fn define_stdgates(&mut self, span: Span) {
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
            gate_symbol("ry", 1, 1),
            gate_symbol("rz", 1, 1),
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
                self.push_redefined_symbol_error(name.as_str(), span);
            }
        }
    }

    /// Define the Qiskit standard gates in the symbol table.
    /// Qiskit emits QASM3 that can't compile because it omits
    /// definitions for many gates that aren't included in the
    /// standard gates include file. We define them here so that
    /// the symbol table is complete and we can lower the QASM3.
    /// We must also define the gates in the `Std.OpenQASM` module so
    /// that we can compile the QASM3 to Q#.
    fn define_qiskit_standard_gate_if_needed<S>(&mut self, name: S, span: Span)
    where
        S: AsRef<str>,
    {
        const QISKIT_STDGATES: [&str; 20] = [
            "rxx",
            "ryy",
            "rzz",
            "dcx",
            "ecr",
            "r",
            "rzx",
            "cs",
            "csdg",
            "sxdg",
            "csx",
            "cu1",
            "cu3",
            "rccx",
            "c3sqrtx",
            "c3x",
            "rc3x",
            "xx_minus_yy",
            "xx_plus_yy",
            "ccz",
        ];
        // only define the gate if it is not already defined
        // and it is in the list of Qiskit standard gates
        if self.symbols.get_symbol_by_name(&name).is_none()
            && QISKIT_STDGATES.contains(&name.as_ref())
        {
            self.define_qiskit_standard_gate(name, span);
        }
    }

    fn define_qiskit_standard_gate<S>(&mut self, name: S, span: Span)
    where
        S: AsRef<str>,
    {
        fn gate_symbol(name: &str, cargs: u32, qargs: u32) -> Symbol {
            Symbol::new(
                name,
                Span::default(),
                Type::Gate(cargs, qargs),
                Default::default(),
                Default::default(),
            )
        }
        // QIR intrinsics missing from qasm std library, that Qiskit won't emit qasm defs for
        // rxx, ryy, rzz;

        // Remaining gates that are not in the qasm std library, but are standard gates in Qiskit
        // that Qiskit wont emit correctly.
        // dcx, ecr, r, rzx, cs, csdg, sxdg, csx, cu1, cu3, rccx, c3sqrtx, c3x, rc3x, xx_minus_yy, xx_plus_yy, ccz;
        let gates = FxHashMap::from_iter([
            ("rxx", gate_symbol("rxx", 1, 2)),
            ("ryy", gate_symbol("ryy", 1, 2)),
            ("rzz", gate_symbol("rzz", 1, 2)),
            ("dcx", gate_symbol("dcx", 0, 2)),
            ("ecr", gate_symbol("ecr", 0, 2)),
            ("r", gate_symbol("r", 2, 1)),
            ("rzx", gate_symbol("rzx", 1, 2)),
            ("cs", gate_symbol("cs", 0, 2)),
            ("csdg", gate_symbol("csdg", 0, 2)),
            ("sxdg", gate_symbol("sxdg", 0, 1)),
            ("csx", gate_symbol("csx", 0, 2)),
            ("cu1", gate_symbol("cu1", 1, 2)),
            ("cu3", gate_symbol("cu3", 3, 2)),
            ("rccx", gate_symbol("rccx", 0, 3)),
            ("c3sqrtx", gate_symbol("c3sqrtx", 0, 4)),
            ("c3x", gate_symbol("c3x", 0, 4)),
            ("rc3x", gate_symbol("rc3x", 0, 4)),
            ("xx_minus_yy", gate_symbol("xx_minus_yy", 2, 2)),
            ("xx_plus_yy", gate_symbol("xx_plus_yy", 2, 2)),
            ("ccz", gate_symbol("ccz", 0, 3)),
        ]);
        let gate = gates.get(name.as_ref()).expect("missing gate symbol");
        if self.symbols.insert_symbol(gate.clone()).is_err() {
            self.push_redefined_symbol_error(name.as_ref(), span);
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

    /// This helper method is meant to be used when failing to lower a declaration statement
    /// and returning `StmtKind::Err` before getting to build the symbol due to insufficient
    /// type information. This usually happens when there is a const evaluation error when
    /// trying to compute the size of a sized type.
    fn try_insert_err_symbol_or_push_redefined_symbol_error<S>(&mut self, name: S, span: Span)
    where
        S: AsRef<str>,
    {
        let err_symbol = Symbol::err(name.as_ref(), span);
        let _ = self.symbols.try_insert_or_get_existing(err_symbol);
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

    fn lower_assign_stmt(&mut self, stmt: &syntax::AssignStmt) -> semantic::StmtKind {
        match &*stmt.lhs {
            syntax::IdentOrIndexedIdent::Ident(ident) => {
                self.lower_simple_assign_stmt(ident, &stmt.rhs, stmt.span)
            }
            syntax::IdentOrIndexedIdent::IndexedIdent(indexed_ident) => {
                self.lower_indexed_assign_stmt(indexed_ident, &stmt.rhs, stmt.span)
            }
        }
    }

    fn lower_simple_assign_stmt(
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
        }

        semantic::StmtKind::Assign(semantic::AssignStmt {
            symbol_id,
            lhs_span: ident.span,
            rhs,
            span,
        })
    }

    fn lower_indexed_assign_stmt(
        &mut self,
        indexed_ident: &syntax::IndexedIdent,
        rhs: &syntax::ValueExpr,
        span: Span,
    ) -> semantic::StmtKind {
        assert!(!indexed_ident.indices.is_empty());

        let lhs = self.lower_indexed_ident_expr(indexed_ident);
        let indexed_ty = &lhs.ty;
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

        if lhs.ty.is_const() {
            let kind = SemanticErrorKind::CannotUpdateConstVariable(
                indexed_ident.ident.name.to_string(),
                indexed_ident.ident.span,
            );
            self.push_semantic_error(kind);
        }

        let semantic::ExprKind::IndexedIdent(indexed_ident) = *lhs.kind else {
            return semantic::StmtKind::Err;
        };

        semantic::StmtKind::IndexedAssign(semantic::IndexedAssignStmt {
            span,
            indexed_ident,
            rhs,
        })
    }

    fn lower_assign_op_stmt(&mut self, stmt: &syntax::AssignOpStmt) -> semantic::StmtKind {
        match &*stmt.lhs {
            syntax::IdentOrIndexedIdent::Ident(ident) => {
                self.lower_simple_assign_op_stmt(ident, stmt.op, &stmt.rhs, stmt.span)
            }
            syntax::IdentOrIndexedIdent::IndexedIdent(indexed_ident) => {
                self.lower_indexed_assign_op_stmt(indexed_ident, stmt.op, &stmt.rhs, stmt.span)
            }
        }
    }

    fn lower_simple_assign_op_stmt(
        &mut self,
        ident: &syntax::Ident,
        op: syntax::BinOp,
        rhs: &syntax::ValueExpr,
        span: Span,
    ) -> semantic::StmtKind {
        let (symbol_id, symbol) =
            self.try_get_existing_or_insert_err_symbol(&ident.name, ident.span);

        let ty = symbol.ty.clone();
        if ty.is_const() {
            let kind =
                SemanticErrorKind::CannotUpdateConstVariable(ident.name.to_string(), ident.span);
            self.push_semantic_error(kind);
        }

        let lhs = self.lower_ident_expr(ident);
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

        let binary_expr = self.lower_binary_op_expr(op, lhs, rhs, span);

        semantic::StmtKind::Assign(semantic::AssignStmt {
            symbol_id,
            lhs_span: ident.span,
            rhs: binary_expr,
            span,
        })
    }

    fn lower_indexed_assign_op_stmt(
        &mut self,
        indexed_ident: &syntax::IndexedIdent,
        op: syntax::BinOp,
        rhs: &syntax::ValueExpr,
        span: Span,
    ) -> semantic::StmtKind {
        assert!(!indexed_ident.indices.is_empty());

        let lhs = self.lower_indexed_ident_expr(indexed_ident);
        let indexed_ty = &lhs.ty;
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

        let binary_expr = self.lower_binary_op_expr(op, lhs.clone(), rhs, span);

        if lhs.ty.is_const() {
            let kind = SemanticErrorKind::CannotUpdateConstVariable(
                indexed_ident.ident.name.to_string(),
                indexed_ident.ident.span,
            );
            self.push_semantic_error(kind);
        }

        let semantic::ExprKind::IndexedIdent(indexed_ident) = *lhs.kind else {
            return semantic::StmtKind::Err;
        };

        semantic::StmtKind::IndexedAssign(semantic::IndexedAssignStmt {
            span,
            indexed_ident,
            rhs: binary_expr,
        })
    }

    fn lower_expr(&mut self, expr: &syntax::Expr) -> semantic::Expr {
        match &*expr.kind {
            syntax::ExprKind::BinaryOp(bin_op_expr) => {
                let lhs = self.lower_expr(&bin_op_expr.lhs);
                let rhs = self.lower_expr(&bin_op_expr.rhs);
                self.lower_binary_op_expr(bin_op_expr.op, lhs, rhs, expr.span)
            }
            syntax::ExprKind::Cast(expr) => self.lower_cast_expr(expr),
            syntax::ExprKind::Err => err_expr!(Type::Err, expr.span),
            syntax::ExprKind::FunctionCall(expr) => self.lower_function_call_expr(expr),
            syntax::ExprKind::Ident(ident) => self.lower_ident_expr(ident),
            syntax::ExprKind::IndexExpr(expr) => self.lower_index_expr(expr),
            syntax::ExprKind::Lit(lit) => self.lower_lit_expr(lit, None),
            syntax::ExprKind::Paren(pexpr) => self.lower_paren_expr(pexpr, expr.span),
            syntax::ExprKind::UnaryOp(expr) => self.lower_unary_op_expr(expr),
        }
    }

    fn lower_decl_expr(
        &mut self,
        expr: &syntax::Expr,
        ty: &Type,
        lowering_array: bool,
    ) -> semantic::Expr {
        let expr = match &*expr.kind {
            syntax::ExprKind::Lit(lit) => self.lower_lit_expr(lit, Some(ty)),
            _ => self.lower_expr(expr),
        };

        // If we are not lowering an array OR are lowering one but haven't reached a leaf node
        // we return expr without casting.
        if !lowering_array || ty.is_array() {
            expr
        } else {
            // We take this branch if we are trying to lower an array but reached a leaf node.
            self.cast_expr_to_type(ty, &expr)
        }
    }

    fn lower_cast_expr(&mut self, cast: &syntax::Cast) -> semantic::Expr {
        let cast_span = cast.span;
        let expr = self.lower_expr(&cast.arg);
        let ty = self.get_semantic_type_from_tydef(&cast.ty, expr.ty.is_const());
        let mut cast = self.cast_expr_to_type_with_span(&ty, &expr, cast_span);

        // This is an explicit cast, so we know its span. If casting
        // succeded, override the default span with the cast span.
        cast.span = cast_span;
        if let semantic::ExprKind::Cast(cast_ref) = cast.kind.as_mut() {
            cast_ref.span = cast_span;
        }

        cast
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
        let is_symbol_declaration_outside_gate_or_function_scope = self
            .symbols
            .is_symbol_outside_most_inner_gate_or_function_scope(symbol_id);

        let need_to_capture_symbol = is_symbol_inside_gate_or_function_scope
            && is_symbol_declaration_outside_gate_or_function_scope;

        let kind = if need_to_capture_symbol && symbol.ty.is_const() {
            if let Some(val) = symbol.get_const_value() {
                semantic::ExprKind::Lit(val)
            } else {
                // If the const evaluation fails, we return Err but don't push
                // any additional error. The error was already pushed in the
                // const_eval function.
                semantic::ExprKind::Ident(symbol_id)
            }
        } else if need_to_capture_symbol && !symbol.ty.is_err() && !symbol.ty.is_const() {
            self.push_semantic_error(SemanticErrorKind::ExprMustBeConst(
                "a captured variable".into(),
                ident.span,
            ));
            semantic::ExprKind::Ident(symbol_id)
        } else {
            semantic::ExprKind::Ident(symbol_id)
        };

        semantic::Expr::new(ident.span, kind, symbol.ty.clone())
    }

    fn check_lit_array_size(&mut self, expr: &syntax::Lit, ty: &Type) {
        if let Some(dims) = ty.array_dims() {
            let expected_size = dims.into_iter().next();

            let Some(expected_size) = expected_size else {
                // If we don't have at least one dimension_size, it means the dimension was Err.
                // In that case we already issued an error before.
                return;
            };

            let syntax::LiteralKind::Array(array) = &expr.kind else {
                let kind = SemanticErrorKind::ArrayDeclarationTypeError(
                    format!(
                        "expected an array of size {} but found {}",
                        expected_size, &expr.kind
                    ),
                    expr.span,
                );
                self.push_semantic_error(kind);
                return;
            };

            let actual_size = array.len();

            if actual_size != expected_size as usize {
                let kind = SemanticErrorKind::ArrayDeclarationTypeError(
                    format!("expected an array of size {expected_size} but found one of size {actual_size}"),
                    expr.span
                );
                self.push_semantic_error(kind);
            }
        }
    }

    fn lower_lit_expr(&mut self, expr: &syntax::Lit, ty: Option<&Type>) -> semantic::Expr {
        if let Some(ty) = ty {
            self.check_lit_array_size(expr, ty);
        }

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
                Type::BitArray(*size, true),
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
                self.push_unsupported_error_message("string literals", expr.span);
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
                if let Some(ty) = ty {
                    let dummy_index = semantic::Expr::new(
                        Span::default(),
                        semantic::ExprKind::Lit(semantic::LiteralKind::Int(0)),
                        Type::Int(None, true),
                    );

                    let indexed_ty = self.get_indexed_type(
                        ty,
                        Span::default(),
                        &[semantic::Index::Expr(dummy_index)],
                    );

                    let texprs = exprs
                        .iter()
                        .map(|expr| self.lower_decl_expr(expr, &indexed_ty, true))
                        .collect::<Vec<_>>();

                    let Some(dims) = ty.array_dims() else {
                        return err_expr!(Type::Err, expr.span);
                    };
                    let array = semantic::Array { data: texprs, dims };
                    let kind = semantic::ExprKind::Lit(semantic::LiteralKind::Array(array));

                    (kind, ty.clone())
                } else {
                    // array literals are only valid in classical decals (const and mut)
                    // and we have to know the expected type of the array in order to lower it
                    // So we can't lower array literals in general.
                    self.push_semantic_error(SemanticErrorKind::ArrayLiteralInNonClassicalDecl(
                        expr.span,
                    ));
                    (semantic::ExprKind::Err, Type::Err)
                }
            }
        };
        semantic::Expr::new(expr.span, kind, ty)
    }

    fn lower_paren_expr(&mut self, expr: &syntax::Expr, span: Span) -> semantic::Expr {
        let expr = self.lower_expr(expr);
        let ty = expr.ty.clone();
        let kind = semantic::ExprKind::Paren(expr);
        semantic::Expr::new(span, kind, ty)
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
                semantic::Expr::new(span, semantic::ExprKind::UnaryOp(unary), ty)
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
                semantic::Expr::new(span, semantic::ExprKind::UnaryOp(unary), ty)
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

                semantic::Expr::new(
                    expr.span,
                    semantic::ExprKind::UnaryOp(semantic::UnaryOpExpr {
                        span: expr.span,
                        op: semantic::UnaryOp::NotL,
                        expr,
                    }),
                    ty,
                )
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
        if ty.is_array() && matches!(ty.array_dims(), Some(super::types::ArrayDimensions::Err)) {
            self.push_unsupported_error_message("arrays with more than 7 dimensions", span);
            return crate::types::Type::Err;
        }

        let is_const = ty.is_const();
        match ty {
            Type::Bit(_) => crate::types::Type::Result(is_const),
            Type::Qubit => crate::types::Type::Qubit,
            Type::HardwareQubit => {
                let message = "hardware qubit to Q# type";
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
                self.push_unsupported_error_message("duration type values", span);
                crate::types::Type::Err
            }
            Type::Stretch(_) => {
                self.push_unsupported_error_message("stretch type values", span);
                crate::types::Type::Err
            }
            Type::BitArray(size, _) => crate::types::Type::ResultArray(
                crate::types::ArrayDimensions::One(*size as usize),
                is_const,
            ),
            Type::QubitArray(size) => {
                crate::types::Type::QubitArray(crate::types::ArrayDimensions::One(*size as usize))
            }
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
            Type::ComplexArray(_, dims) => crate::types::Type::ComplexArray(dims.into(), is_const),
            Type::AngleArray(_, dims) => crate::types::Type::AngleArray(dims.into(), is_const),
            Type::Gate(cargs, qargs) => {
                crate::types::Type::Callable(crate::types::CallableKind::Operation, *cargs, *qargs)
            }
            Type::Range => crate::types::Type::Range,
            Type::Void => crate::types::Type::Tuple(vec![]),
            Type::Err => crate::types::Type::Err,
            _ => {
                let msg = format!("converting `{ty}` to Q# type");
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

    fn lower_block(&mut self, block: &syntax::Block) -> semantic::Block {
        self.symbols.push_scope(ScopeKind::Block);
        let mut stmts = Vec::new();
        for stmt in &block.stmts {
            stmts.append(&mut self.lower_stmt(stmt));
        }
        let stmts = list_from_iter(stmts);
        self.symbols.pop_scope();

        semantic::Block {
            span: block.span,
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

        // Arrays are only allowed in the global scope.
        // If we are not in the global scope, we push an error.
        if ty.is_array()
            && !matches!(ty, Type::BitArray(..))
            && !self.symbols.is_current_scope_global()
        {
            let kind = SemanticErrorKind::ArrayDeclarationInNonGlobalScope(stmt.span);
            self.push_semantic_error(kind);
        }

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
                    let expr = self.lower_decl_expr(expr, &ty, false);
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
        let mut init_expr = match &stmt.init_expr {
            syntax::ValueExpr::Expr(expr) => {
                let expr = self.lower_decl_expr(expr, &ty, false);
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
            init_expr = init_expr.with_const_value(self);
            symbol = symbol.with_const_expr(Rc::new(init_expr.clone()));
        }

        let symbol_id = self.try_insert_or_get_existing_symbol_id(name, symbol);

        if !init_expr.ty.is_err() && !init_expr.ty.is_const() {
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
            (Rc::new(ty), qsharp_ty)
        } else {
            (
                Rc::new(crate::semantic::types::Type::Void),
                crate::types::Type::Tuple(Default::default()),
            )
        };

        // 2. Push the function symbol to the symbol table.
        #[allow(clippy::cast_possible_truncation)]
        let arity = stmt.params.len() as u32;
        let name = stmt.name.name.clone();
        let name_span = stmt.name.span;
        let ty = crate::semantic::types::Type::Function(param_types.into(), return_ty.clone());

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
        self.symbols.push_scope(ScopeKind::Function(return_ty));

        let params = param_symbols
            .into_iter()
            .map(|symbol| {
                let name = symbol.name.clone();
                self.try_insert_or_get_existing_symbol_id(name, symbol)
            })
            .collect();

        let mut stmts = Vec::new();
        for stmt in &stmt.body.stmts {
            stmts.append(&mut self.lower_stmt(stmt));
        }

        let body = semantic::Block {
            span: stmt.body.span,
            stmts: list_from_iter(stmts),
        };

        // Pop the scope where the def lives.
        self.symbols.pop_scope();

        if let Some(return_ty) = &stmt.return_type {
            self.check_that_def_returns_in_all_code_paths(&body, return_ty.span);
        }

        semantic::StmtKind::Def(semantic::DefStmt {
            span: stmt.span,
            symbol_id,
            has_qubit_params,
            params,
            body,
            return_type: qsharp_return_ty,
        })
    }

    fn check_that_def_returns_in_all_code_paths(&mut self, body: &semantic::Block, span: Span) {
        if !Self::block_always_returns(&body.stmts) {
            self.push_semantic_error(SemanticErrorKind::NonVoidDefShouldAlwaysReturn(span));
        }
    }

    fn block_always_returns<'a>(stmts: impl IntoIterator<Item = &'a Box<semantic::Stmt>>) -> bool {
        for stmt in stmts {
            if Self::stmt_always_returns(stmt) {
                return true;
            }
        }
        false
    }

    fn stmt_always_returns(stmt: &semantic::Stmt) -> bool {
        match &*stmt.kind {
            semantic::StmtKind::Block(block) => Self::block_always_returns(&block.stmts),
            semantic::StmtKind::Box(stmt) => Self::block_always_returns(&stmt.body),
            semantic::StmtKind::If(stmt) => {
                if let Some(else_body) = &stmt.else_body {
                    Self::stmt_always_returns(&stmt.if_body) && Self::stmt_always_returns(else_body)
                } else {
                    false
                }
            }
            // We don't know if the user's switch is exhaustive.
            // We take a best effort approach and check if all the branches always return.
            semantic::StmtKind::Switch(stmt) => {
                let mut all_cases_return = true;
                for case in &stmt.cases {
                    all_cases_return &= Self::block_always_returns(&case.block.stmts);
                }
                if let Some(default_case) = &stmt.default {
                    all_cases_return &= Self::block_always_returns(&default_case.stmts);
                }
                all_cases_return
            }
            // We don't know if the iterable of the loop is empty at compiletime.
            // We take a best effort approach and check if the body always returns.
            semantic::StmtKind::For(stmt) => Self::stmt_always_returns(&stmt.body),
            semantic::StmtKind::WhileLoop(stmt) => Self::stmt_always_returns(&stmt.body),
            semantic::StmtKind::Return(..) | semantic::StmtKind::End(..) => true,
            _ => false,
        }
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
            let size = self.const_eval_array_size_designator_expr(size);
            if let Some(size) = size {
                let size = size.get_const_u32().expect("const evaluation succeeded");
                let ty = crate::semantic::types::Type::QubitArray(size);
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
        if matches!(&*expr.kind, semantic::ExprKind::Err) {
            semantic::StmtKind::Err
        } else {
            semantic::StmtKind::ExprStmt(semantic::ExprStmt {
                span: stmt.span,
                expr,
            })
        }
    }

    fn lower_extern(&mut self, stmt: &syntax::ExternDecl) -> semantic::StmtKind {
        // 1. Check that we are in the global scope. QASM3 semantics
        //    only allow extern declarations in the global scope.
        if !self.symbols.is_current_scope_global() {
            let kind = SemanticErrorKind::ExternDeclarationInNonGlobalScope(stmt.span);
            self.push_semantic_error(kind);
        }

        // 2. Build the parameter's type.
        let mut params = Vec::with_capacity(stmt.params.len());
        let mut qsharp_params = Vec::with_capacity(stmt.params.len());

        for param in &stmt.params {
            let ty = self.lower_extern_param(param);
            let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty, param.span());
            params.push(ty);
            qsharp_params.push(qsharp_ty);
        }

        // 2. Build the return type.
        let (return_ty, qsharp_return_ty) = if let Some(ty) = &stmt.return_type {
            let ty_span = ty.span;
            let tydef = syntax::TypeDef::Scalar(ty.clone());
            let ty = self.get_semantic_type_from_tydef(&tydef, false);
            let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty, ty_span);
            (Rc::new(ty), qsharp_ty)
        } else {
            (
                Rc::new(crate::semantic::types::Type::Void),
                crate::types::Type::Tuple(Default::default()),
            )
        };

        // 3. Push the extern symbol to the symbol table.
        #[allow(clippy::cast_possible_truncation)]
        let arity = stmt.params.len() as u32;
        let name = stmt.ident.name.clone();
        let name_span = stmt.ident.span;
        let ty = crate::semantic::types::Type::Function(params.into(), return_ty.clone());
        let kind = crate::types::CallableKind::Function;
        let qsharp_ty = crate::types::Type::Callable(kind, arity, 0);
        let symbol = Symbol::new(&name, name_span, ty, qsharp_ty, IOKind::Default);
        let symbol_id = self.try_insert_or_get_existing_symbol_id(name, symbol);

        semantic::StmtKind::ExternDecl(semantic::ExternDecl {
            span: stmt.span,
            symbol_id,
            params: qsharp_params.into(),
            return_type: qsharp_return_ty,
        })
    }

    fn lower_extern_param(&mut self, param: &syntax::ExternParameter) -> Type {
        let tydef = match param {
            syntax::ExternParameter::ArrayReference(array_reference_type, _) => {
                syntax::TypeDef::ArrayReference(array_reference_type.clone())
            }
            syntax::ExternParameter::Scalar(scalar_type, _) => {
                syntax::TypeDef::Scalar(scalar_type.clone())
            }
        };
        self.get_semantic_type_from_tydef(&tydef, false)
    }

    /// If the body is a block, lowers the block pushing a scope.
    /// If the body is a single statement, also creates a block and
    /// pushes a scope, this is needed to handle broadcasting gate calls.
    fn lower_stmt_or_block_body(&mut self, stmt: &syntax::Stmt) -> semantic::Stmt {
        if matches!(&*stmt.kind, syntax::StmtKind::Block(..)) {
            let stmts = self.lower_stmt(stmt);
            assert_eq!(stmts.len(), 1);
            stmts
                .into_iter()
                .next()
                .expect("there is exactly one element in the vector")
        } else {
            self.symbols.push_scope(ScopeKind::Block);
            let stmts = self.lower_stmt(stmt);
            self.symbols.pop_scope();

            semantic::Stmt {
                span: stmt.span,
                annotations: Box::default(),
                kind: Box::new(semantic::StmtKind::Block(Box::new(semantic::Block {
                    span: stmt.span,
                    stmts: list_from_iter(stmts),
                }))),
            }
        }
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
        let body = self.lower_stmt_or_block_body(&stmt.body);

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

        let if_body = self.lower_stmt_or_block_body(&stmt.if_body);

        let else_body = stmt
            .else_body
            .as_ref()
            .map(|else_body| self.lower_stmt_or_block_body(else_body));

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
            self.push_semantic_error(SemanticErrorKind::CannotCallNonFunction(expr.span));
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

        let kind = semantic::ExprKind::FunctionCall(semantic::FunctionCall {
            span: expr.span,
            fn_name_span: expr.name.span,
            symbol_id,
            args,
        });

        semantic::Expr::new(expr.span, kind, return_ty)
    }

    /// A broadcasted gate call unfolds into multiple statements.
    /// If the original gate call had annotations, we apply them
    /// to all the generated statements.
    fn lower_gate_call_stmts(
        &mut self,
        stmt: &syntax::GateCall,
        annotations: &List<semantic::Annotation>,
    ) -> Vec<semantic::Stmt> {
        let mut stmts = Vec::new();
        for kind in self.lower_gate_call_stmt(stmt) {
            let stmt = semantic::Stmt {
                span: stmt.span,
                annotations: annotations.clone(),
                kind: Box::new(kind),
            };
            stmts.push(stmt);
        }
        stmts
    }

    #[allow(clippy::too_many_lines)]
    fn lower_gate_call_stmt(&mut self, stmt: &syntax::GateCall) -> Vec<semantic::StmtKind> {
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
            return vec![semantic::StmtKind::Err];
        }

        //   1.3. Lower the args.
        let carg_ty = crate::semantic::types::Type::Angle(None, false);
        let args = stmt.args.iter().map(|arg| {
            let arg = self.lower_expr(arg);
            match &arg.kind.as_ref() {
                semantic::ExprKind::Lit(kind) => {
                    if can_cast_literal(&carg_ty, &arg.ty)
                        || can_cast_literal_with_value_knowledge(&carg_ty, kind)
                    {
                        self.coerce_literal_expr_to_type(&carg_ty, &arg, kind)
                    } else {
                        self.cast_expr_to_type(&carg_ty, &arg)
                    }
                }
                _ => self.cast_expr_to_type(&carg_ty, &arg),
            }
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
        if let Some((gate_name, implicit_modifier)) =
            try_get_qsharp_name_and_implicit_modifiers(&name, stmt.name.span)
        {
            // Override the gate name if we mapped with modifiers.
            name = gate_name;

            // 2. Get implicit modifiers and make them explicit.
            //    Q: Do we need this during lowering?
            //    A: Yes, we need it to check the gate_call arity.
            modifiers.push(implicit_modifier);
        }

        // need a workaround for qiskit generating gate calls without having declared the gate
        self.define_qiskit_standard_gate_if_needed(&name, stmt.name.span);

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
                    quantum_arity_with_modifiers +=
                        n.get_const_u32().expect("const evaluation succeeded");
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

        // 5.1 Reverse the modifiers to match the order expected by the compiler.
        modifiers.reverse();
        let modifiers = list_from_iter(modifiers);

        // 6. Check if we need to do broadcasting.
        let mut register_type = None;

        for qubit in &qubits {
            if let semantic::GateOperandKind::Expr(expr) = &qubit.kind {
                if matches!(expr.ty, Type::QubitArray(..)) {
                    register_type = Some(&expr.ty);
                    break;
                }
            }
        }

        // If at least one of the quantum args was a register
        // we try to do broadcasting.
        if let Some(register_type) = register_type {
            // 6.1 Check that all the quantum registers are of the same type/size.
            let mut resgisters_sizes_disagree = false;
            for qubit in &qubits {
                if let semantic::GateOperandKind::Expr(expr) = &qubit.kind {
                    if !matches!(&expr.ty, Type::Qubit)
                        && !base_types_equal(register_type, &expr.ty)
                    {
                        resgisters_sizes_disagree = true;
                        self.push_semantic_error(
                            SemanticErrorKind::BroadcastCallQuantumArgsDisagreeInSize(
                                register_type.to_string(),
                                expr.ty.to_string(),
                                expr.span,
                            ),
                        );
                    }
                }
            }

            // If the register sizes disagree, we don't have sane information
            // to generate multiple statements based on the sizes. So, we
            // return ::Err in this case.
            if resgisters_sizes_disagree {
                return vec![semantic::StmtKind::Err];
            }

            // 6.2 Convert the broadcast call in a list of simple stmts.
            let mut stmts = Vec::new();

            let Type::QubitArray(indexed_dim_size) = register_type else {
                unreachable!("we set register_type iff we find a QubitArray");
            };

            for index in 0..(*indexed_dim_size) {
                let qubits = qubits
                    .iter()
                    .map(|qubit| Self::index_into_qubit_register((**qubit).clone(), index));

                let qubits = list_from_iter(qubits);

                stmts.push(semantic::StmtKind::GateCall(semantic::GateCall {
                    span: stmt.span,
                    modifiers: modifiers.clone(),
                    symbol_id,
                    gate_name_span: stmt.name.span,
                    args: args.clone(),
                    qubits,
                    duration: duration.clone(),
                    classical_arity,
                    quantum_arity,
                }));
            }

            return stmts;
        }

        // 7. This is the base case with no broadcasting. We return:
        //   7.1. Gate symbol_id.
        //   7.2. All controls made explicit.
        //   7.3. Classical args.
        //   7.4. Quantum args in the order expected by the compiler.
        vec![semantic::StmtKind::GateCall(semantic::GateCall {
            span: stmt.span,
            modifiers,
            symbol_id,
            gate_name_span: stmt.name.span,
            args,
            qubits,
            duration,
            classical_arity,
            quantum_arity,
        })]

        // The compiler will be left to do all things that need explicit Q# knowledge.
        // But it won't need to check arities, know about implicit modifiers, or do
        // any casting of classical args. There is still some inherit complexity to
        // building a Q# gate call with this information, but it won't be cluttered
        // by all the QASM semantic analysis.
    }

    fn index_into_qubit_register(op: semantic::GateOperand, index: u32) -> semantic::GateOperand {
        let index = semantic::Index::Expr(semantic::Expr::new(
            op.span,
            semantic::ExprKind::Lit(semantic::LiteralKind::Int(index.into())),
            Type::UInt(None, true),
        ));

        match op.kind {
            semantic::GateOperandKind::Expr(expr) => {
                // Single qubits are allowed in a broadcast call.
                match &expr.ty {
                    Type::Qubit => semantic::GateOperand {
                        span: op.span,
                        kind: semantic::GateOperandKind::Expr(expr),
                    },
                    Type::QubitArray(..) => semantic::GateOperand {
                        span: op.span,
                        kind: semantic::GateOperandKind::Expr(Box::new(semantic::Expr::new(
                            op.span,
                            semantic::ExprKind::IndexExpr(semantic::IndexExpr {
                                span: op.span,
                                collection: *expr,
                                indices: list_from_iter([index]),
                            }),
                            Type::Qubit,
                        ))),
                    },
                    _ => unreachable!("we set register_type iff we find a QubitArray"),
                }
            }
            _ => unreachable!("by this point `op` is guaranteed to be a quantum register"),
        }
    }

    /// A broadcasted gphase unfolds into multiple statements.
    /// If the original gphase had annotations, we apply them
    /// to all the generated statements.
    fn lower_gphase_stmts(
        &mut self,
        stmt: &syntax::GPhase,
        annotations: &List<semantic::Annotation>,
    ) -> Vec<semantic::Stmt> {
        let mut stmts = Vec::new();
        for kind in self.lower_gphase_stmt(stmt) {
            let stmt = semantic::Stmt {
                span: stmt.span,
                annotations: annotations.clone(),
                kind: Box::new(kind),
            };
            stmts.push(stmt);
        }
        stmts
    }

    /// This is just syntax sugar around a gate call.
    fn lower_gphase_stmt(&mut self, stmt: &syntax::GPhase) -> Vec<semantic::StmtKind> {
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
            modifier_keyword_span: modifier.modifier_keyword_span,
            kind,
        })
    }

    fn lower_modifier_ctrl_args(&mut self, expr: Option<&syntax::Expr>) -> Option<semantic::Expr> {
        let Some(expr) = expr else {
            return Some(
                semantic::Expr::new(
                    Span::default(),
                    semantic::ExprKind::Lit(semantic::LiteralKind::Int(1)),
                    Type::Int(None, true),
                )
                .with_const_value(self),
            );
        };

        let expr = self.lower_expr(expr);

        let target_ty = &Type::UInt(None, true);
        let Some(expr) = Self::try_cast_expr_to_type(target_ty, &expr) else {
            self.push_invalid_cast_error(target_ty, &expr.ty, expr.span);
            return None;
        };

        let expr = expr.with_const_value(self);

        let Some(lit) = expr.get_const_value() else {
            // const_eval must have pushed an error unless the ty is Err
            // in which case there is already an error pushed for the ty.
            return None;
        };

        let semantic::LiteralKind::Int(n) = lit else {
            // A CannotCastLiteral error was already pushed.
            return None;
        };

        if u32::try_from(n).is_err() {
            self.push_semantic_error(SemanticErrorKind::ExprMustFitInU32(
                "ctrl modifier argument".into(),
                expr.span,
            ));
            return None;
        }

        Some(expr)
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
        panic!("include should have been handled in lower_source")
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

    /// `measure q -> c;` is syntax sugar for `c = measure q;`
    fn lower_measure_arrow_stmt(&mut self, stmt: &syntax::MeasureArrowStmt) -> semantic::StmtKind {
        if let Some(target) = &stmt.target {
            self.lower_assign_stmt(&syntax::AssignStmt {
                span: stmt.span,
                lhs: Box::new(*target.clone()),
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
            let kind = SemanticErrorKind::GateDeclarationInNonGlobalScope(stmt.span);
            self.push_semantic_error(kind);
        }

        // 2. Push the gate symbol to the symbol table.
        #[allow(clippy::cast_possible_truncation)]
        let classical_arity = stmt
            .params
            .iter()
            .filter_map(|seq_item| seq_item.item_as_ref())
            .count() as u32;
        #[allow(clippy::cast_possible_truncation)]
        let quantum_arity = stmt
            .qubits
            .iter()
            .filter_map(|seq_item| seq_item.item_as_ref())
            .count() as u32;
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

        // Design Note: If a formal parameter is missing (i.e. there are two consecutive commas and we
        //              have a missing item in the formal parameters list), we have two options:
        //                 1. Treat the missing item as if it wasn't there, and just push a parser
        //                    error saying there is a missing item. This is what Rust does.
        //                 2. Treat the missing item as a Type::Err and make it part of the gate
        //                    signature, this is what Q# does.
        //              We decided to go with (1) because it avoids propagating the SeqItem enum
        //              to the compiler, which is simpler.
        let params = stmt
            .params
            .iter()
            .filter_map(|seq_item| seq_item.item_as_ref())
            .map(|arg| {
                let ty = crate::semantic::types::Type::Angle(None, false);
                let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty, Span::default());
                let symbol = Symbol::new(&arg.name, arg.span, ty, qsharp_ty, IOKind::Default);
                self.try_insert_or_get_existing_symbol_id(&arg.name, symbol)
            })
            .collect::<Box<_>>();

        let qubits = stmt
            .qubits
            .iter()
            .filter_map(|seq_item| seq_item.item_as_ref())
            .map(|arg| {
                let ty = crate::semantic::types::Type::Qubit;
                let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty, Span::default());
                let symbol = Symbol::new(&arg.name, arg.span, ty, qsharp_ty, IOKind::Default);
                self.try_insert_or_get_existing_symbol_id(&arg.name, symbol)
            })
            .collect::<Box<_>>();

        let mut stmts = Vec::new();
        for stmt in &stmt.body.stmts {
            stmts.append(&mut self.lower_stmt(stmt));
        }

        let body = semantic::Block {
            span: stmt.body.span,
            stmts: list_from_iter(stmts),
        };

        // Pop the scope where the gate definition lives.
        self.symbols.pop_scope();

        semantic::StmtKind::QuantumGateDefinition(semantic::QuantumGateDefinition {
            span: stmt.span,
            name_span: stmt.ident.span,
            symbol_id,
            params,
            qubits,
            body,
        })
    }

    fn lower_quantum_decl(&mut self, stmt: &syntax::QubitDeclaration) -> semantic::StmtKind {
        if !self.symbols.is_current_scope_global() {
            let kind = SemanticErrorKind::QubitDeclarationInNonGlobalScope(stmt.span);
            self.push_semantic_error(kind);
        }

        // If there wasn't an explicit size, infer the size to be 1.
        let (ty, size_and_span) = if let Some(size_expr) = &stmt.size {
            let span = size_expr.span;
            let size_expr = self.const_eval_quantum_register_size_expr(size_expr);

            let Some(size_expr) = size_expr else {
                // We insert an err symbol if the symbol was not previously defined.
                self.try_insert_err_symbol_or_push_redefined_symbol_error(
                    &stmt.qubit.name,
                    stmt.qubit.span,
                );

                // Any errors would have already been pushed by `const_eval_quantum_register_size`.
                return semantic::StmtKind::Err;
            };

            let Some(size) = size_expr.get_const_u32() else {
                // We insert an err symbol if the symbol was not previously defined.
                self.try_insert_err_symbol_or_push_redefined_symbol_error(
                    &stmt.qubit.name,
                    stmt.qubit.span,
                );

                // Any errors would have already been pushed by `const_eval_quantum_register_size`.
                return semantic::StmtKind::Err;
            };

            (Type::QubitArray(size), Some((size_expr, span)))
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
        let mut expr = stmt
            .expr
            .as_ref()
            .map(|expr| match &**expr {
                syntax::ValueExpr::Expr(expr) => self.lower_expr(expr),
                syntax::ValueExpr::Measurement(expr) => self.lower_measure_expr(expr),
            })
            .map(Box::new);

        let return_ty = self.symbols.get_subroutine_return_ty();

        match (&mut expr, return_ty) {
            // If we don't have a return type then we are not rooted in a subroutine scope.
            (_, None) => {
                self.push_semantic_error(SemanticErrorKind::InvalidScope(
                    "return statements".into(),
                    "subroutine".into(),
                    stmt.span,
                ));
                return semantic::StmtKind::Err;
            }
            (None, Some(ty)) => {
                if !matches!(ty.as_ref(), Type::Void) {
                    self.push_semantic_error(
                        SemanticErrorKind::MissingTargetExpressionInReturnStmt(stmt.span),
                    );
                }
            }
            (Some(expr), Some(ty)) => {
                if matches!(ty.as_ref(), Type::Void) {
                    self.push_semantic_error(
                        SemanticErrorKind::ReturningExpressionFromVoidSubroutine(expr.span),
                    );
                    return semantic::StmtKind::Err;
                }
                *expr = Box::new(self.cast_expr_to_type(&ty, expr));
            }
        }

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
        let body = self.lower_stmt_or_block_body(&stmt.body);

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
    fn const_eval_designator(&mut self, expr: &syntax::Expr) -> Option<semantic::Expr> {
        let expr = self.lower_expr(expr);
        let expr_span = expr.span;
        let expr = self
            .cast_expr_with_target_type_or_default(Some(expr), &Type::UInt(None, true), expr_span)
            .with_const_value(self);

        // const_eval would have pushed an error unless the ty is Err
        // in which case there is already an error pushed for the ty
        // so there is no need to add more errors here.
        expr.const_value.as_ref()?;

        Some(expr)
    }

    fn const_eval_array_size_designator_expr(
        &mut self,
        expr: &syntax::Expr,
    ) -> Option<semantic::Expr> {
        let size = self.const_eval_designator(expr)?;

        let Some(semantic::LiteralKind::Int(val)) = size.get_const_value() else {
            let msg = "array size".to_string();
            self.push_semantic_error(SemanticErrorKind::ExprMustBeInt(msg, expr.span));
            return None;
        };

        if val < 0 {
            let msg = "array size".to_string();
            self.push_semantic_error(SemanticErrorKind::ExprMustBeNonNegativeInt(msg, expr.span));
            return None;
        }

        if u32::try_from(val).is_err() {
            self.push_semantic_error(SemanticErrorKind::DesignatorTooLarge(expr.span));
        }

        Some(size)
    }

    fn const_eval_type_width_designator_expr(
        &mut self,
        expr: &syntax::Expr,
    ) -> Option<semantic::Expr> {
        let width = self.const_eval_designator(expr)?;

        let Some(semantic::LiteralKind::Int(val)) = width.get_const_value() else {
            let msg = "type width".to_string();
            self.push_semantic_error(SemanticErrorKind::ExprMustBeInt(msg, expr.span));
            return None;
        };

        if val < 1 {
            let msg = "type width".to_string();
            self.push_semantic_error(SemanticErrorKind::ExprMustBePositiveInt(msg, expr.span));
            return None;
        }

        if u32::try_from(val).is_err() {
            self.push_semantic_error(SemanticErrorKind::DesignatorTooLarge(expr.span));
        }

        Some(width)
    }

    fn const_eval_quantum_register_size_expr(
        &mut self,
        expr: &syntax::Expr,
    ) -> Option<semantic::Expr> {
        let size = self.const_eval_designator(expr)?;

        let Some(semantic::LiteralKind::Int(val)) = size.get_const_value() else {
            let msg = "quantum register size".into();
            self.push_semantic_error(SemanticErrorKind::ExprMustBeInt(msg, size.span));
            return None;
        };

        if val < 1 {
            let msg = "quantum register size".into();
            self.push_semantic_error(SemanticErrorKind::ExprMustBePositiveInt(msg, expr.span));
            return None;
        }

        if u32::try_from(val).is_err() {
            self.push_semantic_error(SemanticErrorKind::DesignatorTooLarge(expr.span));
        }

        // We already verified that the expression as a non-negative int,
        // so, this const cast will succeed.
        Self::try_cast_expr_to_type(&Type::UInt(None, true), &size)
            .map(|expr| expr.with_const_value(self))
    }

    #[allow(clippy::too_many_lines)]
    fn get_semantic_type_from_scalar_ty(
        &mut self,
        scalar_ty: &syntax::ScalarType,
        is_const: bool,
    ) -> crate::semantic::types::Type {
        match &scalar_ty.kind {
            syntax::ScalarTypeKind::Bit(bit_type) => match &bit_type.size {
                Some(size) => {
                    let Some(size_expr) = self.const_eval_array_size_designator_expr(size) else {
                        return crate::semantic::types::Type::Err;
                    };
                    let Some(size) = size_expr.get_const_u32() else {
                        return crate::semantic::types::Type::Err;
                    };
                    crate::semantic::types::Type::BitArray(size, is_const)
                }
                None => crate::semantic::types::Type::Bit(is_const),
            },
            syntax::ScalarTypeKind::Int(int_type) => match &int_type.size {
                Some(size) => {
                    let Some(size_expr) = self.const_eval_type_width_designator_expr(size) else {
                        return crate::semantic::types::Type::Err;
                    };
                    let Some(size) = size_expr.get_const_u32() else {
                        return crate::semantic::types::Type::Err;
                    };
                    crate::semantic::types::Type::Int(Some(size), is_const)
                }
                None => crate::semantic::types::Type::Int(None, is_const),
            },
            syntax::ScalarTypeKind::UInt(uint_type) => match &uint_type.size {
                Some(size) => {
                    let Some(size_expr) = self.const_eval_type_width_designator_expr(size) else {
                        return crate::semantic::types::Type::Err;
                    };
                    let Some(size) = size_expr.get_const_u32() else {
                        return crate::semantic::types::Type::Err;
                    };
                    crate::semantic::types::Type::UInt(Some(size), is_const)
                }
                None => crate::semantic::types::Type::UInt(None, is_const),
            },
            syntax::ScalarTypeKind::Float(float_type) => match &float_type.size {
                Some(size) => {
                    let Some(size_expr) = self.const_eval_type_width_designator_expr(size) else {
                        return crate::semantic::types::Type::Err;
                    };
                    let Some(size) = size_expr.get_const_u32() else {
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
                        let Some(size_expr) = self.const_eval_type_width_designator_expr(size)
                        else {
                            return crate::semantic::types::Type::Err;
                        };
                        let Some(size) = size_expr.get_const_u32() else {
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
                    let Some(size_expr) = self.const_eval_type_width_designator_expr(size) else {
                        return crate::semantic::types::Type::Err;
                    };
                    let Some(size) = size_expr.get_const_u32() else {
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
        is_const: bool,
    ) -> crate::semantic::types::Type {
        let base_tydef = match &array_ty.base_type {
            syntax::ArrayBaseTypeKind::Int(ty) => syntax::TypeDef::Scalar(syntax::ScalarType {
                span: array_ty.span,
                kind: syntax::ScalarTypeKind::Int(ty.clone()),
            }),
            syntax::ArrayBaseTypeKind::UInt(ty) => syntax::TypeDef::Scalar(syntax::ScalarType {
                span: array_ty.span,
                kind: syntax::ScalarTypeKind::UInt(ty.clone()),
            }),
            syntax::ArrayBaseTypeKind::Float(ty) => syntax::TypeDef::Scalar(syntax::ScalarType {
                span: array_ty.span,
                kind: syntax::ScalarTypeKind::Float(ty.clone()),
            }),
            syntax::ArrayBaseTypeKind::Complex(ty) => syntax::TypeDef::Scalar(syntax::ScalarType {
                span: array_ty.span,
                kind: syntax::ScalarTypeKind::Complex(ty.clone()),
            }),
            syntax::ArrayBaseTypeKind::Angle(ty) => syntax::TypeDef::Scalar(syntax::ScalarType {
                span: array_ty.span,
                kind: syntax::ScalarTypeKind::Angle(ty.clone()),
            }),
            syntax::ArrayBaseTypeKind::BoolType => syntax::TypeDef::Scalar(syntax::ScalarType {
                span: array_ty.span,
                kind: syntax::ScalarTypeKind::BoolType,
            }),
            syntax::ArrayBaseTypeKind::Duration => syntax::TypeDef::Scalar(syntax::ScalarType {
                span: array_ty.span,
                kind: syntax::ScalarTypeKind::Duration,
            }),
        };

        let base_ty = self.get_semantic_type_from_tydef(&base_tydef, is_const);

        let dims = array_ty
            .dimensions
            .iter()
            .filter_map(|expr| {
                self.const_eval_array_size_designator_expr(expr)
                    .and_then(|expr| expr.get_const_u32())
            })
            .collect::<Vec<_>>();

        if dims.len() != array_ty.dimensions.len() {
            return Type::Err;
        }

        Type::make_array_ty(&dims, &base_ty)
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
            self.push_invalid_literal_cast_error(ty, &rhs.ty, span);
            return err_expr!(Type::Err, span);
        }
        // the lhs has a type, but the rhs may be of a different type with
        // implicit and explicit conversions. We need to cast the rhs to the
        // lhs type, but if that cast fails, we will have already pushed an error
        // and we can't proceed
        self.cast_expr_to_type(ty, &rhs)
    }

    fn lower_measure_expr(&mut self, expr: &syntax::MeasureExpr) -> semantic::Expr {
        let operand = self.lower_gate_operand(&expr.operand);
        let ty = get_measurement_ty_from_gate_operand(&operand);

        let measurement = semantic::MeasureExpr {
            span: expr.span,
            measure_token_span: expr.measure_token_span,
            operand: self.lower_gate_operand(&expr.operand),
        };

        semantic::Expr::new(expr.span, semantic::ExprKind::Measure(measurement), ty)
    }

    #[allow(clippy::too_many_lines)]
    fn get_default_value(&mut self, ty: &Type, span: Span) -> semantic::Expr {
        use semantic::Expr;
        use semantic::ExprKind;
        use semantic::LiteralKind;
        let from_lit_kind =
            |kind| -> Expr { Expr::new(Span::default(), ExprKind::Lit(kind), ty.as_const()) };
        let expr = match ty {
            Type::Angle(_, _) => Some(from_lit_kind(LiteralKind::Angle(Default::default()))),
            Type::Bit(_) => Some(from_lit_kind(LiteralKind::Bit(false))),
            Type::Int(_, _) | Type::UInt(_, _) => Some(from_lit_kind(LiteralKind::Int(0))),
            Type::Bool(_) => Some(from_lit_kind(LiteralKind::Bool(false))),
            Type::Float(_, _) => Some(from_lit_kind(LiteralKind::Float(0.0))),
            Type::Complex(_, _) => Some(from_lit_kind(LiteralKind::Complex(0.0, 0.0))),
            Type::Stretch(_) => {
                let message = "stretch default values";
                self.push_unsupported_error_message(message, span);
                None
            }
            Type::Qubit => {
                let message = "qubit default values";
                self.push_unsupported_error_message(message, span);
                None
            }
            Type::HardwareQubit => {
                let message = "hardware qubit default values";
                self.push_unsupported_error_message(message, span);
                None
            }
            Type::QubitArray(_) => {
                let message = "qubit array default values";
                self.push_unsupported_error_message(message, span);
                None
            }
            Type::BitArray(size, _) => Some(from_lit_kind(semantic::LiteralKind::Bitstring(
                BigInt::ZERO,
                *size,
            ))),
            Type::Duration(_) => Some(from_lit_kind(LiteralKind::Duration(
                0.0,
                semantic::TimeUnit::Ns,
            ))),
            Type::BoolArray(dims) => {
                let base_ty = Type::Bool(false);
                let default = || self.get_default_value(&base_ty, span);
                Some(from_lit_kind(LiteralKind::Array(
                    semantic::Array::from_default(dims.clone(), default, &base_ty),
                )))
            }
            Type::DurationArray(dims) => {
                let base_ty = Type::Duration(false);
                let default = || self.get_default_value(&Type::Duration(true), span);
                Some(from_lit_kind(LiteralKind::Array(
                    semantic::Array::from_default(dims.clone(), default, &base_ty),
                )))
            }
            Type::AngleArray(width, dims) => {
                let base_ty = Type::Angle(*width, false);
                let default = || self.get_default_value(&base_ty, span);
                Some(from_lit_kind(LiteralKind::Array(
                    semantic::Array::from_default(dims.clone(), default, &base_ty),
                )))
            }
            Type::ComplexArray(width, dims) => {
                let base_ty = Type::Complex(*width, false);
                let default = || self.get_default_value(&base_ty, span);
                Some(from_lit_kind(LiteralKind::Array(
                    semantic::Array::from_default(dims.clone(), default, &base_ty),
                )))
            }
            Type::FloatArray(width, dims) => {
                let base_ty = Type::Float(*width, false);
                let default = || self.get_default_value(&base_ty, span);
                Some(from_lit_kind(LiteralKind::Array(
                    semantic::Array::from_default(dims.clone(), default, &base_ty),
                )))
            }
            Type::IntArray(width, dims) => {
                let base_ty = Type::Int(*width, false);
                let default = || self.get_default_value(&base_ty, span);
                Some(from_lit_kind(LiteralKind::Array(
                    semantic::Array::from_default(dims.clone(), default, &base_ty),
                )))
            }
            Type::UIntArray(width, dims) => {
                let base_ty = Type::UInt(*width, false);
                let default = || self.get_default_value(&base_ty, span);
                Some(from_lit_kind(LiteralKind::Array(
                    semantic::Array::from_default(dims.clone(), default, &base_ty),
                )))
            }
            Type::Gate(_, _) | Type::Function(..) | Type::Range | Type::Set | Type::Void => {
                let message = format!("default values for {ty}");
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

    pub(crate) fn coerce_literal_expr_to_type(
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
        assert!(rhs.ty.is_const(), "literals must have const types");

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
            match kind {
                semantic::LiteralKind::Int(value) => {
                    // can_cast_literal_with_value_knowledge guarantees that value is 0 or 1
                    return Some(semantic::Expr::new(
                        span,
                        semantic::ExprKind::Lit(semantic::LiteralKind::Bit(*value != 0)),
                        lhs_ty.as_const(),
                    ));
                }
                semantic::LiteralKind::Bool(value) => {
                    return Some(semantic::Expr::new(
                        span,
                        semantic::ExprKind::Lit(semantic::LiteralKind::Bit(*value)),
                        lhs_ty.as_const(),
                    ));
                }
                &semantic::LiteralKind::Angle(value) => {
                    return Some(semantic::Expr::new(
                        span,
                        semantic::ExprKind::Lit(semantic::LiteralKind::Bit(value.value != 0)),
                        lhs_ty.as_const(),
                    ));
                }
                _ => (),
            }
        }
        // if lhs_ty is 1 dim bitarray and rhs is int/uint, we can cast
        let (is_int_to_bit_array, size) = match &lhs_ty {
            Type::BitArray(size, _) => {
                if matches!(rhs.ty, Type::Int(..) | Type::UInt(..)) {
                    (true, *size)
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

                return Some(semantic::Expr::new(
                    span,
                    semantic::ExprKind::Lit(semantic::LiteralKind::Bitstring(value, size)),
                    lhs_ty.as_const(),
                ));
            }
        }
        if matches!(lhs_ty, Type::UInt(..)) {
            if let semantic::LiteralKind::Int(value) = kind {
                // this should have been validated by can_cast_literal_with_value_knowledge
                return Some(semantic::Expr::new(
                    span,
                    semantic::ExprKind::Lit(semantic::LiteralKind::Int(*value)),
                    lhs_ty.as_const(),
                ));
            }
        }
        let result = match (&lhs_ty, &rhs_ty) {
            (Type::Float(..), Type::Int(..) | Type::UInt(..)) => {
                if let semantic::LiteralKind::Int(value) = kind {
                    if let Some(value) = safe_i64_to_f64(*value) {
                        return Some(semantic::Expr::new(
                            span,
                            semantic::ExprKind::Lit(semantic::LiteralKind::Float(value)),
                            lhs_ty.as_const(),
                        ));
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
                    return Some(semantic::Expr::new(
                        span,
                        semantic::ExprKind::Lit(semantic::LiteralKind::Angle(
                            Angle::from_f64_maybe_sized(*value, *width),
                        )),
                        lhs_ty.as_const(),
                    ));
                }
                None
            }
            (Type::Angle(width, _), Type::Int(..) | Type::UInt(..)) => {
                // compatibility case for existing code
                if let semantic::LiteralKind::Int(value) = kind {
                    if *value == 0 {
                        return Some(semantic::Expr::new(
                            span,
                            semantic::ExprKind::Lit(semantic::LiteralKind::Angle(
                                Angle::from_u64_maybe_sized(0, *width),
                            )),
                            lhs_ty.as_const(),
                        ));
                    }
                }
                None
            }
            (Type::Float(..), Type::Float(..)) => {
                if let semantic::LiteralKind::Float(value) = kind {
                    return Some(semantic::Expr::new(
                        span,
                        semantic::ExprKind::Lit(semantic::LiteralKind::Float(*value)),
                        lhs_ty.as_const(),
                    ));
                }
                None
            }
            (Type::Complex(..), Type::Complex(..)) => {
                if let semantic::LiteralKind::Complex(real, imag) = kind {
                    return Some(semantic::Expr::new(
                        span,
                        semantic::ExprKind::Lit(semantic::LiteralKind::Complex(*real, *imag)),
                        lhs_ty.as_const(),
                    ));
                }
                None
            }
            (Type::Complex(..), Type::Float(..)) => {
                if let semantic::LiteralKind::Float(value) = kind {
                    return Some(semantic::Expr::new(
                        span,
                        semantic::ExprKind::Lit(semantic::LiteralKind::Complex(*value, 0.0)),
                        lhs_ty.as_const(),
                    ));
                }
                None
            }
            (Type::Complex(..), Type::Int(..) | Type::UInt(..)) => {
                // complex requires a double as input, so we need to
                // convert the int to a double, then create the complex
                if let semantic::LiteralKind::Int(value) = kind {
                    if let Some(value) = safe_i64_to_f64(*value) {
                        return Some(semantic::Expr::new(
                            span,
                            semantic::ExprKind::Lit(semantic::LiteralKind::Complex(value, 0.0)),
                            lhs_ty.as_const(),
                        ));
                    }
                    let kind = SemanticErrorKind::InvalidCastValueRange(
                        "int".to_string(),
                        "float".to_string(),
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
                        return Some(semantic::Expr::new(
                            span,
                            semantic::ExprKind::Lit(semantic::LiteralKind::Int(*value)),
                            lhs_ty.as_const(),
                        ));
                    }
                    panic!("value must be 0 or 1");
                } else {
                    panic!("literal must be an Int");
                }
            }
            (Type::Int(width, _), Type::Int(_, _) | Type::UInt(_, _)) => {
                // we've already checked that this conversion can happen from a signed to unsigned int
                match kind {
                    semantic::LiteralKind::Int(value) => {
                        return Some(semantic::Expr::new(
                            span,
                            semantic::ExprKind::Lit(semantic::LiteralKind::Int(*value)),
                            lhs_ty.as_const(),
                        ));
                    }
                    semantic::LiteralKind::BigInt(value) => {
                        if let Some(width) = width {
                            let mut cap = BigInt::from_i64(1).expect("1 is a valid i64");
                            BigInt::shl_assign(&mut cap, width);
                            if *value >= cap {
                                self.push_semantic_error(SemanticErrorKind::InvalidCastValueRange(
                                    value.to_string(),
                                    format!("int[{width}]"),
                                    span,
                                ));
                                return None;
                            }
                        }
                        return Some(semantic::Expr::new(
                            span,
                            semantic::ExprKind::Lit(semantic::LiteralKind::BigInt(value.clone())),
                            lhs_ty.as_const(),
                        ));
                    }
                    _ => panic!("literal must be an Int or BigInt"),
                }
            }
            _ => None,
        };
        if result.is_none() {
            // we assert that the type can be casted
            // but we didn't cast it, so this is a bug
            panic!("literal type cast failed lhs: {:?}, rhs: {:?}", ty, rhs.ty);
        } else {
            result
        }
    }

    fn cast_expr_to_type(&mut self, ty: &Type, expr: &semantic::Expr) -> semantic::Expr {
        self.cast_expr_to_type_with_span(ty, expr, expr.span)
    }

    fn cast_expr_to_type_with_span(
        &mut self,
        ty: &Type,
        expr: &semantic::Expr,
        span: Span,
    ) -> semantic::Expr {
        let Some(cast_expr) = Self::try_cast_expr_to_type(ty, expr) else {
            self.push_invalid_cast_error(ty, &expr.ty, span);
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
        if ty.is_err() || expr.ty.is_err() {
            // if either type is an error, we can't cast
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
                    return Some(wrap_expr_in_cast_expr(ty.clone(), expr.clone()));
                }

                if *w1 >= *w2 {
                    return Some(wrap_expr_in_cast_expr(ty.clone(), expr.clone()));
                }
            }
            _ => {}
        }
        // Casting of literals is handled elsewhere. This is for casting expressions
        // which cannot be bypassed and must be handled by built-in Q# casts from
        // the standard library.
        match &expr.ty {
            Type::Angle(width, _) => Self::cast_angle_expr_to_type(ty, expr, *width),
            Type::Bit(..) => Self::cast_bit_expr_to_type(ty, expr),
            Type::Bool(..) => Self::cast_bool_expr_to_type(ty, expr),
            Type::Complex(..) => cast_complex_expr_to_type(ty, expr),
            Type::Float(..) => Self::cast_float_expr_to_type(ty, expr),
            Type::Int(width, _) | Type::UInt(width, _) => {
                Self::cast_int_expr_to_type(ty, expr, *width)
            }
            Type::BitArray(size, _) => Self::cast_bitarray_expr_to_type(*size, ty, expr),
            _ => None,
        }
    }

    /// +----------------+----------------------------------------------------------------+
    /// | Allowed casts  | Casting To                                                     |
    /// +----------------+-------+-----+------+-------+-------+--------+----------+-------+
    /// | Casting From   | bool  | int | uint | float | angle | bit[n] | duration | qubit |
    /// +----------------+-------+-----+------+-------+-------+--------+----------+-------+
    /// | angle          | Yes   | No  | No   | No    | -     | Yes    | No       | No    |
    /// +----------------+-------+-----+------+-------+-------+--------+----------+-------+
    fn cast_angle_expr_to_type(
        ty: &Type,
        rhs: &semantic::Expr,
        width: Option<u32>,
    ) -> Option<semantic::Expr> {
        assert!(matches!(rhs.ty, Type::Angle(..)));
        match ty {
            Type::Angle(..) | Type::Bit(..) | Type::Bool(..) => {
                Some(wrap_expr_in_cast_expr(ty.clone(), rhs.clone()))
            }
            Type::BitArray(size, _) if Some(*size) == width => {
                Some(wrap_expr_in_cast_expr(ty.clone(), rhs.clone()))
            }
            _ => None,
        }
    }

    /// +----------------+----------------------------------------------------------------+
    /// | Allowed casts  | Casting To                                                     |
    /// +----------------+-------+-----+------+-------+-------+--------+----------+-------+
    /// | Casting From   | bool  | int | uint | float | angle | bit[n] | duration | qubit |
    /// +----------------+-------+-----+------+-------+-------+--------+----------+-------+
    /// | bit            | Yes   | Yes | Yes  | No    | Yes   | -      | No       | No    |
    /// +----------------+-------+-----+------+-------+-------+--------+----------+-------+
    fn cast_bit_expr_to_type(ty: &Type, rhs: &semantic::Expr) -> Option<semantic::Expr> {
        assert!(matches!(rhs.ty, Type::Bit(..)));
        // There is no operand, choosing the span of the node
        // but we could use the expr span as well.
        match ty {
            Type::Float(..)
            | Type::Bool(..)
            | Type::Int(..)
            | Type::UInt(..)
            | Type::BitArray(..) => Some(wrap_expr_in_cast_expr(ty.clone(), rhs.clone())),

            _ => None,
        }
    }

    /// +----------------+----------------------------------------------------------------+
    /// | Allowed casts  | Casting To                                                     |
    /// +----------------+-------+-----+------+-------+-------+--------+----------+-------+
    /// | Casting From   | bool  | int | uint | float | angle | bit[n] | duration | qubit |
    /// +----------------+-------+-----+------+-------+-------+--------+----------+-------+
    /// | float          | Yes   | Yes | Yes  | -     | Yes   | No     | No       | No    |
    /// +----------------+-------+-----+------+-------+-------+--------+----------+-------+
    ///
    /// Additional cast to complex
    fn cast_float_expr_to_type(ty: &Type, rhs: &semantic::Expr) -> Option<semantic::Expr> {
        assert!(matches!(rhs.ty, Type::Float(..)));
        match ty {
            Type::Angle(..)
            | Type::Int(..)
            | Type::UInt(..)
            | Type::Float(..)
            | Type::Bool(..)
            | Type::Bit(..) => Some(wrap_expr_in_cast_expr(ty.clone(), rhs.clone())),
            Type::Complex(..) => {
                // Even though the spec doesn't say it, we need to allow
                // casting from float to complex, else this kind of expression
                // would be invalid: 2.0 + sin(pi) + 1.0i
                Some(wrap_expr_in_cast_expr(ty.clone(), rhs.clone()))
            }
            _ => None,
        }
    }

    /// +----------------+----------------------------------------------------------------+
    /// | Allowed casts  | Casting To                                                     |
    /// +----------------+-------+-----+------+-------+-------+--------+----------+-------+
    /// | Casting From   | bool  | int | uint | float | angle | bit[n] | duration | qubit |
    /// +----------------+-------+-----+------+-------+-------+--------+----------+-------+
    /// | bool           | -     | Yes | Yes  | Yes   | No    | Yes    | No       | No    |
    /// +----------------+-------+-----+------+-------+-------+--------+----------+-------+
    fn cast_bool_expr_to_type(ty: &Type, rhs: &semantic::Expr) -> Option<semantic::Expr> {
        assert!(matches!(rhs.ty, Type::Bool(..)));
        match ty {
            Type::Bit(..)
            | Type::BitArray(..)
            | Type::Float(..)
            | Type::Int(..)
            | Type::UInt(..) => Some(wrap_expr_in_cast_expr(ty.clone(), rhs.clone())),
            _ => None,
        }
    }

    /// +----------------+----------------------------------------------------------------+
    /// | Allowed casts  | Casting To                                                     |
    /// +----------------+-------+-----+------+-------+-------+--------+----------+-------+
    /// | Casting From   | bool  | int | uint | float | angle | bit[n] | duration | qubit |
    /// +----------------+-------+-----+------+-------+-------+--------+----------+-------+
    /// | int            | Yes   | -   | Yes  | Yes   | No    | Yes    | No       | No    |
    /// +----------------+-------+-----+------+-------+-------+--------+----------+-------+
    /// | uint           | Yes   | Yes | -    | Yes   | No    | Yes    | No       | No    |
    /// +----------------+-------+-----+------+-------+-------+--------+----------+-------+
    ///
    /// Additional cast to ``BigInt``
    #[allow(clippy::too_many_lines)]
    fn cast_int_expr_to_type(
        ty: &Type,
        rhs: &semantic::Expr,
        width: Option<u32>,
    ) -> Option<semantic::Expr> {
        assert!(matches!(rhs.ty, Type::Int(..) | Type::UInt(..)));

        match ty {
            Type::Float(..)
            | Type::Int(..)
            | Type::UInt(..)
            | Type::Bool(..)
            | Type::Bit(..)
            // Even though the spec doesn't say it, we need to allow
            // casting from int to complex, else this kind of expression
            // would be invalid: 2 + 1i
            | Type::Complex(..) => Some(wrap_expr_in_cast_expr(ty.clone(), rhs.clone())),
            Type::BitArray(size, _) if Some(*size) == width => {
                Some(wrap_expr_in_cast_expr(ty.clone(), rhs.clone()))
            }
            _ => None,
        }
    }

    fn cast_bitarray_expr_to_type(
        array_width: u32,
        ty: &Type,
        rhs: &semantic::Expr,
    ) -> Option<semantic::Expr> {
        match ty {
            Type::Bool(..) | Type::Bit(..) | Type::Int(None, _) | Type::UInt(None, _) => {
                Some(wrap_expr_in_cast_expr(ty.clone(), rhs.clone()))
            }
            Type::Angle(Some(width), _)
            | Type::Int(Some(width), _)
            | Type::UInt(Some(width), _)
                if *width == array_width =>
            {
                Some(wrap_expr_in_cast_expr(ty.clone(), rhs.clone()))
            }
            _ => None,
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

        // Bit shifts with int lhs are not allowed in the spec.
        // If we have an int literal in the lhs, we reinterpret it as a uint.
        let lhs = if matches!(op, syntax::BinOp::Shl | syntax::BinOp::Shr)
            && matches!(
                &*lhs.kind,
                semantic::ExprKind::Lit(semantic::LiteralKind::Int(val)) if *val >= 0
            ) {
            semantic::Expr::new(lhs.span, *lhs.kind, Type::UInt(lhs.ty.width(), true))
        } else {
            lhs
        };

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
                    let kind = SemanticErrorKind::CannotCast(
                        left_type.to_string(),
                        target_ty.to_string(),
                        lhs.span,
                    );
                    self.push_semantic_error(kind);
                }
                if rhs_uint_promotion.is_none() {
                    let kind = SemanticErrorKind::CannotCast(
                        right_type.to_string(),
                        target_ty.to_string(),
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
                let expr = semantic::Expr::new(span, kind, target_ty);
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
            let expr = semantic::Expr::new(span, kind, ty);

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
        } else if binop_requires_symmetric_uint_conversion(op) {
            let ty = Type::UInt(None, ty_constness);
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
                semantic::Expr::new(span, kind, ty.clone())
            } else {
                let kind =
                    SemanticErrorKind::OperatorNotAllowedForComplexValues(format!("{op:?}"), span);
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
            semantic::Expr::new(span, kind, ty.clone())
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
            Type::Bit(..) | Type::UInt(..) | Type::BitArray(_, _)
        )) {
            return true;
        }
        if (matches!(op, syntax::BinOp::Shl | syntax::BinOp::Shr)
            && matches!(
                left_type,
                Type::Bit(..) | Type::UInt(..) | Type::BitArray(_, _)
            ))
        {
            return true;
        }
        false
    }

    fn binop_requires_bitwise_symmetric_conversion(op: syntax::BinOp) -> bool {
        matches!(
            op,
            syntax::BinOp::AndB
                | syntax::BinOp::OrB
                | syntax::BinOp::XorB
                | syntax::BinOp::Shl
                | syntax::BinOp::Shr
        )
    }

    fn lower_index(&mut self, index: &syntax::Index) -> Option<Vec<semantic::Index>> {
        match index {
            syntax::Index::IndexSet(set) => {
                // According to the grammar: <https://openqasm.com/grammar/index.html>
                //   "`setExpression` is only valid when being used as a single index.
                //    Registers can support it for creating aliases, but arrays cannot."
                self.push_semantic_error(SemanticErrorKind::IndexSetOnlyAllowedInAliasStmt(
                    set.span,
                ));
                None
            }
            syntax::Index::IndexList(multidimensional_index) => {
                self.lower_index_list(multidimensional_index)
            }
        }
    }

    fn lower_enumerable_set(&mut self, set: &syntax::EnumerableSet) -> semantic::EnumerableSet {
        match set {
            syntax::EnumerableSet::Set(set) => semantic::EnumerableSet::Set(self.lower_set(set)),
            syntax::EnumerableSet::Range(range_definition) => {
                semantic::EnumerableSet::Range(self.lower_range(range_definition))
            }
            syntax::EnumerableSet::Expr(expr) => {
                semantic::EnumerableSet::Expr(self.lower_expr(expr))
            }
        }
    }

    fn lower_set(&mut self, set: &syntax::Set) -> semantic::Set {
        let items = set
            .values
            .iter()
            .map(|expr| self.lower_expr(expr))
            .collect::<Vec<_>>();

        semantic::Set {
            span: set.span,
            values: syntax::list_from_iter(items),
        }
    }

    fn lower_index_list(&mut self, list: &syntax::IndexList) -> Option<Vec<semantic::Index>> {
        let indices: Vec<_> = list
            .values
            .iter()
            .filter_map(|index| match &**index {
                syntax::IndexListItem::RangeDefinition(range) => {
                    self.lower_const_range(range).map(semantic::Index::Range)
                }
                syntax::IndexListItem::Expr(expr) => {
                    Some(semantic::Index::Expr(self.lower_expr(expr)))
                }
                syntax::IndexListItem::Err => Some(semantic::Index::Expr(err_expr!(Type::Err))),
            })
            .collect();

        if list.values.len() != indices.len() {
            return None;
        }

        Some(indices)
    }

    /// Ranges used for register and array slicing must be const evaluatable.
    fn lower_const_range(&mut self, range: &syntax::Range) -> Option<semantic::Range> {
        let mut lower_and_const_eval = |expr| {
            let lowered_expr = self.lower_expr(expr);
            let lit_expr = Self::try_cast_expr_to_type(&Type::Int(None, true), &lowered_expr);
            let Some(lowered_expr) = lit_expr else {
                self.push_invalid_cast_error(&Type::Int(None, true), &lowered_expr.ty, expr.span);
                return None;
            };
            // const_eval will push any needed errors
            let lowered_expr = lowered_expr.with_const_value(self);

            let lit = lowered_expr.get_const_value()?;

            Some(semantic::Expr::new(
                lowered_expr.span,
                semantic::ExprKind::Lit(lit.clone()),
                Type::Int(None, true),
            ))
        };

        let start = range.start.as_ref().map(&mut lower_and_const_eval);
        let step = range.step.as_ref().map(&mut lower_and_const_eval);
        let end = range.end.as_ref().map(&mut lower_and_const_eval);

        // The spec says that the step cannot be zero, so we push an error in that case.
        // <https://openqasm.com/language/types.html#register-concatenation-and-slicing>
        if let Some(Some(step)) = &step {
            if let semantic::ExprKind::Lit(semantic::LiteralKind::Int(val)) = &*step.kind {
                if *val == 0 {
                    self.push_semantic_error(SemanticErrorKind::ZeroStepInRange(range.span));
                    return None;
                }
            }
        }

        macro_rules! shortcircuit_inner {
            ($nested_option:expr) => {
                match $nested_option {
                    Some(inner) => Some(inner?),
                    None => None,
                }
            };
        }

        Some(semantic::Range {
            span: range.span,
            start: shortcircuit_inner!(start),
            step: shortcircuit_inner!(step),
            end: shortcircuit_inner!(end),
        })
    }

    /// These ranges as iterators in for loops. The spec says
    /// that `start` and `stop` are mandatory in this case.
    ///
    /// Spec: <https://openqasm.com/language/classical.html#for-loops>
    fn lower_range(&mut self, range: &syntax::Range) -> semantic::Range {
        let start = range.start.as_ref().map(|e| self.lower_expr(e));
        let step = range.step.as_ref().map(|e| self.lower_expr(e));
        let end = range.end.as_ref().map(|e| self.lower_expr(e));

        if start.is_none() {
            self.push_semantic_error(SemanticErrorKind::RangeExpressionsMustHaveStart(range.span));
        }

        if end.is_none() {
            self.push_semantic_error(SemanticErrorKind::RangeExpressionsMustHaveStop(range.span));
        }

        semantic::Range {
            span: range.span,
            start,
            step,
            end,
        }
    }

    fn lower_index_expr(&mut self, expr: &syntax::IndexExpr) -> semantic::Expr {
        let collection = self.lower_expr(&expr.collection);
        let Some(indices) = self.lower_index(&expr.index) else {
            // Since we can't evaluate the indices, we can't know the indexed type.
            return err_expr!(Type::Err, expr.span);
        };

        // The spec says:
        // "One or more dimension(s) of an array can be zero,
        //  in which case the array has size zero. An array of
        //  size zero cannot be indexed, e.g. given
        //  `array[float[32], 0] myArray;`
        //  it is an error to access either myArray[0] or myArray[-1]."
        if collection.ty.has_zero_size() {
            let kind = SemanticErrorKind::ZeroSizeArrayAccess(expr.span);
            self.push_semantic_error(kind);
            return err_expr!(Type::Err, expr.span);
        }

        let indexed_ty = self.get_indexed_type(&collection.ty, expr.span, &indices);

        semantic::Expr::new(
            expr.span,
            semantic::ExprKind::IndexExpr(semantic::IndexExpr {
                span: expr.span,
                collection,
                indices: list_from_iter(indices),
            }),
            indexed_ty,
        )
    }

    fn get_indexed_type(
        &mut self,
        ty: &Type,
        span: Span,
        indices: &[semantic::Index],
    ) -> super::types::Type {
        if !ty.is_array() {
            let kind = SemanticErrorKind::CannotIndexType(ty.to_string(), span);
            self.push_semantic_error(kind);
            return super::types::Type::Err;
        }

        if indices.len() > ty.num_dims() {
            let kind = SemanticErrorKind::TooManyIndices(span);
            self.push_semantic_error(kind);
            return super::types::Type::Err;
        }

        if let Some(indexed_ty) = ty.get_indexed_type(indices) {
            indexed_ty
        } else {
            // we failed to get the indexed type, unknown error
            // we should have caught this earlier with the two checks above
            let kind = SemanticErrorKind::CannotIndexType(ty.to_string(), span);
            self.push_semantic_error(kind);
            super::types::Type::Err
        }
    }

    /// A `syntax::IndexedIdent` is guaranteed to have at least one index.
    /// This changes the type of expression we return to simplify downstream compilation
    fn lower_indexed_ident_expr(&mut self, indexed_ident: &syntax::IndexedIdent) -> semantic::Expr {
        assert!(!indexed_ident.indices.is_empty());

        // We flatten the multiple square brackets, converting
        // a[1, 2][5, 7][2, 4:8]
        //   to
        // a[1, 2, 5, 7, 2, 4:8]
        let mut indices = Vec::new();
        for qasm_index in &indexed_ident.indices {
            if let Some(mut lowered_indices) = self.lower_index(qasm_index) {
                indices.append(&mut lowered_indices);
            }
        }

        if indices.len()
            != indexed_ident
                .indices
                .iter()
                .map(|i| i.num_indices())
                .sum::<usize>()
        {
            // Since we can't evaluate all the indices, we can't know the indexed type.
            return err_expr!(Type::Err, indexed_ident.span);
        }

        let ident = indexed_ident.ident.clone();
        let Some((symbol_id, lhs_symbol)) = self.symbols.get_symbol_by_name(&ident.name) else {
            self.push_missing_symbol_error(ident.name, ident.span);
            return err_expr!(Type::Err, indexed_ident.span);
        };

        let ty = lhs_symbol.ty.clone();

        // The spec says:
        // "One or more dimension(s) of an array can be zero,
        //  in which case the array has size zero. An array of
        //  size zero cannot be indexed, e.g. given
        //  `array[float[32], 0] myArray;`
        //  it is an error to access either myArray[0] or myArray[-1]."
        if ty.has_zero_size() {
            let kind = SemanticErrorKind::ZeroSizeArrayAccess(indexed_ident.span);
            self.push_semantic_error(kind);
            return err_expr!(Type::Err, indexed_ident.span);
        }

        // use the supplied number of indices rather than the number of indices we lowered
        let indexed_ty = self.get_indexed_type(&ty, indexed_ident.span, &indices);

        semantic::Expr::new(
            indexed_ident.span,
            semantic::ExprKind::IndexedIdent(semantic::IndexedIdent {
                span: indexed_ident.span,
                name_span: ident.span,
                index_span: indexed_ident.index_span,
                symbol_id,
                indices: list_from_iter(indices),
            }),
            indexed_ty,
        )
    }

    fn lower_gate_operand(&mut self, operand: &syntax::GateOperand) -> semantic::GateOperand {
        let kind = match &operand.kind {
            syntax::GateOperandKind::IdentOrIndexedIdent(ident_or_indexed_ident) => {
                let expr = match &**ident_or_indexed_ident {
                    syntax::IdentOrIndexedIdent::Ident(ident) => self.lower_ident_expr(ident),
                    syntax::IdentOrIndexedIdent::IndexedIdent(indexed_ident) => {
                        self.lower_indexed_ident_expr(indexed_ident)
                    }
                };

                semantic::GateOperandKind::Expr(Box::new(expr))
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
        if target_ty.is_err() || expr_ty.is_err() {
            // if either type is an error, we don't need to push an error
            return;
        }
        let rhs_ty_name = expr_ty.to_string();
        let lhs_ty_name = target_ty.to_string();
        let kind = SemanticErrorKind::CannotCast(rhs_ty_name, lhs_ty_name, span);
        self.push_semantic_error(kind);
    }

    fn push_invalid_literal_cast_error(&mut self, target_ty: &Type, expr_ty: &Type, span: Span) {
        if target_ty.is_err() || expr_ty.is_err() {
            // if either type is an error, we don't need to push an error
            return;
        }

        let rhs_ty_name = expr_ty.to_string();
        let lhs_ty_name = target_ty.to_string();
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

    /// Pushes a const eval error with the given kind.
    pub fn push_const_eval_error(&mut self, kind: ConstEvalError) {
        let kind = crate::ErrorKind::ConstEval(kind);
        let error = self.create_err(kind);
        self.errors.push(error);
    }

    /// Creates an error from the given kind with the current source mapping.
    fn create_err(&self, kind: crate::ErrorKind) -> WithSource<crate::Error> {
        let error = crate::Error(kind);
        WithSource::from_map(&self.source_map, error)
    }
}

fn wrap_expr_in_cast_expr(ty: Type, rhs: semantic::Expr) -> semantic::Expr {
    semantic::Expr::new(
        rhs.span,
        semantic::ExprKind::Cast(semantic::Cast {
            span: Span::default(),
            expr: rhs,
            ty: ty.clone(),
        }),
        ty,
    )
}

fn get_measurement_ty_from_gate_operand(operand: &semantic::GateOperand) -> Type {
    if let semantic::GateOperandKind::Expr(ref expr) = &operand.kind {
        if let Type::QubitArray(size) = expr.ty {
            return Type::BitArray(size, false);
        }
    }

    Type::Bit(false)
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
        // complex can only cast to complex. We do the same as for floats
        // when handling the widths, that is, ignoring them, since complex
        // numbers are a pair of floats.
        return Some(rhs.clone());
    }
    None
}

fn get_identifier_name(identifier: &syntax::IdentOrIndexedIdent) -> std::rc::Rc<str> {
    match identifier {
        syntax::IdentOrIndexedIdent::Ident(ident) => ident.name.clone(),
        syntax::IdentOrIndexedIdent::IndexedIdent(ident) => ident.ident.name.clone(),
    }
}

fn try_get_qsharp_name_and_implicit_modifiers<S: AsRef<str>>(
    gate_name: S,
    name_span: Span,
) -> Option<(String, semantic::QuantumGateModifier)> {
    use semantic::GateModifierKind::*;

    let make_modifier = |kind| semantic::QuantumGateModifier {
        span: name_span,
        modifier_keyword_span: name_span,
        kind,
    };

    let ctrl_expr = Expr::uint(1, Span::default());

    match gate_name.as_ref() {
        "cy" => Some(("y".to_string(), make_modifier(Ctrl(ctrl_expr)))),
        "cz" => Some(("z".to_string(), make_modifier(Ctrl(ctrl_expr)))),
        "ch" => Some(("h".to_string(), make_modifier(Ctrl(ctrl_expr)))),
        "crx" => Some(("rx".to_string(), make_modifier(Ctrl(ctrl_expr)))),
        "cry" => Some(("ry".to_string(), make_modifier(Ctrl(ctrl_expr)))),
        "crz" => Some(("rz".to_string(), make_modifier(Ctrl(ctrl_expr)))),
        "cswap" => Some(("swap".to_string(), make_modifier(Ctrl(ctrl_expr)))),
        "sdg" => Some(("s".to_string(), make_modifier(Inv))),
        "tdg" => Some(("t".to_string(), make_modifier(Inv))),
        // Gates for OpenQASM 2 backwards compatibility
        "CX" => Some(("x".to_string(), make_modifier(Ctrl(ctrl_expr)))),
        "cphase" => Some(("phase".to_string(), make_modifier(Ctrl(ctrl_expr)))),
        _ => None,
    }
}
