use std::{
    fmt::{Display, Write},
    mem::take,
};

use crate::circuit;
use log::debug;
use qsc_circuit::{
    Circuit, Component, ComponentColumn, ComponentGrid, Config, GenerationMethod, Ket, Measurement,
    Operation, Qubit, Register, Unitary, group_qubits, operation_list_to_grid,
};
use qsc_data_structures::{index_map::IndexMap, line_column::Encoding, span::Span};
use qsc_frontend::{compile::PackageStore, location::Location};
use qsc_hir::hir::PackageId;
use qsc_partial_eval::{
    Callable, CallableType, ConditionCode, FcmpConditionCode, Instruction, Literal, Operand,
    VariableId,
    rir::{
        BlockId, BlockWithMetadata, InstructionMetadata, InstructionWithMetadata, Program, Ty,
        Variable,
    },
};
use rustc_hash::FxHashSet;

type ResultId = (usize, usize);

#[derive(Clone)]
enum Op {
    Real(RealOp),
    UnexpandedBranch(UnexpandedBranch),
}

#[derive(Clone)]
struct UnexpandedBranch {
    cond_str: String,
    true_block: BlockId,
    false_block: BlockId,
    control_results: Vec<ResultId>,
    metadata: Option<InstructionMetadata>,
}

#[derive(Clone)]
struct RealOp {
    kind: OperationKind,
    label: String,
    target_qubits: Vec<usize>,
    control_qubits: Vec<usize>,
    target_results: Vec<ResultId>,
    control_results: Vec<ResultId>,
    is_adjoint: bool,
    children: Vec<Op>,
    args: Vec<String>,
    metadata: Option<InstructionMetadata>,
}

impl Op {
    fn control_qubits(&self) -> &[usize] {
        match self {
            Op::Real(r) => &r.control_qubits,
            Op::UnexpandedBranch(_) => &[],
        }
    }
    fn target_qubits(&self) -> &[usize] {
        match self {
            Op::Real(r) => &r.target_qubits,
            Op::UnexpandedBranch(_u) => &[],
        }
    }
    fn args(&self) -> &[String] {
        match self {
            Op::Real(r) => &r.args,
            Op::UnexpandedBranch(_u) => &[],
        }
    }
    fn metadata(&self) -> Option<&InstructionMetadata> {
        match self {
            Op::Real(r) => r.metadata.as_ref(),
            Op::UnexpandedBranch(u) => u.metadata.as_ref(),
        }
    }
}

#[derive(Clone)]
enum OperationKind {
    Unitary,
    Measurement,
    Ket,
}

impl From<RealOp> for Operation {
    fn from(value: RealOp) -> Self {
        let args = value
            .args
            .into_iter()
            .chain(value.metadata.map(|md| format!("metadata={md}")))
            .collect();

        let children = vec![ComponentColumn {
            components: value
                .children
                .into_iter()
                .map(|o| {
                    match o {
                        Op::Real(real_op) => real_op,
                        Op::UnexpandedBranch(_) => {
                            panic!("branches shouldn't appear here")
                        }
                    }
                    .into()
                })
                .collect(),
        }];
        let targets = value
            .target_qubits
            .into_iter()
            .map(|q| Register {
                qubit: q,
                result: None,
            })
            .chain(value.target_results.into_iter().map(|(q, r)| Register {
                qubit: q,
                result: Some(r),
            }))
            .collect();
        let controls = value
            .control_qubits
            .into_iter()
            .map(|q| Register {
                qubit: q,
                result: None,
            })
            .chain(value.control_results.into_iter().map(|(q, r)| Register {
                qubit: q,
                result: Some(r),
            }))
            .collect();

        match value.kind {
            OperationKind::Unitary => Operation::Unitary(Unitary {
                gate: value.label,
                args,
                children,
                targets,
                controls,
                is_adjoint: value.is_adjoint,
            }),
            OperationKind::Measurement => Operation::Measurement(Measurement {
                gate: value.label,
                args,
                children,
                qubits: controls,
                results: targets,
            }),
            OperationKind::Ket => Operation::Ket(Ket {
                gate: value.label,
                args,
                children,
                targets,
            }),
        }
    }
}

pub(crate) fn make_circuit(
    program: &Program,
    package_store: &PackageStore,
    position_encoding: Encoding,
    config: Config,
) -> std::result::Result<Circuit, circuit::Error> {
    assert!(config.generation_method == GenerationMethod::Static);
    debug!("make_circuit: program={}", program);
    let mut state = ProgramMap::new(program.num_qubits);
    let callables = &program.callables;

    for (id, block) in program.blocks.iter() {
        let block_operations =
            operations_in_block(&mut state, callables, block, config.group_scopes)?;
        state.blocks.insert(id, block_operations);
    }

    expand_branches(program, &mut state)?;

    let entry_block = program
        .callables
        .get(program.entry)
        .expect("entry callable should exist")
        .body
        .expect("entry callable should have a body");

    let entry_block = state
        .blocks
        .get(entry_block)
        .expect("entry block should have been processed");

    if matches!(entry_block.terminator, Some(Terminator::Unconditional(_))) {
        return Err(circuit::Error::UnsupportedFeature(
            "entry block should not have an unconditional successor".to_owned(),
        ));
    }
    let entry_operations = entry_block
        .operations
        .iter()
        .map(|op| match op {
            Op::Real(real_op) => real_op.clone(),
            Op::UnexpandedBranch { .. } => {
                panic!("entry block should not have unexpanded branches")
            }
        })
        .collect();

    let operations = expand_successors(&state, entry_operations);

    let qubits = state.into_qubits();
    let operations = operations.into_iter().map(Into::into).collect();

    let (operations, qubits) = if config.collapse_qubit_registers && qubits.len() > 2 {
        // TODO: dummy values for now
        group_qubits(operations, qubits, &[0, 1])
    } else {
        (operations, qubits)
    };

    let mut component_grid = operation_list_to_grid(operations, &qubits, config.loop_detection);

    fill_in_dbg_metadata(&mut component_grid, package_store, position_encoding)?;

    let circuit = Circuit {
        qubits,
        component_grid,
    };
    Ok(circuit)
}

fn expand_branches(program: &Program, state: &mut ProgramMap) -> Result<(), qsc_circuit::Error> {
    let mut more_work = true;
    while more_work {
        // TODO: stop if no progress to avoid infinite loop
        more_work = false;
        for block in program.blocks.iter() {
            let mut circuit_block = state
                .blocks
                .get(block.0)
                .expect("block should exist")
                .clone();
            for operation in &mut circuit_block.operations {
                if expand_branch_children(&*state, operation)? {
                    more_work = true;
                }
            }
            state.blocks.insert(block.0, circuit_block);
        }
    }
    Ok(())
}

