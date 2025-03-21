// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{path::Path, rc::Rc, sync::Arc};

use num_bigint::BigInt;
use qsc_data_structures::span::Span;
use qsc_frontend::{compile::SourceMap, error::WithSource};

use crate::{
    ast_builder::{
        build_arg_pat, build_array_reverse_expr, build_assignment_statement, build_barrier_call,
        build_binary_expr, build_cast_call, build_cast_call_two_params, build_classical_decl,
        build_complex_from_expr, build_convert_call_expr, build_expr_array_expr,
        build_gate_call_param_expr, build_gate_call_with_params_and_callee,
        build_if_expr_then_expr_else_expr, build_implicit_return_stmt,
        build_indexed_assignment_statement, build_lit_bigint_expr, build_lit_bool_expr,
        build_lit_complex_expr, build_lit_double_expr, build_lit_int_expr,
        build_lit_result_array_expr_from_bitstring, build_lit_result_expr,
        build_managed_qubit_alloc, build_math_call_from_exprs, build_math_call_no_params,
        build_measure_call, build_operation_with_stmts, build_path_ident_expr, build_reset_call,
        build_stmt_semi_from_expr, build_stmt_semi_from_expr_with_span,
        build_top_level_ns_with_item, build_tuple_expr, build_unary_op_expr,
        build_unmanaged_qubit_alloc, build_unmanaged_qubit_alloc_array, build_while_stmt,
        build_wrapped_block_expr, managed_qubit_alloc_array, map_qsharp_type_to_ast_ty,
        wrap_expr_in_parens,
    },
    io::{InMemorySourceResolver, SourceResolver},
    parser::ast::{list_from_iter, List},
    runtime::{get_runtime_function_decls, RuntimeFunctions},
    semantic::{
        ast::{
            BinaryOpExpr, Cast, DiscreteSet, Expr, FunctionCall, GateOperand, GateOperandKind,
            IndexElement, IndexExpr, IndexSet, IndexedIdent, LiteralKind, MeasureExpr, TimeUnit,
            UnaryOpExpr,
        },
        symbols::{IOKind, Symbol, SymbolId, SymbolTable},
        types::{ArrayDimensions, Type},
        SemanticErrorKind,
    },
    CompilerConfig, OperationSignature, OutputSemantics, ProgramType, QasmCompileUnit,
    QubitSemantics,
};

use crate::semantic::ast as semast;
use qsc_ast::ast::{self as qsast, NodeId, Package};

pub fn compile_anon_with_config<S>(
    source: S,
    config: CompilerConfig,
) -> miette::Result<QasmCompileUnit>
where
    S: AsRef<str>,
{
    let path = std::path::PathBuf::from("Test.qasm");
    let sources = [(
        Arc::from(path.display().to_string().as_str()),
        Arc::from(source.as_ref()),
    )];
    let resolver = InMemorySourceResolver::from_iter(sources);
    let source = resolver.resolve(&path)?.1;
    compile_with_config(source, &path, &resolver, config)
}

pub fn compile_all_with_config<P>(
    path: P,
    sources: impl IntoIterator<Item = (Arc<str>, Arc<str>)>,
    config: CompilerConfig,
) -> miette::Result<QasmCompileUnit>
where
    P: AsRef<Path>,
{
    let resolver = InMemorySourceResolver::from_iter(sources);
    let source = resolver.resolve(path.as_ref())?.1;
    compile_with_config(source, path, &resolver, config)
}

pub fn compile_with_config<S, P, R>(
    source: S,
    path: P,
    resolver: &R,
    config: CompilerConfig,
) -> miette::Result<QasmCompileUnit>
where
    S: AsRef<str>,
    P: AsRef<Path>,
    R: SourceResolver,
{
    let res = crate::semantic::parse_source(source, path, resolver)?;
    let program = res.program;

    let compiler = crate::compiler::QasmCompiler {
        source_map: res.source_map,
        config,
        stmts: vec![],
        runtime: RuntimeFunctions::empty(),
        symbols: res.symbols,
        errors: res.errors,
    };

    Ok(compiler.compile(&program))
}

pub struct QasmCompiler {
    /// The source map of QASM sources for error reporting.
    pub source_map: SourceMap,
    /// The configuration for the compiler.
    /// This includes the qubit semantics to follow when compiling to Q# AST.
    /// The output semantics to follow when compiling to Q# AST.
    /// The program type to compile to.
    pub config: CompilerConfig,
    /// The compiled statments accumulated during compilation.
    pub stmts: Vec<qsast::Stmt>,
    /// The runtime functions that need to be included at the end of
    /// compilation
    pub runtime: RuntimeFunctions,
    pub symbols: SymbolTable,
    pub errors: Vec<WithSource<crate::Error>>,
}

impl QasmCompiler {
    /// The main entry into compilation. This function will compile the
    /// source file and build the appropriate package based on the
    /// configuration.
    pub fn compile(mut self, program: &crate::semantic::ast::Program) -> QasmCompileUnit {
        self.compile_stmts(&program.statements);
        self.prepend_runtime_decls();
        let program_ty = self.config.program_ty.clone();
        let (package, signature) = match program_ty {
            ProgramType::File => self.build_file(),
            ProgramType::Operation => self.build_operation(),
            ProgramType::Fragments => (self.build_fragments(), None),
        };

        QasmCompileUnit::new(self.source_map, self.errors, Some(package), signature)
    }

