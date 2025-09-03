use crate::circuit;
use qsc_circuit::{
    Circuit, Component, Ket, Measurement, Qubit, Register, Unitary, operation_list_to_grid,
};
use qsc_data_structures::index_map::IndexMap;
use qsc_partial_eval::{
    Callable, CallableType, ConditionCode, Instruction, Literal, Operand, VariableId, rir::Program,
    rir::Variable,
};

pub(crate) fn make_circuit(program: &Program) -> std::result::Result<Circuit, circuit::Error> {
    let mut operations = vec![];

    let mut state = QubitMap::new(program.num_qubits);

    for (id, block) in &program.blocks {
        let mut done = false;
        for instruction in &block.0 {
            assert!(!done, "instructions after return in block {id:?}");
            match instruction {
                Instruction::Call(callable_id, operands, var) => {
                    let callable = program
                        .callables
                        .get(*callable_id)
                        .expect("callable should exist");

                    let this_operations =
                        map_callable_to_operations(&mut state, callable, operands, var.as_ref());

                    operations.extend(this_operations);
                }
                Instruction::Icmp(condition_code, operand, operand1, _variable) => {
                    match condition_code {
                        ConditionCode::Eq => {
                            let _result = result_from_operand(&state, operand);
                            let _result1 = result_from_operand(&state, operand1);
                            // state.link_variable_to_condition(_variable.variable_id);
                            return Err(circuit::Error::ResultComparisonUnsupported);
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
                instruction => {
                    todo!("unsupported instruction in circuit generation: {instruction:?}");
                }
            }
        }
    }

    let circuit = Circuit {
        qubits: state.into_qubits(),
        component_grid: operation_list_to_grid(
            operations,
            program
                .num_qubits
                .try_into()
                .expect("num_qubits should fit in usize"),
        ),
    };
    Ok(dbg!(circuit))
}

fn result_from_operand(state: &QubitMap, operand: &Operand) -> u32 {
    match operand {
        Operand::Literal(_literal) => todo!(),
        Operand::Variable(variable) => state.result_for_variable(variable.variable_id),
    }
}

struct QubitMap {
    /// qubit decl, result idx -> result id
    qubits: Vec<(Qubit, Vec<u32>)>,
    /// variable id -> result id
    variables: IndexMap<VariableId, u32>,
}

impl QubitMap {
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
        }
    }

    fn result_register(&mut self, qubit_id: u32, result_id: u32) -> Register {
        let qubit_result_idx = self.link_result_to_qubit(qubit_id, result_id);

        Register {
            qubit: usize::try_from(qubit_id).expect("qubit id should fit in usize"),
            result: Some(qubit_result_idx),
        }
    }

    fn result_for_variable(&self, variable_id: VariableId) -> u32 {
        *self
            .variables
            .get(variable_id)
            .expect("variable should be linked to a result")
    }

    fn link_result_to_qubit(&mut self, qubit_id: u32, result_id: u32) -> usize {
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

    fn link_variable_to_result(&mut self, result_id: u32, variable_id: VariableId) {
        if let Some(old_result_id) = self.variables.get(variable_id) {
            assert_eq!(
                *old_result_id, result_id,
                "variable {variable_id:?} already linked to result {old_result_id}, cannot link to {result_id}"
            );
        }
        self.variables.insert(variable_id, result_id);
    }

    // fn link_variable_to_condition(&mut self, variable_id: VariableId) {
    //     eprintln!("linking variable {variable_id:?} to condition");
    //     // For now, we don't have a way to represent condition results in the circuit
    // }
}

fn map_callable_to_operations(
    state: &mut QubitMap,
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
                                state.link_variable_to_result(*r, var.variable_id);
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
    state: &mut QubitMap,
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
                    let result_register = state.result_register(q, *r);
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