fn expand_successors(state: &ProgramMap, block_operations: Vec<RealOp>) -> Vec<RealOp> {
    let mut operations = vec![];
    let mut operations_stack = block_operations;
    operations_stack.reverse();

    while let Some(mut operation) = operations_stack.pop() {
        if let unitary @ RealOp {
            kind: OperationKind::Unitary,
            ..
        } = &mut operation
        {
            if unitary.label == "successor" {
                let successor_block_id = BlockId(unitary.args[0].parse().unwrap_or_else(|_| {
                    panic!("successor block id should parse: {}", unitary.args[0])
                }));
                unitary.args.remove(0);
                unitary.label = "check ".into();
                let successor_block = state
                    .blocks
                    .get(successor_block_id)
                    .expect("successor block should exist");
                for successor_operation in successor_block.operations.iter().rev() {
                    let successor_operation = match successor_operation {
                        Op::Real(real_op) => real_op,
                        Op::UnexpandedBranch { .. } => {
                            panic!("successor block should not have unexpanded branches")
                        }
                    };
                    operations_stack.push(successor_operation.clone());
                }
                if unitary.children.is_empty() {
                    // empty block, skip adding
                    continue;
                }
            }

            if !unitary.children.is_empty() {
                let children = take(&mut unitary.children)
                    .into_iter()
                    .map(|o| match o {
                        Op::Real(real_op) => real_op,
                        Op::UnexpandedBranch(_) => {
                            panic!("branches shouldn't appear here")
                        }
                    })
                    .collect();
                let children = expand_successors(state, children);
                unitary.children = children.into_iter().map(Op::Real).collect();
            }
        }
        operations.push(operation.clone());
    }
    operations
}