    /// Build a package with namespace and an operation
    /// containing the compiled statements.
    fn build_file(&mut self) -> (Package, Option<OperationSignature>) {
        let whole_span = Span::default();
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
    fn build_operation(&mut self) -> (qsast::Package, Option<OperationSignature>) {
        let whole_span = Span::default();
        let operation_name = self.config.operation_name();
        let (operation, signature) = self.create_entry_operation(operation_name, whole_span);
        (
            Package {
                nodes: Box::new([qsast::TopLevelNode::Stmt(Box::new(qsast::Stmt {
                    kind: Box::new(qsast::StmtKind::Item(Box::new(operation))),
                    span: whole_span,
                    id: qsast::NodeId::default(),
                }))]),
                ..Default::default()
            },
            Some(signature),
        )
    }

    /// Turns the compiled statements into package of top level nodes
    fn build_fragments(&mut self) -> qsast::Package {
        let nodes = self
            .stmts
            .drain(..)
            .map(Box::new)
            .map(qsast::TopLevelNode::Stmt)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        qsast::Package {
            nodes,
            ..Default::default()
        }
    }

    fn create_entry_operation<S: AsRef<str>>(
        &mut self,
        name: S,
        whole_span: Span,
    ) -> (qsast::Item, OperationSignature) {
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

    fn create_entry_item<S: AsRef<str>>(
        &mut self,
        name: S,
        stmts: Vec<qsast::Stmt>,
        input: Option<Vec<Rc<Symbol>>>,
        output: Option<Vec<Rc<Symbol>>>,
        whole_span: Span,
        output_semantics: OutputSemantics,
    ) -> (qsast::Item, OperationSignature) {
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
                    .filter(|symbol| {
                        matches!(symbol.ty, crate::semantic::types::Type::BitArray(..))
                    })
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
                    .filter(|symbol| {
                        matches!(symbol.ty, crate::semantic::types::Type::BitArray(..))
                    })
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
                s.into_iter().map(|s| {
                    build_arg_pat(
                        s.name.clone(),
                        s.span,
                        map_qsharp_type_to_ast_ty(&s.qsharp_ty),
                    )
                })
            })
            .collect::<Vec<_>>();

        (
            build_operation_with_stmts(name, input_pats, ast_ty, stmts, whole_span),
            signature,
        )
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

    fn compile_stmts(&mut self, smtms: &[Box<crate::semantic::ast::Stmt>]) {
        for stmt in smtms {
            let compiled_stmt = self.compile_stmt(stmt.as_ref());
            if let Some(stmt) = compiled_stmt {
                self.stmts.push(stmt);
            }
        }
    }

    fn compile_stmt(&mut self, stmt: &crate::semantic::ast::Stmt) -> Option<qsast::Stmt> {
        match stmt.kind.as_ref() {
            semast::StmtKind::Alias(stmt) => self.compile_alias_decl_stmt(stmt),
            semast::StmtKind::Assign(stmt) => self.compile_assign_stmt(stmt),
            semast::StmtKind::IndexedAssign(stmt) => self.compile_indexed_assign_stmt(stmt),
            semast::StmtKind::AssignOp(stmt) => self.compile_assign_op_stmt(stmt),
            semast::StmtKind::Barrier(stmt) => self.compile_barrier_stmt(stmt),
            semast::StmtKind::Box(stmt) => self.compile_box_stmt(stmt),
            semast::StmtKind::Block(stmt) => self.compile_block_stmt(stmt),
            semast::StmtKind::CalibrationGrammar(stmt) => {
                self.compile_calibration_grammar_stmt(stmt)
            }
            semast::StmtKind::ClassicalDecl(stmt) => self.compile_classical_decl(stmt),
            semast::StmtKind::Def(stmt) => self.compile_def_stmt(stmt),
            semast::StmtKind::DefCal(stmt) => self.compile_def_cal_stmt(stmt),
            semast::StmtKind::Delay(stmt) => self.compile_delay_stmt(stmt),
            semast::StmtKind::End(stmt) => self.compile_end_stmt(stmt),
            semast::StmtKind::ExprStmt(stmt) => self.compile_expr_stmt(stmt),
            semast::StmtKind::ExternDecl(stmt) => self.compile_extern_stmt(stmt),
            semast::StmtKind::For(stmt) => self.compile_for_stmt(stmt),
            semast::StmtKind::If(stmt) => self.compile_if_stmt(stmt),
            semast::StmtKind::GateCall(stmt) => self.compile_gate_call_stmt(stmt),
            semast::StmtKind::GPhase(stmt) => self.compile_gphase_stmt(stmt),
            semast::StmtKind::Include(stmt) => self.compile_include_stmt(stmt),
            semast::StmtKind::InputDeclaration(stmt) => self.compile_input_decl_stmt(stmt),
            semast::StmtKind::OutputDeclaration(stmt) => self.compile_output_decl_stmt(stmt),
            semast::StmtKind::MeasureArrow(stmt) => self.compile_measure_stmt(stmt),
            semast::StmtKind::Pragma(stmt) => self.compile_pragma_stmt(stmt),
            semast::StmtKind::QuantumGateDefinition(stmt) => self.compile_gate_decl_stmt(stmt),
            semast::StmtKind::QubitDecl(stmt) => self.compile_qubit_decl_stmt(stmt),
            semast::StmtKind::QubitArrayDecl(stmt) => self.compile_qubit_array_decl_stmt(stmt),
            semast::StmtKind::Reset(stmt) => self.compile_reset_stmt(stmt),
            semast::StmtKind::Return(stmt) => self.compile_return_stmt(stmt),
            semast::StmtKind::Switch(stmt) => self.compile_switch_stmt(stmt),
            semast::StmtKind::WhileLoop(stmt) => self.compile_while_stmt(stmt),
            semast::StmtKind::Err => {
                // todo: determine if we should push an error here
                // Are we going to allow trying to compile a program with semantic errors?
                None
            }
        }
    }

    fn compile_alias_decl_stmt(&mut self, stmt: &semast::AliasDeclStmt) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("alias statements", stmt.span);
        None
    }

    fn compile_assign_stmt(&mut self, stmt: &semast::AssignStmt) -> Option<qsast::Stmt> {
        let symbol = self.symbols[stmt.symbol_id].clone();
        let name = &symbol.name;

        let stmt_span = stmt.span;
        let name_span = stmt.name_span;

        let rhs = self.compile_expr(&stmt.rhs)?;
        let stmt = build_assignment_statement(name_span, name, rhs, stmt_span);

        Some(stmt)
    }

