// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::path::PathBuf;

use super::types::Type;
use qsc_data_structures::span::Span;
use qsc_frontend::{compile::SourceMap, error::WithSource};

use super::symbols::{IOKind, Symbol, SymbolTable};

use crate::parser::QasmSource;

use super::{
    ast::{list_from_iter, Stmt, Version},
    SemanticErrorKind,
};

pub(super) struct Analyzer {
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
impl Analyzer {
    pub fn analyze(mut self) -> crate::semantic::QasmSemanticParseResult {
        // Should we fail if we see a version in included files?
        let source = &self.source.clone();
        self.version = self.analyze_version(source.program().version);

        self.analyze_source(source);

        let program = super::ast::Program {
            version: self.version,
            statements: list_from_iter(self.stmts),
        };

        super::QasmSemanticParseResult {
            source: self.source,
            source_map: self.source_map,
            symbols: self.symbols,
            program,
            errors: self.errors,
        }
    }

    fn analyze_version(&mut self, version: Option<crate::ast::Version>) -> Option<Version> {
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

    /// Root recursive function for analyzing the source.
    fn analyze_source(&mut self, source: &QasmSource) {
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
                crate::ast::StmtKind::Include(include) => {
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
                    self.analyze_source(include);
                }
                _ => {
                    if let Some(stmt) = self.analyze_stmt(stmt) {
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
    fn analyze_stmt(&mut self, stmt: &crate::ast::Stmt) -> Option<super::ast::Stmt> {
        let kind = match &*stmt.kind {
            crate::ast::StmtKind::Alias(stmt) => {
                super::ast::StmtKind::Alias(self.analyze_alias(stmt)?)
            }
            crate::ast::StmtKind::Assign(stmt) => {
                super::ast::StmtKind::Assign(self.analyze_assign(stmt)?)
            }
            crate::ast::StmtKind::AssignOp(stmt) => {
                super::ast::StmtKind::AssignOp(self.analyze_assign_op(stmt)?)
            }
            crate::ast::StmtKind::Barrier(stmt) => {
                super::ast::StmtKind::Barrier(self.analyze_barrier(stmt)?)
            }
            crate::ast::StmtKind::Box(stmt) => super::ast::StmtKind::Box(self.analyze_box(stmt)?),
            crate::ast::StmtKind::Break(stmt) => {
                super::ast::StmtKind::Break(self.analyze_break(stmt)?)
            }
            crate::ast::StmtKind::Block(stmt) => {
                super::ast::StmtKind::Block(Box::new(self.analyze_block(stmt)?))
            }
            crate::ast::StmtKind::Cal(stmt) => {
                super::ast::StmtKind::Cal(self.analyze_calibration(stmt)?)
            }
            crate::ast::StmtKind::CalibrationGrammar(stmt) => {
                super::ast::StmtKind::CalibrationGrammar(self.analyze_calibration_grammar(stmt)?)
            }
            crate::ast::StmtKind::ClassicalDecl(stmt) => {
                super::ast::StmtKind::ClassicalDecl(self.analyze_classical_decl(stmt)?)
            }
            crate::ast::StmtKind::ConstDecl(stmt) => {
                super::ast::StmtKind::ConstDecl(self.analyze_const_decl(stmt)?)
            }
            crate::ast::StmtKind::Continue(stmt) => {
                super::ast::StmtKind::Continue(self.analyze_continue_stmt(stmt)?)
            }
            crate::ast::StmtKind::Def(stmt) => super::ast::StmtKind::Def(self.analyze_def(stmt)?),
            crate::ast::StmtKind::DefCal(stmt) => {
                super::ast::StmtKind::DefCal(self.analyze_def_cal(stmt)?)
            }
            crate::ast::StmtKind::Delay(stmt) => {
                super::ast::StmtKind::Delay(self.analyze_delay(stmt)?)
            }
            crate::ast::StmtKind::Empty => {
                // we ignore empty statements
                None?
            }
            crate::ast::StmtKind::End(stmt) => {
                super::ast::StmtKind::End(self.analyze_end_stmt(stmt)?)
            }
            crate::ast::StmtKind::ExprStmt(stmt) => {
                super::ast::StmtKind::ExprStmt(self.analyze_expr_stmt(stmt)?)
            }
            crate::ast::StmtKind::ExternDecl(extern_decl) => {
                super::ast::StmtKind::ExternDecl(self.analyze_extern(extern_decl)?)
            }
            crate::ast::StmtKind::For(stmt) => {
                super::ast::StmtKind::For(self.analyze_for_stmt(stmt)?)
            }
            crate::ast::StmtKind::If(stmt) => super::ast::StmtKind::If(self.analyze_if_stmt(stmt)?),
            crate::ast::StmtKind::GateCall(stmt) => {
                super::ast::StmtKind::GateCall(self.analyze_gate_call(stmt)?)
            }
            crate::ast::StmtKind::GPhase(stmt) => {
                super::ast::StmtKind::GPhase(self.analyze_gphase(stmt)?)
            }
            crate::ast::StmtKind::Include(stmt) => {
                super::ast::StmtKind::Include(self.analyze_include(stmt)?)
            }
            crate::ast::StmtKind::IODeclaration(stmt) => {
                super::ast::StmtKind::IODeclaration(self.analyze_io_decl(stmt)?)
            }
            crate::ast::StmtKind::Measure(stmt) => {
                super::ast::StmtKind::Measure(self.analyze_measure(stmt)?)
            }
            crate::ast::StmtKind::Pragma(stmt) => {
                super::ast::StmtKind::Pragma(self.analyze_pragma(stmt)?)
            }
            crate::ast::StmtKind::QuantumGateDefinition(stmt) => {
                super::ast::StmtKind::QuantumGateDefinition(self.analyze_gate_def(stmt)?)
            }
            crate::ast::StmtKind::QuantumDecl(stmt) => {
                super::ast::StmtKind::QuantumDecl(self.analyze_quantum_decl(stmt)?)
            }
            crate::ast::StmtKind::Reset(stmt) => {
                super::ast::StmtKind::Reset(self.analyze_reset(stmt)?)
            }
            crate::ast::StmtKind::Return(stmt) => {
                super::ast::StmtKind::Return(self.analyze_return(stmt)?)
            }
            crate::ast::StmtKind::Switch(stmt) => {
                super::ast::StmtKind::Switch(self.analyze_switch(stmt)?)
            }
            crate::ast::StmtKind::WhileLoop(stmt) => {
                super::ast::StmtKind::WhileLoop(self.analyze_while_loop(stmt)?)
            }
            crate::ast::StmtKind::Err => {
                self.push_semantic_error(SemanticErrorKind::UnexpectedParserError(
                    "Unexpected error".to_string(),
                    stmt.span,
                ));
                return None;
            }
        };
        let annotations = self.analyze_annotations(&stmt.annotations, &stmt.kind);
        Some(super::ast::Stmt {
            span: stmt.span,
            annotations: list_from_iter(annotations),
            kind: Box::new(kind),
        })
    }

    /// Define the standard gates in the symbol table.
    /// The sdg, tdg, crx, cry, crz, and ch are defined
    /// as their bare gates, and modifiers are applied
    /// when calling them.
    fn define_stdgates(&mut self, include: &crate::ast::IncludeStmt) {
        fn gate_symbol(name: &str, cargs: usize, qargs: usize) -> Symbol {
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
        let kind = crate::ErrorKind::Semantic(crate::semantic::Error(kind));
        let error = self.create_err(kind);
        self.errors.push(error);
    }

    /// Pushes a redefined symbol error with the given name and span.
    /// This is a convenience method for pushing a `SemanticErrorKind::RedefinedSymbol` error.
    pub fn push_redefined_symbol_error<S: AsRef<str>>(&mut self, name: S, span: Span) {
        let kind = SemanticErrorKind::RedefinedSymbol(name.as_ref().to_string(), span);
        self.push_semantic_error(kind);
    }

    /// Pushes a semantic error with the given kind.
    pub fn push_semantic_error(&mut self, kind: SemanticErrorKind) {
        let kind = crate::ErrorKind::Semantic(crate::semantic::Error(kind));
        let error = self.create_err(kind);
        self.errors.push(error);
    }

    /// Pushes an unsupported error with the supplied message.
    pub fn push_unsupported_error_message<S: AsRef<str>>(&mut self, message: S, span: Span) {
        let kind = crate::ErrorKind::Semantic(crate::semantic::Error(
            SemanticErrorKind::NotSupported(message.as_ref().to_string(), span),
        ));
        let error = self.create_err(kind);
        self.errors.push(error);
    }

    /// Pushes an unimplemented error with the supplied message.
    pub fn push_unimplemented_error_message<S: AsRef<str>>(&mut self, message: S, span: Span) {
        let kind = crate::ErrorKind::Semantic(crate::semantic::Error(
            SemanticErrorKind::Unimplemented(message.as_ref().to_string(), span),
        ));
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

    fn analyze_alias(
        &mut self,
        alias: &crate::ast::AliasDeclStmt,
    ) -> Option<super::ast::AliasDeclStmt> {
        let name = get_identifier_name(&alias.ident);
        // alias statements do their types backwards, you read the right side
        // and assign it to the left side.
        // the types of the rhs should be in the symbol table.
        let rhs = alias
            .exprs
            .iter()
            .filter_map(|expr| self.analyze_expr(expr))
            .collect::<Vec<_>>();
        // TODO: handle multiple rhs
        // TODO: validate consistency of rhs types
        let first = rhs.first().expect("missing rhs");
        let symbol = Symbol {
            name: name.to_string(),
            ty: first.ty.clone(),
            qsharp_ty: self.convert_semantic_type_to_qsharp_type(&first.ty, alias.span)?,
            span: alias.span,
            io_kind: IOKind::Default,
        };
        let Ok(symbol_id) = self.symbols.insert_symbol(symbol) else {
            self.push_redefined_symbol_error(name, alias.span);
            return None;
        };

        if rhs.len() != alias.exprs.len() {
            // we failed
            return None;
        }
        Some(super::ast::AliasDeclStmt {
            span: alias.span,
            symbol_id,
            exprs: list_from_iter(rhs),
        })
    }

    fn analyze_expr(&mut self, expr: &crate::ast::Expr) -> Option<super::ast::Expr> {
        match &*expr.kind {
            crate::ast::ExprKind::Assign(assign_expr) => todo!(),
            crate::ast::ExprKind::AssignOp(assign_op_expr) => todo!(),
            crate::ast::ExprKind::BinaryOp(binary_op_expr) => todo!(),
            crate::ast::ExprKind::Cast(cast_expr) => todo!(),
            crate::ast::ExprKind::Err => {
                unreachable!("Err expr should not be analyzed");
            }
            crate::ast::ExprKind::FunctionCall(function_call) => todo!(),
            crate::ast::ExprKind::Ident(ident) => self.analyze_ident_expr(ident),
            crate::ast::ExprKind::IndexExpr(index_expr) => todo!(),

            crate::ast::ExprKind::Lit(lit) => self.analyze_lit_expr(lit),

            crate::ast::ExprKind::Paren(expr) => self.analyze_paren_expr(expr),
            crate::ast::ExprKind::UnaryOp(unary_op_expr) => todo!(),
        }
    }

    fn analyze_ident_expr(&mut self, ident: &crate::ast::Ident) -> Option<super::ast::Expr> {
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

    fn analyze_lit_expr(&mut self, expr: &crate::ast::Lit) -> Option<super::ast::Expr> {
        let (kind, ty) = match &expr.kind {
            crate::ast::LiteralKind::BigInt(value) => {
                // todo: this case is only valid when there is an integer literal
                // that requires more than 64 bits to represent. We should probably
                // introduce a new type for this as openqasm promotion rules don't
                // cover this case as far as I know.
                (super::ast::LiteralKind::BigInt(value.clone()), Type::Err)
            }
            crate::ast::LiteralKind::Bitstring(value, size) => (
                super::ast::LiteralKind::Bitstring(value.clone(), *size),
                Type::BitArray(super::types::ArrayDimensions::One(*size), true),
            ),
            crate::ast::LiteralKind::Bool(value) => {
                (super::ast::LiteralKind::Bool(*value), Type::Bool(true))
            }
            crate::ast::LiteralKind::Int(value) => {
                (super::ast::LiteralKind::Int(*value), Type::Int(None, true))
            }
            crate::ast::LiteralKind::Float(value) => (
                super::ast::LiteralKind::Float(*value),
                Type::Float(None, true),
            ),
            crate::ast::LiteralKind::Imaginary(value) => (
                super::ast::LiteralKind::Complex(0.0, *value),
                Type::Complex(None, true),
            ),
            crate::ast::LiteralKind::String(_) => {
                self.push_unsupported_error_message("String literals", expr.span);
                return None;
            }
            crate::ast::LiteralKind::Duration { value, unit } => {
                self.push_unsupported_error_message("Duration literals", expr.span);
                return None;
            }
            crate::ast::LiteralKind::Array(exprs) => {
                // array literals are only valid in classical decals (const and mut)
                // and we have to know the expected type of the array in order to analyze it
                // So we can't analyze array literals in general.
                self.push_semantic_error(SemanticErrorKind::ArrayLiteralInNonClassicalDecl(
                    expr.span,
                ));
                // place holder for now, this code will need to move to the correct place when we
                // add support for classical decls
                let texprs = exprs
                    .iter()
                    .filter_map(|expr| self.analyze_expr(expr))
                    .collect::<Vec<_>>();
                if texprs.len() != exprs.len() {
                    // we failed to analyze all the entries and an error was pushed
                    return None;
                }

                (
                    super::ast::LiteralKind::Array(list_from_iter(texprs)),
                    Type::Err,
                )
            }
        };
        Some(super::ast::Expr {
            span: expr.span,
            kind: Box::new(super::ast::ExprKind::Lit(super::ast::Lit {
                span: expr.span,
                kind,
            })),
            ty,
        })
    }

    fn analyze_paren_expr(&mut self, expr: &crate::ast::Expr) -> Option<super::ast::Expr> {
        let expr = self.analyze_expr(expr)?;
        let span = expr.span;
        let ty = expr.ty.clone();
        let kind = super::ast::ExprKind::Paren(expr);
        Some(super::ast::Expr {
            span,
            kind: Box::new(kind),
            ty,
        })
    }

    fn analyze_annotations(
        &mut self,
        annotations: &[Box<crate::ast::Annotation>],
        kind: &crate::ast::StmtKind,
    ) -> Vec<super::ast::Annotation> {
        annotations
            .iter()
            .map(|annotation| self.analyze_annotation(annotation, kind))
            .collect::<Vec<_>>()
    }

    fn analyze_annotation(
        &mut self,
        annotation: &crate::ast::Annotation,
        kind: &crate::ast::StmtKind,
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

        if let crate::ast::StmtKind::GateCall(_) = &kind {
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

    fn analyze_assign(&mut self, stmt: &crate::ast::Assign) -> Option<super::ast::Assign> {
        self.push_unimplemented_error_message("assign stmt", stmt.span);
        None
    }

    fn analyze_assign_op(&mut self, stmt: &crate::ast::AssignOp) -> Option<super::ast::AssignOp> {
        self.push_unimplemented_error_message("assign op stmt", stmt.span);
        None
    }

    fn analyze_barrier(
        &mut self,
        stmt: &crate::ast::BarrierStmt,
    ) -> Option<super::ast::BarrierStmt> {
        self.push_unimplemented_error_message("barrier stmt", stmt.span);
        None
    }

    fn analyze_box(&mut self, stmt: &crate::ast::BoxStmt) -> Option<super::ast::BoxStmt> {
        self.push_unimplemented_error_message("box stmt", stmt.span);
        None
    }

    fn analyze_break(&mut self, stmt: &crate::ast::BreakStmt) -> Option<super::ast::BreakStmt> {
        self.push_unimplemented_error_message("break stmt", stmt.span);
        None
    }

    fn analyze_block(&mut self, stmt: &crate::ast::Block) -> Option<super::ast::Block> {
        self.push_unimplemented_error_message("block stmt", stmt.span);
        None
    }

    fn analyze_calibration(
        &mut self,
        stmt: &crate::ast::CalibrationStmt,
    ) -> Option<super::ast::CalibrationStmt> {
        self.push_unimplemented_error_message("calibration stmt", stmt.span);
        None
    }

    fn analyze_calibration_grammar(
        &mut self,
        stmt: &crate::ast::CalibrationGrammarStmt,
    ) -> Option<super::ast::CalibrationGrammarStmt> {
        self.push_unimplemented_error_message("calibration stmt", stmt.span);
        None
    }

    fn analyze_classical_decl(
        &mut self,
        stmt: &crate::ast::ClassicalDeclarationStmt,
    ) -> Option<super::ast::ClassicalDeclarationStmt> {
        self.push_unimplemented_error_message("classical decl stmt", stmt.span);
        None
    }

    fn analyze_const_decl(
        &mut self,
        stmt: &crate::ast::ConstantDeclStmt,
    ) -> Option<super::ast::ConstantDeclStmt> {
        self.push_unimplemented_error_message("const decl stmt", stmt.span);
        None
    }

    fn analyze_continue_stmt(
        &mut self,
        stmt: &crate::ast::ContinueStmt,
    ) -> Option<super::ast::ContinueStmt> {
        self.push_unimplemented_error_message("continue stmt", stmt.span);
        None
    }

    fn analyze_def(&mut self, stmt: &crate::ast::DefStmt) -> Option<super::ast::DefStmt> {
        self.push_unimplemented_error_message("def stmt", stmt.span);
        None
    }

    fn analyze_def_cal(&mut self, stmt: &crate::ast::DefCalStmt) -> Option<super::ast::DefCalStmt> {
        self.push_unimplemented_error_message("def cal stmt", stmt.span);
        None
    }

    fn analyze_delay(&mut self, stmt: &crate::ast::DelayStmt) -> Option<super::ast::DelayStmt> {
        self.push_unimplemented_error_message("delay stmt", stmt.span);
        None
    }

    fn analyze_end_stmt(&mut self, stmt: &crate::ast::EndStmt) -> Option<super::ast::EndStmt> {
        self.push_unimplemented_error_message("end stmt", stmt.span);
        None
    }

    fn analyze_expr_stmt(&mut self, stmt: &crate::ast::ExprStmt) -> Option<super::ast::ExprStmt> {
        self.push_unimplemented_error_message("expr stmt", stmt.span);
        None
    }

    fn analyze_extern(&mut self, stmt: &crate::ast::ExternDecl) -> Option<super::ast::ExternDecl> {
        self.push_unimplemented_error_message("extern stmt", stmt.span);
        None
    }

    fn analyze_for_stmt(&mut self, stmt: &crate::ast::ForStmt) -> Option<super::ast::ForStmt> {
        self.push_unimplemented_error_message("for stmt", stmt.span);
        None
    }

    fn analyze_if_stmt(&mut self, stmt: &crate::ast::IfStmt) -> Option<super::ast::IfStmt> {
        self.push_unimplemented_error_message("if stmt", stmt.span);
        None
    }

    fn analyze_gate_call(&mut self, stmt: &crate::ast::GateCall) -> Option<super::ast::GateCall> {
        self.push_unimplemented_error_message("gate call stmt", stmt.span);
        None
    }

    fn analyze_gphase(&mut self, stmt: &crate::ast::GPhase) -> Option<super::ast::GPhase> {
        self.push_unimplemented_error_message("gphase stmt", stmt.span);
        None
    }

    fn analyze_include(
        &mut self,
        stmt: &crate::ast::IncludeStmt,
    ) -> Option<super::ast::IncludeStmt> {
        self.push_unimplemented_error_message("include stmt", stmt.span);
        None
    }

    fn analyze_io_decl(
        &mut self,
        stmt: &crate::ast::IODeclaration,
    ) -> Option<super::ast::IODeclaration> {
        self.push_unimplemented_error_message("io decl stmt", stmt.span);
        None
    }

    fn analyze_measure(
        &mut self,
        stmt: &crate::ast::MeasureStmt,
    ) -> Option<super::ast::MeasureStmt> {
        self.push_unimplemented_error_message("measure stmt", stmt.span);
        None
    }

    fn analyze_pragma(&mut self, stmt: &crate::ast::Pragma) -> Option<super::ast::Pragma> {
        self.push_unimplemented_error_message("pragma stmt", stmt.span);
        None
    }

    fn analyze_gate_def(
        &mut self,
        stmt: &crate::ast::QuantumGateDefinition,
    ) -> Option<super::ast::QuantumGateDefinition> {
        self.push_unimplemented_error_message("gate def stmt", stmt.span);
        None
    }

    fn analyze_quantum_decl(
        &mut self,
        stmt: &crate::ast::QubitDeclaration,
    ) -> Option<super::ast::QubitDeclaration> {
        self.push_unimplemented_error_message("qubit decl stmt", stmt.span);
        None
    }

    fn analyze_reset(&mut self, stmt: &crate::ast::ResetStmt) -> Option<super::ast::ResetStmt> {
        self.push_unimplemented_error_message("reset stmt", stmt.span);
        None
    }

    fn analyze_return(&mut self, stmt: &crate::ast::ReturnStmt) -> Option<super::ast::ReturnStmt> {
        self.push_unimplemented_error_message("return stmt", stmt.span);
        None
    }

    fn analyze_switch(&mut self, stmt: &crate::ast::SwitchStmt) -> Option<super::ast::SwitchStmt> {
        self.push_unimplemented_error_message("switch stmt", stmt.span);
        None
    }

    fn analyze_while_loop(
        &mut self,
        stmt: &crate::ast::WhileLoop,
    ) -> Option<super::ast::WhileLoop> {
        self.push_unimplemented_error_message("while loop stmt", stmt.span);
        None
    }
}

fn get_identifier_name(identifier: &crate::ast::Identifier) -> std::rc::Rc<str> {
    match identifier {
        crate::ast::Identifier::Ident(ident) => ident.name.clone(),
        crate::ast::Identifier::IndexedIdent(ident) => ident.name.name.clone(),
    }
}