fn fill_in_dbg_metadata(
    component_grid: &mut ComponentGrid,
    package_store: &PackageStore,
    position_encoding: Encoding,
) -> Result<(), qsc_circuit::Error> {
    for column in component_grid {
        for component in &mut column.components {
            let children = match component {
                Component::Unitary(unitary) => &mut unitary.children,
                Component::Measurement(measurement) => &mut measurement.children,
                Component::Ket(ket) => &mut ket.children,
            };

            fill_in_dbg_metadata(children, package_store, position_encoding)?;

            let args = match component {
                Component::Unitary(unitary) => &mut unitary.args,
                Component::Measurement(measurement) => &mut measurement.args,
                Component::Ket(ket) => &mut ket.args,
            };

            // if last arg starts with metadata=, pop it
            let metadata_str = args
                .last()
                .and_then(|last_arg| last_arg.strip_prefix("metadata="))
                .map(ToOwned::to_owned);

            if let Some(metadata_str) = metadata_str {
                args.pop();
                let dbg_metadata = parse_dbg_metadata(&metadata_str);
                if let Some(((package_id, span), scope, discriminator, _)) = dbg_metadata {
                    let location =
                        Location::from(span, package_id, package_store, position_encoding);
                    let mut json = String::new();
                    writeln!(&mut json, "metadata={{").expect("writing to string should work");
                    writeln!(&mut json, r#""source": {:?},"#, location.source)
                        .expect("writing to string should work");
                    write!(
                        &mut json,
                        r#""span": {{"start": {{"line": {}, "character": {}}}, "end": {{"line": {}, "character": {}}}}}"#,
                        location.range.start.line,
                        location.range.start.column,
                        location.range.end.line,
                        location.range.end.column
                    )
                    .expect("writing to string should work");
                    if let Some((block_id, package_id, span)) = scope {
                        writeln!(&mut json, ",").expect("writing to string should work");
                        let scope_location =
                            Location::from(span, package_id, package_store, position_encoding);
                        write!(&mut json, "\"scope\": {},", block_id.0)
                            .expect("writing to string should work");
                        write!(
                            &mut json,
                            r#""scope_location": {{"start": {{"line": {}, "character": {}}}, "end": {{"line": {}, "character": {}}}}}"#,
                            scope_location.range.start.line,
                            scope_location.range.start.column,
                            scope_location.range.end.line,
                            scope_location.range.end.column
                        )
                        .expect("writing to string should work");
                    }
                    if let Some(discriminator) = discriminator {
                        writeln!(&mut json, ",").expect("writing to string should work");
                        write!(&mut json, "\"discriminator\": {discriminator}")
                            .expect("writing to string should work");
                    }
                    write!(&mut json, "}}").expect("writing to string should work");
                    args.push(json);
                }
            }
        }
    }
    Ok(())
}

type DbgMetadata = (
    (PackageId, Span),                  // source location
    Option<(BlockId, PackageId, Span)>, // scope id and location
    Option<usize>,                      // discriminator
    Option<String>,                     // callable name
);

fn parse_dbg_metadata(metadata_str: &str) -> Option<DbgMetadata> {
    // metadata is of the format "!dbg package_id=0 span=[2161-2172] scope=0 scope_package_id=1 scope_span=[123-345] discriminator=1 callable=foo"
    if let Some(rest) = metadata_str.strip_prefix("!dbg ") {
        let mut package_id: Option<usize> = None;
        let mut span = None;
        let mut scope: Option<usize> = None;
        let mut discriminator = None;
        let mut callable = None;
        let mut scope_package_id: Option<usize> = None;
        let mut scope_span: Option<Span> = None;

        for token in rest.split_whitespace() {
            if let Some(rest) = token.strip_prefix("package_id=") {
                package_id = rest.parse().ok();
            } else if let Some(rest) = token.strip_prefix("span=") {
                span = parse_span(rest);
            } else if let Some(rest) = token.strip_prefix("scope=") {
                scope = rest.parse().ok();
            } else if let Some(rest) = token.strip_prefix("scope_package_id=") {
                scope_package_id = rest.parse().ok();
            } else if let Some(rest) = token.strip_prefix("scope_span=") {
                scope_span = parse_span(rest);
            } else if let Some(rest) = token.strip_prefix("discriminator=") {
                discriminator = rest.parse().ok();
            } else if let Some(rest) = token.strip_prefix("callable=") {
                callable = Some(rest.to_string());
            }
        }

        if let (Some(package_id), Some(span)) = (package_id, span) {
            let package_id = package_id.into();
            let scope: Option<BlockId> = scope.map(Into::into);
            let scope_package_id: Option<PackageId> = scope_package_id.map(Into::into);
            let scope = match (scope, scope_package_id, scope_span) {
                (Some(s), Some(p), Some(sp)) => Some((s, p, sp)),
                _ => None,
            };
            return Some(((package_id, span), scope, discriminator, callable));
        }
    }
    None
}

fn parse_span(rest: &str) -> Option<Span> {
    if let Some(span_str) = rest.strip_prefix("[") {
        if let Some(span_str) = span_str.strip_suffix("]") {
            let span_parts: Vec<&str> = span_str.split('-').collect();
            if span_parts.len() == 2 {
                if let (Ok(start), Ok(end)) =
                    (span_parts[0].parse::<u32>(), span_parts[1].parse::<u32>())
                {
                    return Some(Span { lo: start, hi: end });
                }
            }
        }
    }
    None
}

fn expand_branch_children(
    state: &ProgramMap,
    operation: &mut Op,
) -> Result<bool, qsc_circuit::Error> {
    if let Op::UnexpandedBranch(UnexpandedBranch {
        cond_str,
        true_block,
        false_block,
        control_results,
        metadata,
    }) = operation
    {
        let true_block = *true_block;
        let false_block = *false_block;
        if let Some(branch_operations) = operations_from_branch(state, true_block, false_block)? {
            let (true_operations, true_targets) = branch_operations.true_block;
            let true_container = RealOp {
                kind: OperationKind::Unitary,
                label: "true".into(),
                args: vec![],
                children: true_operations.iter().cloned().map(Op::Real).collect(),
                target_qubits: true_targets.clone(),
                control_qubits: vec![],
                target_results: vec![],
                control_results: control_results.clone(),
                is_adjoint: false,
                metadata: None,
            };

            let false_container =
                branch_operations
                    .false_block
                    .map(|(false_operations, false_targets)| {
                        (
                            RealOp {
                                kind: OperationKind::Unitary,
                                label: "false".into(),
                                target_qubits: false_targets.clone(),
                                control_qubits: vec![],
                                target_results: vec![],
                                control_results: control_results.clone(),
                                args: vec![],
                                is_adjoint: false,
                                children: false_operations.iter().cloned().map(Op::Real).collect(),
                                metadata: None,
                            },
                            false_targets,
                        )
                    });

            let true_container = if true_container.children.is_empty() {
                None
            } else {
                Some(true_container)
            };
            let false_container = false_container.and_then(|f| {
                if f.0.children.is_empty() {
                    None
                } else {
                    Some(f)
                }
            });

            let mut children = vec![];
            let mut target_qubits = vec![];

            if let Some(true_container) = true_container {
                children.push(true_container);
                target_qubits.extend(true_targets);
            }

            if let Some((false_container, false_targets)) = false_container {
                children.push(false_container);
                target_qubits.extend(false_targets);
            }

            // dedup targets
            target_qubits.sort_unstable();
            target_qubits.dedup();
            // TODO: target results for container? measurements in branches?

            let args = vec![branch_operations.successor.0.to_string(), cond_str.clone()];
            let label = "successor".to_string();
            *operation = Op::Real(RealOp {
                kind: OperationKind::Unitary,
                label,
                target_qubits,
                control_qubits: vec![],
                target_results: vec![],
                control_results: control_results.clone(),
                is_adjoint: false,
                children: children.into_iter().map(Op::Real).collect(),
                args,
                metadata: metadata.clone(),
            });
        } else {
            return Ok(true); // more work needed to fill in the branch children
        }
    }
    Ok(false)
}

#[derive(Clone)]
struct CircuitBlock {
    _predecessors: Vec<BlockId>,
    operations: Vec<Op>,
    terminator: Option<Terminator>,
}

fn operations_in_block(
    state: &mut ProgramMap,
    callables: &IndexMap<qsc_partial_eval::CallableId, Callable>,
    block: &BlockWithMetadata,
    group_scopes: bool,
) -> Result<CircuitBlock, qsc_circuit::Error> {
    // TODO: use get_block_successors from utils
    let mut terminator = None;
    let mut predecessors = vec![];
    let mut operations = vec![];
    let mut done = false;

    let mut current_scope = vec![];
    let mut last_scope = None;
    // let mut last_discriminator = None;
    for instruction in &block.0 {
        if done {
            return Err(qsc_circuit::Error::UnsupportedFeature(
                "instructions after return or jump in block".to_owned(),
            ));
        }
        let BlockUpdate {
            operations: new_operations,
            terminator: new_terminator,
        } = get_operations_for_instruction(
            state,
            callables,
            &mut predecessors,
            &mut done,
            instruction,
        )?;

        if let Some(new_terminator) = new_terminator {
            let old = terminator.replace(new_terminator);
            assert!(
                old.is_none(),
                "did not expect more than one unconditional successor for block, old: {old:?} new: {terminator:?}"
            );
        }

        extend_operations(
            &mut operations,
            &mut current_scope,
            &mut last_scope,
            new_operations,
            group_scopes,
        );
    }

    if !current_scope.is_empty() {
        flush_scope(
            &mut operations,
            &mut current_scope,
            &mut last_scope,
            group_scopes,
        );
    }

    Ok(CircuitBlock {
        _predecessors: predecessors,
        operations,
        terminator, // TODO: make this exhaustive, and detect corrupt blocks
    })
}

fn extend_operations(
    operations: &mut Vec<Op>,
    current_scope: &mut Vec<Op>,
    last_scope: &mut Option<String>,
    new_operations: Vec<Op>,
    group_scopes: bool,
) {
    for op in new_operations {
        let metadata = match &op {
            Op::Real(r) => &r.metadata,
            Op::UnexpandedBranch(u) => &u.metadata,
        };
        if let Some(metadata) = metadata {
            if let (Some(callable), Some(block_id)) =
                (&metadata.current_callable_name, metadata.scope_id)
            {
                let scope = format!("{callable}_{block_id}");
                if last_scope
                    .as_ref()
                    .is_some_and(|last_scope| last_scope == &scope)
                    && matches!(
                        &op,
                        Op::Real(RealOp {
                            kind: OperationKind::Unitary,
                            ..
                        }) | Op::UnexpandedBranch { .. }
                    )
                // only group unitaries
                {
                    // Add to current group
                    current_scope.push(op);
                } else {
                    // flush group
                    if !current_scope.is_empty() {
                        flush_scope(operations, current_scope, last_scope, group_scopes);
                    }

                    current_scope.push(op);
                }
                *last_scope = Some(scope);

                continue;
            }
        }
        // no scope grouping, flush current scope if any, then add this one right away

        // flush group
        if !current_scope.is_empty() {
            flush_scope(operations, current_scope, last_scope, group_scopes);
        }

        // reset last scope
        *last_scope = None;

        // add this operation
        operations.push(op);
    }
}

fn flush_scope(
    operations: &mut Vec<Op>,
    current_scope: &mut Vec<Op>,
    last_scope: &mut Option<String>,
    group_scopes: bool,
) {
    let mut should_group = current_scope.len() > 1;
    if !group_scopes {
        should_group = false;
    }
    // don't group if there is no overlap at all in the qubits
    if should_group {
        let mut some_overlap = false;
        let mut qubits: FxHashSet<usize> = FxHashSet::default();
        for op in current_scope.iter() {
            // TODO: I feel like this breaks down if `op` is an unexpanded branch?
            let op_qubits: Vec<usize> = op
                .control_qubits()
                .iter()
                .chain(op.target_qubits())
                .copied()
                .collect();
            let initial_len = qubits.len();
            qubits.extend(op_qubits.clone());
            if qubits.len() < initial_len + op_qubits.len() {
                some_overlap = true;
                break;
            }
        }
        if !some_overlap {
            should_group = false;
        }
    }

    if should_group {
        let args = current_scope[0].args().to_vec();
        let metadata = current_scope[0].metadata().cloned();
        let qubits: FxHashSet<usize> = current_scope
            .iter()
            .flat_map(|op| {
                op.control_qubits()
                    .iter()
                    .chain(op.target_qubits())
                    .copied()
            })
            .collect();

        let group = Op::Real(RealOp {
            kind: OperationKind::Unitary,
            label: last_scope
                .as_ref()
                .expect("last scope should exist here")
                .to_string(),
            target_qubits: qubits.into_iter().collect(),
            control_qubits: vec![],
            target_results: vec![],
            control_results: vec![],
            is_adjoint: false,
            children: current_scope.clone(),
            args,
            metadata,
        });
        operations.push(group);
        current_scope.clear();
    } else {
        for op in current_scope.drain(..) {
            operations.push(op);
        }
    }
}

struct BlockUpdate {
    operations: Vec<Op>,
    terminator: Option<Terminator>,
}

#[derive(Debug, Clone)]
enum Terminator {
    Unconditional(BlockId),
    Conditional(Variable, BlockId, BlockId),
}

fn get_operations_for_instruction(
    state: &mut ProgramMap,
    callables: &IndexMap<qsc_partial_eval::CallableId, Callable>,
    predecessors: &mut Vec<BlockId>,
    done: &mut bool,
    instruction: &InstructionWithMetadata,
) -> Result<BlockUpdate, qsc_circuit::Error> {
    let mut unconditional_successor: Option<BlockId> = None;
    let mut conditional_successor: Option<(Variable, BlockId, BlockId)> = None;
    let operations = match &instruction.instruction {
        Instruction::Call(callable_id, operands, var) => extend_block_with_call_instruction(
            state,
            callables,
            instruction,
            *callable_id,
            operands,
            *var,
        )?,
        Instruction::Fcmp(condition_code, operand, operand1, variable) => {
            extend_block_with_fcmp_instruction(
                state,
                *condition_code,
                operand,
                operand1,
                *variable,
            )?;
            vec![]
        }
        Instruction::Icmp(condition_code, operand, operand1, variable) => {
            extend_block_with_icmp_instruction(
                state,
                *condition_code,
                operand,
                operand1,
                *variable,
            )?;
            vec![]
        }
        Instruction::Return => {
            *done = true;
            vec![]
        }
        Instruction::Branch(variable, block_id_1, block_id_2) => {
            *done = true;
            extend_block_with_branch_instruction(
                &mut conditional_successor,
                state,
                instruction,
                *variable,
                *block_id_1,
                *block_id_2,
            )?
        }
        Instruction::Jump(block_id) => {
            extend_block_with_jump_instruction(&mut unconditional_successor, *block_id)?;
            *done = true;
            vec![]
        }
        Instruction::Phi(pres, variable) => {
            extend_block_with_phi_instruction(state, predecessors, pres, *variable)?;
            vec![]
        }
        Instruction::Add(operand, operand1, variable)
        | Instruction::Sub(operand, operand1, variable)
        | Instruction::Mul(operand, operand1, variable)
        | Instruction::Sdiv(operand, operand1, variable)
        | Instruction::Srem(operand, operand1, variable)
        | Instruction::Shl(operand, operand1, variable)
        | Instruction::Ashr(operand, operand1, variable)
        | Instruction::Fadd(operand, operand1, variable)
        | Instruction::Fsub(operand, operand1, variable)
        | Instruction::Fmul(operand, operand1, variable)
        | Instruction::Fdiv(operand, operand1, variable)
        | Instruction::LogicalAnd(operand, operand1, variable)
        | Instruction::LogicalOr(operand, operand1, variable)
        | Instruction::BitwiseAnd(operand, operand1, variable)
        | Instruction::BitwiseOr(operand, operand1, variable)
        | Instruction::BitwiseXor(operand, operand1, variable) => {
            extend_block_with_binop_instruction(state, operand, operand1, *variable)?;
            vec![]
        }
        instruction @ (Instruction::LogicalNot(..) | Instruction::BitwiseNot(..)) => {
            // Leave the variable unassigned, if it's used in anything that's going to be shown in the circuit, we'll raise an error then
            debug!("ignoring not instruction: {instruction:?}");
            vec![]
        }
        instruction @ Instruction::Store(..) => {
            return Err(qsc_circuit::Error::UnsupportedFeature(format!(
                "unsupported instruction in block: {instruction:?}"
            )));
        }
    };
    Ok(BlockUpdate {
        operations,
        terminator: if let Some(block_id) = unconditional_successor {
            Some(Terminator::Unconditional(block_id))
        } else if let Some((var, block_id_1, block_id_2)) = conditional_successor {
            Some(Terminator::Conditional(var, block_id_1, block_id_2))
        } else {
            None
        },
    })
}

fn extend_block_with_binop_instruction(
    state: &mut ProgramMap,
    operand: &Operand,
    operand1: &Operand,
    variable: Variable,
) -> Result<(), qsc_circuit::Error> {
    let expr_left = expr_from_operand(state, operand)?;
    let expr_right = expr_from_operand(state, operand1)?;
    let expr = Expr::RichExpr(RichExpr::FunctionOf(vec![expr_left, expr_right]));
    state.store_expr_in_variable(variable, expr)?;
    Ok(())
}

fn extend_block_with_phi_instruction(
    state: &mut ProgramMap,
    predecessors: &mut Vec<BlockId>,
    pres: &Vec<(Operand, BlockId)>,
    variable: Variable,
) -> Result<(), qsc_circuit::Error> {
    let mut exprs = vec![];
    for pre in pres {
        exprs.push(expr_from_operand(state, &pre.0)?);
    }
    let expr = if exprs.iter().all(|e| matches!(e, Expr::BoolExpr(_))) {
        // fold into pairs of FancyBinOp
        exprs
            .into_iter()
            .reduce(|left, right| {
                Expr::BoolExpr(BoolExpr::BinOp(
                    left.into(),
                    right.into(),
                    "or maybe".into(),
                ))
            })
            .expect("there should be at least one expression")
    } else {
        Expr::RichExpr(RichExpr::Options(exprs))
    };
    state.store_expr_in_variable(variable, expr)?;
    predecessors.extend(pres.iter().map(|p| p.1));
    Ok(())
}

fn extend_block_with_jump_instruction(
    successor: &mut Option<BlockId>,
    block_id: BlockId,
) -> Result<(), qsc_circuit::Error> {
    let old = successor.replace(block_id);
    let r = if old.is_some() {
        Err(qsc_circuit::Error::UnsupportedFeature(
            "block contains more than one jump".to_owned(),
        ))
    } else {
        Ok(())
    };
    r?;
    Ok(())
}

fn extend_block_with_branch_instruction(
    conditional_successors: &mut Option<(Variable, BlockId, BlockId)>,
    state: &mut ProgramMap,
    instruction: &InstructionWithMetadata,
    variable: Variable,
    block_id_1: BlockId,
    block_id_2: BlockId,
) -> Result<Vec<Op>, qsc_circuit::Error> {
    let old = conditional_successors.replace((variable, block_id_1, block_id_2));
    if old.is_some() {
        return Err(qsc_circuit::Error::UnsupportedFeature(
            "block contains more than one branch".to_owned(),
        ));
    }
    Ok(vec![operation_for_branch(
        state,
        instruction,
        variable,
        block_id_1,
        block_id_2,
    )?])
}

fn extend_block_with_icmp_instruction(
    state: &mut ProgramMap,
    condition_code: ConditionCode,
    operand: &Operand,
    operand1: &Operand,
    variable: Variable,
) -> Result<(), qsc_circuit::Error> {
    match condition_code {
        ConditionCode::Eq => {
            let expr_left = expr_from_operand(state, operand)?;
            let expr_right = expr_from_operand(state, operand1)?;
            let expr = eq_expr(expr_left, expr_right)?;
            state.store_expr_in_variable(variable, Expr::BoolExpr(expr))
        }
        condition_code => Err(qsc_circuit::Error::UnsupportedFeature(format!(
            "unsupported condition code in icmp: {condition_code:?}"
        ))),
    }
}

fn extend_block_with_fcmp_instruction(
    state: &mut ProgramMap,
    condition_code: FcmpConditionCode,
    operand: &Operand,
    operand1: &Operand,
    variable: Variable,
) -> Result<(), qsc_circuit::Error> {
    let expr_left = expr_from_operand(state, operand)?;
    let expr_right = expr_from_operand(state, operand1)?;
    let expr = match condition_code {
        FcmpConditionCode::False => BoolExpr::LiteralBool(false),
        FcmpConditionCode::True => BoolExpr::LiteralBool(true),
        cmp => BoolExpr::BinOp(expr_left.into(), expr_right.into(), cmp.to_string()),
    };
    state.store_expr_in_variable(variable, Expr::BoolExpr(expr))?;
    Ok(())
}

fn extend_block_with_call_instruction(
    state: &mut ProgramMap,
    callables: &IndexMap<qsc_partial_eval::CallableId, Callable>,
    instruction: &InstructionWithMetadata,
    callable_id: qsc_partial_eval::CallableId,
    operands: &Vec<Operand>,
    var: Option<Variable>,
) -> Result<Vec<Op>, qsc_circuit::Error> {
    map_callable_to_operations(
        state,
        callables.get(callable_id).expect("callable should exist"),
        operands,
        var,
        instruction.metadata.as_ref(),
    )
    .map(|ops| ops.into_iter().map(Op::Real).collect())
}

fn operation_for_branch(
    state: &mut ProgramMap,
    instruction: &InstructionWithMetadata,
    variable: Variable,
    true_block: BlockId,
    false_block: BlockId,
) -> Result<Op, qsc_circuit::Error> {
    let (results, cond_str) = state.condition_for_variable(variable.variable_id)?;
    // TODO: let's allow this for now, though it's weird (see phi nodes)
    if results.is_empty() {
        return Err(qsc_circuit::Error::UnsupportedFeature(format!(
            "branching on a condition that doesn't involve at least one result: {cond_str}"
        )));
    }
    let controls = results.into_iter().map(|r| state.result_register(r));

    let op = Op::UnexpandedBranch(UnexpandedBranch {
        cond_str,
        true_block,
        false_block,
        control_results: controls
            .into_iter()
            .map(|r| {
                (
                    r.qubit,
                    r.result.expect("result register must have result idx"),
                )
            })
            .collect(),
        metadata: instruction.metadata.clone(),
    });
    Ok(op)
}

fn eq_expr(expr_left: Expr, expr_right: Expr) -> Result<BoolExpr, qsc_circuit::Error> {
    Ok(match (expr_left, expr_right) {
        (Expr::BoolExpr(BoolExpr::LiteralBool(b1)), Expr::BoolExpr(BoolExpr::LiteralBool(b2))) => {
            BoolExpr::LiteralBool(b1 == b2)
        }
        (Expr::BoolExpr(BoolExpr::Result(r)), Expr::BoolExpr(BoolExpr::LiteralBool(b)))
        | (Expr::BoolExpr(BoolExpr::LiteralBool(b)), Expr::BoolExpr(BoolExpr::Result(r))) => {
            if b {
                BoolExpr::Result(r)
            } else {
                BoolExpr::NotResult(r)
            }
        }
        (Expr::BoolExpr(BoolExpr::Result(left)), Expr::BoolExpr(BoolExpr::Result(right))) => {
            BoolExpr::TwoResultCondition(TwoResultCondition {
                results: (left, right),
                filter: (true, false, false, true), // 00 and 11
            })
        }
        (left, right) => {
            return Err(qsc_circuit::Error::UnsupportedFeature(format!(
                "unsupported equality expression combination: left={left:?}, right={right:?}"
            )));
        }
    })
}

struct BranchOperations {
    true_block: (Vec<RealOp>, Vec<usize>),
    false_block: Option<(Vec<RealOp>, Vec<usize>)>,
    successor: BlockId,
}

/// Can only handle basic branches
fn operations_from_branch(
    state: &ProgramMap,
    true_block: BlockId,
    false_block: BlockId,
) -> Result<Option<BranchOperations>, qsc_circuit::Error> {
    let CircuitBlock {
        operations: true_operations,
        terminator: true_terminator,
        ..
    } = state.blocks.get(true_block).expect("block should exist");
    let CircuitBlock {
        operations: false_operations,
        terminator: false_terminator,
        ..
    } = state.blocks.get(false_block).expect("block should exist");

    let true_successor = match true_terminator {
        Some(Terminator::Unconditional(s)) => Some(*s),
        _ => None,
    };
    let false_successor = match false_terminator {
        Some(Terminator::Unconditional(s)) => Some(*s),
        _ => None,
    };

    if true_successor.is_some_and(|c| c == false_block) && false_successor.is_none() {
        // simple if
        let (early_return, target_qubits, true_operations) =
            expand_real_branch_block(true_operations);
        if let Some(e) = early_return {
            return e.map(|_| None);
        }

        Ok(Some(BranchOperations {
            true_block: (true_operations, target_qubits),
            false_block: None,
            successor: false_block,
        }))
    } else if false_successor.is_some_and(|c| c == true_block) && true_successor.is_none() {
        // simple if, but flipped
        // TODO: we need to flip the condition!!
        let (early_return, target_qubits, true_operations) =
            expand_real_branch_block(true_operations);
        if let Some(e) = early_return {
            return e.map(|()| None);
        }

        Ok(Some(BranchOperations {
            true_block: (true_operations, target_qubits),
            false_block: None,
            successor: false_block,
        }))
    } else if true_successor
        .and_then(|true_successor| {
            false_successor.map(|false_successor| (true_successor, false_successor))
        })
        .is_some_and(|(true_successor, false_successor)| true_successor == false_successor)
    {
        let (early_return, true_target_qubits, true_operations) =
            expand_real_branch_block(true_operations);
        if let Some(e) = early_return {
            return e.map(|_| None);
        }

        let true_block = (true_operations, true_target_qubits);

        let (early_return, false_target_qubits, false_operations) =
            expand_real_branch_block(false_operations);

        if let Some(e) = early_return {
            return e.map(|_| None);
        }

        let false_block = (false_operations, false_target_qubits);

        Ok(Some(BranchOperations {
            true_block,
            false_block: Some(false_block),
            successor: true_successor.expect("true_successor should exist"),
        }))
    } else if false_successor
        .and_then(|false_successor| {
            true_successor.map(|true_successor| (true_successor, false_successor))
        })
        .is_some_and(|(true_successor, false_successor)| true_successor == false_successor)
    {
        // if/else, but flipped

        let (early_return, true_target_qubits, true_operations) =
            expand_real_branch_block(true_operations);

        if let Some(e) = early_return {
            return e.map(|_| None);
        }

        let true_block = (true_operations, true_target_qubits);

        let (early_return, false_target_qubits, false_operations) =
            expand_real_branch_block(false_operations);

        if let Some(e) = early_return {
            return e.map(|_| None);
        }

        let false_block = (false_operations.clone(), false_target_qubits);

        Ok(Some(BranchOperations {
            true_block: false_block,
            false_block: Some(true_block),
            successor: true_successor.expect("true_successor should exist"),
        }))
    } else {
        Err(qsc_circuit::Error::UnsupportedFeature(format!(
            "complex branch: true_block={true_block:?} successor={true_successor:?}, false_block={false_block:?} successor={false_successor:?}"
        )))
    }
}

#[must_use]
fn expand_real_branch_block(
    operations: &Vec<Op>,
) -> (
    Option<Result<(), qsc_circuit::Error>>,
    Vec<usize>,
    Vec<RealOp>,
) {
    let mut seen = FxHashSet::default();
    let mut early_return: Option<Result<(), qsc_circuit::Error>> = None;
    let mut real_ops = vec![];
    for op in operations {
        let Op::Real(op) = op else {
            // Skip this one for now, the branch block itself has an unexpanded branch
            early_return = Some(Ok(()));
            break;
        };
        real_ops.push(op.clone());
        for q in op.target_qubits.iter().chain(&op.control_qubits) {
            seen.insert((*q, None));
        }
        for r in op.target_results.iter().chain(&op.control_results) {
            seen.insert((r.0, Some(r.1)));
        }
    }

    if seen.iter().any(|(_, r)| r.is_some()) {
        early_return = Some(Err(qsc_circuit::Error::UnsupportedFeature(
            "measurement operation in a branch block".to_owned(),
        )));
    }

    // TODO: everything is a target. Don't know how else we would do this.
    let target_qubits = seen.into_iter().map(|(q, _)| q).collect();
    (early_return, target_qubits, real_ops)
}

fn expr_from_operand(state: &ProgramMap, operand: &Operand) -> Result<Expr, qsc_circuit::Error> {
    match operand {
        Operand::Literal(literal) => match literal {
            Literal::Result(r) => Ok(Expr::BoolExpr(BoolExpr::Result(*r))),
            Literal::Bool(b) => Ok(Expr::BoolExpr(BoolExpr::LiteralBool(*b))),
            Literal::Integer(i) => Ok(Expr::RichExpr(RichExpr::Literal(i.to_string()))),
            Literal::Double(d) => Ok(Expr::RichExpr(RichExpr::Literal(d.to_string()))),
            _ => Err(qsc_circuit::Error::UnsupportedFeature(format!(
                "unsupported literal operand: {literal:?}"
            ))),
        },
        Operand::Variable(variable) => state.expr_for_variable(variable.variable_id).cloned(),
    }
}

struct ProgramMap {
    /// qubit decl, result idx -> result id
    qubits: Vec<(Qubit, Vec<u32>)>,
    /// result id -> qubit id
    results: IndexMap<usize, u32>,
    /// variable id -> result id
    variables: IndexMap<VariableId, Expr>,
    /// block id -> (operations, successor)
    blocks: IndexMap<BlockId, CircuitBlock>,
}

#[derive(Debug, Clone, Copy)]
struct TwoResultCondition {
    results: (u32, u32),
    // 00, 01, 10, 11
    filter: (bool, bool, bool, bool),
}

#[derive(Debug, Clone)]
enum Expr {
    RichExpr(RichExpr),
    BoolExpr(BoolExpr),
}

#[derive(Debug, Clone)]
enum BoolExpr {
    Result(u32),
    NotResult(u32),
    TwoResultCondition(TwoResultCondition),
    LiteralBool(bool),
    BinOp(Box<Expr>, Box<Expr>, String),
}

#[derive(Debug, Clone)]
enum RichExpr {
    Literal(String),
    Options(Vec<Expr>),
    FunctionOf(Vec<Expr>),
}

impl Expr {
    fn linked_results(&self) -> Vec<u32> {
        match self {
            Expr::RichExpr(rich_expr) => match rich_expr {
                RichExpr::Options(exprs) => exprs.iter().flat_map(Expr::linked_results).collect(),
                RichExpr::Literal(_) => vec![],
                RichExpr::FunctionOf(exprs) => {
                    exprs.iter().flat_map(Expr::linked_results).collect()
                }
            },
            Expr::BoolExpr(condition_expr) => match condition_expr {
                BoolExpr::Result(result_id) | BoolExpr::NotResult(result_id) => {
                    vec![*result_id]
                }
                BoolExpr::TwoResultCondition(two_result_cond) => {
                    vec![two_result_cond.results.0, two_result_cond.results.1]
                }
                BoolExpr::LiteralBool(_) => vec![],
                BoolExpr::BinOp(condition_expr, condition_expr1, _) => condition_expr
                    .linked_results()
                    .into_iter()
                    .chain(condition_expr1.linked_results())
                    .collect(),
            },
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::RichExpr(complicated_expr) => match complicated_expr {
                RichExpr::Options(exprs) => {
                    let exprs_str: Vec<String> = exprs.iter().map(ToString::to_string).collect();
                    write!(f, "one of: ({})", exprs_str.join(", "))
                }
                RichExpr::Literal(literal_str) => write!(f, "{literal_str}"),
                RichExpr::FunctionOf(exprs) => write!(
                    f,
                    "function of: ({})",
                    exprs
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            },
            Expr::BoolExpr(condition_expr) => write!(f, "{condition_expr}"),
        }
    }
}

impl Display for BoolExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoolExpr::Result(_) => write!(f, "a = |1〉"),
            BoolExpr::NotResult(_) => write!(f, "a = |0〉"),
            BoolExpr::LiteralBool(true) => write!(f, "true"),
            BoolExpr::LiteralBool(false) => write!(f, "false"),
            BoolExpr::TwoResultCondition(two_result_cond) => {
                let (f00, f01, f10, f11) = two_result_cond.filter;
                let mut conditions = vec![];
                if f00 {
                    conditions.push("ab = |00〉".to_string());
                }
                if f01 {
                    conditions.push("ab = |01〉".to_string());
                }
                if f10 {
                    conditions.push("ab = |10〉".to_string());
                }
                if f11 {
                    conditions.push("ab = |11〉".to_string());
                }
                write!(f, "{}", conditions.join(" or "))
            }
            BoolExpr::BinOp(condition_expr, condition_expr1, op) => {
                write!(f, "({condition_expr} {op} {condition_expr1})")
            }
        }
    }
}