    fn compile_indexed_assign_stmt(
        &mut self,
        stmt: &semast::IndexedAssignStmt,
    ) -> Option<qsast::Stmt> {
        let symbol = self.symbols[stmt.symbol_id].clone();

        let indices: Vec<_> = stmt
            .indices
            .iter()
            .filter_map(|elem| self.compile_index_element(elem))
            .collect();

        let rhs = self.compile_expr(&stmt.rhs);

        if stmt.indices.len() != 1 {
            self.push_unimplemented_error_message(
                "multi-dimensional array index expressions",
                stmt.span,
            );
            return None;
        }

        if indices.len() != stmt.indices.len() {
            return None;
        }

        // Use the `?` operator after compiling checking all other errors.
        let (rhs, index_expr) = (rhs?, indices[0].clone());

        let stmt = build_indexed_assignment_statement(
            symbol.span,
            symbol.name.clone(),
            index_expr,
            rhs,
            stmt.span,
        );

        Some(stmt)
    }

    fn compile_assign_op_stmt(&mut self, stmt: &semast::AssignOpStmt) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("assignment op statements", stmt.span);
        None
    }

    fn compile_barrier_stmt(&mut self, stmt: &semast::BarrierStmt) -> Option<qsast::Stmt> {
        let qubits: Vec<_> = stmt
            .qubits
            .iter()
            .filter_map(|q| self.compile_gate_operand(q))
            .collect();

        if stmt.qubits.len() != qubits.len() {
            // if any of the qubit arguments failed to compile we can't proceed.
            // This can happen if the qubit is not defined.
            return None;
        }

        self.runtime.insert(RuntimeFunctions::Barrier);
        Some(build_barrier_call(stmt.span))
    }

    fn compile_box_stmt(&mut self, stmt: &semast::BoxStmt) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("box statements", stmt.span);
        None
    }

    fn compile_block(&mut self, block: &semast::Block) -> qsast::Block {
        let stmts = block
            .stmts
            .iter()
            .filter_map(|stmt| self.compile_stmt(stmt))
            .collect::<Vec<_>>();
        qsast::Block {
            id: qsast::NodeId::default(),
            stmts: list_from_iter(stmts),
            span: block.span,
        }
    }

    fn compile_block_stmt(&mut self, block: &semast::Block) -> Option<qsast::Stmt> {
        let block = self.compile_block(block);
        Some(build_stmt_semi_from_expr(build_wrapped_block_expr(block)))
    }

    fn compile_calibration_grammar_stmt(
        &mut self,
        stmt: &semast::CalibrationGrammarStmt,
    ) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("calibration grammar statements", stmt.span);
        None
    }

    fn compile_classical_decl(
        &mut self,
        decl: &semast::ClassicalDeclarationStmt,
    ) -> Option<qsast::Stmt> {
        let symbol = &self.symbols[decl.symbol_id].clone();
        let name = &symbol.name;
        let is_const = symbol.ty.is_const();
        let ty_span = decl.ty_span;
        let decl_span = decl.span;
        let name_span = symbol.span;
        let qsharp_ty = &symbol.qsharp_ty;
        let expr = decl.init_expr.as_ref();

        let expr = self.compile_expr(expr)?;
        let stmt = build_classical_decl(
            name, is_const, ty_span, decl_span, name_span, qsharp_ty, expr,
        );

        Some(stmt)
    }

    fn compile_def_stmt(&mut self, stmt: &semast::DefStmt) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("def statements", stmt.span);
        None
    }

    fn compile_def_cal_stmt(&mut self, stmt: &semast::DefCalStmt) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("def cal statements", stmt.span);
        None
    }

    fn compile_delay_stmt(&mut self, stmt: &semast::DelayStmt) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("delay statements", stmt.span);
        None
    }

    fn compile_end_stmt(&mut self, stmt: &semast::EndStmt) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("end statements", stmt.span);
        None
    }

    fn compile_expr_stmt(&mut self, stmt: &semast::ExprStmt) -> Option<qsast::Stmt> {
        let expr = self.compile_expr(&stmt.expr)?;
        Some(build_stmt_semi_from_expr_with_span(expr, stmt.span))
    }

    fn compile_extern_stmt(&mut self, stmt: &semast::ExternDecl) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("extern statements", stmt.span);
        None
    }

    fn compile_for_stmt(&mut self, stmt: &semast::ForStmt) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("for statements", stmt.span);
        None
    }

    fn compile_if_stmt(&mut self, stmt: &semast::IfStmt) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("if statements", stmt.span);
        None
    }

    fn compile_gate_call_stmt(&mut self, stmt: &semast::GateCall) -> Option<qsast::Stmt> {
        let symbol = self.symbols[stmt.symbol_id].clone();
        let mut qubits: Vec<_> = stmt
            .qubits
            .iter()
            .filter_map(|q| self.compile_gate_operand(q))
            .collect();
        let args: Vec<_> = stmt
            .args
            .iter()
            .filter_map(|arg| self.compile_expr(arg))
            .collect();

        if qubits.len() != stmt.qubits.len() || args.len() != stmt.args.len() {
            return None;
        }

        // Take the number of qubit args that the gates expects from the source qubits.
        let gate_qubits = qubits.split_off(qubits.len() - stmt.quantum_arity as usize);
        // Then merge the classical args with the qubit args. This will give
        // us the args for the call prior to wrapping in tuples for controls.
        let args: Vec<_> = args.into_iter().chain(gate_qubits).collect();
        let mut args = build_gate_call_param_expr(args, qubits.len());
        let mut callee = build_path_ident_expr(&symbol.name, symbol.span, stmt.span);

        for modifier in &stmt.modifiers {
            match &modifier.kind {
                semast::GateModifierKind::Inv => {
                    callee = build_unary_op_expr(
                        qsast::UnOp::Functor(qsast::Functor::Adj),
                        callee,
                        modifier.span,
                    );
                }
                semast::GateModifierKind::Pow(expr) => {
                    let exponent_expr = self.compile_expr(expr)?;
                    self.runtime |= RuntimeFunctions::Pow;
                    args = build_tuple_expr(vec![exponent_expr, callee, args]);
                    callee = build_path_ident_expr("__Pow__", modifier.span, stmt.span);
                }
                semast::GateModifierKind::Ctrl(num_ctrls) => {
                    // remove the last n qubits from the qubit list
                    if qubits.len() < *num_ctrls as usize {
                        let kind = SemanticErrorKind::InvalidNumberOfQubitArgs(
                            *num_ctrls as usize,
                            qubits.len(),
                            modifier.span,
                        );
                        self.push_semantic_error(kind);
                        return None;
                    }
                    let ctrl = qubits.split_off(qubits.len() - *num_ctrls as usize);
                    let ctrls = build_expr_array_expr(ctrl, modifier.span);
                    args = build_tuple_expr(vec![ctrls, args]);
                    callee = build_unary_op_expr(
                        qsast::UnOp::Functor(qsast::Functor::Ctl),
                        callee,
                        modifier.span,
                    );
                }
                semast::GateModifierKind::NegCtrl(num_ctrls) => {
                    // remove the last n qubits from the qubit list
                    if qubits.len() < *num_ctrls as usize {
                        let kind = SemanticErrorKind::InvalidNumberOfQubitArgs(
                            *num_ctrls as usize,
                            qubits.len(),
                            modifier.span,
                        );
                        self.push_semantic_error(kind);
                        return None;
                    }
                    let ctrl = qubits.split_off(qubits.len() - *num_ctrls as usize);
                    let ctrls = build_expr_array_expr(ctrl, modifier.span);
                    let lit_0 = build_lit_int_expr(0, Span::default());
                    args = build_tuple_expr(vec![lit_0, callee, ctrls, args]);
                    callee =
                        build_path_ident_expr("ApplyControlledOnInt", modifier.span, stmt.span);
                }
            }
        }

        // This should never be reached, since semantic analysis during lowering
        // makes sure the arities match.
        if !qubits.is_empty() {
            return None;
        }

        let expr = build_gate_call_with_params_and_callee(args, callee, stmt.span);
        Some(build_stmt_semi_from_expr(expr))
    }

    fn compile_gphase_stmt(&mut self, stmt: &semast::GPhase) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("gphase statements", stmt.span);
        None
    }

    fn compile_include_stmt(&mut self, stmt: &semast::IncludeStmt) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("include statements", stmt.span);
        None
    }

    #[allow(clippy::unused_self)]
    fn compile_input_decl_stmt(&mut self, _stmt: &semast::InputDeclaration) -> Option<qsast::Stmt> {
        None
    }

    fn compile_output_decl_stmt(
        &mut self,
        stmt: &semast::OutputDeclaration,
    ) -> Option<qsast::Stmt> {
        let symbol = &self.symbols[stmt.symbol_id];

        // input decls should have been pushed to symbol table,
        // but should not be the stmts list.
        // TODO: This may be an issue for tooling as there isn't a way to have a forward
        // declared varible in Q#.
        if symbol.io_kind != IOKind::Output {
            //self.push_semantic_error(SemanticErrorKind::InvalidIODeclaration(stmt.span));
            return None;
        }

        let symbol = symbol.clone();
        let name = &symbol.name;
        let is_const = symbol.ty.is_const();
        let ty_span = stmt.ty_span; // todo
        let decl_span = stmt.span;
        let name_span = symbol.span;
        let qsharp_ty = &symbol.qsharp_ty;

        let expr = stmt.init_expr.as_ref();

        let expr = self.compile_expr(expr)?;
        let stmt = build_classical_decl(
            name, is_const, ty_span, decl_span, name_span, qsharp_ty, expr,
        );

        Some(stmt)
    }

    fn compile_measure_stmt(&mut self, stmt: &semast::MeasureArrowStmt) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("measure statements", stmt.span);
        None
    }

    fn compile_pragma_stmt(&mut self, stmt: &semast::Pragma) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("pragma statements", stmt.span);
        None
    }

    fn compile_gate_decl_stmt(
        &mut self,
        stmt: &semast::QuantumGateDefinition,
    ) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("gate decl statements", stmt.span);
        None
    }

    fn compile_qubit_decl_stmt(&mut self, stmt: &semast::QubitDeclaration) -> Option<qsast::Stmt> {
        let symbol = self.symbols[stmt.symbol_id].clone();
        let name = &symbol.name;
        let name_span = symbol.span;

        let stmt = match self.config.qubit_semantics {
            QubitSemantics::QSharp => build_managed_qubit_alloc(name, stmt.span, name_span),
            QubitSemantics::Qiskit => build_unmanaged_qubit_alloc(name, stmt.span, name_span),
        };
        Some(stmt)
    }

    fn compile_qubit_array_decl_stmt(
        &mut self,
        stmt: &semast::QubitArrayDeclaration,
    ) -> Option<qsast::Stmt> {
        let symbol = self.symbols[stmt.symbol_id].clone();
        let name = &symbol.name;
        let name_span = symbol.span;

        let stmt = match self.config.qubit_semantics {
            QubitSemantics::QSharp => {
                managed_qubit_alloc_array(name, stmt.size, stmt.span, name_span, stmt.size_span)
            }
            QubitSemantics::Qiskit => build_unmanaged_qubit_alloc_array(
                name,
                stmt.size,
                stmt.span,
                name_span,
                stmt.size_span,
            ),
        };
        Some(stmt)
    }

    fn compile_reset_stmt(&mut self, stmt: &semast::ResetStmt) -> Option<qsast::Stmt> {
        let operand = self.compile_gate_operand(&stmt.operand)?;
        let operand_span = operand.span;
        let expr = build_reset_call(operand, stmt.reset_token_span, operand_span);
        Some(build_stmt_semi_from_expr(expr))
    }

    fn compile_return_stmt(&mut self, stmt: &semast::ReturnStmt) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("return statements", stmt.span);
        None
    }

    fn compile_switch_stmt(&mut self, stmt: &semast::SwitchStmt) -> Option<qsast::Stmt> {
        self.push_unimplemented_error_message("switch statements", stmt.span);
        None
    }

    fn compile_while_stmt(&mut self, stmt: &semast::WhileLoop) -> Option<qsast::Stmt> {
        let condition = self.compile_expr(&stmt.condition)?;
        match &*stmt.body.kind {
            semast::StmtKind::Block(block) => {
                let block = self.compile_block(block);
                Some(build_while_stmt(condition, block, stmt.span))
            }
            semast::StmtKind::Err => Some(qsast::Stmt {
                id: NodeId::default(),
                span: stmt.body.span,
                kind: Box::new(qsast::StmtKind::Err),
            }),
            _ => {
                let block_stmt = self.compile_stmt(&stmt.body)?;
                let block = qsast::Block {
                    id: qsast::NodeId::default(),
                    stmts: list_from_iter([block_stmt]),
                    span: stmt.span,
                };
                Some(build_while_stmt(condition, block, stmt.span))
            }
        }
    }

    fn compile_expr(&mut self, expr: &semast::Expr) -> Option<qsast::Expr> {
        match expr.kind.as_ref() {
            semast::ExprKind::Err => {
                // todo: determine if we should push an error here
                // Are we going to allow trying to compile a program with semantic errors?
                None
            }
            semast::ExprKind::Ident(symbol_id) => self.compile_ident_expr(*symbol_id),
            semast::ExprKind::IndexedIdentifier(indexed_ident) => {
                self.compile_indexed_ident_expr(indexed_ident)
            }
            semast::ExprKind::UnaryOp(unary_op_expr) => self.compile_unary_op_expr(unary_op_expr),
            semast::ExprKind::BinaryOp(binary_op_expr) => {
                self.compile_binary_op_expr(binary_op_expr)
            }
            semast::ExprKind::Lit(literal_kind) => {
                self.compile_literal_expr(literal_kind, expr.span)
            }
            semast::ExprKind::FunctionCall(function_call) => self.compile_call_expr(function_call),
            semast::ExprKind::Cast(cast) => self.compile_cast_expr(cast),
            semast::ExprKind::IndexExpr(index_expr) => self.compile_index_expr(index_expr),
            semast::ExprKind::Paren(pexpr) => self.compile_paren_expr(pexpr, expr.span),
            semast::ExprKind::Measure(expr) => self.compile_measure_expr(expr),
        }
    }

    fn compile_ident_expr(&mut self, symbol_id: SymbolId) -> Option<qsast::Expr> {
        let symbol = &self.symbols[symbol_id];
        let span = symbol.span;
        let expr = match symbol.name.as_str() {
            "euler" | "ℇ" => build_math_call_no_params("E", span),
            "pi" | "π" => build_math_call_no_params("PI", span),
            "tau" | "τ" => {
                let expr = build_math_call_no_params("PI", span);
                qsast::Expr {
                    kind: Box::new(qsast::ExprKind::BinOp(
                        qsast::BinOp::Mul,
                        Box::new(build_lit_double_expr(2.0, span)),
                        Box::new(expr),
                    )),
                    span,
                    id: qsast::NodeId::default(),
                }
            }
            _ => build_path_ident_expr(&symbol.name, span, span),
        };
        Some(expr)
    }

    /// The lowerer eliminated indexed identifiers with zero indices.
    /// So we are safe to assume that the indices are non-empty.
    fn compile_indexed_ident_expr(&mut self, indexed_ident: &IndexedIdent) -> Option<qsast::Expr> {
        let span = indexed_ident.span;
        let index: Vec<_> = indexed_ident
            .indices
            .iter()
            .filter_map(|elem| self.compile_index_element(elem))
            .collect();

        if index.len() != 1 {
            self.push_unimplemented_error_message(
                "multi-dimensional array index expressions",
                span,
            );
            return None;
        }

        let symbol = &self.symbols[indexed_ident.symbol_id];

        let ident =
            build_path_ident_expr(&symbol.name, indexed_ident.name_span, indexed_ident.span);
        let expr = qsast::Expr {
            id: qsast::NodeId::default(),
            span,
            kind: Box::new(qsast::ExprKind::Index(
                Box::new(ident),
                Box::new(index[0].clone()),
            )),
        };
        Some(expr)
    }

    fn compile_unary_op_expr(&mut self, unary: &UnaryOpExpr) -> Option<qsast::Expr> {
        match unary.op {
            semast::UnaryOp::Neg => self.compile_neg_expr(&unary.expr, unary.span),
            semast::UnaryOp::NotB => self.compile_bitwise_not_expr(&unary.expr, unary.span),
            semast::UnaryOp::NotL => self.compile_logical_not_expr(&unary.expr, unary.span),
        }
    }
    fn compile_neg_expr(&mut self, expr: &Expr, span: Span) -> Option<qsast::Expr> {
        let expr = self.compile_expr(expr)?;
        Some(build_unary_op_expr(qsast::UnOp::Neg, expr, span))
    }

    fn compile_bitwise_not_expr(&mut self, expr: &Expr, span: Span) -> Option<qsast::Expr> {
        let expr = self.compile_expr(expr)?;
        Some(build_unary_op_expr(qsast::UnOp::NotB, expr, span))
    }

    fn compile_logical_not_expr(&mut self, expr: &Expr, span: Span) -> Option<qsast::Expr> {
        let expr = self.compile_expr(expr)?;
        Some(build_unary_op_expr(qsast::UnOp::NotL, expr, span))
    }

    fn compile_binary_op_expr(&mut self, binary: &BinaryOpExpr) -> Option<qsast::Expr> {
        let lhs = self.compile_expr(&binary.lhs);
        let rhs = self.compile_expr(&binary.rhs);
        let (lhs, rhs) = (lhs?, rhs?);
        let op = Self::map_bin_op(binary.op);
        let is_assignment = false;
        Some(build_binary_expr(
            is_assignment,
            op,
            lhs,
            rhs,
            binary.span(),
        ))
    }

    fn compile_literal_expr(&mut self, lit: &LiteralKind, span: Span) -> Option<qsast::Expr> {
        match lit {
            LiteralKind::Array(value) => self.compile_array_literal(value, span),
            LiteralKind::Bitstring(big_int, width) => {
                Self::compile_bitstring_literal(big_int, *width, span)
            }
            LiteralKind::Bit(value) => Self::compile_bit_literal(*value, span),
            LiteralKind::Bool(value) => Self::compile_bool_literal(*value, span),
            LiteralKind::Duration(value, time_unit) => {
                self.compile_duration_literal(*value, *time_unit, span)
            }
            LiteralKind::Float(value) => Self::compile_float_literal(*value, span),
            LiteralKind::Complex(real, imag) => Self::compile_complex_literal(*real, *imag, span),
            LiteralKind::Int(value) => Self::compile_int_literal(*value, span),
            LiteralKind::BigInt(value) => Self::compile_bigint_literal(value, span),
            LiteralKind::String(value) => self.compile_string_literal(value, span),
        }
    }

    fn compile_call_expr(&mut self, call: &FunctionCall) -> Option<qsast::Expr> {
        self.push_unimplemented_error_message("function call expresssions", call.span);
        None
    }

    fn compile_cast_expr(&mut self, cast: &Cast) -> Option<qsast::Expr> {
        let expr = self.compile_expr(&cast.expr)?;
        let cast_expr = match cast.expr.ty {
            crate::semantic::types::Type::Bit(_) => {
                self.cast_bit_expr_to_ty(expr, &cast.expr.ty, &cast.ty, cast.span)
            }
            crate::semantic::types::Type::Bool(_) => {
                self.cast_bool_expr_to_ty(expr, &cast.expr.ty, &cast.ty, cast.span)
            }
            crate::semantic::types::Type::Duration(_) => {
                self.cast_duration_expr_to_ty(expr, &cast.expr.ty, &cast.ty, cast.span)
            }
            crate::semantic::types::Type::Angle(_, _) => {
                self.cast_angle_expr_to_ty(&expr, &cast.expr.ty, &cast.ty, cast.span)
            }
            crate::semantic::types::Type::Complex(_, _) => {
                self.cast_complex_expr_to_ty(expr, &cast.expr.ty, &cast.ty, cast.span)
            }
            crate::semantic::types::Type::Float(_, _) => {
                self.cast_float_expr_to_ty(expr, &cast.expr.ty, &cast.ty, cast.span)
            }
            crate::semantic::types::Type::Int(_, _) | crate::semantic::types::Type::UInt(_, _) => {
                self.cast_int_expr_to_ty(expr, &cast.expr.ty, &cast.ty, cast.span)
            }
            crate::semantic::types::Type::BitArray(ArrayDimensions::One(size), _) => {
                self.cast_bit_array_expr_to_ty(expr, &cast.expr.ty, &cast.ty, size, cast.span)
            }
            _ => None,
        };
        if cast_expr.is_none() {
            self.push_unsupported_error_message(
                format!("casting {} to {} type", cast.expr.ty, cast.ty),
                cast.span,
            );
        }
        cast_expr
    }

    fn compile_index_expr(&mut self, index: &IndexExpr) -> Option<qsast::Expr> {
        self.push_unimplemented_error_message("index expressions", index.span);
        None
    }

    fn compile_paren_expr(&mut self, paren: &Expr, span: Span) -> Option<qsast::Expr> {
        let expr = self.compile_expr(paren)?;
        Some(wrap_expr_in_parens(expr, span))
    }

    fn compile_measure_expr(&mut self, expr: &MeasureExpr) -> Option<qsast::Expr> {
        let call_span = expr.span;
        let name_span = expr.measure_token_span;
        let arg = self.compile_gate_operand(&expr.operand)?;
        let operand_span = expr.operand.span;
        let expr = build_measure_call(arg, name_span, operand_span, call_span);
        Some(expr)
    }

    fn compile_gate_operand(&mut self, op: &GateOperand) -> Option<qsast::Expr> {
        match &op.kind {
            GateOperandKind::HardwareQubit(hw) => {
                // We don't support hardware qubits, so we need to push an error
                // but we can still create an identifier for the hardware qubit
                // and let the rest of the containing expression compile to
                // catch any other errors
                let message = "Hardware qubit operands";
                self.push_unsupported_error_message(message, op.span);
                let ident = build_path_ident_expr(hw.name.clone(), hw.span, op.span);
                Some(ident)
            }
            GateOperandKind::Expr(expr) => self.compile_expr(expr),
            GateOperandKind::Err => None,
        }
    }

    fn compile_index_element(&mut self, elem: &IndexElement) -> Option<qsast::Expr> {
        match elem {
            IndexElement::DiscreteSet(discrete_set) => self.compile_discrete_set(discrete_set),
            IndexElement::IndexSet(index_set) => self.compile_index_set(index_set),
        }
    }

    fn compile_discrete_set(&mut self, set: &DiscreteSet) -> Option<qsast::Expr> {
        self.push_unimplemented_error_message("discrete set expressions", set.span);
        None
    }

    fn compile_index_set(&mut self, set: &IndexSet) -> Option<qsast::Expr> {
        // This is a temporary limitation. We can only handle
        // single index expressions for now.
        if set.values.len() == 1 {
            if let semast::IndexSetItem::Expr(expr) = &*set.values[0] {
                return self.compile_expr(expr);
            }
        }

        self.push_unimplemented_error_message("index set expressions", set.span);
        None
    }

    fn compile_array_literal(&mut self, _value: &List<Expr>, span: Span) -> Option<qsast::Expr> {
        self.push_unimplemented_error_message("array literals", span);
        None
    }

    fn compile_bit_literal(value: bool, span: Span) -> Option<qsast::Expr> {
        Some(build_lit_result_expr(value.into(), span))
    }

    fn compile_bool_literal(value: bool, span: Span) -> Option<qsast::Expr> {
        Some(build_lit_bool_expr(value, span))
    }

    fn compile_duration_literal(
        &mut self,
        _value: f64,
        _unit: TimeUnit,
        span: Span,
    ) -> Option<qsast::Expr> {
        self.push_unsupported_error_message("timing literals", span);
        None
    }

    fn compile_bitstring_literal(value: &BigInt, width: u32, span: Span) -> Option<qsast::Expr> {
        let width = width as usize;
        let bitstring = format!("Bitstring(\"{:0>width$}\")", value.to_str_radix(2));
        Some(build_lit_result_array_expr_from_bitstring(bitstring, span))
    }

    fn compile_complex_literal(real: f64, imag: f64, span: Span) -> Option<qsast::Expr> {
        Some(build_lit_complex_expr(
            crate::types::Complex::new(real, imag),
            span,
        ))
    }

    fn compile_float_literal(value: f64, span: Span) -> Option<qsast::Expr> {
        Some(build_lit_double_expr(value, span))
    }

    fn compile_int_literal(value: i64, span: Span) -> Option<qsast::Expr> {
        Some(build_lit_int_expr(value, span))
    }

    fn compile_bigint_literal(value: &BigInt, span: Span) -> Option<qsast::Expr> {
        Some(build_lit_bigint_expr(value.clone(), span))
    }

    fn compile_string_literal(&mut self, _value: &Rc<str>, span: Span) -> Option<qsast::Expr> {
        self.push_unimplemented_error_message("string literal expressions", span);
        None
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
        let error = crate::Error(kind);
        let error = WithSource::from_map(&self.source_map, error);
        self.errors.push(error);
    }

    /// +----------------+-------------------------------------------------------------+
    /// | Allowed casts  | Casting To                                                  |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | Casting From   | bool  | int | uint | float | angle | bit | duration | qubit |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | angle          | Yes   | No  | No   | No    | -     | Yes | No       | No    |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    fn cast_angle_expr_to_ty(
        &mut self,
        expr: &qsast::Expr,
        expr_ty: &crate::semantic::types::Type,
        ty: &crate::semantic::types::Type,
        _span: Span,
    ) -> Option<qsast::Expr> {
        assert!(matches!(expr_ty, Type::Angle(..)));
        // https://openqasm.com/language/types.html#casting-from-angle
        match ty {
            Type::Angle(..) => {
                let msg = "Cast angle to angle";
                self.push_unimplemented_error_message(msg, expr.span);
                None
            }
            Type::Bit(..) => {
                let msg = "Cast angle to bit";
                self.push_unimplemented_error_message(msg, expr.span);
                None
            }
            Type::BitArray(..) => {
                let msg = "Cast angle to bit array";
                self.push_unimplemented_error_message(msg, expr.span);
                None
            }
            Type::Bool(..) => {
                let msg = "Cast angle to bool";
                self.push_unimplemented_error_message(msg, expr.span);
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
    fn cast_bit_expr_to_ty(
        &mut self,
        expr: qsast::Expr,
        expr_ty: &crate::semantic::types::Type,
        ty: &crate::semantic::types::Type,
        span: Span,
    ) -> Option<qsast::Expr> {
        assert!(matches!(expr_ty, Type::Bit(..)));
        // There is no operand, choosing the span of the node
        // but we could use the expr span as well.
        let operand_span = expr.span;
        let name_span = span;
        match ty {
            &Type::Angle(..) => {
                let msg = "Cast bit to angle";
                self.push_unimplemented_error_message(msg, expr.span);
                None
            }
            &Type::Bool(..) => {
                self.runtime |= RuntimeFunctions::ResultAsBool;
                Some(build_cast_call(
                    RuntimeFunctions::ResultAsBool,
                    expr,
                    name_span,
                    operand_span,
                ))
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
                let expr = build_cast_call(function, expr, name_span, operand_span);
                Some(expr)
            }

            _ => None,
        }
    }

    fn cast_bit_array_expr_to_ty(
        &mut self,
        expr: qsast::Expr,
        expr_ty: &crate::semantic::types::Type,
        ty: &crate::semantic::types::Type,
        size: u32,
        span: Span,
    ) -> Option<qsast::Expr> {
        assert!(matches!(
            expr_ty,
            Type::BitArray(ArrayDimensions::One(_), _)
        ));

        let name_span = expr.span;
        let operand_span = span;

        if !matches!(ty, Type::Int(..) | Type::UInt(..)) {
            return None;
        }
        // we know we have a bit array being cast to an int/uint
        // verfiy widths
        let int_width = ty.width();

        if int_width.is_none() || (int_width == Some(size)) {
            self.runtime |= RuntimeFunctions::ResultArrayAsIntBE;
            let expr = build_cast_call(
                RuntimeFunctions::ResultArrayAsIntBE,
                expr,
                name_span,
                operand_span,
            );
            Some(expr)
        } else {
            None
        }
    }

    /// +----------------+-------------------------------------------------------------+
    /// | Allowed casts  | Casting To                                                  |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | Casting From   | bool  | int | uint | float | angle | bit | duration | qubit |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    /// | bool           | -     | Yes | Yes  | Yes   | No    | Yes | No       | No    |
    /// +----------------+-------+-----+------+-------+-------+-----+----------+-------+
    fn cast_bool_expr_to_ty(
        &mut self,
        expr: qsast::Expr,
        expr_ty: &crate::semantic::types::Type,
        ty: &crate::semantic::types::Type,
        span: Span,
    ) -> Option<qsast::Expr> {
        assert!(matches!(expr_ty, Type::Bool(..)));

        let name_span = expr.span;
        let operand_span = span;
        match ty {
            Type::Bit(..) => {
                self.runtime |= RuntimeFunctions::BoolAsResult;
                let expr = build_cast_call(
                    RuntimeFunctions::BoolAsResult,
                    expr,
                    name_span,
                    operand_span,
                );
                Some(expr)
            }
            Type::Float(..) => {
                self.runtime |= RuntimeFunctions::BoolAsDouble;
                let expr = build_cast_call(
                    RuntimeFunctions::BoolAsDouble,
                    expr,
                    name_span,
                    operand_span,
                );
                Some(expr)
            }
            Type::Int(w, _) | Type::UInt(w, _) => {
                let function = if let Some(width) = w {
                    if *width > 64 {
                        RuntimeFunctions::BoolAsBigInt
                    } else {
                        RuntimeFunctions::BoolAsInt
                    }
                } else {
                    RuntimeFunctions::BoolAsInt
                };
                self.runtime |= function;
                let expr = build_cast_call(function, expr, name_span, operand_span);
                Some(expr)
            }
            _ => None,
        }
    }

    fn cast_complex_expr_to_ty(
        &mut self,
        _expr: qsast::Expr,
        _expr_ty: &crate::semantic::types::Type,
        _ty: &crate::semantic::types::Type,
        span: Span,
    ) -> Option<qsast::Expr> {
        self.push_unimplemented_error_message("cast complex expressions", span);
        None
    }

    fn cast_duration_expr_to_ty(
        &mut self,
        _expr: qsast::Expr,
        _expr_ty: &crate::semantic::types::Type,
        _ty: &crate::semantic::types::Type,
        span: Span,
    ) -> Option<qsast::Expr> {
        self.push_unimplemented_error_message("cast duration expressions", span);
        None
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
    fn cast_float_expr_to_ty(
        &mut self,
        expr: qsast::Expr,
        expr_ty: &crate::semantic::types::Type,
        ty: &crate::semantic::types::Type,
        span: Span,
    ) -> Option<qsast::Expr> {
        assert!(matches!(expr_ty, Type::Float(..)));
        match ty {
            &Type::Complex(..) => {
                let expr = build_complex_from_expr(expr);
                Some(expr)
            }
            &Type::Angle(..) => {
                let msg = "Cast float to angle";
                self.push_unimplemented_error_message(msg, expr.span);
                None
            }
            &Type::Int(w, _) | &Type::UInt(w, _) => {
                let expr = build_math_call_from_exprs("Truncate", vec![expr], span);
                let expr = if let Some(w) = w {
                    if w > 64 {
                        build_convert_call_expr(expr, "IntAsBigInt")
                    } else {
                        expr
                    }
                } else {
                    expr
                };

                Some(expr)
            }
            &Type::Bool(..) => {
                let span = expr.span;
                let expr = build_math_call_from_exprs("Truncate", vec![expr], span);
                let const_int_zero_expr = build_lit_int_expr(0, span);
                let qsop = qsast::BinOp::Eq;
                let cond = build_binary_expr(false, qsop, expr, const_int_zero_expr, span);
                let coerce_expr = build_if_expr_then_expr_else_expr(
                    cond,
                    build_lit_bool_expr(false, span),
                    build_lit_bool_expr(true, span),
                    span,
                );
                Some(coerce_expr)
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
    /// With the exception of casting to ``BigInt``, there is no checking for overflow,
    /// widths, truncation, etc. Qiskit doesn't do these kinds of casts. For general
    /// `OpenQASM` support this will need to be fleshed out.
    #[allow(clippy::too_many_lines)]
    fn cast_int_expr_to_ty(
        &mut self,
        expr: qsast::Expr,
        expr_ty: &crate::semantic::types::Type,
        ty: &crate::semantic::types::Type,
        span: Span,
    ) -> Option<qsast::Expr> {
        assert!(matches!(expr_ty, Type::Int(..) | Type::UInt(..)));
        let name_span = expr.span;
        let operand_span = span;
        match ty {
            Type::BitArray(dims, _) => {
                self.runtime |= RuntimeFunctions::IntAsResultArrayBE;
                let ArrayDimensions::One(size) = dims else {
                    return None;
                };
                let size = i64::from(*size);

                let size_expr = build_lit_int_expr(size, Span::default());
                let expr = build_cast_call_two_params(
                    RuntimeFunctions::IntAsResultArrayBE,
                    expr,
                    size_expr,
                    name_span,
                    operand_span,
                );
                Some(expr)
            }
            Type::Float(..) => {
                let expr = build_convert_call_expr(expr, "IntAsDouble");
                Some(expr)
            }
            Type::Int(tw, _) | Type::UInt(tw, _) => {
                // uint to int, or int/uint to BigInt
                if let Some(tw) = tw {
                    if *tw > 64 {
                        let expr = build_convert_call_expr(expr, "IntAsBigInt");
                        Some(expr)
                    } else {
                        Some(expr)
                    }
                } else {
                    Some(expr)
                }
            }
            Type::Bool(..) => {
                let expr_span = expr.span;
                let const_int_zero_expr = build_lit_int_expr(0, expr.span);
                let qsop = qsast::BinOp::Eq;
                let cond = build_binary_expr(false, qsop, expr, const_int_zero_expr, expr_span);
                let coerce_expr = build_if_expr_then_expr_else_expr(
                    cond,
                    build_lit_bool_expr(false, expr_span),
                    build_lit_bool_expr(true, expr_span),
                    expr_span,
                );
                Some(coerce_expr)
            }
            Type::Bit(..) => {
                let expr_span = expr.span;
                let const_int_zero_expr = build_lit_int_expr(0, expr.span);
                let qsop = qsast::BinOp::Eq;
                let cond = build_binary_expr(false, qsop, expr, const_int_zero_expr, expr_span);
                let coerce_expr = build_if_expr_then_expr_else_expr(
                    cond,
                    build_lit_result_expr(qsast::Result::One, expr_span),
                    build_lit_result_expr(qsast::Result::Zero, expr_span),
                    expr_span,
                );
                Some(coerce_expr)
            }
            Type::Complex(..) => {
                let expr = build_convert_call_expr(expr, "IntAsDouble");
                let expr = build_complex_from_expr(expr);
                Some(expr)
            }
            _ => None,
        }
    }

    fn map_bin_op(op: semast::BinOp) -> qsast::BinOp {
        match op {
            semast::BinOp::Add => qsast::BinOp::Add,
            semast::BinOp::AndB => qsast::BinOp::AndB,
            semast::BinOp::AndL => qsast::BinOp::AndL,
            semast::BinOp::Div => qsast::BinOp::Div,
            semast::BinOp::Eq => qsast::BinOp::Eq,
            semast::BinOp::Exp => qsast::BinOp::Exp,
            semast::BinOp::Gt => qsast::BinOp::Gt,
            semast::BinOp::Gte => qsast::BinOp::Gte,
            semast::BinOp::Lt => qsast::BinOp::Lt,
            semast::BinOp::Lte => qsast::BinOp::Lte,
            semast::BinOp::Mod => qsast::BinOp::Mod,
            semast::BinOp::Mul => qsast::BinOp::Mul,
            semast::BinOp::Neq => qsast::BinOp::Neq,
            semast::BinOp::OrB => qsast::BinOp::OrB,
            semast::BinOp::OrL => qsast::BinOp::OrL,
            semast::BinOp::Shl => qsast::BinOp::Shl,
            semast::BinOp::Shr => qsast::BinOp::Shr,
            semast::BinOp::Sub => qsast::BinOp::Sub,
            semast::BinOp::XorB => qsast::BinOp::XorB,
        }
    }
}
