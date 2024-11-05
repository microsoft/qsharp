// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::panic;
use std::path::PathBuf;

use crate::ast_builder::{
    self, build_arg_pat, build_array_reverse_expr, build_assignment_statement, build_attr,
    build_barrier_call, build_binary_expr, build_cast_call, build_cast_call_two_params,
    build_classical_decl, build_complex_binary_expr, build_complex_from_expr,
    build_convert_call_expr, build_default_result_array_expr, build_expr_array_expr,
    build_gate_call_param_expr, build_gate_decl_lambda, build_if_expr_then_block,
    build_if_expr_then_block_else_block, build_if_expr_then_block_else_expr,
    build_if_expr_then_expr_else_expr, build_implicit_return_stmt,
    build_indexed_assignment_statement, build_lit_bigint_expr, build_lit_bool_expr,
    build_lit_complex_expr, build_lit_double_expr, build_lit_int_expr,
    build_lit_result_array_expr_from_bitstring, build_lit_result_expr, build_managed_qubit_alloc,
    build_math_call_no_params, build_measure_call, build_operation_with_stmts,
    build_path_ident_expr, build_range_expr, build_reset_call, build_stmt_semi_from_expr,
    build_stmt_wrapped_block_expr, build_top_level_ns_with_item, build_tuple_expr,
    build_unary_op_expr, build_unmanaged_qubit_alloc, build_unmanaged_qubit_alloc_array,
    build_wrapped_block_expr, is_complex_binop_supported, managed_qubit_alloc_array,
    map_qsharp_type_to_ast_ty,
};

use crate::oqasm_helpers::{
    binop_requires_symmetric_int_conversion, can_cast_literal_with_value_knowledge,
    extract_dims_from_designator, get_designator_from_scalar_type, requires_symmetric_conversion,
    requires_types_already_match_conversion, safe_u128_to_f64, span_for_named_item,
    span_for_syntax_node, span_for_syntax_token,
};
use crate::oqasm_types::{promote_to_uint_ty, promote_types, types_equal_except_const};
use crate::runtime::{get_runtime_function_decls, RuntimeFunctions};
use crate::symbols::IOKind;
use crate::symbols::Symbol;
use crate::symbols::SymbolTable;
use crate::types::{get_indexed_type, get_qsharp_gate_name, GateModifier, QasmTypedExpr};
use crate::{
    CompilerConfig, OperationSignature, OutputSemantics, ProgramType, QubitSemantics,
    SemanticError, SemanticErrorKind,
};

use ast::NodeId;
use ast::Package;
use ast::TopLevelNode;
use num_bigint::BigInt;
use oq3_semantics::types::{ArrayDims, IsConst, Type};
use oq3_syntax::ast::{
    AnnotationStatement, ArithOp, BinaryOp, BitString, CastExpression, DelayStmt, Expr,
    GateOperand, HasArgList, HasName, Literal, LiteralKind, Modifier, ParamList, ParenExpr,
    PragmaStatement, Stmt, TimeUnit, TimingLiteral, UnaryOp,
};
use oq3_syntax::SyntaxNode;
use oq3_syntax::{AstNode, HasTextName};
use qsc::ast;
use qsc::Span;
use qsc::{error::WithSource, SourceMap};

use crate::{parse::QasmSource, QasmCompileUnit};

#[cfg(test)]
pub(crate) mod tests;

/// Compiles a QASM3 source to a Q# AST package.
#[must_use]
pub fn qasm_to_program(
    source: QasmSource,
    source_map: SourceMap,
    config: CompilerConfig,
) -> QasmCompileUnit {
    assert!(!source.has_errors(), "Source has errors");
    assert!(
        source.parse_result().have_parse(),
        "Source has not been successfully parsed"
    );
    let compiler = QasmCompiler {
        source,
        source_map,
        config,
        stmts: Vec::new(),
        runtime: RuntimeFunctions::default(),
        errors: Vec::new(),
        file_stack: Vec::new(),
        symbols: SymbolTable::new(),
        version: None,
        next_gate_as_item: false,
    };
    compiler.compile_program()
}

struct QasmCompiler {
    /// The root QASM source to compile.
    source: QasmSource,
    /// The source map of QASM sources for error reporting.
    source_map: SourceMap,
    /// The configuration for the compiler.
    /// This includes the qubit semantics to follow when compiling to Q# AST.
    /// The output semantics to follow when compiling to Q# AST.
    /// The program type to compile to.
    config: CompilerConfig,
    /// The compiled statments accumulated during compilation.
    stmts: Vec<ast::Stmt>,
    /// The runtime functions that need to be included at the end of
    /// compilation
    runtime: RuntimeFunctions,
    errors: Vec<WithSource<crate::Error>>,
    /// The file stack is used to track the current file for error reporting.
    /// When we include a file, we push the file path to the stack and pop it
    /// when we are done with the file.
    /// This allows us to report errors with the correct file path.
    file_stack: Vec<PathBuf>,
    symbols: SymbolTable,
    /// The QASM version parsed from the source file. This is a placeholder
    /// for future use. We may want to use this to generate/parse different code
    /// based on the QASM version.
    version: Option<String>,
    /// If the next gate should be compiled as a top level item instead of a lambda.
    /// In order to close over captured variables, we compile gates as a lambda
    /// operations; however, if the gate is annotated, we need to compile it as a
    /// top level item as attributes are not supported on lambdas. This isn't an
    /// issue as any gates that need attributes can't be used in a lambda anyway.
    /// This value is set once we encounter an annotation statement and is reset
    /// after the next gate is compiled, we run out of statements, or we encounter
    /// an error.
    next_gate_as_item: bool,
}

impl QasmCompiler {
    /// The main entry into compilation. This function will compile the
    /// source file and build the appropriate package based on the
    /// configuration.
    fn compile_program(mut self) -> QasmCompileUnit {
        self.compile_source(&self.source.clone());
        self.prepend_runtime_decls();
        let program_ty = self.config.program_ty.clone();
        let (package, signature) = match program_ty {
            ProgramType::File => self.build_file(),
            ProgramType::Operation => self.build_operation(),
            ProgramType::Fragments => (self.build_fragments(), None),
        };

        QasmCompileUnit::new(self.source_map, self.errors, Some(package), signature)
    }

    /// Prepends the runtime declarations to the beginning of the statements.
    /// Any runtime functions that are required by the compiled code are set
    /// in the `self.runtime` field during compilation.
    ///
    /// We could declare these as top level functions when compiling to
    /// `ProgramType::File`, but prepending them to the statements is the
    /// most flexible approach.
    fn prepend_runtime_decls(&mut self) {
        let mut runtime = get_runtime_function_decls(self.runtime);
        self.stmts.splice(0..0, runtime.drain(..));
    }

    /// Build a package with namespace and an operation
    /// containing the compiled statements.
    fn build_file(&mut self) -> (Package, Option<OperationSignature>) {
        let tree = self.source.tree();
        let whole_span = span_for_syntax_node(tree.syntax());
        let operation_name = self.config.operation_name();
        let (operation, mut signature) = self.create_entry_operation(operation_name, whole_span);
        let ns = self.config.namespace();
        signature.ns = Some(ns.to_string());
        let top = build_top_level_ns_with_item(whole_span, ns, operation);
        (
            Package {
                nodes: Box::new([top]),
                ..Default::default()
            },
            Some(signature),
        )
    }

    /// Creates an operation with the given name.
    fn build_operation(&mut self) -> (Package, Option<OperationSignature>) {
        let tree = self.source.tree();
        let whole_span = span_for_syntax_node(tree.syntax());
        let operation_name = self.config.operation_name();
        let (operation, signature) = self.create_entry_operation(operation_name, whole_span);
        (
            Package {
                nodes: Box::new([TopLevelNode::Stmt(Box::new(ast::Stmt {
                    kind: Box::new(ast::StmtKind::Item(Box::new(operation))),
                    span: whole_span,
                    id: NodeId::default(),
                }))]),
                ..Default::default()
            },
            Some(signature),
        )
    }

    /// Turns the compiled statements into package of top level nodes
    fn build_fragments(&mut self) -> Package {
        let nodes = self
            .stmts
            .drain(..)
            .map(Box::new)
            .map(TopLevelNode::Stmt)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Package {
            nodes,
            ..Default::default()
        }
    }

    /// Root recursive function for compiling the source.
    fn compile_source(&mut self, source: &QasmSource) {
        // we push the file path to the stack so we can track the current file
        // for reporting errors. This saves us from having to pass around
        // the current QasmSource value.
        self.file_stack.push(source.path());

        let mut annotations = Vec::new();

        // we keep an iterator of the includes so we can match them with the
        // source includes. The include statements only have the path, but
        // we have already loaded all of source files in the
        // `source.includes()`
        let mut includes = source.includes().iter();
        for stmt in source.tree().statements() {
            match stmt {
                Stmt::AnnotationStatement(annotation) => {
                    if let Some(annotation) = self.compile_annotation_stmt(&annotation) {
                        annotations.push(annotation);
                        self.next_gate_as_item = true;
                    }
                    continue;
                }
                Stmt::Include(include) => {
                    let Some(Some(path)) = include.file().map(|f| f.to_string()) else {
                        let span = span_for_syntax_node(include.syntax());
                        let kind = SemanticErrorKind::IncludeStatementMissingPath(span);
                        self.push_semantic_error(kind);
                        continue;
                    };

                    // if we are not in the root  we should not be able to include
                    // as this is a limitation of the QASM3 language
                    if !self.symbols.is_current_scope_global() {
                        let kind = SemanticErrorKind::IncludeNotInGlobalScope(
                            path.to_string(),
                            span_for_syntax_node(include.syntax()),
                        );
                        self.push_semantic_error(kind);
                        continue;
                    }

                    // special case for stdgates.inc
                    // it won't be in the includes list
                    if path.to_lowercase() == "stdgates.inc" {
                        self.define_stdgates(&include);
                        continue;
                    }

                    let include = includes.next().expect("missing include");
                    self.compile_source(include);
                }
                _ => {
                    if let Some(stmt) = self.compile_stmt(&stmt) {
                        if annotations.is_empty() {
                            self.stmts.push(stmt);
                            continue;
                        }

                        // we drain the annotations regardless of whether the statement
                        // can have them attached or not. This is to prevent the attrs
                        // from being attached to the wrong statement.

                        // If there is an error, we record the error, push the stmt
                        // without the annotations, and continue.
                        // The error is fatal for overall compilation, but this way we
                        // can continue to compile the rest of the statements
                        let mut stmt = stmt;
                        self.apply_annotations_to_stmt(&mut annotations, &mut stmt);
                        self.stmts.push(stmt);
                        self.next_gate_as_item = false;
                    }
                }
            }
        }

        if !annotations.is_empty() {
            let span = annotations.last().map(|x| x.span).unwrap_or_default();
            let kind = SemanticErrorKind::AnnotationWithoutStatement(span);
            self.push_semantic_error(kind);
            self.next_gate_as_item = false;
        }

        // Finally we pop the file path from the stack so that we
        // can return to the previous file for error handling.
        self.file_stack.pop();
    }

    fn compile_stmts(&mut self, stmt: &[oq3_syntax::ast::Stmt]) -> Vec<ast::Stmt> {
        let mut annotations = Vec::new();
        let mut stmts = Vec::new();
        for stmt in stmt {
            if let Stmt::AnnotationStatement(annotation) = stmt {
                // we compile the annotation and push it to the annotations.
                // If compiling fails, we record the error and continue.
                if let Some(annotation) = self.compile_annotation_stmt(annotation) {
                    annotations.push(annotation);
                    self.next_gate_as_item = true;
                }
                continue;
            }
            let stmt = self.compile_stmt(stmt);
            if stmt.is_none() {
                continue;
            }
            let stmt = stmt.expect("stmt is not None");
            if annotations.is_empty() {
                stmts.push(stmt);
                continue;
            }
            let mut stmt = stmt;

            // we drain the annotations regardless of whether the statement
            // can have them attached or not. This is to prevent the attrs
            // from being attached to the wrong statement.

            // If there is an error, we record the error, push the stmt
            // without the annotations, and continue.
            // The error is fatal for overall compilation, but this way we
            // can continue to compile the rest of the statements

            self.apply_annotations_to_stmt(&mut annotations, &mut stmt);
            self.next_gate_as_item = false;
        }
        if !annotations.is_empty() {
            let span = annotations.last().map(|x| x.span).unwrap_or_default();
            let kind = SemanticErrorKind::AnnotationWithoutStatement(span);
            self.push_semantic_error(kind);
            self.next_gate_as_item = false;
        }
        stmts
    }

    fn apply_annotations_to_stmt(
        &mut self,
        annotations: &mut Vec<ast::Attr>,
        stmt: &mut ast::Stmt,
    ) {
        let current_annotations: Vec<_> = annotations.drain(..).map(Box::new).collect();
        if let ast::StmtKind::Item(item) = stmt.kind.as_mut() {
            if let ast::ItemKind::Callable(_) = item.kind.as_ref() {
                let mut existing_attrs = item.attrs.to_vec();
                existing_attrs.extend(current_annotations);
                item.attrs = existing_attrs.into_boxed_slice();
            } else {
                let kind = SemanticErrorKind::InvalidAnnotationTarget(stmt.span);
                self.push_semantic_error(kind);
            }
        } else {
            let kind = SemanticErrorKind::InvalidAnnotationTarget(stmt.span);
            self.push_semantic_error(kind);
        }
    }

    /// Match against the different types of statements and compile them
    /// to the appropriate AST statement. There should be no logic here.
    fn compile_stmt(&mut self, stmt: &oq3_syntax::ast::Stmt) -> Option<ast::Stmt> {
        match stmt {
            Stmt::AliasDeclarationStatement(alias) => self.compile_alias_decl(alias),
            Stmt::AssignmentStmt(assignment) => self.compile_assignment_stmt(assignment),
            Stmt::Barrier(barrier) => self.compile_barrier_stmt(barrier),
            Stmt::BreakStmt(break_stmt) => self.compile_break_stmt(break_stmt),
            Stmt::ClassicalDeclarationStatement(decl) => self.compile_classical_decl(decl),
            Stmt::ContinueStmt(continue_stmt) => self.compile_continue_stmt(continue_stmt),
            Stmt::Def(def) => self.compile_def_decl(def),
            Stmt::EndStmt(end) => Some(compile_end_stmt(end)),
            Stmt::ExprStmt(expr) => self.compile_expr_stmt(expr),
            Stmt::ForStmt(for_stmt) => self.compile_for_stmt(for_stmt),
            Stmt::Gate(gate) => self.compile_gate_decl(gate),
            Stmt::IfStmt(if_stmt) => self.compile_if_stmt(if_stmt),
            Stmt::IODeclarationStatement(io_decl) => self.compile_io_decl_stmt(io_decl),
            Stmt::LetStmt(let_stmt) => self.compile_let_stmt(let_stmt),
            Stmt::Measure(measure) => self.compile_measure_stmt(measure),
            Stmt::QuantumDeclarationStatement(decl) => self.compile_quantum_decl(decl),
            Stmt::Reset(reset) => self.compile_reset_call(reset),
            Stmt::SwitchCaseStmt(switch_case) => self.compile_switch_stmt(switch_case),
            Stmt::VersionString(version) => self.compile_version_stmt(version),
            Stmt::WhileStmt(while_stmt) => self.compile_while_stmt(while_stmt),
            Stmt::Include(include) => self.compile_include_stmt(include),
            Stmt::Cal(..) | Stmt::DefCal(..) | Stmt::DefCalGrammar(..) => {
                self.compile_calibration_stmt(stmt)
            }
            Stmt::AnnotationStatement(..) => {
                panic!("Annotation statements should have been handled in compile_stmts")
            }
            Stmt::DelayStmt(delay) => self.compile_delay_stmt(delay),
            Stmt::PragmaStatement(pragma) => self.compile_pragma_stmt(pragma),
        }
    }

    fn compile_pragma_stmt(&mut self, stmt: &PragmaStatement) -> Option<ast::Stmt> {
        self.push_unsupported_error_message("Pragma statements", stmt.syntax());
        None
    }

    fn compile_delay_stmt(&mut self, stmt: &DelayStmt) -> Option<ast::Stmt> {
        self.push_unsupported_error_message("Delay statements", stmt.syntax());
        None
    }

    /// Annotations are defined by the compiler and their behavior are not part of the QASM3
    /// specification. They start with an `@` symbol and are used to define attributes for
    /// the next statement. Their values are not validated by the compiler and are passed
    /// directly to the AST as a string containing the whole line. It is up to the compiler
    /// to interpret the annotation.
    ///
    /// We use annotations to define intrinsic gates that are simulatable in Q# mapping the
    /// annotation to a Q# attribute. We also only allow the annotation to be attached to
    /// gates. Any other usage will result in a semantic error.
    fn compile_annotation_stmt(&mut self, stmt: &AnnotationStatement) -> Option<ast::Attr> {
        let text = stmt.annotation_text();
        let span = span_for_syntax_node(stmt.syntax());
        if let "@SimulatableIntrinsic" = text.as_str() {
            let (_at, name) = text.split_at(1);
            Some(build_attr(name.to_string(), span))
        } else {
            let span = span_for_syntax_node(stmt.syntax());
            let kind = SemanticErrorKind::UnknownAnnotation(text.to_string(), span);
            self.push_semantic_error(kind);
            None
        }
    }

    /// We don't support callibration statements in Q# so we push an error
    fn compile_calibration_stmt(&mut self, stmt: &Stmt) -> Option<ast::Stmt> {
        self.push_calibration_error(stmt.syntax());
        None
    }

    /// This function is always a indication of a error. Either the
    /// program is declaring include in a non-global scope or the
    /// include is not handled in `self.compile_source` properly.
    fn compile_include_stmt(&mut self, include: &oq3_syntax::ast::Include) -> Option<ast::Stmt> {
        // if we are not in the root we should not be able to include
        if !self.symbols.is_current_scope_global() {
            let name = include.to_string();
            let span = span_for_syntax_node(include.syntax());
            let kind = SemanticErrorKind::IncludeNotInGlobalScope(name, span);
            self.push_semantic_error(kind);
            return None;
        }
        // if we are at the root and we have an include, we should have
        // already handled it and we are in an invalid state
        panic!("Include should have been handled in compile_source")
    }

    /// Aliasing allows declared qubit bits and registers to be referred to by another name.
    /// Aliasing can slice a qubit array or register. Aliasing uses the `let` keyword.
    /// Example:
    /// ```qasm
    /// qubit[4] = q;
    /// let qubit = q[1:3]; // qubit now points to the middle two qubits of q
    /// // qubit[0] is now q[1], and qubit[1] is now q[2]
    /// ```
    /// We can eventually support this for qubits, but we don't support it for registers.
    /// Creating an array slice of results in Q# would be difficult as it would normally
    /// copy the conents of the array. We could potentially create a view of the array, but
    /// further investigation is needed.
    fn compile_alias_decl(
        &mut self,
        alias: &oq3_syntax::ast::AliasDeclarationStatement,
    ) -> Option<ast::Stmt> {
        let name = alias.name().expect("Alias declaration must have a name");
        let name_span = span_for_named_item(alias);
        let decl_span = span_for_syntax_node(alias.syntax());
        let rhs = alias.expr().and_then(|f| self.compile_expr(&f))?;

        if !matches!(rhs.ty, Type::Qubit | Type::QubitArray(_)) {
            let kind = SemanticErrorKind::CannotAliasType(format!("{:?}", rhs.ty), decl_span);
            self.push_semantic_error(kind);
            return None;
        }

        let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&rhs.ty, name.syntax())?;
        let symbol = Symbol {
            name: name.to_string(),
            span: span_for_syntax_node(name.syntax()),
            ty: rhs.ty.clone(),
            qsharp_ty,
            io_kind: IOKind::Default,
        };