impl ProgramMap {
    fn into_qubits(self) -> Vec<Qubit> {
        self.qubits
            .into_iter()
            .map(|(q, results)| Qubit {
                id: q.id,
                num_results: results.len(),
            })
            .collect()
    }

    fn new(num_qubits: u32) -> Self {
        Self {
            qubits: (0..num_qubits)
                .map(|id| {
                    (
                        Qubit {
                            id: usize::try_from(id).expect("qubit id should fit in usize"),
                            num_results: 0,
                        },
                        vec![],
                    )
                })
                .collect::<Vec<_>>(),
            variables: IndexMap::new(),
            blocks: IndexMap::new(),
            results: IndexMap::new(),
        }
    }

    fn result_register(&mut self, result_id: u32) -> Register {
        let qubit_id = self
            .results
            .get(usize::try_from(result_id).expect("result id should fit into usize"))
            .copied()
            .expect("result should be linked to a qubit");

        let qubit_result_idx = self.link_result_to_qubit(qubit_id, result_id);

        Register {
            qubit: usize::try_from(qubit_id).expect("qubit id should fit in usize"),
            result: Some(qubit_result_idx),
        }
    }

    fn expr_for_variable(&self, variable_id: VariableId) -> Result<&Expr, qsc_circuit::Error> {
        let expr = self.variables.get(variable_id);
        expr.ok_or_else(|| {
            qsc_circuit::Error::UnsupportedFeature(format!(
                "variable {variable_id:?} is not linked to a result or expression"
            ))
        })
    }

