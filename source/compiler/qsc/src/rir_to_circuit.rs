use crate::circuit;
use qsc_circuit::{
    Circuit, Component, ComponentColumn, ComponentGrid, Ket, Measurement, Operation, Qubit,
    Register, Unitary, operation_list_to_grid,
};
use qsc_data_structures::index_map::IndexMap;
use qsc_partial_eval::{
    Callable, CallableType, ConditionCode, Instruction, Literal, Operand, VariableId,
    rir::{Block, BlockId, Program, Variable},
};
use rustc_hash::FxHashSet;

pub(crate) fn make_circuit(program: &Program) -> std::result::Result<Circuit, circuit::Error> {
    let mut state = ProgramMap::new(program.num_qubits);
    let callables = &program.callables;

    // eprintln!("{program}");
    for (id, block) in &program.blocks {
        let block_operations = operations_in_block(&mut state, callables, block)?;
        state.blocks.insert(id, block_operations);
    }

    let mut more_work = true;
    while more_work {
        more_work = false;
        for block in &program.blocks {
            let mut circuit_block = state
                .blocks
                .get(block.0)
                .expect("block should exist")
                .clone();
            for operation in &mut circuit_block.operations {
                if expand_branch_children(&state, operation) {
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

    assert!(
        entry_block.successor.is_none(),
        "entry block should not have a successor"
    );

    let mut operations = vec![];

    let mut operations_stack = entry_block.operations.clone();
    operations_stack.reverse();

    while let Some(mut operation) = operations_stack.pop() {
        if let Component::Unitary(unitary) = &mut operation {
            if unitary.gate == "successor" {
                let successor_block_id =
                    BlockId(unitary.args[0].parse().expect("block id should parse"));
                unitary.args.remove(0);
                unitary.gate = "if ".into();
                // TODO:  gate name & args are bogus
                let successor_block = state
                    .blocks
                    .get(successor_block_id)
                    .expect("successor block should exist");
                for successor_operation in successor_block.operations.iter().rev() {
                    operations_stack.push(successor_operation.clone());
                }
            }
        }
        operations.push(operation.clone());
    }

    let component_grid = operation_list_to_grid(
        operations,
        program
            .num_qubits
            .try_into()
            .expect("num_qubits should fit in usize"),
    );

    let circuit = Circuit {
        qubits: state.into_qubits(),
        component_grid,
    };
    Ok(circuit)
}

#[must_use]
fn expand_branch_children(state: &ProgramMap, operation: &mut Operation) -> bool {
    if let Component::Unitary(unitary) = operation {
        if unitary.gate == "branch" {
            let block_id_1 = BlockId(unitary.args[0].parse().expect("block id should parse"));
            let block_id_2 = BlockId(unitary.args[1].parse().expect("block id should parse"));
            let cond_str = unitary.args[2].clone();
            if let Some((branch_operations, targets)) =
                operations_from_branch(state, block_id_1, block_id_2)
            {
                unitary.targets = targets;
                unitary.children = branch_operations;
                unitary.args = vec![block_id_2.0.to_string(), cond_str];
                unitary.gate = "successor".to_string();
            }
            return true; // more work needed to fill in the branch children
        }
    }
    false
}

#[derive(Clone)]
struct CircuitBlock {
    operations: Vec<Operation>,
    successor: Option<BlockId>,
}

fn operations_in_block(
    state: &mut ProgramMap,
    callables: &IndexMap<qsc_partial_eval::CallableId, Callable>,
    block: &Block,
) -> Result<CircuitBlock, qsc_circuit::Error> {
    let mut successor = None;
    let mut operations = vec![];
    let mut done = false;
    for instruction in &block.0 {
        assert!(!done, "instructions after return or jump in block");
        match instruction {
            Instruction::Call(callable_id, operands, var) => {
                operations.extend(map_callable_to_operations(
                    state,
                    callables.get(*callable_id).expect("callable should exist"),
                    operands,
                    var.as_ref(),
                ));
            }
            Instruction::Icmp(condition_code, operand, operand1, variable) => {
                match condition_code {
                    ConditionCode::Eq => {
                        let expr_left = expr_from_operand(&*state, operand);
                        let expr_right = expr_from_operand(&*state, operand1);
                        let expr = eq_expr(expr_left, expr_right);
                        state.store_expr_in_variable(variable.variable_id, expr);
                    }
                    ConditionCode::Ne => todo!(),
                    ConditionCode::Slt => todo!(),
                    ConditionCode::Sle => todo!(),
                    ConditionCode::Sgt => todo!(),
                    ConditionCode::Sge => todo!(),
                }
            }
            Instruction::Return => {
                done = true;
            }
            Instruction::Branch(variable, block_id_1, block_id_2) => {
                let (results, cond_str) = state.condition_for_variable(variable.variable_id);
                assert!(
                    !results.is_empty(),
                    "branch variable should have at least one result"
                );
                let controls = results
                    .into_iter()
                    .map(|r| state.result_register(r))
                    .collect();
                operations.push(Component::Unitary(Unitary {
                    gate: "branch".to_string(),
                    args: vec![block_id_1.0.to_string(), block_id_2.0.to_string(), cond_str],
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
                }));
            }
            Instruction::Jump(block_id) => {
                let old = successor.replace(*block_id);
                assert!(old.is_none(), "block should not already have a successor");
                done = true;
            }
            instruction => {
                todo!("unsupported instruction in circuit generation: {instruction:?}");
            }
        }
    }
    Ok(CircuitBlock {
        operations,
        successor,
    })
}

fn eq_expr(expr_left: ConditionExpr, expr_right: ConditionExpr) -> ConditionExpr {
    match (expr_left, expr_right) {
        (ConditionExpr::LiteralBool(b1), ConditionExpr::LiteralBool(b2)) => {
            ConditionExpr::LiteralBool(b1 == b2)
        }
        (ConditionExpr::Result(r), ConditionExpr::LiteralBool(b))
        | (ConditionExpr::LiteralBool(b), ConditionExpr::Result(r)) => {
            if b {
                ConditionExpr::Result(r)
            } else {
                ConditionExpr::NotResult(r)
            }
        }
        (ConditionExpr::Result(left), ConditionExpr::Result(right)) => {
            ConditionExpr::TwoResultCondition(TwoResultCondition {
                results: (left, right),
                filter: (true, false, false, true), // 00 and 11
            })
        }
        (left, right) => {
            todo!("unsupported equality expression combination: left={left:?}, right={right:?}")
        }
    }
}

/// Can only handle basic branches, if x { ... } without an else
fn operations_from_branch(
    state: &ProgramMap,
    branch_block: BlockId,
    merge_block: BlockId,
) -> Option<(ComponentGrid, Vec<Register>)> {
    let CircuitBlock {
        operations: branch_operations,
        successor: branch_successor,
    } = state.blocks.get(branch_block).expect("block should exist");
    let CircuitBlock {
        operations: _,
        successor: merge_successor,
    } = state.blocks.get(merge_block).expect("block should exist");
    assert!(
        branch_successor.is_some_and(|c| c == merge_block),
        "successor for branch block should be the merge block"
    );
    assert!(
        merge_successor.is_none(),
        "did not expect a successor for the merge block"
    );

    let mut seen = FxHashSet::default();
    for op in branch_operations {
        match op {
            Operation::Measurement(measurement) => {
                for q in &measurement.qubits {
                    seen.insert((q.qubit, q.result));
                }
                for r in &measurement.results {
                    seen.insert((r.qubit, r.result));
                }
            }
            Operation::Unitary(unitary) => {
                if unitary.gate == "branch" {
                    // Skip this one for now, the branch block itself has an unexpanded branch
                    return None;
                }
                for q in &unitary.targets {
                    seen.insert((q.qubit, q.result));
                }
                for q in &unitary.controls {
                    seen.insert((q.qubit, q.result));
                }
            }
            Operation::Ket(ket) => {
                for q in &ket.targets {
                    seen.insert((q.qubit, q.result));
                }
            }
        }
    }

    assert!(
        !seen.iter().any(|(_, r)| r.is_some()),
        "grouped operations in a branch should only touch qubit registers"
    );

    let component_grid_1 = operation_list_to_grid(branch_operations.clone(), seen.len());

    // TODO: everything is a target. Don't know how else we would do this.

    let targets = seen
        .into_iter()
        .map(|(q, r)| Register {
            qubit: q,
            result: r,
        })
        .collect();
    Some((component_grid_1, targets))
}

fn expr_from_operand(state: &ProgramMap, operand: &Operand) -> ConditionExpr {
    match operand {
        Operand::Literal(literal) => match literal {
            Literal::Result(r) => ConditionExpr::Result(*r),
            Literal::Bool(b) => ConditionExpr::LiteralBool(*b),
            _ => panic!("unsupported literal operand for expression: {literal:?}"),
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
    variables: IndexMap<VariableId, ConditionExpr>,
    /// block id -> (operations, successor)
    blocks: IndexMap<BlockId, CircuitBlock>,
}

#[derive(Debug, Clone, Copy)]
struct TwoResultCondition {
    results: (u32, u32),
    // 00, 01, 10, 11
    filter: (bool, bool, bool, bool),
}

#[derive(Debug, Clone, Copy)]
enum ConditionExpr {
    Result(u32),
    NotResult(u32),
    TwoResultCondition(TwoResultCondition),
    LiteralBool(bool),
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

    fn expr_for_variable(&self, variable_id: VariableId) -> ConditionExpr {
        *self
            .variables
            .get(variable_id)
            .expect("variable should be linked to a result")
    }

    fn condition_for_variable(&self, variable_id: VariableId) -> (Vec<u32>, String) {
        let var_expr = *self
            .variables
            .get(variable_id)
            .unwrap_or_else(|| panic!("variable {variable_id:?} should be linked to a result"));
        let results = match var_expr {
            ConditionExpr::Result(result_id) | ConditionExpr::NotResult(result_id) => {
                vec![result_id]
            }
            ConditionExpr::TwoResultCondition(two_result_cond) => {
                vec![two_result_cond.results.0, two_result_cond.results.1]
            }
            ConditionExpr::LiteralBool(_) => vec![],
        };

        let str = match var_expr {
            ConditionExpr::Result(_) => "a = |1〉".to_string(),
            ConditionExpr::NotResult(_) => "a = |0〉".to_string(),
            ConditionExpr::LiteralBool(_) => todo!("did not expect a literal bool condition"),
            ConditionExpr::TwoResultCondition(two_result_cond) => {
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
                conditions.join(" or ")
            }
        };

        (results, str)
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

    fn store_result_in_variable(&mut self, variable_id: VariableId, result_id: u32) {
        if let Some(old_value) = self.variables.get(variable_id) {
            panic!(
                "variable {variable_id:?} already stored {old_value:?}, cannot store {result_id}"
            );
        }
        self.variables
            .insert(variable_id, ConditionExpr::Result(result_id));
    }

    fn store_expr_in_variable(&mut self, variable_id: VariableId, expr: ConditionExpr) {
        if let Some(old_value) = self.variables.get(variable_id) {
            panic!("variable {variable_id:?} already stored {old_value:?}, cannot store {expr:?}");
        }
        self.variables.insert(variable_id, expr);
    }
}

fn map_callable_to_operations(
    state: &mut ProgramMap,
    callable: &Callable,
    operands: &Vec<Operand>,
    var: Option<&Variable>,
) -> Vec<qsc_circuit::Operation> {
    match callable.call_type {
        CallableType::Measurement => {
            let gate = match callable.name.as_str() {
                "__quantum__qis__m__body" => "M",
                "__quantum__qis__mresetz__body" => "MResetZ",
                _ => panic!("unsupported measurement {callable:?}"),
            };

            let (this_qubits, this_results) = gather_measurement_operands(state, operands);

            if gate == "MResetZ" {
                vec![
                    Component::Measurement(Measurement {
                        gate: gate.to_string(),
                        args: vec![],
                        children: vec![],
                        qubits: this_qubits.clone(),
                        results: this_results,
                    }),
                    Component::Ket(Ket {
                        gate: "0".to_string(),
                        args: vec![],
                        children: vec![],
                        targets: this_qubits,
                    }),
                ]
            } else {
                vec![Component::Measurement(Measurement {
                    gate: gate.to_string(),
                    args: vec![],
                    children: vec![],
                    qubits: this_qubits,
                    results: this_results,
                })]
            }
        }
        CallableType::Reset => match callable.name.as_str() {
            "__quantum__qis__reset__body" => {
                let operand_types = vec![QubitOperandType::Target];
                let (targets, _, _) = gather_operands(&operand_types, operands);

                vec![Component::Ket(Ket {
                    gate: "0".to_string(),
                    args: vec![],
                    children: vec![],
                    targets,
                })]
            }
            _ => {
                panic!("unsupported reset {callable:?}")
            }
        },
        CallableType::Readout => match callable.name.as_str() {
            "__quantum__qis__read_result__body" => {
                for operand in operands {
                    match operand {
                        Operand::Literal(literal) => match literal {
                            Literal::Result(r) => {
                                let var = var
                                    .expect("read_result must have a variable to store the result");
                                state.store_result_in_variable(var.variable_id, *r);
                            }
                            _ => todo!(),
                        },
                        Operand::Variable(_variable) => todo!(),
                    }
                }
                vec![]
            }
            _ => {
                panic!("unsupported readout {callable:?}")
            }
        },
        CallableType::OutputRecording => {
            vec![]
        }
        CallableType::Regular => {
            let (gate, operand_types) = callable_spec(callable, operands);

            let (targets, controls, args) = gather_operands(&operand_types, operands);

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
    }
}

fn callable_spec<'a>(
    callable: &'a Callable,
    operands: &[Operand],
) -> (&'a str, Vec<QubitOperandType>) {
    match callable.name.as_str() {
        // single-qubit gates
        "__quantum__qis__x__body" => ("X", vec![QubitOperandType::Target]),
        "__quantum__qis__y__body" => ("Y", vec![QubitOperandType::Target]),
        "__quantum__qis__h__body" => ("H", vec![QubitOperandType::Target]),
        "__quantum__qis__rx__body" => ("Rx", vec![QubitOperandType::Arg, QubitOperandType::Target]),
        "__quantum__qis__ry__body" => ("Ry", vec![QubitOperandType::Arg, QubitOperandType::Target]),
        // multi-qubit gates
        "__quantum__qis__cx__body" => (
            "X",
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
            (
                custom,
                operands
                    .iter()
                    .map(|o| match o {
                        Operand::Literal(literal) => match literal {
                            Literal::Qubit(_) => QubitOperandType::Target, // assume all qubit operands are targets for custom gates
                            Literal::Result(_) => todo!(),
                            Literal::Bool(_) => todo!(),
                            Literal::Integer(_) | Literal::Double(_) => QubitOperandType::Arg,
                            Literal::Pointer => todo!(),
                        },
                        Operand::Variable(_variable) => todo!(),
                    })
                    .collect::<Vec<_>>(),
            )
        }
    }
}

fn gather_measurement_operands(
    state: &mut ProgramMap,
    operands: &Vec<Operand>,
) -> (Vec<Register>, Vec<Register>) {
    let mut qubit_registers = vec![];
    let mut result_registers = vec![];
    let mut qubit_id = None;
    for operand in operands {
        match operand {
            Operand::Literal(literal) => match literal {
                Literal::Qubit(q) => {
                    let old = qubit_id.replace(q);
                    assert!(
                        old.is_none(),
                        "measurement should only have one qubit operand, found {old:?} and {q}"
                    );
                    qubit_registers.push(Register {
                        qubit: usize::try_from(*q).expect("qubit id should fit in usize"),
                        result: None,
                    });
                }
                Literal::Result(r) => {
                    let q = *qubit_id.expect("measurement should have a qubit operand");
                    state.link_result_to_qubit(q, *r);
                    let result_register = state.result_register(*r);
                    result_registers.push(result_register);
                }
                Literal::Bool(_) => todo!(),
                Literal::Integer(i) => todo!("integer {i}"),
                Literal::Double(_) => todo!(),
                Literal::Pointer => todo!(),
            },
            Operand::Variable(variable) => todo!("variable {variable:?}"),
        }
    }
    (qubit_registers, result_registers)
}

enum QubitOperandType {
    Control,
    Target,
    Arg,
}

fn gather_operands(
    operand_types: &[QubitOperandType],
    operands: &[Operand],
) -> (Vec<Register>, Vec<Register>, Vec<String>) {
    let mut targets = vec![];
    let mut controls = vec![];
    let mut args = vec![];
    assert!(
        operand_types.len() == operands.len(),
        "operand types and operands must have the same length"
    );
    for (operand, operand_type) in operands.iter().zip(operand_types) {
        match operand {
            Operand::Literal(literal) => match literal {
                Literal::Qubit(q) => {
                    let operands_array = match operand_type {
                        QubitOperandType::Control => &mut controls,
                        QubitOperandType::Target => &mut targets,
                        QubitOperandType::Arg => {
                            panic!("expected qubit operand")
                        }
                    };
                    operands_array.push(Register {
                        qubit: usize::try_from(*q).expect("qubit id should fit in usize"),
                        result: None,
                    });
                }
                Literal::Result(r) => {
                    panic!("result {r} cannot be a target of a unitary operation")
                }
                Literal::Bool(_) => todo!(),
                Literal::Integer(i) => match operand_type {
                    QubitOperandType::Arg => {
                        args.push(i.to_string());
                    }
                    _ => panic!("expected argument operand"),
                },
                Literal::Double(d) => match operand_type {
                    QubitOperandType::Arg => {
                        args.push(format!("{d:.4}"));
                    }
                    _ => panic!("expected argument operand"),
                },
                Literal::Pointer => todo!(),
            },
            Operand::Variable(variable) => todo!("variable {variable:?}"),
        }
    }
    (targets, controls, args)
}
