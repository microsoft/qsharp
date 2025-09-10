use std::fmt::Display;

use crate::circuit;
use log::debug;
use qsc_circuit::{
    Circuit, Component, ComponentColumn, ComponentGrid, Ket, Measurement, Operation, Qubit,
    Register, Unitary, operation_list_to_grid,
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

pub(crate) fn make_circuit(
    program: &Program,
    package_store: &PackageStore,
    position_encoding: Encoding,
    loop_detection: bool,
) -> std::result::Result<Circuit, circuit::Error> {
    debug!("make_circuit: program={}", program);
    let mut state = ProgramMap::new(program.num_qubits);
    let callables = &program.callables;

    for (id, block) in program.blocks.iter() {
        let block_operations = operations_in_block(&mut state, callables, block)?;
        state.blocks.insert(id, block_operations);
    }

    let mut more_work = true;
    while more_work {
        more_work = false;
        for block in program.blocks.iter() {
            let mut circuit_block = state
                .blocks
                .get(block.0)
                .expect("block should exist")
                .clone();
            for operation in &mut circuit_block.operations {
                if expand_branch_children(&state, operation, loop_detection)? {
                    more_work = true;
                }
            }
            state.blocks.insert(block.0, circuit_block);
        }
    }

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

    if entry_block.successor.is_some() {
        return Err(circuit::Error::UnsupportedFeature(
            "entry block should not have a successor".to_owned(),
        ));
    }
    let entry_operations = entry_block.operations.clone();

    let num_qubits = program
        .num_qubits
        .try_into()
        .expect("num_qubits should fit in usize");

    let mut component_grid =
        expand_successors(&state, entry_operations, num_qubits, loop_detection);

    fill_in_dbg_metadata(&mut component_grid, package_store, position_encoding)?;

    let circuit = Circuit {
        qubits: state.into_qubits(),
        component_grid,
    };
    Ok(circuit)
}

fn expand_successors(
    state: &ProgramMap,
    block_operations: Vec<Operation>,
    num_qubits: usize,
    loop_detection: bool,
) -> Vec<ComponentColumn> {
    let mut operations = vec![];
    let mut operations_stack = block_operations;
    operations_stack.reverse();

    while let Some(mut operation) = operations_stack.pop() {
        if let Component::Unitary(unitary) = &mut operation {
            if unitary.gate == "successor" {
                let successor_block_id = BlockId(unitary.args[0].parse().unwrap_or_else(|_| {
                    panic!("successor block id should parse: {}", unitary.args[0])
                }));
                unitary.args.remove(0);
                unitary.gate = "check ".into();
                let successor_block = state
                    .blocks
                    .get(successor_block_id)
                    .expect("successor block should exist");
                for successor_operation in successor_block.operations.iter().rev() {
                    operations_stack.push(successor_operation.clone());
                }
                if unitary.children.is_empty()
                    || unitary.children.iter().all(|col| col.components.is_empty())
                {
                    // empty block, skip adding
                    continue;
                }
            }

            if !unitary.children.is_empty() {
                dbg!(
                    "expanding successors for unitary with children: {:?}",
                    &unitary
                );
                assert!(unitary.children.len() == 1);
                let next_column = unitary.children.remove(0);
                let next_column =
                    expand_successors(state, next_column.components, num_qubits, loop_detection);
                unitary.children = next_column;
            }
        }
        operations.push(operation.clone());
    }

    operation_list_to_grid(&operations, num_qubits, loop_detection)
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
            let metadata_str = if let Some(last_arg) = args.last() {
                last_arg.strip_prefix("metadata=").map(ToOwned::to_owned)
            } else {
                None
            };

            if let Some(metadata_str) = metadata_str {
                args.pop();
                // metadata is of the format "!dbg package_id=0 span=[2161-2172] scope=0 discriminator=1"
                // parse it out manually
                let parts: Vec<&str> = metadata_str.split_whitespace().collect();
                if parts.len() >= 3 {
                    let dbg_part = parts[0];
                    if dbg_part != "!dbg" {
                        return Err(qsc_circuit::Error::UnsupportedFeature(format!(
                            "unexpected metadata format, expected !dbg but got: {metadata_str}"
                        )));
                    }
                    let package_id_part = parts[1];
                    let span_part = parts[2];
                    let scope = parts
                        .get(3)
                        .and_then(|s| s.strip_prefix("scope="))
                        .unwrap_or("-1");
                    let discriminator = parts
                        .get(4)
                        .and_then(|s| s.strip_prefix("discriminator="))
                        .unwrap_or("-1");
                    if let Some(package_id_str) = package_id_part.strip_prefix("package_id=") {
                        if let Ok(package_id) = package_id_str.parse::<usize>() {
                            if let Some(span_str) = span_part.strip_prefix("span=[") {
                                if let Some(span_str) = span_str.strip_suffix("]") {
                                    let span_parts: Vec<&str> = span_str.split('-').collect();
                                    if span_parts.len() == 2 {
                                        if let (Ok(start), Ok(end)) = (
                                            span_parts[0].parse::<u32>(),
                                            span_parts[1].parse::<u32>(),
                                        ) {
                                            let span = Span { lo: start, hi: end };
                                            let package_id: PackageId = package_id.into();
                                            let location = Location::from(
                                                span,
                                                package_id,
                                                package_store,
                                                position_encoding,
                                            );
                                            args.push(format!(
                                                r#"metadata={{
    "source": {:?},
    "span": {{
        "start": {{
            "line": {},
            "character": {}
        }},
        "end": {{
            "line": {},
            "character": {}
        }}
    }},
    "scope": {},
    "discriminator": {}
}}"#,
                                                location.source,
                                                location.range.start.line,
                                                location.range.start.column,
                                                location.range.end.line,
                                                location.range.end.column,
                                                scope,
                                                discriminator
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn expand_branch_children(
    state: &ProgramMap,
    operation: &mut Operation,
    loop_detection: bool,
) -> Result<bool, qsc_circuit::Error> {
    if let Component::Unitary(unitary) = operation {
        if unitary.gate == "branch" {
            debug!("found a branch with args: {:?}", unitary.args);
            let true_arg = &unitary.args[0];
            let true_block = BlockId(
                true_arg
                    .parse()
                    .unwrap_or_else(|_| panic!("block id should parse: {true_arg}")),
            );
            let false_arg = &unitary.args[1];
            let false_block = BlockId(
                false_arg
                    .parse()
                    .unwrap_or_else(|_| panic!("block id should parse: {false_arg}")),
            );

            debug!(
                "branching on expr: {:?}, true_block: {:?}, false_block: {:?}",
                unitary.args[2], true_block, false_block
            );
            if let Some(branch_operations) =
                operations_from_branch(state, true_block, false_block, loop_detection)?
            {
                let (true_operations, true_targets) = branch_operations.true_block;
                let true_container = Some(Component::Unitary(Unitary {
                    gate: "true".into(),
                    args: vec![],
                    children: true_operations,
                    targets: true_targets.clone(),
                    controls: unitary.controls.clone(),
                    is_adjoint: unitary.is_adjoint,
                }));

                let false_container =
                    branch_operations
                        .false_block
                        .map(|(false_operations, false_targets)| {
                            (
                                Component::Unitary(Unitary {
                                    gate: "false".into(),
                                    args: vec![],
                                    children: false_operations,
                                    targets: false_targets.clone(),
                                    controls: unitary.controls.clone(),
                                    is_adjoint: unitary.is_adjoint,
                                }),
                                false_targets,
                            )
                        });

                let true_container = true_container.and_then(|t| {
                    if t.children().iter().any(|col| !col.components.is_empty()) {
                        Some(t)
                    } else {
                        None
                    }
                });
                let false_container = false_container.and_then(|f| {
                    if f.0.children().iter().any(|col| !col.components.is_empty()) {
                        Some(f)
                    } else {
                        None
                    }
                });

                let mut children = vec![];

                if let Some(true_container) = true_container {
                    children.push(true_container);
                    unitary.targets.extend(true_targets);
                }

                if let Some((false_container, false_targets)) = false_container {
                    children.push(false_container);
                    unitary.targets.extend(false_targets);
                }

                unitary.children = vec![ComponentColumn {
                    components: children,
                }];

                // dedup targets
                unitary.targets.sort_by_key(|r| (r.qubit, r.result));
                unitary.targets.dedup_by_key(|r| (r.qubit, r.result));

                unitary.args.remove(0);
                unitary.args.remove(0);
                unitary
                    .args
                    .insert(0, branch_operations.successor.0.to_string());
                unitary.gate = "successor".to_string();
            } else {
                return Ok(true); // more work needed to fill in the branch children
            }
        }
    }
    Ok(false)
}

#[derive(Clone)]
struct CircuitBlock {
    _predecessors: Vec<BlockId>,
    operations: Vec<Operation>,
    successor: Option<BlockId>,
}

fn operations_in_block(
    state: &mut ProgramMap,
    callables: &IndexMap<qsc_partial_eval::CallableId, Callable>,
    block: &BlockWithMetadata,
) -> Result<CircuitBlock, qsc_circuit::Error> {
    // TODO: use get_block_successors from utils
    let mut successor = None;
    let mut predecessors = vec![];
    let mut operations = vec![];
    let mut done = false;
    for instruction in &block.0 {
        if done {
            return Err(qsc_circuit::Error::UnsupportedFeature(
                "instructions after return or jump in block".to_owned(),
            ));
        }
        extend_with_instruction(
            state,
            callables,
            &mut successor,
            &mut predecessors,
            &mut operations,
            &mut done,
            instruction,
        )?;
    }
    Ok(CircuitBlock {
        _predecessors: predecessors,
        operations,
        successor,
    })
}

fn extend_with_instruction(
    state: &mut ProgramMap,
    callables: &IndexMap<qsc_partial_eval::CallableId, Callable>,
    successor: &mut Option<BlockId>,
    predecessors: &mut Vec<BlockId>,
    operations: &mut Vec<Operation>,
    done: &mut bool,
    instruction: &InstructionWithMetadata,
) -> Result<(), qsc_circuit::Error> {
    match &instruction.instruction {
        Instruction::Call(callable_id, operands, var) => {
            operations.extend(map_callable_to_operations(
                state,
                callables.get(*callable_id).expect("callable should exist"),
                operands,
                var.as_ref(),
                instruction.metadata.as_ref(),
            )?);
        }
        Instruction::Fcmp(condition_code, operand, operand1, variable) => {
            let expr_left = expr_from_operand(state, operand)?;
            let expr_right = expr_from_operand(state, operand1)?;
            let expr = match condition_code {
                FcmpConditionCode::False => BoolExpr::LiteralBool(false),
                FcmpConditionCode::True => BoolExpr::LiteralBool(true),
                cmp => BoolExpr::BinOp(expr_left.into(), expr_right.into(), cmp.to_string()),
            };

            state.store_expr_in_variable(*variable, Expr::BoolExpr(expr))?;
        }
        Instruction::Icmp(condition_code, operand, operand1, variable) => match condition_code {
            ConditionCode::Eq => {
                let expr_left = expr_from_operand(state, operand)?;
                let expr_right = expr_from_operand(state, operand1)?;
                let expr = eq_expr(expr_left, expr_right)?;
                state.store_expr_in_variable(*variable, Expr::BoolExpr(expr))?;
            }
            condition_code => {
                return Err(qsc_circuit::Error::UnsupportedFeature(format!(
                    "unsupported condition code in icmp: {condition_code:?}"
                )));
            }
        },

        Instruction::Return => {
            *done = true;
        }
        Instruction::Branch(variable, block_id_1, block_id_2) => {
            operations.push(operation_for_branch(
                state,
                instruction,
                *variable,
                *block_id_1,
                *block_id_2,
            )?);
        }
        Instruction::Jump(block_id) => {
            let old = successor.replace(*block_id);
            if old.is_some() {
                return Err(qsc_circuit::Error::UnsupportedFeature(
                    "block contains more than one jump".to_owned(),
                ));
            }
            *done = true;
        }
        Instruction::Phi(pres, variable) => {
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
                Expr::RichExpr(NotBoolExpr::Options(exprs))
            };
            state.store_expr_in_variable(*variable, expr)?;
            predecessors.extend(pres.iter().map(|p| p.1));
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
            let expr_left = expr_from_operand(state, operand)?;
            let expr_right = expr_from_operand(state, operand1)?;
            let expr = Expr::RichExpr(NotBoolExpr::BinOp(
                expr_left.into(),
                expr_right.into(),
                "***".into(),
            ));
            state.store_expr_in_variable(*variable, expr)?;
        }
        Instruction::LogicalNot(..) | Instruction::BitwiseNot(..) => {
            // Leave the variable unassigned, if it's used in anything that's going to be shown in the circuit, we'll raise an error then
        }
        instruction @ Instruction::Store(..) => {
            return Err(qsc_circuit::Error::UnsupportedFeature(format!(
                "unsupported instruction in block: {instruction:?}"
            )));
        }
    }
    Ok(())
}

fn operation_for_branch(
    state: &mut ProgramMap,
    instruction: &InstructionWithMetadata,
    variable: Variable,
    true_block: BlockId,
    false_block: BlockId,
) -> Result<Operation, qsc_circuit::Error> {
    let (results, cond_str) = state.condition_for_variable(variable.variable_id)?;
    // TODO: let's allow this for now, though it's weird (see phi nodes)
    // if results.is_empty() {
    //     return Err(qsc_circuit::Error::UnsupportedFeature(format!(
    //         "branching on a condition that doesn't involve at least one result: {cond_str}"
    //     )));
    // }
    let controls = results
        .into_iter()
        .map(|r| state.result_register(r))
        .collect();
    let mut args = vec![
        true_block.0.to_string(),
        false_block.0.to_string(),
        cond_str,
    ];
    if let Some(metadata) = instruction.metadata.as_ref() {
        args.push(metadata_arg(metadata));
    }
    let op = Component::Unitary(Unitary {
        gate: "branch".to_string(),
        args,
        children: vec![ComponentColumn {
            components: vec![Component::Unitary(Unitary {
                gate: "block_placeholder".to_string(),
                args: vec![],
                children: vec![],
                targets: vec![],
                controls: vec![],
                is_adjoint: false,
            })],
        }],
        targets: vec![],
        controls,
        is_adjoint: false,
    });
    debug!("pushed a branch with args: {:?}", op.args());
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
    true_block: (ComponentGrid, Vec<Register>),
    false_block: Option<(ComponentGrid, Vec<Register>)>,
    successor: BlockId,
}

/// Can only handle basic branches
fn operations_from_branch(
    state: &ProgramMap,
    true_block: BlockId,
    false_block: BlockId,
    loop_detection: bool,
) -> Result<Option<BranchOperations>, qsc_circuit::Error> {
    let CircuitBlock {
        operations: true_operations,
        successor: true_successor,
        ..
    } = state.blocks.get(true_block).expect("block should exist");
    let CircuitBlock {
        operations: false_operations,
        successor: false_successor,
        ..
    } = state.blocks.get(false_block).expect("block should exist");

    if true_successor.is_some_and(|c| c == false_block) && false_successor.is_none() {
        // simple if
        let mut seen = FxHashSet::default();
        let mut max_qubit_id = 0;
        for op in true_operations {
            match op {
                Operation::Measurement(measurement) => {
                    for q in &measurement.qubits {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                    for r in &measurement.results {
                        max_qubit_id = max_qubit_id.max(r.qubit);
                        seen.insert((r.qubit, r.result));
                    }
                }
                Operation::Unitary(unitary) => {
                    if unitary.gate == "branch" {
                        // Skip this one for now, the branch block itself has an unexpanded branch
                        return Ok(None);
                    }
                    for q in &unitary.targets {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                    for q in &unitary.controls {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                }
                Operation::Ket(ket) => {
                    for q in &ket.targets {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                }
            }
        }

        if seen.iter().any(|(_, r)| r.is_some()) {
            return Err(qsc_circuit::Error::UnsupportedFeature(
                "measurement operation in a branch block".to_owned(),
            ));
        }

        let component_grid =
            operation_list_to_grid(true_operations, max_qubit_id + 1, loop_detection);

        // TODO: everything is a target. Don't know how else we would do this.

        let targets = seen
            .into_iter()
            .map(|(q, r)| Register {
                qubit: q,
                result: r,
            })
            .collect();
        Ok(Some(BranchOperations {
            true_block: (component_grid, targets),
            false_block: None,
            successor: false_block,
        }))
    } else if false_successor.is_some_and(|c| c == true_block) && true_successor.is_none() {
        // simple if, but flipped
        // TODO: we need to flip the condition!!
        let mut seen = FxHashSet::default();
        let mut max_qubit_id = 0;
        for op in true_operations {
            match op {
                Operation::Measurement(measurement) => {
                    for q in &measurement.qubits {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                    for r in &measurement.results {
                        max_qubit_id = max_qubit_id.max(r.qubit);
                        seen.insert((r.qubit, r.result));
                    }
                }
                Operation::Unitary(unitary) => {
                    if unitary.gate == "branch" {
                        // Skip this one for now, the branch block itself has an unexpanded branch
                        return Ok(None);
                    }
                    for q in &unitary.targets {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                    for q in &unitary.controls {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                }
                Operation::Ket(ket) => {
                    for q in &ket.targets {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                }
            }
        }

        if seen.iter().any(|(_, r)| r.is_some()) {
            return Err(qsc_circuit::Error::UnsupportedFeature(
                "measurement operation in a branch block".to_owned(),
            ));
        }

        let component_grid =
            operation_list_to_grid(true_operations, max_qubit_id + 1, loop_detection);

        // TODO: everything is a target. Don't know how else we would do this.

        let targets = seen
            .into_iter()
            .map(|(q, r)| Register {
                qubit: q,
                result: r,
            })
            .collect();
        Ok(Some(BranchOperations {
            true_block: (component_grid, targets),
            false_block: None,
            successor: false_block,
        }))
    } else if true_successor
        .and_then(|true_successor| {
            false_successor.map(|false_successor| (true_successor, false_successor))
        })
        .is_some_and(|(true_successor, false_successor)| true_successor == false_successor)
    {
        let mut seen = FxHashSet::default();
        let mut max_qubit_id = 0;
        for op in true_operations {
            match op {
                Operation::Measurement(measurement) => {
                    for q in &measurement.qubits {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                    for r in &measurement.results {
                        max_qubit_id = max_qubit_id.max(r.qubit);
                        seen.insert((r.qubit, r.result));
                    }
                }
                Operation::Unitary(unitary) => {
                    if unitary.gate == "branch" {
                        // Skip this one for now, the branch block itself has an unexpanded branch
                        return Ok(None);
                    }
                    for q in &unitary.targets {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                    for q in &unitary.controls {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                }
                Operation::Ket(ket) => {
                    for q in &ket.targets {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                }
            }
        }

        if seen.iter().any(|(_, r)| r.is_some()) {
            return Err(qsc_circuit::Error::UnsupportedFeature(
                "measurement operation in a branch block".to_owned(),
            ));
        }

        let component_grid =
            operation_list_to_grid(true_operations, max_qubit_id + 1, loop_detection);

        // TODO: everything is a target. Don't know how else we would do this.

        let targets = seen
            .into_iter()
            .map(|(q, r)| Register {
                qubit: q,
                result: r,
            })
            .collect();

        let true_block = (component_grid, targets);

        let mut seen = FxHashSet::default();
        let mut max_qubit_id = 0;
        for op in false_operations {
            match op {
                Operation::Measurement(measurement) => {
                    for q in &measurement.qubits {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                    for r in &measurement.results {
                        max_qubit_id = max_qubit_id.max(r.qubit);
                        seen.insert((r.qubit, r.result));
                    }
                }
                Operation::Unitary(unitary) => {
                    if unitary.gate == "branch" {
                        // Skip this one for now, the branch block itself has an unexpanded branch
                        return Ok(None);
                    }
                    for q in &unitary.targets {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                    for q in &unitary.controls {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                }
                Operation::Ket(ket) => {
                    for q in &ket.targets {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                }
            }
        }

        if seen.iter().any(|(_, r)| r.is_some()) {
            return Err(qsc_circuit::Error::UnsupportedFeature(
                "measurement operation in a branch block".to_owned(),
            ));
        }

        let component_grid =
            operation_list_to_grid(true_operations, max_qubit_id + 1, loop_detection);

        // TODO: everything is a target. Don't know how else we would do this.

        let targets = seen
            .into_iter()
            .map(|(q, r)| Register {
                qubit: q,
                result: r,
            })
            .collect();

        let false_block = (component_grid, targets);

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

        let mut seen = FxHashSet::default();
        let mut max_qubit_id = 0;
        for op in true_operations {
            match op {
                Operation::Measurement(measurement) => {
                    for q in &measurement.qubits {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                    for r in &measurement.results {
                        max_qubit_id = max_qubit_id.max(r.qubit);
                        seen.insert((r.qubit, r.result));
                    }
                }
                Operation::Unitary(unitary) => {
                    if unitary.gate == "branch" {
                        // Skip this one for now, the branch block itself has an unexpanded branch
                        return Ok(None);
                    }
                    for q in &unitary.targets {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                    for q in &unitary.controls {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                }
                Operation::Ket(ket) => {
                    for q in &ket.targets {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                }
            }
        }

        if seen.iter().any(|(_, r)| r.is_some()) {
            return Err(qsc_circuit::Error::UnsupportedFeature(
                "measurement operation in a branch block".to_owned(),
            ));
        }

        let component_grid =
            operation_list_to_grid(true_operations, max_qubit_id + 1, loop_detection);

        // TODO: everything is a target. Don't know how else we would do this.

        let targets = seen
            .into_iter()
            .map(|(q, r)| Register {
                qubit: q,
                result: r,
            })
            .collect();

        let true_block = (component_grid, targets);

        let mut seen = FxHashSet::default();
        let mut max_qubit_id = 0;
        for op in false_operations {
            match op {
                Operation::Measurement(measurement) => {
                    for q in &measurement.qubits {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                    for r in &measurement.results {
                        max_qubit_id = max_qubit_id.max(r.qubit);
                        seen.insert((r.qubit, r.result));
                    }
                }
                Operation::Unitary(unitary) => {
                    if unitary.gate == "branch" {
                        // Skip this one for now, the branch block itself has an unexpanded branch
                        return Ok(None);
                    }
                    for q in &unitary.targets {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                    for q in &unitary.controls {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                }
                Operation::Ket(ket) => {
                    for q in &ket.targets {
                        max_qubit_id = max_qubit_id.max(q.qubit);
                        seen.insert((q.qubit, q.result));
                    }
                }
            }
        }

        if seen.iter().any(|(_, r)| r.is_some()) {
            return Err(qsc_circuit::Error::UnsupportedFeature(
                "measurement operation in a branch block".to_owned(),
            ));
        }

        let component_grid =
            operation_list_to_grid(true_operations, max_qubit_id + 1, loop_detection);

        // TODO: everything is a target. Don't know how else we would do this.

        let targets = seen
            .into_iter()
            .map(|(q, r)| Register {
                qubit: q,
                result: r,
            })
            .collect();

        let false_block = (component_grid, targets);

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

fn expr_from_operand(state: &ProgramMap, operand: &Operand) -> Result<Expr, qsc_circuit::Error> {
    match operand {
        Operand::Literal(literal) => match literal {
            Literal::Result(r) => Ok(Expr::BoolExpr(BoolExpr::Result(*r))),
            Literal::Bool(b) => Ok(Expr::BoolExpr(BoolExpr::LiteralBool(*b))),
            Literal::Integer(i) => Ok(Expr::RichExpr(NotBoolExpr::Literal(i.to_string()))),
            Literal::Double(d) => Ok(Expr::RichExpr(NotBoolExpr::Literal(d.to_string()))),
            _ => Err(qsc_circuit::Error::UnsupportedFeature(format!(
                "unsupported literal operand: {literal:?}"
            ))),
        },
        Operand::Variable(variable) => state.expr_for_variable(variable.variable_id),
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
    RichExpr(NotBoolExpr),
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
enum NotBoolExpr {
    Literal(String),
    Options(Vec<Expr>),
    BinOp(Box<Expr>, Box<Expr>, String),
}

impl Expr {
    fn linked_results(&self) -> Vec<u32> {
        match self {
            Expr::RichExpr(rich_expr) => match rich_expr {
                NotBoolExpr::Options(exprs) => {
                    exprs.iter().flat_map(Expr::linked_results).collect()
                }
                NotBoolExpr::Literal(_) => vec![],
                NotBoolExpr::BinOp(expr, expr1, _) => expr
                    .linked_results()
                    .into_iter()
                    .chain(expr1.linked_results())
                    .collect(),
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
                NotBoolExpr::Options(exprs) => {
                    let exprs_str: Vec<String> = exprs.iter().map(ToString::to_string).collect();
                    write!(f, "one of: ({})", exprs_str.join(", "))
                }
                NotBoolExpr::Literal(literal_str) => write!(f, "{literal_str}"),
                NotBoolExpr::BinOp(expr, expr1, op) => write!(f, "({expr}) {op} ({expr1})"),
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

    fn expr_for_variable(&self, variable_id: VariableId) -> Result<Expr, qsc_circuit::Error> {
        let expr = self.variables.get(variable_id);
        expr.cloned().ok_or_else(|| {
            qsc_circuit::Error::UnsupportedFeature(format!(
                "variable {variable_id:?} is not linked to a result or expression"
            ))
        })
    }

    fn condition_for_variable(
        &self,
        variable_id: VariableId,
    ) -> Result<(Vec<u32>, String), qsc_circuit::Error> {
        let var_expr = self.variables.get(variable_id);
        let var_expr = var_expr.ok_or_else(|| {
            qsc_circuit::Error::UnsupportedFeature(format!(
                "variable {variable_id:?} is not linked to a result or expression"
            ))
        })?;
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
    var: Option<&Variable>,
    metadata: Option<&InstructionMetadata>,
) -> Result<Vec<qsc_circuit::Operation>, qsc_circuit::Error> {
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
                                *var.expect("read_result must have a variable to store the result");
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

            let (targets, controls, mut args) = gather_operands(state, &operand_types, operands)?;
            if let Some(metadata) = metadata {
                args.push(metadata_arg(metadata));
            }

            if targets.is_empty() && controls.is_empty() {
                // Skip operations without targets or controls.
                // Alternative might be to include these anyway, across the entire state,
                // or annotated in the circuit in some way.
                vec![]
            } else {
                vec![Component::Unitary(Unitary {
                    gate: gate.to_string(),
                    args,
                    children: vec![],
                    targets,
                    controls,
                    is_adjoint: false,
                })]
            }
        }
    })
}

fn map_reset_call_into_operations(
    state: &mut ProgramMap,
    callable: &Callable,
    operands: &[Operand],
    metadata: Option<&InstructionMetadata>,
) -> Result<Vec<Operation>, qsc_circuit::Error> {
    Ok(match callable.name.as_str() {
        "__quantum__qis__reset__body" => {
            let operand_types = vec![QubitOperandType::Target];
            let (targets, _, _) = gather_operands(state, &operand_types, operands)?;

            vec![Component::Ket(Ket {
                gate: "0".to_string(),
                args: arg_vec_with_only_metadata(metadata),
                children: vec![],
                targets,
            })]
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
) -> Result<Vec<Operation>, qsc_circuit::Error> {
    let gate = match callable.name.as_str() {
        "__quantum__qis__m__body" => "M",
        "__quantum__qis__mresetz__body" => "MResetZ",
        name => name,
    };
    let (this_qubits, this_results) = gather_measurement_operands(state, operands)?;
    Ok(if gate == "MResetZ" {
        vec![
            Component::Measurement(Measurement {
                gate: gate.to_string(),
                args: arg_vec_with_only_metadata(metadata),
                children: vec![],
                qubits: this_qubits.clone(),
                results: this_results,
            }),
            Component::Ket(Ket {
                gate: "0".to_string(),
                args: arg_vec_with_only_metadata(metadata),
                children: vec![],
                targets: this_qubits,
            }),
        ]
    } else {
        vec![Component::Measurement(Measurement {
            gate: gate.to_string(),
            args: arg_vec_with_only_metadata(metadata),
            children: vec![],
            qubits: this_qubits,
            results: this_results,
        })]
    })
}

fn metadata_arg(m: &InstructionMetadata) -> String {
    format!("metadata={}", m.str)
}

fn arg_vec_with_only_metadata(m: Option<&InstructionMetadata>) -> Vec<String> {
    m.map_or(vec![], |m| vec![metadata_arg(m)])
}

fn callable_spec<'a>(
    callable: &'a Callable,
    operands: &[Operand],
) -> Result<(&'a str, Vec<QubitOperandType>), qsc_circuit::Error> {
    Ok(match callable.name.as_str() {
        // single-qubit gates
        "__quantum__qis__x__body" => ("X", vec![QubitOperandType::Target]),
        "__quantum__qis__y__body" => ("Y", vec![QubitOperandType::Target]),
        "__quantum__qis__z__body" => ("Z", vec![QubitOperandType::Target]),
        "__quantum__qis__s__body" => ("S", vec![QubitOperandType::Target]),
        "__quantum__qis__s__adj" => ("S'", vec![QubitOperandType::Target]),
        "__quantum__qis__h__body" => ("H", vec![QubitOperandType::Target]),
        "__quantum__qis__rx__body" => ("Rx", vec![QubitOperandType::Arg, QubitOperandType::Target]),
        "__quantum__qis__ry__body" => ("Ry", vec![QubitOperandType::Arg, QubitOperandType::Target]),
        "__quantum__qis__rz__body" => ("Rz", vec![QubitOperandType::Arg, QubitOperandType::Target]),
        // multi-qubit gates
        "__quantum__qis__cx__body" => (
            "X",
            vec![QubitOperandType::Control, QubitOperandType::Target],
        ),
        "__quantum__qis__cy__body" => (
            "Y",
            vec![QubitOperandType::Control, QubitOperandType::Target],
        ),
        "__quantum__qis__cz__body" => (
            "Z",
            vec![QubitOperandType::Control, QubitOperandType::Target],
        ),
        "__quantum__qis__ccx__body" => (
            "X",
            vec![
                QubitOperandType::Control,
                QubitOperandType::Control,
                QubitOperandType::Target,
            ],
        ),
        "__quantum__qis__rxx__body" => (
            "Rxx",
            vec![
                QubitOperandType::Arg,
                QubitOperandType::Target,
                QubitOperandType::Target,
            ],
        ),
        custom => {
            let mut operand_types = vec![];
            for o in operands {
                match o {
                    Operand::Literal(Literal::Qubit(_)) => {
                        operand_types.push(QubitOperandType::Target);
                    } // assume all qubit operands are targets for custom gates
                    Operand::Literal(Literal::Integer(_) | Literal::Double(_)) => {
                        operand_types.push(QubitOperandType::Arg);
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

enum QubitOperandType {
    Control,
    Target,
    Arg,
}

type TargetsControlsArgs = (Vec<Register>, Vec<Register>, Vec<String>);

fn gather_operands(
    state: &mut ProgramMap,
    operand_types: &[QubitOperandType],
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
                        QubitOperandType::Control => &mut controls,
                        QubitOperandType::Target => &mut targets,
                        QubitOperandType::Arg => {
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
                    QubitOperandType::Arg => {
                        args.push(i.to_string());
                    }
                    _ => {
                        return Err(qsc_circuit::Error::UnsupportedFeature(
                            "integer operand where qubit was expected".to_owned(),
                        ));
                    }
                },
                Literal::Double(d) => match operand_type {
                    QubitOperandType::Arg => {
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
                if let &QubitOperandType::Arg = operand_type {
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