    fn condition_for_variable(
        &self,
        variable_id: VariableId,
    ) -> Result<(Vec<u32>, String), qsc_circuit::Error> {
        let var_expr = self.expr_for_variable(variable_id)?;
        let results = var_expr.linked_results();

        if let Expr::BoolExpr(var_expr) = var_expr {
            if let BoolExpr::LiteralBool(_) = var_expr {
                return Err(qsc_circuit::Error::UnsupportedFeature(
                    "constant condition in branch".to_owned(),
                ));
            }
        } else {
            return Err(qsc_circuit::Error::UnsupportedFeature(format!(
                "variable {variable_id:?} is not a condition expression, cannot branch on it: {var_expr}"
            )));
        }

        let str = var_expr.to_string();

        Ok((results, str))
    }

    fn link_result_to_qubit(&mut self, qubit_id: u32, result_id: u32) -> usize {
        self.results.insert(
            result_id
                .try_into()
                .expect("result id should fit into usize"),
            qubit_id,
        );
        let result_ids_for_qubit =
            &mut self.qubits[usize::try_from(qubit_id).expect("qubit id should fit in usize")].1;
        let qubit_result_idx = result_ids_for_qubit
            .iter_mut()
            .enumerate()
            .find(|(_, qubit_r)| **qubit_r == result_id)
            .map(|(a, _)| a);

        qubit_result_idx.unwrap_or_else(|| {
            result_ids_for_qubit.push(result_id);
            result_ids_for_qubit.len() - 1
        })
    }