        if self.symbols.insert_symbol(symbol).is_err() {
            self.push_redefined_symbol_error(name.to_string(), name_span);
            return None;
        }

        let name = name.to_string();
        let is_const = true;
        let ty_span = Span::default();
        let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&rhs.ty, alias.syntax())?;
        let stmt = build_classical_decl(
            name, is_const, ty_span, decl_span, name_span, &qsharp_ty, rhs.expr,
        );
        Some(stmt)
    }

    /// Assignment statements have two forms: simple and indexed.
    /// Simple assignments are of the form `ident = expr;` and indexed
    /// assignments are of the form `ident[index] = expr;`.
    /// This function will dispatch to the appropriate function based
    /// on the type of assignment.
    ///
    /// While they are similar, the indexed assignment is far more complex.
    fn compile_assignment_stmt(
        &mut self,
        assignment: &oq3_syntax::ast::AssignmentStmt,
    ) -> Option<ast::Stmt> {
        if let Some(name) = assignment.identifier() {
            self.compile_simple_assignment_stmt(assignment, &name)
        } else {
            self.compile_indexed_assignment_stmt(assignment)
        }
    }

    /// `ident = expr;`
    fn compile_simple_assignment_stmt(
        &mut self,
        assignment: &oq3_syntax::ast::AssignmentStmt,
        name: &oq3_syntax::ast::Identifier,
    ) -> Option<ast::Stmt> {
        let name_span = span_for_named_item(assignment);
        let assignment_span = span_for_syntax_node(assignment.syntax());
        let lhs_symbol = self
            .symbols
            .get_symbol_by_name(name.to_string().as_str())?
            .clone();
        if lhs_symbol.ty.is_const() {
            let kind = SemanticErrorKind::CannotUpdateConstVariable(name.to_string(), name_span);
            self.push_semantic_error(kind);
            // usually we'd return None here, but we'll continue to compile the rhs
            // looking for more errors. There is nothing in this type of error that
            // would prevent us from compiling the rhs.
        }
        // resolve the rhs expression to match the lhs type
        let rhs = self.compile_expr_to_ty_with_casts(
            assignment.rhs(),
            &lhs_symbol.ty,
            assignment.syntax(),
        )?;
        let stmt = build_assignment_statement(name_span, name.to_string(), rhs, assignment_span);
        Some(stmt)
    }

    /// `ident[index] = expr;`
    fn compile_indexed_assignment_stmt(
        &mut self,
        assignment: &oq3_syntax::ast::AssignmentStmt,
    ) -> Option<ast::Stmt> {
        let indexed_ident = assignment
            .indexed_identifier()
            .expect("assignment without name must have an indexed identifier");
        let name = indexed_ident
            .identifier()
            .expect("indexed identifier must have a name");
        let string_name = name.to_string();
        let name_span = span_for_named_item(&indexed_ident);
        let stmt_span = span_for_syntax_node(assignment.syntax());
        let rhs_span = span_for_syntax_node(assignment.rhs()?.syntax());

        // resolve the index expression
        // we only support single index expressions for now
        // but in the future we may support slice/range/array indexing
        let indices: Vec<_> = indexed_ident
            .index_operators()
            .filter_map(|op| self.compile_index_operator(&op))
            .flatten()
            .collect();

        if indices.len() != 1 {
            // This is a temporary limitation. We can only handle
            // single index expressions for now.
            let kind = SemanticErrorKind::IndexMustBeSingleExpr(span_for_syntax_node(
                indexed_ident.syntax(),
            ));
            self.push_semantic_error(kind);
            return None;
        }
        let index = indices[0].clone();

        let lhs_symbol = self
            .symbols
            .get_symbol_by_name(name.to_string().as_str())?
            .clone();
        if index.ty.num_dims() > lhs_symbol.ty.num_dims() {
            let kind = SemanticErrorKind::TypeRankError(rhs_span);
            self.push_semantic_error(kind);
        }
        let index_expr = index.expr.clone();

        let Some(indexed_ty) = get_indexed_type(&lhs_symbol.ty) else {
            let kind = SemanticErrorKind::CannotIndexType(format!("{:?}", lhs_symbol.ty), rhs_span);
            self.push_semantic_error(kind);
            return None;
        };
        let rhs =
            self.compile_expr_to_ty_with_casts(assignment.rhs(), &indexed_ty, assignment.syntax())?;
        let stmt =
            build_indexed_assignment_statement(name_span, string_name, index_expr, rhs, stmt_span);
        Some(stmt)
    }

    /// Barrier isn't supported in Q# so we insert a runtime function that is
    /// a no-op for simulation but is `@SimulatableIntrinsic()` so that it can
    /// be emited in code gen as `__quantum__qis__barrier__body()`. This
    /// matches the existing `qiskit-qir` behavior. Qiskit barriers are
    /// variadic, but QIR doesn't support variadic gates.
    /// We should look at how to handle this better in the future.
    fn compile_barrier_stmt(&mut self, barrier: &oq3_syntax::ast::Barrier) -> Option<ast::Stmt> {
        let qubit_args: Vec<_> = if let Some(qubits) = barrier.qubit_list() {
            qubits
                .gate_operands()
                .map(|op| self.compile_gate_operand(&op).map(|x| x.expr))
                .collect()
        } else {
            vec![]
        };

        if qubit_args.iter().any(Option::is_none) {
            // if any of the qubit arguments failed to compile, we can't proceed
            // This can happen if the qubit is not defined or if the qubit was
            // a hardware qubit
            return None;
        }
        let call_span = span_for_syntax_node(barrier.syntax());
        // we don't support barrier, but we can insert a runtime function
        // which will generate a barrier call in QIR
        self.runtime.insert(RuntimeFunctions::Barrier);
        Some(build_barrier_call(call_span))
    }

    /// Need to add break support in Q# to support break statements
    fn compile_break_stmt(&mut self, break_stmt: &oq3_syntax::ast::BreakStmt) -> Option<ast::Stmt> {
        self.push_unsupported_error_message("break statements", break_stmt.syntax());
        None
    }

    /// Classical decls are used to declare classical variables. They have two
    /// main forms:
    /// - `type ident;`
    /// - `type ident = expr;`
    ///
    /// Q# requires classical variables to be initialized, so we will use the
    /// default value for the type to initialize the variable. In theory this
    /// isn't a problem as any classical variable that is used should be
    /// initialized before use and would be a bug anyway. This leads to awkward
    /// code in Q# where we have to initialize classical variables that are
    /// always overwritten before use.
    fn compile_classical_decl(
        &mut self,
        decl: &oq3_syntax::ast::ClassicalDeclarationStatement,
    ) -> Option<ast::Stmt> {
        let name = decl.name().expect("classical declaration must have a name");
        let name_span = span_for_syntax_node(name.syntax());
        let scalar_ty = decl
            .scalar_type()
            .expect("Classical declaration must have a scalar type");
        let is_const = decl.const_token().is_some();
        // if we can't convert the scalar type, we can't proceed, an error has been pushed
        let ty = self.get_semantic_type_from_scalar_type(&scalar_ty, is_const)?;
        let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty, name.syntax())?;

        let symbol = Symbol {
            name: name.to_string(),
            span: name_span,
            ty: ty.clone(),
            qsharp_ty: qsharp_ty.clone(),
            io_kind: IOKind::Default,
        };

        if self.symbols.insert_symbol(symbol).is_err() {
            self.push_redefined_symbol_error(name.to_string(), name_span);
            return None;
        }

        // if there is an expression, compile it to match the decl type
        let rhs = self.compile_expr_to_ty_with_casts(decl.expr(), &ty, decl.syntax())?;

        // create the let binding and assign the rhs to the lhs
        let ty_span = span_for_syntax_node(scalar_ty.syntax());
        let stmt_span = span_for_syntax_node(decl.syntax());
        let stmt = build_classical_decl(
            name.to_string(),
            is_const,
            ty_span,
            stmt_span,
            name_span,
            &qsharp_ty,
            rhs,
        );

        Some(stmt)
    }

    /// The expr type is fixed so we try to resolve the expression to match the type
    /// via implicit casts or literal conversion if possible.
    fn compile_expr_to_ty_with_casts(
        &mut self,
        expr: Option<Expr>,
        ty: &Type,
        node: &SyntaxNode,
    ) -> Option<ast::Expr> {
        let Some(expr) = expr else {
            // In OpenQASM, classical variables may be uninitialized, but in Q#,
            // they must be initialized. We will use the default value for the type
            // to initialize the variable.
            return self.get_default_value(ty, node);
        };

        // since we have an expr, we can refine the node for errors
        let span = span_for_syntax_node(expr.syntax());

        let rhs = self.compile_expr(&expr)?;
        let rhs_ty = rhs.ty.clone();

        // if we have an exact type match, we can use the rhs as is
        if types_equal_except_const(ty, &rhs_ty) {
            return Some(rhs.expr);
        }

        if let Expr::Literal(literal) = &expr {
            // if the rhs is a literal, we can try to cast it to the lhs type
            // we can do better than just types given we have a literal value
            if can_cast_literal(ty, &rhs_ty) || can_cast_literal_with_value_knowledge(ty, literal) {
                return Some(self.cast_literal_expr_to_type(ty, &rhs, literal)?.expr);
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
        let is_negated_lit = if let Expr::PrefixExpr(prefix_expr) = &expr {
            if let Some(UnaryOp::Neg) = prefix_expr.op_kind() {
                matches!(&prefix_expr.expr(), Some(Expr::Literal(..)))
            } else {
                false
            }
        } else {
            false
        };
        if matches!(ty, Type::UInt(..)) && is_negated_lit {
            let kind = SemanticErrorKind::CannotAssignToType(
                "Negative Int".to_string(),
                format!("{ty:?}"),
                span,
            );
            self.push_semantic_error(kind);
            return None;
        }
        if let Expr::PrefixExpr(prefix_expr) = &expr {
            if let Some(UnaryOp::Neg) = prefix_expr.op_kind() {
                if let Some(Expr::Literal(literal)) = &prefix_expr.expr() {
                    // if the rhs is a literal, we can try to cast it to the lhs type
                    // we can do better than just types given we have a literal value

                    if can_cast_literal(ty, &rhs_ty)
                        || can_cast_literal_with_value_knowledge(ty, literal)
                    {
                        // if the literal is negated, we need to compile it as a negated literal
                        // This will only work for int/float as we can't express any other
                        // kind of negated literal
                        return Some(self.compile_negated_literal_as_ty(literal, Some(ty))?.expr);
                    }
                    // if we can't cast the literal, we can't proceed
                    // create a semantic error and return
                    let kind = SemanticErrorKind::CannotAssignToType(
                        format!("{:?}", rhs.ty),
                        format!("{ty:?}"),
                        span_for_syntax_node(node),
                    );
                    self.push_semantic_error(kind);
                    return None;
                }
            }
        }
        // the lhs has a type, but the rhs may be of a different type with
        // implicit and explicit conversions. We need to cast the rhs to the
        // lhs type, but if that cast fails, we will have already pushed an error
        // and we can't proceed

        Some(self.cast_expr_to_type(ty, &rhs, node)?.expr)
    }

    /// Need to add continue support in Q# to support continue statements
    fn compile_continue_stmt(
        &mut self,
        continue_stmt: &oq3_syntax::ast::ContinueStmt,
    ) -> Option<ast::Stmt> {
        self.push_unsupported_error_message("continue statements", continue_stmt.syntax());
        None
    }

    /// <https://openqasm.com/language/subroutines.html>
    /// Qiskit can't handle def statements, so we push an error
    /// and return None. We also push a semantic error if the def
    /// statement is not in the global scope.
    fn compile_def_decl(&mut self, def: &oq3_syntax::ast::Def) -> Option<ast::Stmt> {
        let def_span = span_for_syntax_node(def.syntax());
        if !self.symbols.is_current_scope_global() {
            let kind = SemanticErrorKind::QuantumDeclarationInNonGlobalScope(def_span);
            self.push_semantic_error(kind);
            return None;
        }
        self.push_unsupported_error_message("def declarations", def.syntax());
        None
    }

    /// Some statements don't fall into the normal categories, so they
    /// are handled here. This is a catch-all for non-declaration
    /// assignments and calculations.
    /// Example:
    /// ```qasm
    /// input int a;
    /// input int b;
    /// a * b; // this is an expr statement
    /// ```
    fn compile_expr_stmt(&mut self, expr: &oq3_syntax::ast::ExprStmt) -> Option<ast::Stmt> {
        let expr = expr.expr()?;
        let texpr = self.compile_expr(&expr)?;
        Some(build_stmt_semi_from_expr(texpr.expr))
    }

    /// This is the heart of compilation. All statements eventually call this
    /// function and it is where we start to build the AST. From here we start
    /// to return `QasmTypedExpr` so that we can track the QASM type of for the
    /// Q# expresssion that we are building. This is needed for type checking
    /// and casting. We must make sure all types match while building the AST
    /// as Q# doesn't have implicit casting and would otherwise fail to compile.
    fn compile_expr(&mut self, expr: &oq3_syntax::ast::Expr) -> Option<QasmTypedExpr> {
        match expr {
            Expr::ArrayExpr(array_expr) => self.compile_array_expr(array_expr, expr),
            Expr::ArrayLiteral(array_literal) => {
                self.compile_array_literal_expr(array_literal, expr)
            }
            Expr::BinExpr(bin_expr) => self.compile_bin_expr(bin_expr, expr),
            Expr::BlockExpr(_) => {
                // block expressions are handled by their containing statements
                panic!("Block expressions should not be compiled directly")
            }
            Expr::BoxExpr(box_expr) => self.compile_box_expr(box_expr),
            Expr::CallExpr(call_expr) => self.compile_call_expr(call_expr),
            Expr::CastExpression(cast_expr) => self.compile_cast_expr(cast_expr),
            Expr::GateCallExpr(gate_call_expr) => self.compile_gate_call_expr(gate_call_expr, expr),
            Expr::GPhaseCallExpr(_) => {
                panic!("GPhase expressions should not be compiled directly")
            }
            Expr::HardwareQubit(hardware_qubit) => {
                self.compile_hardware_qubit_expr(hardware_qubit, expr)
            }
            Expr::Identifier(identifier) => self.compile_identifier_expr(identifier, expr),
            Expr::IndexExpr(index_expr) => self.compile_index_expr(index_expr),
            Expr::IndexedIdentifier(indexed_identifier) => {
                self.compile_indexed_identifier_expr(indexed_identifier)
            }
            Expr::Literal(lit) => self.compile_literal_expr(lit, expr),
            Expr::TimingLiteral(lit) => self.compile_timing_literal_expr(lit, expr),
            Expr::MeasureExpression(measure_expr) => self.compile_measure_expr(measure_expr, expr),
            Expr::ModifiedGateCallExpr(modified_gate_call_expr) => {
                self.compile_modified_gate_call_expr(modified_gate_call_expr)
            }
            Expr::ParenExpr(paren_expr) => self.compile_paren_expr(paren_expr),
            Expr::PrefixExpr(prefix_expr) => self.compile_prefix_expr(prefix_expr),
            Expr::RangeExpr(range_expr) => self.compile_range_expr(range_expr, expr.syntax()),
            Expr::ReturnExpr(return_expr) => self.compile_return_expr(return_expr),
        }
    }

    /// Qubit and bit registers are handled in the their own statements.
    /// Arrays for classical variables would be handled here. Qiskit
    /// can't handle array expressions yet, so we push an error and return None.
    fn compile_array_expr(
        &mut self,
        _array_expr: &oq3_syntax::ast::ArrayExpr,
        expr: &Expr,
    ) -> Option<QasmTypedExpr> {
        self.push_unimplemented_error_message("array expressions", expr.syntax());
        None
    }

    /// Qubit and bit registers are handled in the their own statements.
    /// Arrays for classical variables would be handled here. Qiskit
    /// can't handle array expressions yet, so we push an error and return None.
    fn compile_array_literal_expr(
        &mut self,
        _array_literal: &oq3_syntax::ast::ArrayLiteral,
        expr: &Expr,
    ) -> Option<QasmTypedExpr> {
        self.push_unimplemented_error_message("array literal expressions", expr.syntax());
        None
    }

    /// Create a binary expression from the given binary expression node
    /// The binary expression is created by recursively compiling the left and right
    /// expressions and then creating a binary expression from the compiled expressions
    ///
    /// This is more complex than it seems because we need to handle type promotion
    /// and casting. The `OpenQASM3` language has a specific set of rules for casting
    /// between types. The rules can be found at:
    ///  <https://openqasm.com/language/types.html#casting-specifics>
    ///
    /// This harder than decl statements as we need to deal with promotion and casting
    /// for both the lhs and rhs expressions instead of having a fixed LHS type.
    ///
    /// complex > float > int/uint
    /// within type widths are promoted to the larger type
    #[allow(clippy::too_many_lines)]
    fn compile_bin_expr(
        &mut self,
        bin_expr: &oq3_syntax::ast::BinExpr,
        expr: &Expr,
    ) -> Option<QasmTypedExpr> {
        // We don't need to worry about semantic errors as binary expression
        // must have a lhs and rhs expression and an operator. This is
        // verified in the binary expression tests.
        let op = bin_expr.op_kind()?;
        let lhs_expr = bin_expr.lhs()?;
        let rhs_expr = bin_expr.rhs()?;

        let lhs = self.compile_expr(&lhs_expr)?;
        let rhs = self.compile_expr(&rhs_expr)?;

        if lhs.ty.is_quantum() {
            let kind = SemanticErrorKind::QuantumTypesInBinaryExpression(lhs.expr.span);
            self.push_semantic_error(kind);
        }
        if rhs.ty.is_quantum() {
            let kind = SemanticErrorKind::QuantumTypesInBinaryExpression(rhs.expr.span);
            self.push_semantic_error(kind);
        }

        let qsop = self.map_bin_op(op, expr.syntax())?;
        let is_assignment = matches!(op, oq3_syntax::ast::BinaryOp::Assignment { op: _ });

        let left_type = lhs.ty.clone();
        let right_type = rhs.ty.clone();

        if binop_requires_bitwise_conversion(op, &left_type) {
            // if the operator requires bitwise conversion, we need to determine
            // what size of UInt to promote to. If we can't promote to a UInt, we
            // push an error and return None.
            let (ty, lhs_uint_promotion, rhs_uint_promotion) =
                promote_to_uint_ty(&left_type, &right_type);
            let Some(ty) = ty else {
                let target_ty = Type::UInt(None, IsConst::False);
                if lhs_uint_promotion.is_none() {
                    let target_str: String = format!("{target_ty:?}");
                    let kind = SemanticErrorKind::CannotCast(
                        format!("{left_type:?}"),
                        target_str,
                        lhs.expr.span,
                    );
                    self.push_semantic_error(kind);
                }
                if rhs_uint_promotion.is_none() {
                    let target_str: String = format!("{target_ty:?}");
                    let kind = SemanticErrorKind::CannotCast(
                        format!("{right_type:?}"),
                        target_str,
                        rhs.expr.span,
                    );
                    self.push_semantic_error(kind);
                }
                return None;
            };
            // Now that we know the effective Uint type, we can cast the lhs and rhs
            // so that operations can be performed on them.
            let new_lhs = self.cast_expr_to_type(&ty, &lhs, lhs_expr.syntax())?;
            // only cast the rhs if the operator requires symmetric conversion
            let new_rhs = if binop_requires_bitwise_symmetric_conversion(op) {
                self.cast_expr_to_type(&ty, &rhs, rhs_expr.syntax())?
            } else {
                rhs
            };
            let expr = build_binary_expr(
                is_assignment,
                qsop,
                new_lhs.expr,
                new_rhs.expr,
                span_for_syntax_node(expr.syntax()),
            );
            let texpr = QasmTypedExpr { ty, expr };
            let final_expr = self.cast_expr_to_type(&left_type, &texpr, bin_expr.syntax())?;
            return Some(final_expr);
        }

        let span = span_for_syntax_node(expr.syntax());

        if requires_types_already_match_conversion(op) {
            if !types_equal_except_const(&left_type, &right_type) {
                let kind = SemanticErrorKind::CannotApplyOperatorToTypes(
                    format!("{op:?}"),
                    format!("{left_type:?}"),
                    format!("{right_type:?}"),
                    span,
                );
                self.push_semantic_error(kind);
                return None;
            }
            let expr = build_binary_expr(is_assignment, qsop, lhs.expr, rhs.expr, span);
            return Some(QasmTypedExpr {
                ty: left_type,
                expr,
            });
        }

        // for int, uint, float, and complex, the lesser of the two types is cast to
        // the greater of the two. Within each level of complex and float, types with
        // greater width are greater than types with lower width.
        // complex > float > int/uint
        // Q# has built-in functions: IntAsDouble, IntAsBigInt to handle two cases.
        // If the width of a float is greater than 64, we can't represent it as a double.

        let (lhs, rhs, ty) = if binop_requires_bool_conversion_for_type(op) {
            let ty = Type::Bool(IsConst::False);
            let new_lhs = self.cast_expr_to_type(&ty, &lhs, lhs_expr.syntax())?;
            let new_rhs = self.cast_expr_to_type(&ty, &rhs, rhs_expr.syntax())?;
            (new_lhs.expr, new_rhs.expr, ty)
        } else if binop_requires_int_conversion_for_type(op, &left_type, &rhs.ty) {
            let ty = Type::Int(None, IsConst::False);
            let new_lhs = self.cast_expr_to_type(&ty, &lhs, lhs_expr.syntax())?;
            let new_rhs = self.cast_expr_to_type(&ty, &rhs, rhs_expr.syntax())?;
            (new_lhs.expr, new_rhs.expr, ty)
        } else if requires_symmetric_conversion(op) {
            let promoted_type = try_promote_with_casting(&left_type, &right_type);
            let new_left = if promoted_type == left_type {
                lhs
            } else {
                let node = lhs_expr.syntax();
                match &lhs_expr {
                    Expr::Literal(literal) => {
                        if can_cast_literal(&promoted_type, &left_type)
                            || can_cast_literal_with_value_knowledge(&promoted_type, literal)
                        {
                            self.cast_literal_expr_to_type(&promoted_type, &lhs, literal)?
                        } else {
                            self.cast_expr_to_type(&promoted_type, &lhs, node)?
                        }
                    }
                    _ => self.cast_expr_to_type(&promoted_type, &lhs, node)?,
                }
            };
            let new_right = if promoted_type == right_type {
                rhs
            } else {
                let node = rhs_expr.syntax();
                match &rhs_expr {
                    Expr::Literal(literal) => {
                        if can_cast_literal(&promoted_type, &right_type)
                            || can_cast_literal_with_value_knowledge(&promoted_type, literal)
                        {
                            self.cast_literal_expr_to_type(&promoted_type, &rhs, literal)?
                        } else {
                            self.cast_expr_to_type(&promoted_type, &rhs, node)?
                        }
                    }
                    _ => self.cast_expr_to_type(&promoted_type, &rhs, node)?,
                }
            };
            (new_left.expr, new_right.expr, promoted_type)
        } else {
            // we don't have symmetric promotion, so we need to promote the rhs only
            if is_assignment {
                let oq3_syntax::ast::BinaryOp::Assignment { op: arith_op } = op else {
                    unreachable!()
                };
                let (lhs, rhs) =
                    if arith_op.is_some() && binop_requires_symmetric_int_conversion(op) {
                        let ty = Type::Int(None, IsConst::False);
                        let lhs = self.cast_expr_to_type(&ty, &lhs, lhs_expr.syntax())?.expr;
                        let rhs = self.cast_expr_to_type(&ty, &rhs, rhs_expr.syntax())?.expr;
                        (lhs, rhs)
                    } else {
                        let rhs = self.compile_expr_to_ty_with_casts(
                            Some(rhs_expr.clone()),
                            &left_type,
                            rhs_expr.syntax(),
                        )?;
                        (lhs.expr, rhs)
                    };

                (lhs, rhs, left_type)
            } else if binop_requires_symmetric_int_conversion(op) {
                let ty = Type::Int(None, IsConst::False);
                let new_rhs = self.cast_expr_to_type(&ty, &rhs, rhs_expr.syntax())?;
                (lhs.expr, new_rhs.expr, left_type)
            } else {
                (lhs.expr, rhs.expr, left_type)
            }
        };

        // now that we have the lhs and rhs expressions, we can create the binary expression
        // but we need to check if the chosen operator is supported by the types after
        // promotion and conversion.

        let expr = if matches!(ty, Type::Complex(..)) {
            if is_assignment {
                let kind = SemanticErrorKind::ComplexBinaryAssignment(span);
                self.push_semantic_error(kind);
                None
            } else if is_complex_binop_supported(qsop) {
                Some(build_complex_binary_expr(
                    is_assignment,
                    qsop,
                    lhs,
                    rhs,
                    span,
                ))
            } else {
                let kind = SemanticErrorKind::OperatorNotSupportedForTypes(
                    format!("{qsop:?}"),
                    format!("{ty:?}"),
                    format!("{ty:?}"),
                    span,
                );
                self.push_semantic_error(kind);
                None
            }
        } else {
            Some(build_binary_expr(is_assignment, qsop, lhs, rhs, span))
        };
        let expr = expr?;
        let ty = match &op {
            BinaryOp::CmpOp(..) | BinaryOp::LogicOp(..) => Type::Bool(IsConst::False),
            _ => ty,
        };
        Some(QasmTypedExpr { ty, expr })
    }

    fn map_bin_op(
        &mut self,
        op: oq3_syntax::ast::BinaryOp,
        node: &SyntaxNode,
    ) -> Option<ast::BinOp> {
        match op {
            oq3_syntax::ast::BinaryOp::LogicOp(logic_op) => Some(match logic_op {
                oq3_syntax::ast::LogicOp::And => ast::BinOp::AndL,
                oq3_syntax::ast::LogicOp::Or => ast::BinOp::OrL,
            }),
            oq3_syntax::ast::BinaryOp::ArithOp(arith) => Some(match arith {
                oq3_syntax::ast::ArithOp::Add => ast::BinOp::Add,
                oq3_syntax::ast::ArithOp::Mul => ast::BinOp::Mul,
                oq3_syntax::ast::ArithOp::Sub => ast::BinOp::Sub,
                oq3_syntax::ast::ArithOp::Div => ast::BinOp::Div,
                oq3_syntax::ast::ArithOp::Rem => ast::BinOp::Mod,
                oq3_syntax::ast::ArithOp::Shl => ast::BinOp::Shl,
                oq3_syntax::ast::ArithOp::Shr => ast::BinOp::Shr,
                oq3_syntax::ast::ArithOp::BitXor => ast::BinOp::XorB,
                oq3_syntax::ast::ArithOp::BitOr => ast::BinOp::OrB,
                oq3_syntax::ast::ArithOp::BitAnd => ast::BinOp::AndB,
            }),
            oq3_syntax::ast::BinaryOp::CmpOp(cmp_op) => Some(match cmp_op {
                oq3_syntax::ast::CmpOp::Eq { negated } => {
                    if negated {
                        ast::BinOp::Neq
                    } else {
                        ast::BinOp::Eq
                    }
                }
                oq3_syntax::ast::CmpOp::Ord { ordering, strict } => match ordering {
                    oq3_syntax::ast::Ordering::Less => {
                        if strict {
                            ast::BinOp::Lt
                        } else {
                            ast::BinOp::Lte
                        }
                    }
                    oq3_syntax::ast::Ordering::Greater => {
                        if strict {
                            ast::BinOp::Gt
                        } else {
                            ast::BinOp::Gte
                        }
                    }
                },
            }),
            oq3_syntax::ast::BinaryOp::ConcatenationOp => {
                // This is only used for types which we don't currently support.
                self.push_unimplemented_error_message("Concatenation operators", node);
                None
            }
            oq3_syntax::ast::BinaryOp::Assignment { op } => op.map(|op| match op {
                oq3_syntax::ast::ArithOp::Add => ast::BinOp::Add,
                oq3_syntax::ast::ArithOp::Mul => ast::BinOp::Mul,
                oq3_syntax::ast::ArithOp::Sub => ast::BinOp::Sub,
                oq3_syntax::ast::ArithOp::Div => ast::BinOp::Div,
                oq3_syntax::ast::ArithOp::Rem => ast::BinOp::Mod,
                oq3_syntax::ast::ArithOp::Shl => ast::BinOp::Shl,
                oq3_syntax::ast::ArithOp::Shr => ast::BinOp::Shr,
                oq3_syntax::ast::ArithOp::BitXor => ast::BinOp::XorB,
                oq3_syntax::ast::ArithOp::BitOr => ast::BinOp::OrB,
                oq3_syntax::ast::ArithOp::BitAnd => ast::BinOp::AndB,
            }),
        }
    }

    fn compile_block_expr(&mut self, block_expr: &oq3_syntax::ast::BlockExpr) -> ast::Block {
        let stmts = self.compile_stmts(&block_expr.statements().collect::<Vec<_>>());
        let stmts = stmts
            .into_iter()
            .map(Box::new)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let span = span_for_syntax_node(block_expr.syntax());
        ast::Block {
            id: NodeId::default(),
            span,
            stmts,
        }
    }

    /// <https://openqasm.com/language/delays.html#boxed-expressions>
    /// Qiskit can't emit the box/delay, so we push an error and return None.
    /// The QASM grammar specifies this is a statement, but the parser has it as an expression.
    ///
    /// We don't really have anything in Q# the box statement is for scoping the
    /// timing of a particular part of the circuit. We could generates call around
    /// a new block, but that would be a bit of a hack.
    fn compile_box_expr(&mut self, box_expr: &oq3_syntax::ast::BoxExpr) -> Option<QasmTypedExpr> {
        self.push_unimplemented_error_message("box expressions", box_expr.syntax());
        None
    }

    /// Qiskit can't handle call expressions yet, so we push an error and return None.
    fn compile_call_expr(
        &mut self,
        call_expr: &oq3_syntax::ast::CallExpr,
    ) -> Option<QasmTypedExpr> {
        self.push_unimplemented_error_message("call expressions", call_expr.syntax());
        None
    }

    /// explicit casts have no defined behavior AFAICT from the spec. I'm
    /// guessing that they are a best effort for the compiler implementor.
    fn compile_cast_expr(&mut self, cast_expr: &CastExpression) -> Option<QasmTypedExpr> {
        let scalar_ty = cast_expr
            .scalar_type()
            .expect("cast expression must have a scalar type");
        let is_const = false;
        let ty = self.get_semantic_type_from_scalar_type(&scalar_ty, is_const)?;
        let expr = self.compile_expr_to_ty_with_casts(cast_expr.expr(), &ty, cast_expr.syntax())?;
        Some(QasmTypedExpr { ty, expr })
    }

    /// Gate call expression. We delegate compilation to
    /// `compile_gate_call_expr_impl` with empty modifiers so that we can reuse
    /// gate call compilation logic.
    fn compile_gate_call_expr(
        &mut self,
        gate_call_expr: &oq3_syntax::ast::GateCallExpr,
        expr: &Expr,
    ) -> Option<QasmTypedExpr> {
        let expr_span = span_for_syntax_node(expr.syntax());
        self.compile_gate_call_expr_impl(gate_call_expr, expr_span, &[])
    }

    /// Compile gate call expression with modifiers.
    /// We have to resolve the gate name and modifiers as some of the stdgates
    /// have implicit modifiers and different gate names that we have to map
    /// into Q# names with the appropriate modifiers/fuctors.
    /// - The `inv` modifier is the `Adjoint` fuctor.
    /// - The `pow` modifier is the `__Pow__` function which we define as a
    ///     runtime function at the end of code generation if it is used.
    /// - the `ctrl` modifier is the `Controlled` functor.
    ///     the `negctrl` modifier is a special case equivalent to
    ///     `ApplyControlledOnInt(0, _, _, _)`.
    ///
    /// Apply the modifiers are applied in reverse order to the gate call.
    /// A simplified binding of the modifiers to the gate call with all
    /// operations being on a single qubit gate would look like:
    /// `a @ b @ c g(r) q0, q1, q2` `=>` `(a @ (b @ (c g(r) q0), q1), q2)`
    ///
    /// This get more complex when we have multiple qubit gates and controls.
    #[allow(clippy::too_many_lines)]
    fn compile_gate_call_expr_impl(
        &mut self,
        gate_call_expr: &oq3_syntax::ast::GateCallExpr,
        expr_span: Span,
        modifiers: &[crate::types::GateModifier],
    ) -> Option<QasmTypedExpr> {
        let name = gate_call_expr
            .identifier()
            .expect("gate call must have a name");
        let name_span = span_for_syntax_node(name.syntax());
        let name_text = name.to_string();
        let call_span = span_for_syntax_node(gate_call_expr.syntax());
        // if we fail to map the name, we don't have a valid Q# gate
        // but the user may have defined their own. We check the symbol
        // table looking for such a definition.
        let gate_name = get_qsharp_gate_name(&name_text).unwrap_or(&name_text);
        let (gate_name, additional_modifiers) = get_implicit_modifiers(gate_name, name_span);
        let Some(sym) = self.symbols.get_symbol_by_name(&gate_name) else {
            self.push_missing_symbol_error(name_text, name.syntax());
            return None;
        };
        let Type::Gate(cargs_len, qargs_len) = sym.ty else {
            let kind = SemanticErrorKind::CannotCallNonGate(call_span);
            self.push_semantic_error(kind);
            return None;
        };

        let classical_args = self.compile_gate_call_classical_args(gate_call_expr, cargs_len)?;
        let mut qubit_args = self.compile_gate_call_quantum_args(gate_call_expr)?;

        // at this point we all of the information we need, but we have to deal with modifiers
        // We have the modifiers which we have been given, plus the implicit modifiers
        // from the gate definition. We need to merge these two sets of modifiers
        // See: ch, crx, cry, crz, sdg, and tdg
        let modifiers = modifiers
            .iter()
            .chain(additional_modifiers.iter())
            .rev()
            .collect::<Vec<_>>();
        let num_ctrls = calculate_num_ctrls(&modifiers);
        self.verify_qubit_args_match_gate_and_ctrls(
            &qubit_args,
            qargs_len,
            num_ctrls,
            gate_call_expr,
        )?;
        // take the nuber of qubit args that the gates expects from the source qubits
        let gate_qubits = qubit_args.split_off(qubit_args.len() - qargs_len);
        // and then merge the classical args with the qubit args
        // this will give us the args for the call prior to wrapping in tuples
        // for controls
        let args: Vec<_> = classical_args.into_iter().chain(gate_qubits).collect();
        let mut args = build_gate_call_param_expr(args, qubit_args.len());
        let mut callee = build_path_ident_expr(&gate_name, name_span, expr_span);

        for modifier in modifiers {
            match modifier {
                GateModifier::Inv(mod_span) => {
                    callee = build_unary_op_expr(
                        ast::UnOp::Functor(ast::Functor::Adj),
                        callee,
                        *mod_span,
                    );
                }
                GateModifier::Pow(exponent, mod_span) => {
                    // The exponent is only an option when initially parsing the gate
                    // call. The stmt would not have been created. If we don't have an
                    // an eponent at this point it is a bug
                    let exponent = exponent.expect("Exponent must be present");
                    let exponent_expr = build_lit_int_expr(exponent, *mod_span);
                    self.runtime |= RuntimeFunctions::Pow;
                    args = build_tuple_expr(vec![exponent_expr, callee, args]);
                    callee = build_path_ident_expr("__Pow__", *mod_span, expr_span);
                }
                GateModifier::Ctrl(controls, mod_span) => {
                    // remove the last n qubits from the qubit list
                    let num_ctrls = controls.unwrap_or(1);
                    if qubit_args.len() < num_ctrls {
                        let kind =
                            SemanticErrorKind::InvalidNumberOfQubitArgs(qargs_len, 0, call_span);
                        self.push_semantic_error(kind);
                        return None;
                    }
                    let ctrl = qubit_args.split_off(qubit_args.len() - num_ctrls);
                    let ctrls = build_expr_array_expr(ctrl, *mod_span);
                    args = build_tuple_expr(vec![ctrls, args]);
                    callee = build_unary_op_expr(
                        ast::UnOp::Functor(ast::Functor::Ctl),
                        callee,
                        *mod_span,
                    );
                }
                GateModifier::NegCtrl(controls, mod_span) => {
                    // remove the last n qubits from the qubit list
                    let num_ctrls = controls.unwrap_or(1);
                    if qubit_args.len() < num_ctrls {
                        let kind =
                            SemanticErrorKind::InvalidNumberOfQubitArgs(qargs_len, 0, call_span);
                        self.push_semantic_error(kind);
                        return None;
                    }
                    let ctrl = qubit_args.split_off(qubit_args.len() - num_ctrls);
                    let ctrls = build_expr_array_expr(ctrl, *mod_span);
                    let lit_0 = build_lit_int_expr(0, Span::default());
                    args = build_tuple_expr(vec![lit_0, callee, ctrls, args]);
                    callee = build_path_ident_expr("ApplyControlledOnInt", *mod_span, expr_span);
                }
            }
        }

        self.validate_all_quantum_args_have_been_consumed(&qubit_args, qargs_len, call_span)?;

        let expr = ast_builder::build_gate_call_with_params_and_callee(args, callee, expr_span);
        Some(QasmTypedExpr {
            ty: Type::Void,
            expr,
        })
    }

    /// Push if all qubit args have not been consumed.
    /// Resurns None for an error, Some(()) for success.
    /// This allows short-circuiting of the function.
    fn validate_all_quantum_args_have_been_consumed(
        &mut self,
        qubit_args: &[ast::Expr],
        qargs_len: usize,
        call_span: Span,
    ) -> Option<()> {
        // This is a safety check. We should have peeled off all the controls
        // but if we haven't, we need to push an error
        if qubit_args.is_empty() {
            return Some(());
        }
        let kind =
            SemanticErrorKind::InvalidNumberOfQubitArgs(qargs_len, qubit_args.len(), call_span);
        self.push_semantic_error(kind);
        None
    }

    /// Raises an error if the number of qubit arguments does not match the number
    /// of qubit arguments expected by the gate and the number of controls.
    fn verify_qubit_args_match_gate_and_ctrls(
        &mut self,
        qubit_args: &[ast::Expr],
        qargs_len: usize,
        num_ctrls: u64,
        gate_call_expr: &oq3_syntax::ast::GateCallExpr,
    ) -> Option<()> {
        let gate_call_span = span_for_syntax_node(gate_call_expr.syntax());
        let Some(num_ctrls) = usize::try_from(num_ctrls).ok() else {
            let kind = SemanticErrorKind::TooManyControls(gate_call_span);
            self.push_semantic_error(kind);
            return None;
        };

        if qubit_args.len() != qargs_len + num_ctrls {
            let span = if qubit_args.is_empty() {
                gate_call_span
            } else {
                span_for_syntax_node(
                    gate_call_expr
                        .qubit_list()
                        .expect("Qubit list must exist")
                        .syntax(),
                )
            };
            let kind =
                SemanticErrorKind::InvalidNumberOfQubitArgs(qargs_len, qubit_args.len(), span);
            self.push_semantic_error(kind);
            return None;
        };
        Some(())
    }

    /// Compiles the gate call qubit arguments. This is a helper function
    fn compile_gate_call_quantum_args(
        &mut self,
        gate_call_expr: &oq3_syntax::ast::GateCallExpr,
    ) -> Option<Vec<ast::Expr>> {
        let qubit_args: Vec<_> = gate_call_expr
            .qubit_list()
            .expect("Cannot call a gate without qubit arguments")
            .gate_operands()
            .map(|op| self.compile_gate_operand(&op).map(|x| x.expr))
            .collect();
        if qubit_args.iter().any(Option::is_none) {
            // if any of the qubit arguments failed to compile, we can't proceed
            // This can happen if the qubit is not defined or if the qubit was
            // a hardware qubit
            return None;
        }
        let qubit_args = qubit_args
            .into_iter()
            .map(|x| x.expect("All items should have value"))
            .collect::<Vec<_>>();
        Some(qubit_args)
    }

    /// Compiles the gate call classical argument expressions. This is a helper function
    fn compile_gate_call_classical_args(
        &mut self,
        gate_call_expr: &oq3_syntax::ast::GateCallExpr,
        cargs_len: usize,
    ) -> Option<Vec<ast::Expr>> {
        let classical_args = match gate_call_expr.arg_list() {
            Some(params) => {
                let list = params
                    .expression_list()
                    .expect("Arg list must have an expression list");

                // the classical args list is a list of expressions
                // but the type of the args is fixed by the gate definition
                // which should always move to float.
                let angle_ty = Type::Float(None, IsConst::False);
                let exprs = list
                    .exprs()
                    .map(|expr| {
                        self.compile_expr_to_ty_with_casts(
                            Some(expr),
                            &angle_ty,
                            gate_call_expr.syntax(),
                        )
                    })
                    .collect::<Vec<_>>();

                if !exprs.iter().all(Option::is_some) {
                    // There was an issue with one of the expressions
                    // and an error was pushed
                    return None;
                }
                exprs
                    .into_iter()
                    .map(|expr| expr.expect("All items should have value"))
                    .collect::<Vec<_>>()
            }
            None => Vec::new(),
        };

        if classical_args.len() != cargs_len {
            let gate_call_span = span_for_syntax_node(gate_call_expr.syntax());
            let span = if classical_args.is_empty() {
                gate_call_span
            } else {
                span_for_syntax_node(
                    gate_call_expr
                        .arg_list()
                        .expect("Qubit list must exist")
                        .syntax(),
                )
            };
            let kind = SemanticErrorKind::InvalidNumberOfClassicalArgs(
                cargs_len,
                classical_args.len(),
                span,
            );
            self.push_semantic_error(kind);
            return None;
        }
        Some(classical_args)
    }

    /// Compiles the expression list. Returns None if any of the expressions
    /// fail to compile. If all expressions compile, returns a vector of
    /// the compiled expressions. An error is pushed if any of the expressions
    /// fail to compile.
    fn compile_expression_list(
        &mut self,
        expr_list: &oq3_syntax::ast::ExpressionList,
    ) -> Option<Vec<QasmTypedExpr>> {
        let exprs: Vec<_> = expr_list.exprs().collect();
        let exprs_len = exprs.len();
        let mapped_exprs: Vec<_> = exprs
            .into_iter()
            .filter_map(|expr| self.compile_expr(&expr))
            .collect();
        if exprs_len == mapped_exprs.len() {
            return Some(mapped_exprs);
        }
        let kind = SemanticErrorKind::FailedToCompileExpressionList(span_for_syntax_node(
            expr_list.syntax(),
        ));
        self.push_semantic_error(kind);
        None
    }

    /// Compiles the expression list attempting to coerce the expressions to a
    /// specific type.
    /// Returns None if any of the expressions fail to compile and an error is
    /// pushed. If all expressions compile, returns a vector of the compiled
    /// expressions.
    fn compile_typed_expression_list(
        &mut self,
        expr_list: &oq3_syntax::ast::ExpressionList,
        ty: &Type,
    ) -> Option<Vec<QasmTypedExpr>> {
        let exprs: Vec<_> = expr_list.exprs().collect();
        let exprs_len = exprs.len();
        let mapped_exprs: Vec<_> = exprs
            .into_iter()
            .filter_map(|expr| {
                self.compile_expr_to_ty_with_casts(Some(expr.clone()), ty, expr.syntax())
                    .map(|expr| QasmTypedExpr {
                        expr,
                        ty: ty.clone(),
                    })
            })
            .collect();
        if exprs_len == mapped_exprs.len() {
            return Some(mapped_exprs);
        }
        let kind = SemanticErrorKind::FailedToCompileExpressionList(span_for_syntax_node(
            expr_list.syntax(),
        ));
        self.push_semantic_error(kind);
        None
    }

    /// Compiles qubit arguments for an instruction call.
    fn compile_gate_operand(&mut self, op: &GateOperand) -> Option<QasmTypedExpr> {
        let op_span = span_for_syntax_node(op.syntax());
        match op {
            GateOperand::HardwareQubit(hw) => {
                // We don't support hardware qubits, so we need to push an error
                // but we can still create an identifier for the hardware qubit
                // and let the rest of the containing expression compile to
                // catch any other errors
                let message = "Hardware qubit operands";
                self.push_unsupported_error_message(message, hw.syntax());

                let name = hw.to_string();
                let name_span = span_for_syntax_node(hw.syntax());
                let ident = build_path_ident_expr(name, name_span, op_span);
                Some(QasmTypedExpr {
                    ty: Type::HardwareQubit,
                    expr: ident,
                })
            }
            GateOperand::Identifier(ident) => {
                let name = ident.to_string();
                let name_span = span_for_syntax_node(ident.syntax());
                let Some(sym) = self.symbols.get_symbol_by_name(name.as_str()) else {
                    self.push_missing_symbol_error(name.as_str(), op.syntax());
                    return None;
                };
                let ty = sym.ty.clone();
                if !matches!(ty, Type::Qubit | Type::QubitArray(_)) {
                    let kind = SemanticErrorKind::InvalidGateOperand(op_span);
                    self.push_semantic_error(kind);
                }
                let ident = build_path_ident_expr(name, name_span, op_span);
                Some(QasmTypedExpr { ty, expr: ident })
            }
            GateOperand::IndexedIdentifier(indexed_ident) => {
                let expr: QasmTypedExpr = self.compile_indexed_identifier_expr(indexed_ident)?;
                // the type of the ident may be been Type::QubitArray, but the type of
                // the returned expression should be Type::Qubit
                if !matches!(expr.ty, Type::Qubit) {
                    let kind = SemanticErrorKind::InvalidIndexedGateOperand(op_span);
                    self.push_semantic_error(kind);
                }
                Some(expr)
            }
        }
    }
    fn compile_index_operator(
        &mut self,
        op: &oq3_syntax::ast::IndexOperator,
    ) -> Option<Vec<QasmTypedExpr>> {
        match op.index_kind() {
            Some(oq3_syntax::ast::IndexKind::SetExpression(expr)) => {
                let expr = expr.expression_list()?;
                self.compile_expression_list(&expr)
            }
            Some(oq3_syntax::ast::IndexKind::ExpressionList(expr)) => {
                self.compile_expression_list(&expr)
            }
            None => {
                let span = span_for_syntax_node(op.syntax());
                let kind = SemanticErrorKind::UnknownIndexedOperatorKind(span);
                self.push_semantic_error(kind);
                None
            }
        }
    }

    #[allow(dead_code)]
    fn compile_gphase_call_expr(
        &mut self,
        gphase_call_expr: &oq3_syntax::ast::GPhaseCallExpr,
        expr: &Expr,
    ) -> Option<QasmTypedExpr> {
        let expr_span = span_for_syntax_node(expr.syntax());
        self.compile_gphase_call_expr_impl(gphase_call_expr, expr_span, &[])
    }

    fn compile_gphase_call_expr_impl(
        &mut self,
        gphase_call_expr: &oq3_syntax::ast::GPhaseCallExpr,
        _expr_span: Span,
        _modifiers: &[crate::types::GateModifier],
    ) -> Option<QasmTypedExpr> {
        self.push_unimplemented_error_message("gphase expressions", gphase_call_expr.syntax());
        None
    }

    fn compile_hardware_qubit_expr(
        &mut self,
        _hardware_qubit: &oq3_syntax::ast::HardwareQubit,
        expr: &Expr,
    ) -> Option<QasmTypedExpr> {
        self.push_unsupported_error_message("hardware qubit expressions", expr.syntax());
        None
    }

    fn compile_identifier_expr(
        &mut self,
        identifier: &oq3_syntax::ast::Identifier,
        expr: &Expr,
    ) -> Option<QasmTypedExpr> {
        let name = identifier.to_string();
        let Some(sym) = self.symbols.get_symbol_by_name(name.as_str()) else {
            self.push_missing_symbol_error(&name, expr.syntax());
            return None;
        };
        let span = span_for_syntax_node(identifier.syntax());
        let expr_span = span_for_syntax_node(expr.syntax());
        match sym.name.as_str() {
            "euler" | "ℇ" => {
                let expr = build_math_call_no_params("E", span);
                let ty = Type::Float(None, IsConst::True);
                Some(QasmTypedExpr { ty, expr })
            }
            "pi" | "π" => {
                let expr = build_math_call_no_params("PI", span);
                let ty = Type::Float(None, IsConst::True);
                Some(QasmTypedExpr { ty, expr })
            }
            "tau" | "τ" => {
                let expr = build_math_call_no_params("PI", span);
                let ty = Type::Float(None, IsConst::True);
                let expr = ast::Expr {
                    kind: Box::new(ast::ExprKind::BinOp(
                        ast::BinOp::Mul,
                        Box::new(build_lit_double_expr(2.0, span)),
                        Box::new(expr),
                    )),
                    span,
                    id: NodeId::default(),
                };
                Some(QasmTypedExpr { ty, expr })
            }
            _ => {
                let expr = build_path_ident_expr(&sym.name, span, expr_span);
                let ty = sym.ty.clone();
                Some(QasmTypedExpr { ty, expr })
            }
        }
    }

    fn compile_index_expr(
        &mut self,
        index_expr: &oq3_syntax::ast::IndexExpr,
    ) -> Option<QasmTypedExpr> {
        let expr = index_expr.expr()?;
        let expr_span = span_for_syntax_node(index_expr.syntax());
        let texpr = self.compile_expr(&expr)?;
        let index = index_expr.index_operator()?;
        let indices = self.compile_index_operator(&index)?;
        let index_span = span_for_syntax_node(index.syntax());

        if indices.len() != 1 {
            // This is a temporary limitation. We can only handle
            // single index expressions for now.
            let kind = SemanticErrorKind::IndexMustBeSingleExpr(index_span);
            self.push_semantic_error(kind);
            return None;
        }
        let index = indices[0].clone();
        if index.ty.num_dims() > texpr.ty.num_dims() {
            let kind = SemanticErrorKind::TypeRankError(index_span);
            self.push_semantic_error(kind);
        }
        let index_expr = index.expr.clone();
        let Some(indexed_ty) = get_indexed_type(&texpr.ty) else {
            let kind =
                SemanticErrorKind::CannotIndexType(format!("{:?}", texpr.ty), texpr.expr.span);
            self.push_semantic_error(kind);
            return None;
        };

        let expr = ast_builder::build_index_expr(texpr.expr, index_expr, expr_span);
        Some(QasmTypedExpr {
            ty: indexed_ty,
            expr,
        })
    }

    /// Compiles a indexed expr `a[i]` where `a` is an identifier and `i` is an expression.
    /// The type of the expression is determined by the indexed type of the identifier
    /// resolved by `self.get_indexed_type`.
    fn compile_indexed_identifier_expr(
        &mut self,
        indexed_ident: &oq3_syntax::ast::IndexedIdentifier,
    ) -> Option<QasmTypedExpr> {
        let name = indexed_ident.identifier()?.to_string();
        let name_span = span_for_syntax_node(indexed_ident.syntax());
        let Some(sym) = self.symbols.get_symbol_by_name(name.as_str()) else {
            self.push_missing_symbol_error(name.as_str(), indexed_ident.syntax());
            return None;
        };
        let sym = sym.clone();
        let op_span = span_for_syntax_node(indexed_ident.syntax());

        let index: Vec<_> = indexed_ident
            .index_operators()
            .filter_map(|op| self.compile_index_operator(&op))
            .flatten()
            .collect();

        assert!(index.len() == 1, "index must be a single expression");
        let ident = build_path_ident_expr(name, name_span, op_span);
        let expr = ast::Expr {
            id: NodeId::default(),
            span: span_for_syntax_node(indexed_ident.syntax()),
            kind: Box::new(ast::ExprKind::Index(
                Box::new(ident),
                Box::new(index[0].expr.clone()),
            )),
        };
        let Some(indexed_ty) = get_indexed_type(&sym.ty) else {
            let kind = SemanticErrorKind::CannotIndexType(format!("{:?}", sym.ty), op_span);
            self.push_semantic_error(kind);
            return None;
        };
        Some(QasmTypedExpr {
            ty: indexed_ty,
            expr,
        })
    }

    fn compile_literal_expr(
        &mut self,
        lit: &oq3_syntax::ast::Literal,
        expr: &Expr,
    ) -> Option<QasmTypedExpr> {
        let span = span_for_syntax_node(lit.syntax());
        match lit.kind() {
            LiteralKind::BitString(bitstring) => compile_bitstring(&bitstring, span),
            LiteralKind::Bool(value) => {
                let expr = build_lit_bool_expr(value, span);
                let ty = Type::Bool(IsConst::True);
                Some(QasmTypedExpr { ty, expr })
            }
            LiteralKind::Byte(_) => {
                self.push_unimplemented_error_message("byte literal expressions", expr.syntax());
                None
            }
            LiteralKind::Char(_) => {
                self.push_unimplemented_error_message("char literal expressions", expr.syntax());
                None
            }
            LiteralKind::FloatNumber(value) => {
                let expr = Self::compile_float_literal(&value, span);
                let ty = Type::Float(None, IsConst::True);
                Some(QasmTypedExpr { ty, expr })
            }
            LiteralKind::IntNumber(value) => {
                let expr = Self::compile_int_literal(&value, span);
                let ty = Type::UInt(None, IsConst::True);
                Some(QasmTypedExpr { ty, expr })
            }
            LiteralKind::String(string) => self.compile_string_literal(&string, expr),
        }
    }

    /// Compiles a complex literal expression from a literal int.
    fn compile_int_to_double_literal_to_complex(
        &mut self,
        value: &oq3_syntax::ast::IntNumber,
        span: Span,
    ) -> Option<ast::Expr> {
        let value = value.value().expect("FloatNumber must have a value");
        if let Some(value) = safe_u128_to_f64(value) {
            Some(build_complex_from_expr(build_lit_double_expr(value, span)))
        } else {
            let kind = SemanticErrorKind::InvalidCastValueRange(
                "Integer".to_string(),
                "Double".to_string(),
                span,
            );
            self.push_semantic_error(kind);
            None
        }
    }

    /// Compiles a double expression from a literal int.
    fn compile_int_to_double_literal(
        &mut self,
        value: &oq3_syntax::ast::IntNumber,
        negate: bool,
        span: Span,
    ) -> Option<ast::Expr> {
        let value = value.value().expect("FloatNumber must have a value");
        if let Some(value) = safe_u128_to_f64(value) {
            let value = if negate { -value } else { value };
            Some(build_lit_double_expr(value, span))
        } else {
            let kind = SemanticErrorKind::InvalidCastValueRange(
                "Integer".to_string(),
                "Double".to_string(),
                span,
            );
            self.push_semantic_error(kind);
            None
        }
    }

    fn compile_float_literal(value: &oq3_syntax::ast::FloatNumber, span: Span) -> ast::Expr {
        build_lit_double_expr(value.value().expect("FloatNumber must have a value"), span)
    }

    fn compile_int_literal(value: &oq3_syntax::ast::IntNumber, span: Span) -> ast::Expr {
        if let Some(value) = value.value() {
            match value.try_into() {
                Ok(value) => build_lit_int_expr(value, span),
                Err(_) => build_lit_bigint_expr(value.into(), span),
            }
        } else {
            panic!("IntNumber must have a value");
        }
    }

    fn compile_string_literal(
        &mut self,
        _string: &oq3_syntax::ast::String,
        expr: &Expr,
    ) -> Option<QasmTypedExpr> {
        self.push_unimplemented_error_message("string literal expressions", expr.syntax());
        None
    }

    fn compile_timing_literal_expr(
        &mut self,
        lit: &oq3_syntax::ast::TimingLiteral,
        expr: &Expr,
    ) -> Option<QasmTypedExpr> {
        self.compile_timing_literal_as_complex(lit, expr, false)
    }

    // OpenQASM parser bundles complex numbers with timing literals
    // so we have to disambiguate them during timing literal compilation
    fn compile_timing_literal_as_complex(
        &mut self,
        lit: &TimingLiteral,
        expr: &Expr,
        negate: bool,
    ) -> Option<QasmTypedExpr> {
        if let Some(TimeUnit::Imaginary) = lit.time_unit() {
            let literal = lit.literal()?;
            match literal.kind() {
                LiteralKind::FloatNumber(value) => {
                    let value = value.value().expect("FloatNumber must have a value");
                    let value = if negate { -value } else { value };
                    let expr = build_lit_complex_expr(
                        crate::types::Complex::new(0.0, value),
                        span_for_syntax_node(lit.syntax()),
                    );
                    let ty = Type::Complex(None, IsConst::True);
                    Some(QasmTypedExpr { ty, expr })
                }
                LiteralKind::IntNumber(value) => {
                    let value = value.value().expect("IntNumber must have a value");

                    if let Some(value) = safe_u128_to_f64(value) {
                        let value = if negate { -value } else { value };
                        let expr = build_lit_complex_expr(
                            crate::types::Complex::new(0.0, value),
                            span_for_syntax_node(lit.syntax()),
                        );
                        let ty = Type::Complex(None, IsConst::True);
                        Some(QasmTypedExpr { ty, expr })
                    } else {
                        let kind = SemanticErrorKind::InvalidCastValueRange(
                            "Complex imaginary".to_string(),
                            "Float".to_string(),
                            span_for_syntax_node(literal.syntax()),
                        );
                        self.push_semantic_error(kind);
                        None
                    }
                }
                _ => {
                    // parser bug
                    unreachable!(
                        "Expected float or int literal, there is a bug in the OpenQASM parser."
                    )
                }
            }
        } else {
            self.push_unsupported_error_message("Timing literal expressions", expr.syntax());
            None
        }
    }

    fn compile_measure_expr(
        &mut self,
        measure_expr: &oq3_syntax::ast::MeasureExpression,
        expr: &Expr,
    ) -> Option<QasmTypedExpr> {
        let Some(measure_token) = measure_expr.measure_token() else {
            let span = span_for_syntax_node(expr.syntax());
            let kind = SemanticErrorKind::MeasureExpressionsMustHaveName(span);
            self.push_semantic_error(kind);
            return None;
        };
        let name_span = span_for_syntax_token(&measure_token);
        let stmt_span = span_for_syntax_node(measure_expr.syntax());

        let Some(operand) = measure_expr.gate_operand() else {
            let span = span_for_syntax_node(expr.syntax());
            let kind = SemanticErrorKind::MeasureExpressionsMustHaveGateOperand(span);
            self.push_semantic_error(kind);
            return None;
        };

        let args = self.compile_gate_operand(&operand)?;
        let operand_span = span_for_syntax_node(operand.syntax());
        let expr = build_measure_call(args.expr, name_span, operand_span, stmt_span);

        Some(QasmTypedExpr {
            ty: Type::Bit(IsConst::False),
            expr,
        })
    }

    fn compile_modified_gate_call_expr(
        &mut self,
        modified_gate_call_expr: &oq3_syntax::ast::ModifiedGateCallExpr,
    ) -> Option<QasmTypedExpr> {
        let expr_span = span_for_syntax_node(modified_gate_call_expr.syntax());
        let modifiers = modified_gate_call_expr
            .modifiers()
            .map(|modifier| {
                let span = span_for_syntax_node(modifier.syntax());
                match modifier {
                    Modifier::InvModifier(_) => GateModifier::Inv(span),
                    Modifier::PowModifier(pow_mod) => {
                        let Some(expr) = pow_mod.paren_expr() else {
                            let kind = SemanticErrorKind::PowModifierMustHaveExponent(span);
                            self.push_semantic_error(kind);
                            return GateModifier::Pow(None, span);
                        };
                        extract_pow_exponent(&expr, span)
                    }
                    Modifier::CtrlModifier(ctrl_mod) => {
                        let ctrls = self.extract_controls_from_modifier(ctrl_mod.paren_expr());
                        GateModifier::Ctrl(ctrls, span)
                    }
                    Modifier::NegCtrlModifier(neg_ctrl_mod) => {
                        let ctrls = self.extract_controls_from_modifier(neg_ctrl_mod.paren_expr());
                        GateModifier::NegCtrl(ctrls, span)
                    }
                }
            })
            .collect::<Vec<_>>();

        if let Some(gate_call_expr) = modified_gate_call_expr.gate_call_expr() {
            self.compile_gate_call_expr_impl(&gate_call_expr, expr_span, &modifiers)
        } else {
            let Some(gphase_call_expr) = modified_gate_call_expr.g_phase_call_expr() else {
                // error
                return None;
            };
            self.compile_gphase_call_expr_impl(&gphase_call_expr, expr_span, &modifiers)
        }
    }

    /// Extracts the literal int from `ctrl(value)` and `negctrl(value)` modifiers.
    /// Returns None if the modifier is invalid and pushes an error.
    /// Returns Some(1) if the modifier is empty.
    /// Returns Some(value) if the modifier is valid.
    fn extract_controls_from_modifier(&mut self, paren_expr: Option<ParenExpr>) -> Option<usize> {
        if let Some(paren_expr) = paren_expr {
            if let Some((ctrl, sign)) = compile_paren_lit_int_expr(&paren_expr) {
                if sign {
                    let kind = SemanticErrorKind::NegativeControlCount(span_for_syntax_node(
                        paren_expr.syntax(),
                    ));
                    self.push_semantic_error(kind);
                }
                Some(ctrl)
            } else {
                let kind = SemanticErrorKind::InvalidControlCount(span_for_syntax_node(
                    paren_expr.syntax(),
                ));
                self.push_semantic_error(kind);
                None
            }
        } else {
            Some(1)
        }
    }

    fn compile_paren_expr(
        &mut self,
        paren_expr: &oq3_syntax::ast::ParenExpr,
    ) -> Option<QasmTypedExpr> {
        let span = span_for_syntax_node(paren_expr.syntax());
        let expr = paren_expr.expr()?;
        let texpr = self.compile_expr(&expr)?;
        let pexpr = ast_builder::wrap_expr_in_parens(texpr.expr, span);
        Some(QasmTypedExpr {
            ty: texpr.ty.clone(),
            expr: pexpr,
        })
    }

    fn compile_negated_literal_as_ty(
        &mut self,
        literal: &Literal,
        ty: Option<&Type>,
    ) -> Option<QasmTypedExpr> {
        let span = span_for_syntax_node(literal.syntax());
        match literal.kind() {
            LiteralKind::IntNumber(value) => match ty {
                Some(Type::Float(..)) => {
                    let expr = self.compile_int_to_double_literal(&value, true, span)?;
                    Some(QasmTypedExpr {
                        ty: ty.expect("Expected type").clone(),
                        expr,
                    })
                }
                _ => Some(compile_intnumber_as_negated_int(&value, span)),
            },
            LiteralKind::FloatNumber(value) => match ty {
                Some(Type::Int(..) | Type::UInt(..)) => {
                    let value = value.value().expect("FloatNumber must have a value");
                    #[allow(clippy::cast_possible_truncation)]
                    let converted_value = value.trunc() as i64;
                    #[allow(clippy::cast_precision_loss)]
                    if (converted_value as f64 - value).abs() > f64::EPSILON {
                        let span = span_for_syntax_node(literal.syntax());
                        let kind = SemanticErrorKind::CastWouldCauseTruncation(
                            "Float".to_string(),
                            format!("{:?}", ty.expect("Expected type")),
                            span,
                        );
                        self.push_semantic_error(kind);
                        None
                    } else {
                        let expr = build_lit_int_expr(-converted_value, span);
                        let ty = ty.expect("Expected type").clone();
                        Some(QasmTypedExpr { ty, expr })
                    }
                }
                _ => Some(compile_floatnumber_as_negated_double(&value, span)),
            },
            _ => {
                self.push_unimplemented_error_message(
                    "negated literal expressions",
                    literal.syntax(),
                );
                None
            }
        }
    }

    fn compile_prefix_expr(
        &mut self,
        prefix_expr: &oq3_syntax::ast::PrefixExpr,
    ) -> Option<QasmTypedExpr> {
        let prefix_span = span_for_syntax_node(prefix_expr.syntax());
        match prefix_expr.op_kind() {
            Some(UnaryOp::Neg) => match prefix_expr.expr() {
                Some(Expr::Literal(lit)) => self.compile_negated_literal_as_ty(&lit, None),
                Some(Expr::TimingLiteral(lit)) => {
                    let expr = prefix_expr
                        .expr()
                        .expect("TimingLiteral must have an expression");
                    self.compile_timing_literal_as_complex(&lit, &expr, true)
                }
                Some(expr) => {
                    let texpr = self.compile_expr(&expr)?;
                    let expr = build_unary_op_expr(ast::UnOp::Neg, texpr.expr, prefix_span);
                    let ty = texpr.ty;
                    Some(QasmTypedExpr { ty, expr })
                }
                None => {
                    self.push_unimplemented_error_message(
                        "negated empty expressions",
                        prefix_expr.syntax(),
                    );
                    None
                }
            },
            Some(UnaryOp::LogicNot) => {
                // bug in QASM parser, logical not and bitwise not are backwards
                if let Some(prefix) = prefix_expr.expr() {
                    let texpr = self.compile_expr(&prefix)?;
                    let expr = build_unary_op_expr(ast::UnOp::NotB, texpr.expr, prefix_span);
                    let ty = texpr.ty;
                    Some(QasmTypedExpr { ty, expr })
                } else {
                    self.push_unimplemented_error_message(
                        "bitwise not empty expressions",
                        prefix_expr.syntax(),
                    );
                    None
                }
            }
            Some(UnaryOp::Not) => {
                // bug in QASM parser, logical not and bitwise not are backwards
                // THIS CODE IS FOR LOGICAL NOT
                let ty = Type::Bool(IsConst::False);
                let expr = self.compile_expr_to_ty_with_casts(
                    prefix_expr.expr(),
                    &ty,
                    prefix_expr.syntax(),
                )?;
                let expr = build_unary_op_expr(ast::UnOp::NotL, expr, prefix_span);
                Some(QasmTypedExpr { ty, expr })
            }
            None => None,
        }
    }

    #[allow(clippy::similar_names)]
    fn compile_range_expr(
        &mut self,
        range_expr: &oq3_syntax::ast::RangeExpr,
        node: &SyntaxNode,
    ) -> Option<QasmTypedExpr> {
        let (start, step, stop) = range_expr.start_step_stop();
        let Some(start) = start else {
            let span = span_for_syntax_node(range_expr.syntax());
            let kind = SemanticErrorKind::RangeExpressionsMustHaveStart(span);
            self.push_semantic_error(kind);
            return None;
        };
        let Some(stop) = stop else {
            let span = span_for_syntax_node(range_expr.syntax());
            let kind = SemanticErrorKind::RangeExpressionsMustHaveStop(span);
            self.push_semantic_error(kind);
            return None;
        };
        let start_texpr = self.compile_expr(&start)?;
        let stop_texpr = self.compile_expr(&stop)?;
        let step_texpr = if let Some(step) = step {
            Some(self.compile_expr(&step)?.expr)
        } else {
            None
        };
        Some(QasmTypedExpr {
            ty: Type::Range,
            expr: build_range_expr(
                start_texpr.expr,
                stop_texpr.expr,
                step_texpr,
                span_for_syntax_node(node),
            ),
        })
    }

    fn compile_return_expr(
        &mut self,
        return_expr: &oq3_syntax::ast::ReturnExpr,
    ) -> Option<QasmTypedExpr> {
        let stmt_span = span_for_syntax_node(return_expr.syntax());
        if !self.symbols.is_scope_rooted_in_subroutine() {
            let kind = SemanticErrorKind::ReturnNotInSubroutine(stmt_span);
            self.push_semantic_error(kind);
        }
        // the containing function will have an explicit return type
        // or default of Void. We don't need to check the return type
        // as that will be handled by Q# type checker. If there is no
        // expression, we return Unit which Void maps to in Q#.
        if let Some(expr) = return_expr.expr() {
            let texpr = self.compile_expr(&expr)?;
            let expr = ast_builder::build_return_expr(texpr.expr, stmt_span);
            Some(QasmTypedExpr { ty: texpr.ty, expr })
        } else {
            let expr = ast_builder::build_return_unit(stmt_span);
            Some(QasmTypedExpr {
                ty: Type::Void,
                expr,
            })
        }
    }

    fn compile_for_stmt(&mut self, for_stmt: &oq3_syntax::ast::ForStmt) -> Option<ast::Stmt> {
        let loop_var = for_stmt
            .loop_var()
            .expect("For statement must have a loop variable");
        let loop_var_span = span_for_syntax_node(loop_var.syntax());
        let loop_var_scalar_ty = for_stmt
            .scalar_type()
            .expect("For statement must have a scalar type");
        let for_iterable = for_stmt
            .for_iterable()
            .expect("For statement must have an iterable");
        let stmt_span = span_for_syntax_node(for_stmt.syntax());
        let iterable = self.compile_for_iterable(&for_iterable)?;
        let loop_var_sem_ty =
            self.get_semantic_type_from_scalar_type(&loop_var_scalar_ty, false)?;
        let qsharp_ty =
            self.convert_semantic_type_to_qsharp_type(&loop_var_sem_ty, loop_var.syntax())?;

        let loop_var_symbol = Symbol {
            name: loop_var.to_string(),
            span: loop_var_span,
            ty: loop_var_sem_ty.clone(),
            qsharp_ty: qsharp_ty.clone(),
            io_kind: IOKind::Default,
        };

        self.symbols.push_scope(crate::symbols::ScopeKind::Block);
        if self.symbols.insert_symbol(loop_var_symbol.clone()).is_err() {
            self.push_redefined_symbol_error(loop_var.to_string(), loop_var_span);
            return None;
        }

        let body = if let Some(stmt) = for_stmt.stmt() {
            let stmt = self.compile_stmt(&stmt);
            self.symbols.pop_scope();
            build_stmt_wrapped_block_expr(stmt?)
        } else if let Some(block) = for_stmt.body() {
            let block = self.compile_block_expr(&block);
            self.symbols.pop_scope();
            block
        } else {
            let span = span_for_syntax_node(for_stmt.syntax());
            let kind = SemanticErrorKind::ForStatementsMustHaveABodyOrStatement(span);
            self.push_semantic_error(kind);
            None?
        };

        Some(ast_builder::build_for_stmt(
            &loop_var_symbol,
            iterable,
            body,
            stmt_span,
        ))
    }

    fn compile_for_iterable(
        &mut self,
        for_iterable: &oq3_syntax::ast::ForIterable,
    ) -> Option<QasmTypedExpr> {
        if let Some(expr) = for_iterable.set_expression() {
            let expr_list = expr.expression_list()?;

            let expression_list = self
                .compile_expression_list(&expr_list)?
                .into_iter()
                .map(|e| e.expr)
                .collect();

            let expr = build_expr_array_expr(expression_list, span_for_syntax_node(expr.syntax()));
            Some(QasmTypedExpr {
                ty: Type::Set,
                expr,
            })
        } else if let Some(expr) = for_iterable.range_expr() {
            self.compile_range_expr(&expr, for_iterable.syntax())
        } else if let Some(expr) = for_iterable.for_iterable_expr() {
            // For iterating over something like bit[n]
            self.compile_expr(&expr)
        } else {
            let span = span_for_syntax_node(for_iterable.syntax());
            let kind = SemanticErrorKind::ForIterableInvalidExpression(span);
            self.push_semantic_error(kind);
            None
        }
    }

    #[allow(clippy::too_many_lines)]
    fn compile_gate_decl(&mut self, gate: &oq3_syntax::ast::Gate) -> Option<ast::Stmt> {
        let name = gate.name()?;

        // Once we support angle types, we will use this as the type for
        // the angle parameters' coersion:
        //let angle_ty = Type::Angle(None, IsConst::True);
        let angle_ty = Type::Float(None, IsConst::True);
        let qs_angle_ty = self.convert_semantic_type_to_qsharp_type(&angle_ty, gate.syntax())?;
        let ast_angle_ty = map_qsharp_type_to_ast_ty(&qs_angle_ty);
        let qubit_ty = Type::Qubit;
        let qs_qubit_ty = self.convert_semantic_type_to_qsharp_type(&qubit_ty, gate.syntax())?;
        let ast_qubit_ty = map_qsharp_type_to_ast_ty(&qs_qubit_ty);

        let gate_span = span_for_syntax_node(gate.syntax());
        if !self.symbols.is_current_scope_global() {
            let kind = SemanticErrorKind::QuantumDeclarationInNonGlobalScope(gate_span);
            self.push_semantic_error(kind);
            return None;
        }
        // get the name of the args and their spans
        let cargs = gate
            .angle_params()
            .iter()
            .flat_map(ParamList::params)
            .map(|e| {
                (
                    e.text().to_string(),
                    ast_angle_ty.clone(),
                    build_arg_pat(
                        e.text().to_string(),
                        span_for_named_item(&e),
                        ast_angle_ty.clone(),
                    ),
                )
            })
            .collect::<Vec<_>>();

        let qargs = gate
            .qubit_params()
            .iter()
            .flat_map(ParamList::params)
            .map(|e| {
                (
                    e.text().to_string(),
                    ast_qubit_ty.clone(),
                    build_arg_pat(
                        e.text().to_string(),
                        span_for_named_item(&e),
                        ast_qubit_ty.clone(),
                    ),
                )
            })
            .collect::<Vec<_>>();

        self.symbols.push_scope(crate::symbols::ScopeKind::Gate);
        // bind the cargs, qargs and body

        for (name, _, pat) in &cargs {
            let symbol = Symbol {
                name: name.clone(),
                span: pat.span,
                ty: angle_ty.clone(),
                qsharp_ty: qs_angle_ty.clone(),
                io_kind: IOKind::Default,
            };
            if self.symbols.insert_symbol(symbol).is_err() {
                self.push_redefined_symbol_error(name, pat.span);
            }
        }
        for (name, _, pat) in &qargs {
            let symbol = Symbol {
                name: name.clone(),
                span: pat.span,
                ty: qubit_ty.clone(),
                qsharp_ty: qs_qubit_ty.clone(),
                io_kind: IOKind::Default,
            };
            if self.symbols.insert_symbol(symbol).is_err() {
                self.push_redefined_symbol_error(name, pat.span);
            }
        }
        let body = gate.body().map(|body| self.compile_block_expr(&body));
        let body_span = gate
            .body()
            .map(|body| span_for_syntax_node(body.syntax()))
            .unwrap_or_default();
        self.symbols.pop_scope();
        // create a gate symbol with type information with num cargs and qargs

        let gate_ty = Type::Gate(cargs.len(), qargs.len());
        let qs_gate_ty = self.convert_semantic_type_to_qsharp_type(&gate_ty, gate.syntax())?;
        let name_span = span_for_syntax_node(name.syntax());
        let symbol = Symbol {
            name: name.to_string(),
            span: name_span,
            ty: gate_ty,
            qsharp_ty: qs_gate_ty,
            io_kind: IOKind::Default,
        };
        if self.symbols.insert_symbol(symbol).is_err() {
            self.push_redefined_symbol_error(name.to_string(), span_for_syntax_node(name.syntax()));
            return None;
        }

        if self.next_gate_as_item {
            Some(ast_builder::build_gate_decl(
                name.to_string(),
                cargs,
                qargs,
                body,
                name_span,
                body_span,
                gate_span,
            ))
        } else {
            Some(build_gate_decl_lambda(
                name.to_string(),
                cargs,
                qargs,
                body,
                name_span,
                body_span,
                gate_span,
            ))
        }
    }

    fn compile_if_stmt(&mut self, if_stmt: &oq3_syntax::ast::IfStmt) -> Option<ast::Stmt> {
        let stmt_span = span_for_syntax_node(if_stmt.syntax());

        let Some(condition) = if_stmt.condition() else {
            let kind =
                SemanticErrorKind::IfStmtMissingExpression("condition".to_string(), stmt_span);
            self.push_semantic_error(kind);
            return None;
        };
        let node = condition.syntax();
        let cond_ty = Type::Bool(IsConst::False);
        let cond = self
            .compile_expr_to_ty_with_casts(Some(condition.clone()), &cond_ty, node)
            .map(|expr| QasmTypedExpr { ty: cond_ty, expr });

        let Some(block) = if_stmt.then_branch() else {
            let kind =
                SemanticErrorKind::IfStmtMissingExpression("then block".to_string(), stmt_span);
            self.push_semantic_error(kind);
            return None;
        };

        self.symbols.push_scope(crate::symbols::ScopeKind::Block);
        let then_block = self.compile_block_expr(&block);
        self.symbols.pop_scope();
        let else_block = if_stmt.else_branch().map(|block_expr| {
            self.symbols.push_scope(crate::symbols::ScopeKind::Block);
            let else_expr = self.compile_block_expr(&block_expr);
            self.symbols.pop_scope();
            else_expr
        });

        // The cond may have failed to compile, in which case we return None
        // we let it go this far so that we could accumulate any errors in
        // the block.
        let cond = cond?;
        let if_expr = if let Some(else_block) = else_block {
            build_if_expr_then_block_else_block(cond.expr, then_block, else_block, stmt_span)
        } else {
            build_if_expr_then_block(cond.expr, then_block, stmt_span)
        };

        Some(build_stmt_semi_from_expr(if_expr))
    }

    fn compile_io_decl_stmt(
        &mut self,
        decl: &oq3_syntax::ast::IODeclarationStatement,
    ) -> Option<ast::Stmt> {
        if decl.array_type().is_some() {
            self.push_unimplemented_error_message("array io declarations", decl.syntax());
            return None;
        }
        let name = decl.name().expect("io declaration must have a name");
        let scalar_ty = decl
            .scalar_type()
            .expect("io declaration must have a scalar type");
        let io_kind = match decl.input_token() {
            Some(_) => IOKind::Input,
            None => IOKind::Output,
        };
        // if we can't convert the scalar type, we can't proceed, an error has been pushed
        let ty = self.get_semantic_type_from_scalar_type(&scalar_ty, false)?;
        let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty, name.syntax())?;
        let symbol = Symbol {
            name: name.to_string(),
            span: span_for_syntax_node(name.syntax()),
            ty: ty.clone(),
            qsharp_ty: qsharp_ty.clone(),
            io_kind: io_kind.clone(),
        };

        if self.symbols.insert_symbol(symbol).is_err() {
            self.push_redefined_symbol_error(name.to_string(), span_for_syntax_node(name.syntax()));
            return None;
        }

        // if we have output, we need to assign a default value to declare the variable
        // if we have input, we can keep return none as we would promote the variable
        // to a parameter in the function signature once we generate the function
        if io_kind == IOKind::Output {
            let rhs = self.get_default_value(&ty, name.syntax())?;
            let stmt = build_classical_decl(
                name.to_string(),
                false,
                span_for_syntax_node(scalar_ty.syntax()),
                span_for_syntax_node(decl.syntax()),
                span_for_syntax_node(name.syntax()),
                &qsharp_ty,
                rhs,
            );
            Some(stmt)
        } else {
            None
        }
    }

    /// Let statements shouldn't make it into parsing
    /// Looking at the parser, this statement seems
    /// anachronistic and should be removed from the parser
    fn compile_let_stmt(&mut self, let_stmt: &oq3_syntax::ast::LetStmt) -> Option<ast::Stmt> {
        self.push_unsupported_error_message("let statements", let_stmt.syntax());
        None
    }

    /// Measure statements shouldn't make it into parsing
    /// Looking at the parser, this statement seems
    /// anachronistic and should be removed from the parser
    fn compile_measure_stmt(&mut self, measure: &oq3_syntax::ast::Measure) -> Option<ast::Stmt> {
        self.push_unsupported_error_message("measure statements", measure.syntax());
        None
    }

    fn compile_quantum_decl(
        &mut self,
        decl: &oq3_syntax::ast::QuantumDeclarationStatement,
    ) -> Option<ast::Stmt> {
        let decl_span = span_for_syntax_node(decl.syntax());
        if !self.symbols.is_current_scope_global() {
            let kind = SemanticErrorKind::QuantumDeclarationInNonGlobalScope(decl_span);
            self.push_semantic_error(kind);
            return None;
        }
        let qubit_ty = decl
            .qubit_type()
            .expect("Quantum declaration must have a qubit type");
        let name = decl.name().expect("Quantum declaration must have a name");

        let designator = match qubit_ty.designator() {
            Some(designator) => {
                let width_span = span_for_syntax_node(designator.syntax());
                let width = extract_dims_from_designator(Some(designator))
                    .expect("Designator must be a literal integer");

                Some((width, width_span))
            }
            None => None,
        };
        let ty = if let Some((width, _)) = designator {
            Type::QubitArray(ArrayDims::D1(width as usize))
        } else {
            Type::Qubit
        };
        let qsharp_ty = self.convert_semantic_type_to_qsharp_type(&ty, name.syntax())?;
        let symbol = Symbol {
            name: name.to_string(),
            span: span_for_syntax_node(name.syntax()),
            ty,
            qsharp_ty,
            io_kind: IOKind::Default,
        };

        if self.symbols.insert_symbol(symbol).is_err() {
            self.push_redefined_symbol_error(name.to_string(), span_for_syntax_node(name.syntax()));
            return None;
        }
        let name = name.to_string();
        let name_span = span_for_named_item(decl);
        let stmt = match self.config.qubit_semantics {
            QubitSemantics::QSharp => {
                if let Some((width, designator_span)) = designator {
                    managed_qubit_alloc_array(name, width, decl_span, name_span, designator_span)
                } else {
                    build_managed_qubit_alloc(name, decl_span, name_span)
                }
            }
            QubitSemantics::Qiskit => {
                if let Some((width, span)) = designator {
                    build_unmanaged_qubit_alloc_array(name, width, decl_span, name_span, span)
                } else {
                    build_unmanaged_qubit_alloc(name, decl_span, name_span)
                }
            }
        };
        Some(stmt)
    }

    fn compile_reset_call(&mut self, expr: &oq3_syntax::ast::Reset) -> Option<ast::Stmt> {
        let Some(token) = expr.reset_token() else {
            let span = span_for_syntax_node(expr.syntax());
            let kind = SemanticErrorKind::ResetExpressionMustHaveName(span);
            self.push_semantic_error(kind);
            return None;
        };
        let name_span = span_for_syntax_token(&token);

        let Some(operand) = expr.gate_operand() else {
            let span = span_for_syntax_node(expr.syntax());
            let kind = SemanticErrorKind::ResetExpressionMustHaveGateOperand(span);
            self.push_semantic_error(kind);
            return None;
        };
        let args = self.compile_gate_operand(&operand)?;
        let operand_span = span_for_syntax_node(operand.syntax());
        let expr = build_reset_call(args.expr, name_span, operand_span);

        Some(build_stmt_semi_from_expr(expr))
    }

    fn compile_switch_stmt(
        &mut self,
        switch_case: &oq3_syntax::ast::SwitchCaseStmt,
    ) -> Option<ast::Stmt> {
        // The condition for the switch statement must be an integer type
        // so instead of using `compile_expr` we use `resolve_rhs_expr_with_casts`
        // forcing the type to be an integer type with implicit casts if necessary
        let cond_ty = Type::Int(None, IsConst::False);
        // We try to compile all expressions first to accumulate errors
        let control = switch_case.control().and_then(|control| {
            self.compile_expr_to_ty_with_casts(Some(control), &cond_ty, switch_case.syntax())
        });
        let cases: Vec<_> = switch_case
            .case_exprs()
            .map(|case| {
                let lhs = case
                    .expression_list()
                    .and_then(|expr| self.compile_typed_expression_list(&expr, &cond_ty));
                self.symbols.push_scope(crate::symbols::ScopeKind::Block);
                let rhs = case
                    .block_expr()
                    .map(|block| self.compile_block_expr(&block));
                self.symbols.pop_scope();
                (lhs, rhs)
            })
            .collect();
        self.symbols.push_scope(crate::symbols::ScopeKind::Block);
        let default_block = switch_case
            .default_block()
            .map(|block| self.compile_block_expr(&block));
        self.symbols.pop_scope();

        // at this point we tried to compile everything, bail if we have any errors
        if control.is_none()
            || cases
                .iter()
                .any(|(lhs, rhs)| lhs.is_none() || rhs.is_none())
            || cases.is_empty()
        {
            // See tests, but it is a parse error to have a switch statement with
            // no cases, even if the default block is present. Getting here means
            // the parser is broken or they changed the grammar.
            panic!("Switch statement must have a control expression and at least one case");
        }

        // update bindings based on what we checked above
        let control = control?;
        let cases: Vec<_> = cases
            .into_iter()
            .map(|(lhs, rhs)| {
                let lhs = lhs.expect("Case must have a lhs");
                let rhs = rhs.expect("Case must have a rhs");
                (lhs, rhs)
            })
            .collect();

        // Semantics of switch case is that the outer block doesn't introduce
        // a new scope but each case rhs does.

        // Can we add a new scope anyway to hold a temporary variable?
        // if we do that, we can refer to a new variable instead of the control
        // expr this would allow us to avoid the need to resolve the control
        // expr multiple times in the case where we have to coerce the control
        // expr to the correct type. Introducing a new variable without a new
        // scope would effect output semantics.

        // For each case, convert the lhs into a sequence of equality checks
        // and then fold them into a single expression of logical ors for
        // the if expr
        let cases: Vec<_> = cases
            .into_iter()
            .map(|(lhs, rhs)| {
                let case = lhs
                    .iter()
                    .map(|texpr| {
                        ast_builder::build_binary_expr(
                            false,
                            ast::BinOp::Eq,
                            control.clone(),
                            texpr.expr.clone(),
                            texpr.expr.span,
                        )
                    })
                    .fold(None, |acc, expr| match acc {
                        None => Some(expr),
                        Some(acc) => {
                            let qsop = ast::BinOp::OrL;
                            let span = Span {
                                lo: acc.span.lo,
                                hi: expr.span.hi,
                            };
                            Some(build_binary_expr(false, qsop, acc, expr, span))
                        }
                    });
                // The type checker doesn't know that we have at least one case
                // so we have to unwrap here since the accumulation is guaranteed
                // to have Some(value)
                let case = case.expect("Case must have at least one expression");
                (case, rhs)
            })
            .collect();

        // cond is resolved, cases are resolved, default is resolved
        // we can now build the if expression backwards. The default block
        // is the last else block, the last case is the then block, and the rest
        // are built as if-else blocks with the last case as the else block
        let default_expr = default_block.map(build_wrapped_block_expr);
        let if_expr = cases
            .into_iter()
            .rev()
            .fold(default_expr, |else_expr, (cond, block)| {
                let span = Span {
                    lo: cond.span.lo,
                    hi: block.span.hi,
                };
                Some(build_if_expr_then_block_else_expr(
                    cond, block, else_expr, span,
                ))
            });
        if_expr.map(build_stmt_semi_from_expr)
    }

    // This is a no-op in Q# but we will save it for future use
    fn compile_version_stmt(
        &mut self,
        version: &oq3_syntax::ast::VersionString,
    ) -> Option<ast::Stmt> {
        if let Some(version) = version.version() {
            let version_str = format!("{version}");
            if !version_str.starts_with("3.") {
                self.push_unsupported_error_message(
                    "OpenQASM versions other than 3",
                    version.syntax(),
                );
            }
            self.version = Some(version_str);
        }
        None
    }

    /// Note: From the ``OpenQASM`` 3.0 specification:
    /// This clearly allows users to write code that does not terminate.
    /// We do not discuss implementation details here, but one possibility
    /// is to compile into target code that imposes iteration limits.
    fn compile_while_stmt(&mut self, while_stmt: &oq3_syntax::ast::WhileStmt) -> Option<ast::Stmt> {
        let stmt_span = span_for_syntax_node(while_stmt.syntax());
        let Some(condition) = while_stmt.condition() else {
            let kind =
                SemanticErrorKind::WhileStmtMissingExpression("condition".to_string(), stmt_span);
            self.push_semantic_error(kind);
            return None;
        };

        let node = condition.syntax();
        let cond_ty = Type::Bool(IsConst::False);
        let cond = self
            .compile_expr_to_ty_with_casts(Some(condition.clone()), &cond_ty, node)
            .map(|expr| QasmTypedExpr { ty: cond_ty, expr });

        // if cond is none, an error was pushed
        // or the expression couldn't be resolved
        // We keep going to catch more errors but only if the condition
        // expression can be compiled
        let cond = match cond {
            Some(cond) => cond,
            None => self.compile_expr(&condition)?,
        };

        let Some(block) = while_stmt.body() else {
            let kind = SemanticErrorKind::WhileStmtMissingExpression("body".to_string(), stmt_span);
            self.push_semantic_error(kind);
            return None;
        };

        self.symbols.push_scope(crate::symbols::ScopeKind::Block);
        let block_body = self.compile_block_expr(&block);
        self.symbols.pop_scope();
        Some(ast_builder::build_while_stmt(
            cond.expr, block_body, stmt_span,
        ))
    }

    fn convert_semantic_type_to_qsharp_type(
        &mut self,
        ty: &Type,
        node: &SyntaxNode,
    ) -> Option<crate::types::Type> {
        let is_const = ty.is_const();
        match ty {
            Type::Bit(_) => Some(crate::types::Type::Result(is_const)),
            Type::Qubit => Some(crate::types::Type::Qubit),
            Type::HardwareQubit => {
                let message = "HardwareQubit to Q# type";
                self.push_unsupported_error_message(message, node);
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
                self.push_unsupported_error_message("Duration type values", node);
                None
            }
            Type::Stretch(_) => {
                self.push_unsupported_error_message("Stretch type values", node);
                None
            }
            Type::BitArray(dims, _) => Some(crate::types::Type::ResultArray(dims.into(), is_const)),
            Type::QubitArray(dims) => Some(crate::types::Type::QubitArray(dims.into())),
            Type::IntArray(dims) | Type::UIntArray(dims) => {
                Some(crate::types::Type::IntArray(dims.into(), is_const))
            }
            Type::FloatArray(dims) => Some(crate::types::Type::DoubleArray(dims.into())),
            Type::AngleArray(_) => todo!("AngleArray to Q# type"),
            Type::ComplexArray(_) => todo!("ComplexArray to Q# type"),
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
                self.push_unimplemented_error_message(msg, node);
                None
            }
        }
    }

    fn get_default_value(&mut self, ty: &Type, node: &SyntaxNode) -> Option<ast::Expr> {
        let span = span_for_syntax_node(node);
        match ty {
            Type::Bit(_) => Some(build_lit_result_expr(ast::Result::Zero, span)),
            Type::Qubit => {
                let message = "Qubit default values";
                self.push_unsupported_error_message(message, node);
                None
            }
            Type::HardwareQubit => {
                let message = "HardwareQubit default values";
                self.push_unsupported_error_message(message, node);
                None
            }
            Type::Int(width, _) | Type::UInt(width, _) => {
                if let Some(width) = width {
                    if *width > 64 {
                        Some(build_lit_bigint_expr(BigInt::from(0), span))
                    } else {
                        Some(build_lit_int_expr(0, span))
                    }
                } else {
                    Some(build_lit_int_expr(0, span))
                }
            }
            Type::Float(_, _) => Some(build_lit_double_expr(0.0, span)),
            Type::Angle(_, _) => todo!("Angle default values"),
            Type::Complex(_, _) => Some(build_lit_complex_expr(
                crate::types::Complex::new(0.0, 0.0),
                span,
            )),
            Type::Bool(_) => Some(build_lit_bool_expr(false, span)),
            Type::Duration(_) => {
                self.push_unsupported_error_message(
                    "Duration type values are not supported.",
                    node,
                );
                None
            }
            Type::Stretch(_) => {
                self.push_unsupported_error_message("Stretch type values are not supported.", node);
                None
            }
            Type::BitArray(dims, _) => match dims {
                ArrayDims::D1(len) => Some(build_default_result_array_expr(*len, span)),
                ArrayDims::D2(_, _) => {
                    self.push_unsupported_error_message(
                        "2-dim Bit Arrays without default values",
                        node,
                    );
                    None
                }
                ArrayDims::D3(_, _, _) => {
                    self.push_unsupported_error_message(
                        "3-dim Bit Arrays without default values",
                        node,
                    );
                    None
                }
            },
            Type::QubitArray(_) => {
                let message = "QubitArray default values";
                self.push_unsupported_error_message(message, node);
                None
            }
            Type::IntArray(_)
            | Type::UIntArray(_)
            | Type::FloatArray(_)
            | Type::AngleArray(_)
            | Type::ComplexArray(_)
            | Type::BoolArray(_) => {
                self.push_unsupported_error_message("Arrays without default values", node);
                None
            }
            Type::DurationArray(_) => {
                self.push_unsupported_error_message(
                    "DurationArray type values are not supported.",
                    node,
                );
                None
            }
            Type::Gate(_, _)
            | Type::Range
            | Type::Set
            | Type::Void
            | Type::ToDo
            | Type::Undefined => {
                let mut message = format!("Default values for {ty:?} are unsupported.");
                message.push_str(" This is likely a bug in the compiler.");
                self.push_unsupported_error_message(message, node);
                None
            }
        }
    }

    /// Define the standard gates in the symbol table.
    /// The sdg, tdg, crx, cry, crz, and ch are defined
    /// as their bare gates, and modifiers are applied
    /// when calling them.
    fn define_stdgates(&mut self, include: &oq3_syntax::ast::Include) {
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
                self.push_redefined_symbol_error(
                    name.as_str(),
                    span_for_syntax_node(include.syntax()),
                );
            }
        }
    }

    fn get_semantic_type_from_scalar_type(
        &mut self,
        scalar_ty: &oq3_syntax::ast::ScalarType,
        is_const: bool,
    ) -> Option<oq3_semantics::types::Type> {
        let designator = get_designator_from_scalar_type(scalar_ty);
        let is_const = is_const.into();
        let width = if let Some(designator) = designator {
            match designator.expr() {
                Some(oq3_syntax::ast::Expr::Literal(ref literal)) => match literal.kind() {
                    oq3_syntax::ast::LiteralKind::IntNumber(int_num) => {
                        let size: u32 = u32::try_from(int_num.value()?).ok()?;
                        Some(size)
                    }
                    _ => None,
                },
                Some(expr) => {
                    let span = span_for_syntax_node(expr.syntax());
                    let kind = SemanticErrorKind::DesignatorMustBeIntLiteral(span);
                    self.push_semantic_error(kind);
                    return None;
                }
                None => None,
            }
        } else {
            None
        };

        let ty = match scalar_ty.kind() {
            oq3_syntax::ast::ScalarTypeKind::Angle => {
                oq3_semantics::types::Type::Angle(width, is_const)
            }
            oq3_syntax::ast::ScalarTypeKind::Bit => match width {
                Some(width) => {
                    oq3_semantics::types::Type::BitArray(ArrayDims::D1(width as usize), is_const)
                }
                None => oq3_semantics::types::Type::Bit(is_const),
            },
            oq3_syntax::ast::ScalarTypeKind::Bool => oq3_semantics::types::Type::Bool(is_const),
            oq3_syntax::ast::ScalarTypeKind::Complex => {
                oq3_semantics::types::Type::Complex(width, is_const)
            }
            oq3_syntax::ast::ScalarTypeKind::Duration => {
                oq3_semantics::types::Type::Duration(is_const)
            }
            oq3_syntax::ast::ScalarTypeKind::Float => {
                oq3_semantics::types::Type::Float(width, is_const)
            }
            oq3_syntax::ast::ScalarTypeKind::Int => {
                oq3_semantics::types::Type::Int(width, is_const)
            }
            oq3_syntax::ast::ScalarTypeKind::Qubit => match width {
                Some(width) => {
                    oq3_semantics::types::Type::QubitArray(ArrayDims::D1(width as usize))
                }
                None => oq3_semantics::types::Type::Qubit,
            },
            oq3_syntax::ast::ScalarTypeKind::Stretch => {
                oq3_semantics::types::Type::Stretch(is_const)
            }
            oq3_syntax::ast::ScalarTypeKind::UInt => {
                oq3_semantics::types::Type::UInt(width, is_const)
            }
            oq3_syntax::ast::ScalarTypeKind::None => {
                let msg = "ScalarTypeKind::None should have been handled by the parser".to_string();
                let span = span_for_syntax_node(scalar_ty.syntax());
                let kind = SemanticErrorKind::UnexpectedParserError(msg, span);
                self.push_semantic_error(kind);
                return None;
            }
        };
        Some(ty)
    }

    fn try_cast_expr_to_type(
        &mut self,
        ty: &Type,
        rhs: &QasmTypedExpr,
        node: &SyntaxNode,
    ) -> Option<QasmTypedExpr> {
        if *ty == rhs.ty {
            // Base case, we shouldn't have gotten here
            // but if we did, we can just return the rhs
            return Some(rhs.clone());
        }
        if types_equal_except_const(ty, &rhs.ty) {
            if rhs.ty.is_const() {
                // lhs isn't const, but rhs is, we can just return the rhs
                return Some(rhs.clone());
            }
            // the lsh is supposed to be const but is being initialized
            // to a non-const value, we can't allow this
            return None;
        }
        // if the target type is wider, we can try to relax the rhs type
        // We only do this for float and complex. Int types
        // require extra handling for BigInts
        match (ty, &rhs.ty) {
            (Type::Float(w1, _), Type::Float(w2, _))
            | (Type::Complex(w1, _), Type::Complex(w2, _)) => {
                if w1.is_none() && w2.is_some() {
                    return Some(QasmTypedExpr {
                        ty: ty.clone(),
                        expr: rhs.expr.clone(),
                    });
                }

                if *w1 >= *w2 {
                    return Some(QasmTypedExpr {
                        ty: ty.clone(),
                        expr: rhs.expr.clone(),
                    });
                }
            }
            _ => {}
        }
        // Casting of literals is handled elsewhere. This is for casting expressions
        // which cannot be bypassed and must be handled by built-in Q# casts from
        // the standard library.
        match &rhs.ty {
            Type::Angle(_, _) => self.cast_angle_expr_to_type(ty, rhs, node),
            Type::Bit(_) => self.cast_bit_expr_to_type(ty, rhs, node),
            Type::Bool(_) => self.cast_bool_expr_to_type(ty, rhs),
            Type::Complex(_, _) => cast_complex_expr_to_type(ty, rhs),
            Type::Float(_, _) => self.cast_float_expr_to_type(ty, rhs, node),
            Type::Int(_, _) | Type::UInt(_, _) => self.cast_int_expr_to_type(ty, rhs),
            Type::BitArray(dims, _) => self.cast_bitarray_expr_to_type(dims, ty, rhs),
            _ => None,
        }
    }

    fn cast_expr_to_type(
        &mut self,
        ty: &Type,
        rhs: &QasmTypedExpr,
        node: &SyntaxNode,
    ) -> Option<QasmTypedExpr> {
        let cast_expr = self.try_cast_expr_to_type(ty, rhs, node);
        if cast_expr.is_none() {
            let rhs_ty_name = format!("{:?}", rhs.ty);
            let lhs_ty_name = format!("{ty:?}");
            let span = span_for_syntax_node(node);
            let kind = SemanticErrorKind::CannotCast(rhs_ty_name, lhs_ty_name, span);
            self.push_semantic_error(kind);
        }
        cast_expr
    }

    #[allow(clippy::too_many_lines)]
    fn cast_literal_expr_to_type(
        &mut self,
        ty: &Type,
        rhs: &QasmTypedExpr,
        literal: &Literal,
    ) -> Option<QasmTypedExpr> {
        if *ty == rhs.ty {
            // Base case, we shouldn't have gotten here
            // but if we did, we can just return the rhs
            return Some(rhs.clone());
        }
        if types_equal_except_const(ty, &rhs.ty) {
            if rhs.ty.is_const() {
                // lhs isn't const, but rhs is, we can just return the rhs
                return Some(rhs.clone());
            }
            // the lsh is supposed to be const but is being initialized
            // to a non-const value, we can't allow this
            return None;
        }
        assert!(
            can_cast_literal(ty, &rhs.ty) || can_cast_literal_with_value_knowledge(ty, literal)
        );
        let lhs_ty = ty.clone();
        let rhs_ty = rhs.ty.clone();
        let span = rhs.expr.span;

        if matches!(lhs_ty, Type::Bit(..)) {
            if let LiteralKind::IntNumber(value) = literal.kind() {
                return compile_intnumber_as_bit(&value, span, ty);
            } else if let LiteralKind::Bool(value) = literal.kind() {
                let expr = build_lit_result_expr(value.into(), span);
                return Some(QasmTypedExpr {
                    ty: ty.clone(),
                    expr,
                });
            }
        }
        // if lhs_ty is 1 dim bitarray and rhs is int/uint, we can cast
        let (is_int_to_bit_array, size) = match &lhs_ty {
            Type::BitArray(dims, _) => {
                if matches!(rhs.ty, Type::Int(..) | Type::UInt(..)) {
                    match dims {
                        ArrayDims::D1(size) => (true, *size),
                        _ => (false, 0),
                    }
                } else {
                    (false, 0)
                }
            }
            _ => (false, 0),
        };
        if is_int_to_bit_array {
            if let LiteralKind::IntNumber(value) = literal.kind() {
                // Value can't be negative as IntNumber is unsigned
                // any sign would come from a prefix expression
                if let Some(value) = value.value() {
                    if let Ok(value) = value.try_into() {
                        let value: i64 = value;
                        if value >= 0 && value < (1 << size) {
                            let bitstring = format!("{value:0size$b}");
                            let expr = build_lit_result_array_expr_from_bitstring(bitstring, span);
                            return Some(QasmTypedExpr {
                                ty: ty.clone(),
                                expr,
                            });
                        }
                        return None;
                    }
                }
            }
        }
        if matches!(lhs_ty, Type::UInt(..)) {
            if let LiteralKind::IntNumber(value) = literal.kind() {
                // Value can't be negative as IntNumber is unsigned
                // any sign would come from a prefix expression
                if let Some(value) = value.value() {
                    if let Ok(value) = value.try_into() {
                        let value: i64 = value;
                        let expr = build_lit_int_expr(value, span);
                        let ty = Type::Int(None, IsConst::True);
                        return Some(QasmTypedExpr { ty, expr });
                    }
                }
            }
        }
        let result = match (&lhs_ty, &rhs_ty) {
            (Type::Float(..), Type::Int(..) | Type::UInt(..)) => {
                // the qasm type is int/uint, but the expr will be q# int
                if let LiteralKind::IntNumber(value) = literal.kind() {
                    let expr = self.compile_int_to_double_literal(&value, false, span)?;
                    Some(QasmTypedExpr {
                        ty: ty.clone(),
                        expr,
                    })
                } else {
                    panic!("Literal must be an IntNumber")
                }
            }
            (Type::Float(..), Type::Float(..)) => {
                if let LiteralKind::FloatNumber(value) = literal.kind() {
                    let value = value.value().expect("FloatNumber must have a value");
                    let expr = build_lit_double_expr(value, span);
                    Some(QasmTypedExpr {
                        ty: ty.clone(),
                        expr,
                    })
                } else {
                    panic!("Literal must be a FloatNumber")
                }
            }
            (Type::Complex(..), Type::Float(..)) => {
                let expr = build_complex_from_expr(rhs.expr.clone());
                Some(QasmTypedExpr {
                    ty: ty.clone(),
                    expr,
                })
            }
            (Type::Complex(..), Type::Int(..) | Type::UInt(..)) => {
                // complex requires a double as input, so we need to
                // convert the int to a double, then create the complex
                if let LiteralKind::IntNumber(value) = literal.kind() {
                    if let Some(expr) = self.compile_int_to_double_literal_to_complex(&value, span)
                    {
                        return Some(QasmTypedExpr {
                            ty: ty.clone(),
                            expr,
                        });
                    }
                }
                panic!("Literal must be an IntNumber")
            }
            (Type::Bit(..), Type::Int(..) | Type::UInt(..)) => {
                // we've already checked that the value is 0 or 1
                if let LiteralKind::IntNumber(value) = literal.kind() {
                    let value = value.value().expect("IntNumber must have a value");
                    if value == 0 || value == 1 {
                        let expr = build_lit_result_expr((value == 1).into(), rhs.expr.span);
                        Some(QasmTypedExpr {
                            ty: ty.clone(),
                            expr,
                        })
                    } else {
                        panic!("Value must be 0 or 1");
                    }
                } else {
                    panic!("Literal must be an IntNumber");
                }
            }
            (Type::Int(..), Type::Int(..) | Type::UInt(..)) => {
                // we've already checked that this conversion can happen
                if let LiteralKind::IntNumber(value) = literal.kind() {
                    let value = value.value().expect("IntNumber must have a value");
                    let expr = if let Ok(value) = value.try_into() {
                        let value: i64 = value;
                        build_lit_int_expr(value, span)
                    } else {
                        build_lit_bigint_expr(BigInt::from(value), span)
                    };
                    Some(QasmTypedExpr {
                        ty: ty.clone(),
                        expr,
                    })
                } else {
                    panic!("Literal must be an IntNumber");
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

    fn create_entry_operation<S: AsRef<str>>(
        &mut self,
        name: S,
        whole_span: Span,
    ) -> (ast::Item, OperationSignature) {
        let stmts = self.stmts.drain(..).collect::<Vec<_>>();
        let input = self.symbols.get_input();
        let output = self.symbols.get_output();
        self.create_entry_item(
            name,
            stmts,
            input,
            output,
            whole_span,
            self.config.output_semantics,
        )
    }

    /// +----------------+-------------------------------------------------------------+
    /// | Allowed casts  | Casting To                                                  |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | Casting From   | bool  | int | uint | float | angle | bit | duration | qubit |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | angle          | Yes   | No  | No   | No    | -     | Yes | No       | No    |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    fn cast_angle_expr_to_type(
        &mut self,
        ty: &Type,
        rhs: &QasmTypedExpr,
        node: &SyntaxNode,
    ) -> Option<QasmTypedExpr> {
        assert!(matches!(rhs.ty, Type::Bit(..)));
        match ty {
            Type::Bit(..) => {
                let msg = "Cast angle to bit";
                self.push_unimplemented_error_message(msg, node);
                None
            }
            Type::Bool(..) => {
                let msg = "Cast angle to bool";
                self.push_unimplemented_error_message(msg, node);
                None
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
        rhs: &QasmTypedExpr,
        node: &SyntaxNode,
    ) -> Option<QasmTypedExpr> {
        assert!(matches!(rhs.ty, Type::Bit(..)));
        // There is no operand, choosing the span of the node
        // but we could use the expr span as well.
        let operand_span = span_for_syntax_node(node);
        let name_span = rhs.expr.span;
        match ty {
            &Type::Angle(..) => {
                let msg = "Cast bit to angle";
                self.push_unimplemented_error_message(msg, node);
                None
            }
            &Type::Bool(..) => {
                self.runtime |= RuntimeFunctions::ResultAsBool;
                Some(QasmTypedExpr {
                    ty: ty.clone(),
                    expr: build_cast_call(
                        RuntimeFunctions::ResultAsBool,
                        rhs.expr.clone(),
                        name_span,
                        operand_span,
                    ),
                })
            }
            &Type::Float(..) => {
                // The spec says that this cast isn't supported, but it
                // casts to other types that case to float. For now, we'll
                // say it is invalid like the spec.
                None
            }
            &Type::Int(w, _) | &Type::UInt(w, _) => {
                let function = if let Some(width) = w {
                    if width > 64 {
                        RuntimeFunctions::ResultAsBigInt
                    } else {
                        RuntimeFunctions::ResultAsInt
                    }
                } else {
                    RuntimeFunctions::ResultAsInt
                };
                self.runtime |= function;
                let expr = build_cast_call(function, rhs.expr.clone(), name_span, operand_span);
                Some(QasmTypedExpr {
                    ty: ty.clone(),
                    expr,
                })
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
    fn cast_float_expr_to_type(
        &mut self,
        ty: &Type,
        rhs: &QasmTypedExpr,
        node: &SyntaxNode,
    ) -> Option<QasmTypedExpr> {
        assert!(matches!(rhs.ty, Type::Float(..)));
        match ty {
            &Type::Complex(..) => {
                let expr = build_complex_from_expr(rhs.expr.clone());
                Some(QasmTypedExpr {
                    ty: ty.clone(),
                    expr,
                })
            }
            &Type::Angle(..) => {
                let msg = "Cast float to angle";
                self.push_unimplemented_error_message(msg, node);
                None
            }
            &Type::Int(w, _) | &Type::UInt(w, _) => {
                let span = span_for_syntax_node(node);
                let expr = ast_builder::build_math_call_from_exprs(
                    "Truncate",
                    vec![rhs.expr.clone()],
                    span,
                );
                let expr = if let Some(w) = w {
                    if w > 64 {
                        build_convert_call_expr(expr, "IntAsBigInt")
                    } else {
                        expr
                    }
                } else {
                    expr
                };

                Some(QasmTypedExpr {
                    ty: ty.clone(),
                    expr,
                })
            }
            &Type::Bool(..) => {
                let span = span_for_syntax_node(node);
                let expr = ast_builder::build_math_call_from_exprs(
                    "Truncate",
                    vec![rhs.expr.clone()],
                    span,
                );
                let const_int_zero_expr = build_lit_int_expr(0, rhs.expr.span);
                let qsop = ast::BinOp::Eq;
                let cond = ast_builder::build_binary_expr(
                    false,
                    qsop,
                    expr,
                    const_int_zero_expr,
                    rhs.expr.span,
                );
                let coerce_expr = build_if_expr_then_expr_else_expr(
                    cond,
                    build_lit_bool_expr(false, rhs.expr.span),
                    build_lit_bool_expr(true, rhs.expr.span),
                    rhs.expr.span,
                );
                Some(QasmTypedExpr {
                    ty: ty.clone(),
                    expr: coerce_expr,
                })
            }
            _ => None,
        }
    }

    fn create_entry_item<S: AsRef<str>>(
        &mut self,
        name: S,
        stmts: Vec<ast::Stmt>,
        input: Option<Vec<Symbol>>,
        output: Option<Vec<Symbol>>,
        whole_span: Span,
        output_semantics: OutputSemantics,
    ) -> (ast::Item, OperationSignature) {
        let mut stmts = stmts;
        let is_qiskit = matches!(output_semantics, OutputSemantics::Qiskit);
        let mut signature = OperationSignature {
            input: vec![],
            output: String::new(),
            name: name.as_ref().to_string(),
            ns: None,
        };
        let output_ty = if matches!(output_semantics, OutputSemantics::ResourceEstimation) {
            // we have no output, but need to set the entry point return type
            crate::types::Type::Tuple(vec![])
        } else if let Some(output) = output {
            let output_exprs = if is_qiskit {
                output
                    .iter()
                    .rev()
                    .filter(|symbol| matches!(symbol.ty, Type::BitArray(..)))
                    .map(|symbol| {
                        let ident =
                            build_path_ident_expr(symbol.name.as_str(), symbol.span, symbol.span);

                        build_array_reverse_expr(ident)
                    })
                    .collect::<Vec<_>>()
            } else {
                output
                    .iter()
                    .map(|symbol| {
                        build_path_ident_expr(symbol.name.as_str(), symbol.span, symbol.span)
                    })
                    .collect::<Vec<_>>()
            };
            // this is the output whether it is inferred or explicitly defined
            // map the output symbols into a return statement, add it to the nodes list,
            // and get the entry point return type
            let output_types = if is_qiskit {
                output
                    .iter()
                    .rev()
                    .filter(|symbol| matches!(symbol.ty, Type::BitArray(..)))
                    .map(|symbol| symbol.qsharp_ty.clone())
                    .collect::<Vec<_>>()
            } else {
                output
                    .iter()
                    .map(|symbol| symbol.qsharp_ty.clone())
                    .collect::<Vec<_>>()
            };

            let (output_ty, output_expr) = if output_types.len() == 1 {
                (output_types[0].clone(), output_exprs[0].clone())
            } else {
                let output_ty = crate::types::Type::Tuple(output_types);
                let output_expr = build_tuple_expr(output_exprs);
                (output_ty, output_expr)
            };

            let return_stmt = build_implicit_return_stmt(output_expr);
            stmts.push(return_stmt);
            output_ty
        } else {
            if is_qiskit {
                let kind = SemanticErrorKind::QiskitEntryPointMissingOutput(whole_span);
                self.push_semantic_error(kind);
            }
            crate::types::Type::Tuple(vec![])
        };

        let ast_ty = map_qsharp_type_to_ast_ty(&output_ty);
        signature.output = format!("{output_ty}");
        // TODO: This can create a collision on multiple compiles when interactive
        // We also have issues with the new entry point inference logic
        let input_desc = input
            .iter()
            .flat_map(|s| {
                s.iter()
                    .map(|s| (s.name.to_string(), format!("{}", s.qsharp_ty)))
            })
            .collect::<Vec<_>>();
        signature.input = input_desc;
        let input_pats = input
            .into_iter()
            .flat_map(|s| {
                s.into_iter()
                    .map(|s| build_arg_pat(s.name, s.span, map_qsharp_type_to_ast_ty(&s.qsharp_ty)))
            })
            .collect::<Vec<_>>();

        (
            build_operation_with_stmts(name, input_pats, ast_ty, stmts, whole_span),
            signature,
        )
    }

    /// +----------------+-------------------------------------------------------------+
    /// | Allowed casts  | Casting To                                                  |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | Casting From   | bool  | int | uint | float | angle | bit | duration | qubit |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | bool           | -     | Yes | Yes  | Yes   | No    | Yes | No       | No    |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    fn cast_bool_expr_to_type(&mut self, ty: &Type, rhs: &QasmTypedExpr) -> Option<QasmTypedExpr> {
        assert!(matches!(rhs.ty, Type::Bool(..)));
        let name_span = rhs.expr.span;
        let operand_span = rhs.expr.span;
        match ty {
            &Type::Bit(..) => {
                self.runtime |= RuntimeFunctions::BoolAsResult;
                let expr = build_cast_call(
                    RuntimeFunctions::BoolAsResult,
                    rhs.expr.clone(),
                    name_span,
                    operand_span,
                );
                Some(QasmTypedExpr {
                    ty: ty.clone(),
                    expr,
                })
            }
            &Type::Float(..) => {
                self.runtime |= RuntimeFunctions::BoolAsDouble;
                let expr = build_cast_call(
                    RuntimeFunctions::BoolAsDouble,
                    rhs.expr.clone(),
                    name_span,
                    operand_span,
                );
                Some(QasmTypedExpr {
                    ty: ty.clone(),
                    expr,
                })
            }
            &Type::Int(w, _) | &Type::UInt(w, _) => {
                let function = if let Some(width) = w {
                    if width > 64 {
                        RuntimeFunctions::BoolAsBigInt
                    } else {
                        RuntimeFunctions::BoolAsInt
                    }
                } else {
                    RuntimeFunctions::BoolAsInt
                };
                self.runtime |= function;
                let expr = build_cast_call(function, rhs.expr.clone(), name_span, operand_span);
                Some(QasmTypedExpr {
                    ty: ty.clone(),
                    expr,
                })
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
    fn cast_int_expr_to_type(&mut self, ty: &Type, rhs: &QasmTypedExpr) -> Option<QasmTypedExpr> {
        assert!(matches!(rhs.ty, Type::Int(..) | Type::UInt(..)));
        let name_span = rhs.expr.span;
        let operand_span = rhs.expr.span;
        match ty {
            Type::BitArray(dims, _) => {
                self.runtime |= RuntimeFunctions::IntAsResultArrayBE;
                let size = match dims {
                    ArrayDims::D1(size) => i64::try_from(*size).ok()?,
                    _ => 0,
                };

                let size_expr = build_lit_int_expr(size, Span::default());
                let expr = build_cast_call_two_params(
                    RuntimeFunctions::IntAsResultArrayBE,
                    rhs.expr.clone(),
                    size_expr,
                    name_span,
                    operand_span,
                );
                Some(QasmTypedExpr {
                    ty: ty.clone(),
                    expr,
                })
            }
            Type::Float(..) => {
                let expr = build_convert_call_expr(rhs.expr.clone(), "IntAsDouble");
                Some(QasmTypedExpr {
                    ty: ty.clone(),
                    expr,
                })
            }
            Type::Int(tw, _) | Type::UInt(tw, _) => {
                // uint to int, or int/uint to BigInt
                if let Some(tw) = tw {
                    if *tw > 64 {
                        let expr = build_convert_call_expr(rhs.expr.clone(), "IntAsBigInt");
                        Some(QasmTypedExpr {
                            ty: ty.clone(),
                            expr,
                        })
                    } else {
                        Some(QasmTypedExpr {
                            ty: ty.clone(),
                            expr: rhs.expr.clone(),
                        })
                    }
                } else {
                    Some(QasmTypedExpr {
                        ty: ty.clone(),
                        expr: rhs.expr.clone(),
                    })
                }
            }
            Type::Bool(..) => {
                let const_int_zero_expr = build_lit_int_expr(0, rhs.expr.span);
                let qsop = ast::BinOp::Eq;
                let cond = ast_builder::build_binary_expr(
                    false,
                    qsop,
                    rhs.expr.clone(),
                    const_int_zero_expr,
                    rhs.expr.span,
                );
                let coerce_expr = build_if_expr_then_expr_else_expr(
                    cond,
                    build_lit_bool_expr(false, rhs.expr.span),
                    build_lit_bool_expr(true, rhs.expr.span),
                    rhs.expr.span,
                );
                Some(QasmTypedExpr {
                    ty: ty.clone(),
                    expr: coerce_expr,
                })
            }
            Type::Bit(..) => {
                let const_int_zero_expr = build_lit_int_expr(0, rhs.expr.span);
                let qsop = ast::BinOp::Eq;
                let cond = ast_builder::build_binary_expr(
                    false,
                    qsop,
                    rhs.expr.clone(),
                    const_int_zero_expr,
                    rhs.expr.span,
                );
                let coerce_expr = build_if_expr_then_expr_else_expr(
                    cond,
                    build_lit_result_expr(ast::Result::One, rhs.expr.span),
                    build_lit_result_expr(ast::Result::Zero, rhs.expr.span),
                    rhs.expr.span,
                );
                Some(QasmTypedExpr {
                    ty: ty.clone(),
                    expr: coerce_expr,
                })
            }
            Type::Complex(..) => {
                let expr = build_convert_call_expr(rhs.expr.clone(), "IntAsDouble");
                let expr = build_complex_from_expr(expr);
                Some(QasmTypedExpr {
                    ty: ty.clone(),
                    expr,
                })
            }
            _ => None,
        }
    }

    fn cast_bitarray_expr_to_type(
        &mut self,
        dims: &ArrayDims,
        ty: &Type,
        rhs: &QasmTypedExpr,
    ) -> Option<QasmTypedExpr> {
        let ArrayDims::D1(array_width) = dims else {
            return None;
        };
        if !matches!(ty, Type::Int(..) | Type::UInt(..)) {
            return None;
        }
        // we know we have a bit array being cast to an int/uint
        // verfiy widths
        let int_width = ty.width();

        if int_width.is_none() || (int_width == Some(u32::try_from(*array_width).ok()?)) {
            let name_span = rhs.expr.span;
            let operand_span = rhs.expr.span;
            self.runtime |= RuntimeFunctions::ResultArrayAsIntBE;
            let expr = build_cast_call(
                RuntimeFunctions::ResultArrayAsIntBE,
                rhs.expr.clone(),
                name_span,
                operand_span,
            );
            Some(QasmTypedExpr {
                ty: ty.clone(),
                expr,
            })
        } else {
            None
        }
    }

    /// Pushes an unimplemented error with the supplied message.
    pub fn push_unimplemented_error_message<S: AsRef<str>>(
        &mut self,
        message: S,
        node: &SyntaxNode,
    ) {
        let span = span_for_syntax_node(node);
        let kind = crate::ErrorKind::Unimplemented(message.as_ref().to_string(), span);
        let error = self.create_err(kind);
        self.errors.push(error);
    }

    /// Pushes a missing symbol error with the given name
    /// This is a convenience method for pushing a `SemanticErrorKind::UndefinedSymbol` error.
    pub fn push_missing_symbol_error<S: AsRef<str>>(&mut self, name: S, node: &SyntaxNode) {
        let span = span_for_syntax_node(node);
        let kind = SemanticErrorKind::UndefinedSymbol(name.as_ref().to_string(), span);
        let kind = crate::ErrorKind::Semantic(SemanticError(kind));
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
        let kind = crate::ErrorKind::Semantic(SemanticError(kind));
        let error = self.create_err(kind);
        self.errors.push(error);
    }

    /// Pushes an unsupported error with the supplied message.
    pub fn push_unsupported_error_message<S: AsRef<str>>(&mut self, message: S, node: &SyntaxNode) {
        let span = span_for_syntax_node(node);
        let kind = crate::ErrorKind::NotSupported(message.as_ref().to_string(), span);
        let error = self.create_err(kind);
        self.errors.push(error);
    }

    /// Pushes an error for a gate not being supported.
    pub fn push_calibration_error(&mut self, node: &SyntaxNode) {
        let span = span_for_syntax_node(node);
        let text = node.text().to_string();
        let kind = crate::ErrorKind::CalibrationsNotSupported(text, span);
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
}

fn compile_end_stmt(end: &oq3_syntax::ast::EndStmt) -> ast::Stmt {
    ast_builder::build_end_stmt(span_for_syntax_node(end.syntax()))
}

/// This is missing bitwise negation, but it is impossible to test
/// as the parser doesn't support it.
fn binop_requires_bitwise_conversion(op: BinaryOp, left_type: &Type) -> bool {
    match op {
        BinaryOp::ArithOp(arith) => match arith {
            ArithOp::BitAnd | ArithOp::BitOr | ArithOp::BitXor => matches!(
                left_type,
                Type::Bit(..)
                    | Type::UInt(..)
                    | Type::Angle(..)
                    | Type::BitArray(ArrayDims::D1(_), _)
            ),
            ArithOp::Shl | ArithOp::Shr => matches!(
                left_type,
                Type::Bit(..)
                    | Type::UInt(..)
                    | Type::Angle(..)
                    | Type::BitArray(ArrayDims::D1(_), _)
            ),
            _ => false,
        },
        _ => false,
    }
}

fn binop_requires_bitwise_symmetric_conversion(op: BinaryOp) -> bool {
    match op {
        BinaryOp::ArithOp(arith) => {
            matches!(arith, ArithOp::BitAnd | ArithOp::BitOr | ArithOp::BitXor)
        }
        _ => false,
    }
}

fn calculate_num_ctrls(modifiers: &[&GateModifier]) -> u64 {
    let num_ctrls: u64 = modifiers
        .iter()
        .map(|m| match m {
            GateModifier::Inv(_) | GateModifier::Pow(_, _) => 0,
            GateModifier::Ctrl(ctls, _) | GateModifier::NegCtrl(ctls, _) => {
                TryInto::<u64>::try_into(ctls.unwrap_or(1))
                    .ok()
                    .unwrap_or(0)
            }
        })
        .sum();
    num_ctrls
}

fn get_implicit_modifiers<S: AsRef<str>>(
    gate_name: S,
    name_span: Span,
) -> (String, Vec<GateModifier>) {
    // ch, crx, cry, crz, sdg, and tdg
    match gate_name.as_ref() {
        "ch" => ("H".to_string(), vec![GateModifier::Ctrl(None, name_span)]),
        "crx" => ("Rx".to_string(), vec![GateModifier::Ctrl(None, name_span)]),
        "cry" => ("Ry".to_string(), vec![GateModifier::Ctrl(None, name_span)]),
        "crz" => ("Rz".to_string(), vec![GateModifier::Ctrl(None, name_span)]),
        "sdg" => ("S".to_string(), vec![GateModifier::Inv(name_span)]),
        "tdg" => ("T".to_string(), vec![GateModifier::Inv(name_span)]),
        _ => (gate_name.as_ref().to_string(), vec![]),
    }
}

/// Bit arrays can be compared, but need to be converted to int first
fn binop_requires_int_conversion_for_type(op: BinaryOp, ty_1: &Type, ty_2: &Type) -> bool {
    match op {
        BinaryOp::CmpOp(_) => match (ty_1, ty_2) {
            (Type::BitArray(ArrayDims::D1(d1), _), Type::BitArray(ArrayDims::D1(d2), _)) => {
                d1 == d2
            }
            _ => false,
        },
        _ => false,
    }
}

fn binop_requires_bool_conversion_for_type(op: BinaryOp) -> bool {
    matches!(op, BinaryOp::LogicOp(..))
}

fn compile_intnumber_as_bit(
    value: &oq3_syntax::ast::IntNumber,
    span: Span,
    ty: &Type,
) -> Option<QasmTypedExpr> {
    let value = value.value().expect("IntNumber must have a value");
    if value == 0 || value == 1 {
        let expr = build_lit_result_expr((value == 1).into(), span);
        Some(QasmTypedExpr {
            ty: ty.clone(),
            expr,
        })
    } else {
        None
    }
}

fn compile_floatnumber_as_negated_double(
    value: &oq3_syntax::ast::FloatNumber,
    span: Span,
) -> QasmTypedExpr {
    let expr = build_lit_double_expr(-value.value().expect("FloatNumber must have a value"), span);
    let ty = Type::Float(None, IsConst::True);
    QasmTypedExpr { ty, expr }
}

fn compile_intnumber_as_negated_int(
    value: &oq3_syntax::ast::IntNumber,
    span: Span,
) -> QasmTypedExpr {
    let value = value.value().expect("IntNumber must have a value");
    if let Ok(value) = value.try_into() {
        let value: i64 = value;
        let expr = build_lit_int_expr(-value, span);
        let ty = Type::Int(None, IsConst::True);
        QasmTypedExpr { ty, expr }
    } else {
        let expr = build_lit_bigint_expr(BigInt::from(-1) * BigInt::from(value), span);
        let ty = Type::Int(Some(128), IsConst::True);
        QasmTypedExpr { ty, expr }
    }
}

/// +----------------+-------------------------------------------------------------+
/// | Allowed casts  | Casting To                                                  |
/// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
/// | Casting From   | bool  | int | uint | float | angle | bit | duration | qubit |
/// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
/// | complex        | ??    | ??  | ??   | ??    | No    | ??  | No       | No    |
/// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
fn cast_complex_expr_to_type(ty: &Type, rhs: &QasmTypedExpr) -> Option<QasmTypedExpr> {
    assert!(matches!(rhs.ty, Type::Complex(..)));

    if matches!((ty, &rhs.ty), (Type::Complex(..), Type::Complex(..))) {
        // we are both complex, but our widths are different. If both
        // had implicit widths, we would have already matched for the
        // (None, None). If the rhs width is bigger, we will return
        // None to let the cast fail

        // Here, we can safely cast the rhs to the lhs type if the
        // lhs width can hold the rhs's width
        if ty.width().is_none() && rhs.ty.width().is_some() {
            return Some(QasmTypedExpr {
                ty: ty.clone(),
                expr: rhs.expr.clone(),
            });
        }
        if ty.width() >= rhs.ty.width() {
            return Some(QasmTypedExpr {
                ty: ty.clone(),
                expr: rhs.expr.clone(),
            });
        }
    }
    None
}

fn try_promote_with_casting(left_type: &Type, right_type: &Type) -> Type {
    let promoted_type = promote_types(left_type, right_type);

    if promoted_type != Type::Void {
        return promoted_type;
    }
    if let Some(value) = try_promote_bitarray_to_int(left_type, right_type) {
        return value;
    }
    // simple promotion failed, try a lossless cast
    // each side to double
    let promoted_rhs = promote_types(&Type::Float(None, IsConst::False), right_type);
    let promoted_lhs = promote_types(left_type, &Type::Float(None, IsConst::False));

    match (promoted_lhs, promoted_rhs) {
        (Type::Void, Type::Void) => Type::Float(None, IsConst::False),
        (Type::Void, promoted_rhs) => promoted_rhs,
        (promoted_lhs, Type::Void) => promoted_lhs,
        (promoted_lhs, promoted_rhs) => {
            // return the greater of the two promoted types
            if matches!(promoted_lhs, Type::Complex(..)) {
                promoted_lhs
            } else if matches!(promoted_rhs, Type::Complex(..)) {
                promoted_rhs
            } else if matches!(promoted_lhs, Type::Float(..)) {
                promoted_lhs
            } else if matches!(promoted_rhs, Type::Float(..)) {
                promoted_rhs
            } else {
                Type::Float(None, IsConst::False)
            }
        }
    }
}

fn try_promote_bitarray_to_int(left_type: &Type, right_type: &Type) -> Option<Type> {
    if matches!(
        (left_type, right_type),
        (Type::Int(..) | Type::UInt(..), Type::BitArray(..))
    ) {
        let ty = left_type;
        let r = right_type.dims().expect("")[0];

        if ty.dims().is_none() || (ty.num_dims() == 1 && ty.dims().expect("")[0] == r) {
            return Some(left_type.clone());
        }
    }
    if matches!(
        (left_type, right_type),
        (Type::BitArray(..), Type::Int(..) | Type::UInt(..))
    ) {
        let ty = right_type;
        let r = left_type.dims().expect("")[0];

        if ty.dims().is_none() || (ty.num_dims() == 1 && ty.dims().expect("")[0] == r) {
            return Some(right_type.clone());
        }
    }
    None
}

fn compile_bitstring(bitstring: &BitString, span: Span) -> Option<QasmTypedExpr> {
    let width = bitstring
        .to_string()
        .chars()
        .filter(|c| *c == '0' || *c == '1')
        .count();
    let expr = bitstring
        .value()
        .map(|value| build_lit_result_array_expr_from_bitstring(value, span))?;
    let ty = Type::BitArray(ArrayDims::D1(width), IsConst::True);
    Some(QasmTypedExpr { ty, expr })
}

pub fn can_cast_literal(lhs_ty: &Type, ty_lit: &Type) -> bool {
    if matches!(lhs_ty, Type::Int(..)) && matches!(ty_lit, Type::UInt(..)) {
        return true;
    }
    if matches!(lhs_ty, Type::UInt(..)) {
        return matches!(ty_lit, Type::Complex(..));
    }
    oq3_semantics::types::can_cast_literal(lhs_ty, ty_lit)
        || {
            matches!(lhs_ty, Type::Bit(..) | Type::Bool(..))
                && matches!(ty_lit, Type::Bit(..) | Type::Bool(..))
        }
        || {
            match lhs_ty {
                Type::BitArray(dims, _) => {
                    matches!(dims, ArrayDims::D1(_))
                        && matches!(ty_lit, Type::Int(..) | Type::UInt(..))
                }
                _ => false,
            }
        }
}

fn extract_pow_exponent(expr: &oq3_syntax::ast::ParenExpr, span: Span) -> GateModifier {
    let lit = compile_paren_lit_int_expr(expr);
    if let Some((exponent, sign)) = lit {
        let exponent = i64::try_from(exponent).ok();
        let Some(exponent) = exponent else {
            return GateModifier::Pow(None, span);
        };
        if sign {
            GateModifier::Pow(Some(-exponent), span)
        } else {
            GateModifier::Pow(Some(exponent), span)
        }
    } else {
        GateModifier::Pow(None, span)
    }
}

fn compile_paren_lit_int_expr(paren_expr: &oq3_syntax::ast::ParenExpr) -> Option<(usize, bool)> {
    let expr = paren_expr.expr()?;
    match expr {
        Expr::Literal(lit) => match lit.kind() {
            LiteralKind::IntNumber(value) => {
                let size: usize = usize::try_from(value.value()?).ok()?;
                Some((size, false))
            }
            _ => None,
        },
        Expr::PrefixExpr(prefix) => match prefix.op_kind() {
            Some(UnaryOp::Neg) => {
                let expr = prefix.expr()?;
                match expr {
                    Expr::Literal(lit) => match lit.kind() {
                        LiteralKind::IntNumber(value) => {
                            let size: usize = usize::try_from(value.value()?).ok()?;
                            Some((size, true))
                        }
                        _ => None,
                    },
                    _ => None,
                }
            }
            _ => None,
        },
        _ => None,
    }
}