    fn store_expr_in_variable(
        &mut self,
        var: Variable,
        expr: Expr,
    ) -> Result<(), qsc_circuit::Error> {
        let variable_id = var.variable_id;
        if let Some(old_value) = self.variables.get(variable_id) {
            panic!("variable {variable_id:?} already stored {old_value:?}, cannot store {expr:?}");
        }
        if let Expr::BoolExpr(condition_expr) = &expr {
            if let Ty::Boolean = var.ty {
            } else {
                return Err(qsc_circuit::Error::UnsupportedFeature(format!(
                    "variable {variable_id:?} has type {var_ty:?} but is being assigned a condition expression: {condition_expr:?}",
                    var_ty = var.ty,
                )));
            }
        }

        self.variables.insert(variable_id, expr);
        Ok(())
    }
}

fn map_callable_to_operations(
    state: &mut ProgramMap,
    callable: &Callable,
    operands: &Vec<Operand>,
    var: Option<Variable>,
    metadata: Option<&InstructionMetadata>,
) -> Result<Vec<RealOp>, qsc_circuit::Error> {
    Ok(match callable.call_type {
        CallableType::Measurement => {
            map_measurement_call_to_operations(state, callable, operands, metadata)?
        }
        CallableType::Reset => map_reset_call_into_operations(state, callable, operands, metadata)?,
        CallableType::Readout => match callable.name.as_str() {
            "__quantum__qis__read_result__body" => {
                for operand in operands {
                    match operand {
                        Operand::Literal(Literal::Result(r)) => {
                            let var =
                                var.expect("read_result must have a variable to store the result");
                            state.store_expr_in_variable(
                                var,
                                Expr::BoolExpr(BoolExpr::Result(*r)),
                            )?;
                        }
                        operand => {
                            return Err(qsc_circuit::Error::UnsupportedFeature(format!(
                                "operand for result readout is not a result: {operand:?}"
                            )));
                        }
                    }
                }
                vec![]
            }
            name => {
                return Err(qsc_circuit::Error::UnsupportedFeature(format!(
                    "unknown readout callable: {name}"
                )));
            }
        },
        CallableType::OutputRecording => {
            vec![]
        }
        CallableType::Regular => {
            let (gate, operand_types) = callable_spec(callable, operands)?;
            if let Some(var) = var {
                let result_expr = Expr::RichExpr(RichExpr::FunctionOf(
                    operands
                        .iter()
                        .map(|o| expr_from_operand(state, o))
                        .collect::<Result<Vec<_>, _>>()?,
                ));

                state.store_expr_in_variable(var, result_expr)?;
            }

            let (targets, controls, args) = gather_operands(state, &operand_types, operands)?;

            if targets.is_empty() && controls.is_empty() {
                // Skip operations without targets or controls.
                // Alternative might be to include these anyway, across the entire state,
                // or annotated in the circuit in some way.
                vec![]
            } else {
                vec![RealOp {
                    kind: OperationKind::Unitary,
                    label: gate.to_string(),
                    target_qubits: targets
                        .iter()
                        .filter_map(|r| {
                            if r.result.is_some() {
                                None
                            } else {
                                Some(r.qubit)
                            }
                        })
                        .collect(),
                    control_qubits: controls
                        .iter()
                        .filter_map(|r| {
                            if r.result.is_some() {
                                None
                            } else {
                                Some(r.qubit)
                            }
                        })
                        .collect(),
                    target_results: targets
                        .iter()
                        .filter_map(|reg| reg.result.map(|r| (reg.qubit, r)))
                        .collect(),
                    control_results: controls
                        .iter()
                        .filter_map(|reg| reg.result.map(|r| (reg.qubit, r)))
                        .collect(),
                    is_adjoint: false,
                    children: vec![],
                    args,
                    metadata: metadata.cloned(),
                }]
            }
        }
    })
}

fn map_reset_call_into_operations(
    state: &mut ProgramMap,
    callable: &Callable,
    operands: &[Operand],
    metadata: Option<&InstructionMetadata>,
) -> Result<Vec<RealOp>, qsc_circuit::Error> {
    Ok(match callable.name.as_str() {
        "__quantum__qis__reset__body" => {
            let operand_types = vec![OperandType::Target];
            let (targets, _, _) = gather_operands(state, &operand_types, operands)?;

            vec![RealOp {
                kind: OperationKind::Ket,
                label: "0".to_string(),
                target_qubits: targets
                    .iter()
                    .filter_map(|r| {
                        if r.result.is_some() {
                            None
                        } else {
                            Some(r.qubit)
                        }
                    })
                    .collect(),
                control_qubits: vec![],
                target_results: targets
                    .iter()
                    .filter_map(|reg| reg.result.map(|r| (reg.qubit, r)))
                    .collect(),
                control_results: vec![],
                is_adjoint: false,
                children: vec![],
                args: vec![],
                metadata: metadata.cloned(),
            }]
        }
        name => {
            return Err(qsc_circuit::Error::UnsupportedFeature(format!(
                "unknown reset callable: {name}"
            )));
        }
    })
}

fn map_measurement_call_to_operations(
    state: &mut ProgramMap,
    callable: &Callable,
    operands: &Vec<Operand>,
    metadata: Option<&InstructionMetadata>,
) -> Result<Vec<RealOp>, qsc_circuit::Error> {
    let gate = match callable.name.as_str() {
        "__quantum__qis__m__body" => "M",
        "__quantum__qis__mresetz__body" => "MResetZ",
        name => name,
    };
    let (this_qubits, this_results) = gather_measurement_operands(state, operands)?;
    Ok(if gate == "MResetZ" {
        vec![
            RealOp {
                kind: OperationKind::Measurement,
                label: gate.to_string(),
                target_qubits: vec![],
                control_qubits: this_qubits
                    .iter()
                    .filter_map(|r| {
                        if r.result.is_some() {
                            None
                        } else {
                            Some(r.qubit)
                        }
                    })
                    .collect(),
                target_results: this_results
                    .iter()
                    .map(|r| {
                        (
                            r.qubit,
                            r.result.expect("result register must have result idx"),
                        )
                    })
                    .collect(),
                control_results: vec![],
                is_adjoint: false,
                children: vec![],
                args: vec![],
                metadata: metadata.cloned(),
            },
            RealOp {
                kind: OperationKind::Ket,
                label: "0".to_string(),
                target_qubits: this_qubits
                    .iter()
                    .filter_map(|r| {
                        if r.result.is_some() {
                            None
                        } else {
                            Some(r.qubit)
                        }
                    })
                    .collect(),
                control_qubits: vec![],
                target_results: vec![],
                control_results: vec![],
                is_adjoint: false,
                children: vec![],
                args: vec![],
                metadata: metadata.cloned(),
            },
        ]
    } else {
        vec![RealOp {
            kind: OperationKind::Measurement,
            label: gate.to_string(),
            target_qubits: vec![],
            control_qubits: this_qubits
                .iter()
                .filter_map(|r| {
                    if r.result.is_some() {
                        None
                    } else {
                        Some(r.qubit)
                    }
                })
                .collect(),
            target_results: this_results
                .iter()
                .map(|r| {
                    (
                        r.qubit,
                        r.result.expect("result register must have result idx"),
                    )
                })
                .collect(),
            control_results: vec![],
            is_adjoint: false,
            children: vec![],
            args: vec![],
            metadata: metadata.cloned(),
        }]
    })
}

fn callable_spec<'a>(
    callable: &'a Callable,
    operands: &[Operand],
) -> Result<(&'a str, Vec<OperandType>), qsc_circuit::Error> {
    Ok(match callable.name.as_str() {
        // single-qubit gates
        "__quantum__qis__x__body" => ("X", vec![OperandType::Target]),
        "__quantum__qis__y__body" => ("Y", vec![OperandType::Target]),
        "__quantum__qis__z__body" => ("Z", vec![OperandType::Target]),
        "__quantum__qis__s__body" => ("S", vec![OperandType::Target]),
        "__quantum__qis__s__adj" => ("S'", vec![OperandType::Target]),
        "__quantum__qis__h__body" => ("H", vec![OperandType::Target]),
        "__quantum__qis__rx__body" => ("Rx", vec![OperandType::Arg, OperandType::Target]),
        "__quantum__qis__ry__body" => ("Ry", vec![OperandType::Arg, OperandType::Target]),
        "__quantum__qis__rz__body" => ("Rz", vec![OperandType::Arg, OperandType::Target]),
        // multi-qubit gates
        "__quantum__qis__cx__body" => ("X", vec![OperandType::Control, OperandType::Target]),
        "__quantum__qis__cy__body" => ("Y", vec![OperandType::Control, OperandType::Target]),
        "__quantum__qis__cz__body" => ("Z", vec![OperandType::Control, OperandType::Target]),
        "__quantum__qis__ccx__body" => (
            "X",
            vec![
                OperandType::Control,
                OperandType::Control,
                OperandType::Target,
            ],
        ),
        "__quantum__qis__rxx__body" => (
            "Rxx",
            vec![OperandType::Arg, OperandType::Target, OperandType::Target],
        ),
        "__quantum__qis__ryy__body" => (
            "Ryy",
            vec![OperandType::Arg, OperandType::Target, OperandType::Target],
        ),
        "__quantum__qis__rzz__body" => (
            "Rzz",
            vec![OperandType::Arg, OperandType::Target, OperandType::Target],
        ),
        custom => {
            let mut operand_types = vec![];
            for o in operands {
                match o {
                    Operand::Literal(Literal::Integer(_) | Literal::Double(_)) => {
                        operand_types.push(OperandType::Arg);
                    }
                    Operand::Variable(Variable {
                        ty: Ty::Boolean | Ty::Integer | Ty::Double,
                        ..
                    }) => operand_types.push(OperandType::Arg),
                    Operand::Variable(Variable { ty: Ty::Qubit, .. })
                    | Operand::Literal(Literal::Qubit(_)) => {
                        // assume all qubit operands are targets for custom gates
                        operand_types.push(OperandType::Target);
                    }
                    o => {
                        return Err(qsc_circuit::Error::UnsupportedFeature(format!(
                            "unsupported operand for custom gate {custom}: {o:?}"
                        )));
                    }
                }
            }

            (custom, operand_types)
        }
    })
}

fn gather_measurement_operands(
    state: &mut ProgramMap,
    operands: &Vec<Operand>,
) -> Result<(Vec<Register>, Vec<Register>), qsc_circuit::Error> {
    let mut qubit_registers = vec![];
    let mut result_registers = vec![];
    let mut qubit_id = None;
    for operand in operands {
        match operand {
            Operand::Literal(Literal::Qubit(q)) => {
                let old = qubit_id.replace(q);
                if old.is_some() {
                    return Err(qsc_circuit::Error::UnsupportedFeature(format!(
                        "measurement should only have one qubit operand, found {old:?} and {q}"
                    )));
                }
                qubit_registers.push(Register {
                    qubit: usize::try_from(*q).expect("qubit id should fit in usize"),
                    result: None,
                });
            }
            Operand::Literal(Literal::Result(r)) => {
                let q = *qubit_id.expect("measurement should have a qubit operand");
                state.link_result_to_qubit(q, *r);
                let result_register = state.result_register(*r);
                result_registers.push(result_register);
            }
            o => {
                return Err(qsc_circuit::Error::UnsupportedFeature(format!(
                    "unsupported operand for measurement: {o:?}"
                )));
            }
        }
    }
    Ok((qubit_registers, result_registers))
}

enum OperandType {
    Control,
    Target,
    Arg,
}

type TargetsControlsArgs = (Vec<Register>, Vec<Register>, Vec<String>);

fn gather_operands(
    state: &mut ProgramMap,
    operand_types: &[OperandType],
    operands: &[Operand],
) -> Result<TargetsControlsArgs, qsc_circuit::Error> {
    let mut targets = vec![];
    let mut controls = vec![];
    let mut args = vec![];
    if operand_types.len() != operands.len() {
        return Err(qsc_circuit::Error::UnsupportedFeature(
            "unexpected number of operands for known operation".to_owned(),
        ));
    }
    for (operand, operand_type) in operands.iter().zip(operand_types) {
        match operand {
            Operand::Literal(literal) => match literal {
                Literal::Qubit(q) => {
                    let operands_array = match operand_type {
                        OperandType::Control => &mut controls,
                        OperandType::Target => &mut targets,
                        OperandType::Arg => {
                            return Err(qsc_circuit::Error::UnsupportedFeature(
                                "qubit operand cannot be an argument".to_owned(),
                            ));
                        }
                    };
                    operands_array.push(Register {
                        qubit: usize::try_from(*q).expect("qubit id should fit in usize"),
                        result: None,
                    });
                }
                Literal::Result(_r) => {
                    return Err(qsc_circuit::Error::UnsupportedFeature(
                        "result operand cannot be a target of a unitary operation".to_owned(),
                    ));
                }
                Literal::Integer(i) => match operand_type {
                    OperandType::Arg => {
                        args.push(i.to_string());
                    }
                    _ => {
                        return Err(qsc_circuit::Error::UnsupportedFeature(
                            "integer operand where qubit was expected".to_owned(),
                        ));
                    }
                },
                Literal::Double(d) => match operand_type {
                    OperandType::Arg => {
                        args.push(format!("{d:.4}"));
                    }
                    _ => {
                        return Err(qsc_circuit::Error::UnsupportedFeature(
                            "double operand where qubit was expected".to_owned(),
                        ));
                    }
                },
                l => {
                    return Err(qsc_circuit::Error::UnsupportedFeature(format!(
                        "unsupported literal operand for unitary operation: {l:?}"
                    )));
                }
            },
            o @ Operand::Variable(var) => {
                if let &OperandType::Arg = operand_type {
                    args.push(state.expr_for_variable(var.variable_id)?.to_string());
                } else {
                    return Err(qsc_circuit::Error::UnsupportedFeature(format!(
                        "variable operand cannot be a target or control of a unitary operation: {o:?}"
                    )));
                }
            }
        }
    }
    Ok((targets, controls, args))
}
