// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use rustc_hash::FxHashMap;

use crate::{Circuit, Operation};

// pub fn circ_to_qsharp(circuit: Circuit) -> String {
//     let mut qsharp = String::new();
//     qsharp.push_str(&format!("namespace {} {{\n", circuit.namespace));
//     for operation in circuit.operations {
//         qsharp.push_str(&format!("    operation {}(", operation.name));
//         for (i, arg) in operation.args.iter().enumerate() {
//             qsharp.push_str(&format!("{} : {}, ", arg.name, arg.ty));
//         }
//         qsharp.push_str(") : ");
//         qsharp.push_str(&operation.ret_ty);
//         qsharp.push_str(" {\n");
//         for instr in operation.body {
//             qsharp.push_str(&format!("        {};\n", instr));
//         }
//         qsharp.push_str("    }\n");
//     }
//     qsharp.push_str("}\n");
//     qsharp
// }

pub fn circ_to_qsharp(circuit_name: String, circuit_json: String) -> String {
    match serde_json::from_str::<Circuit>(circuit_json.as_str()) {
        Ok(circuit) => build_qsharp(circuit_name, circuit),
        Err(e) => format!("Error: {}", e),
    }
}

pub fn build_qsharp(circuit_name: String, circuit: Circuit) -> String {
    let mut indentation_level = 0;
    let qubits = circuit
        .qubits
        .iter()
        .map(|q| (q.id, format!("q{}", q.id)))
        .collect::<FxHashMap<_, _>>();

    let mut parameters = qubits.iter().collect::<Vec<_>>();
    parameters.sort_by_key(|(id, _)| *id);
    let parameters = parameters
        .iter()
        .map(|(_, name)| format!("{} : Qubit", name))
        .collect::<Vec<_>>()
        .join(", ");

    // The return type is determined by the number of qubits "children".
    // However, the actual return statement is determined by the variables storing measurements.
    // If there is an inconsistency between these, which would happen if there was a mismatch between
    // the number of qubit children specified on the circuit and the number of qubit children specified
    // on the measurements, incorrect Q# could be generated.
    let return_type = match circuit.qubits.iter().fold(0, |sum, q| sum + q.num_children) {
        0 => "Unit",
        1 => "Result",
        _ => "Result[]",
    };

    let mut qsharp_str = format!("operation {circuit_name}({parameters}) : {return_type} {{\n");
    indentation_level += 1;

    let mut measure_results = vec![];
    let indent = "    ".repeat(indentation_level);
    // ToDo: Add support for children operations
    for op in &circuit.operations {
        if op.is_measurement {
            let operation_str = measurement_call(op, &qubits);
            let mut op_results = vec![];
            for t in &op.targets {
                if let Some(c_id) = t.c_id {
                    let result = (format!("c{}_{}", t.q_id, c_id), (t.q_id, c_id));
                    op_results.push(result.clone());
                }
            }

            // Sort first by q_id, then by c_id
            op_results.sort_by_key(|(_, (q_id, c_id))| (*q_id, *c_id));
            let result = op_results
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            match op_results.len() {
                0 => {
                    qsharp_str.push_str(&format!("{indent}{operation_str};\n"));
                }
                1 => {
                    qsharp_str.push_str(&format!("{indent}let {result} = {operation_str};\n"));
                    measure_results.extend(op_results);
                }
                _ => {
                    qsharp_str.push_str(&format!("{indent}let ({result}) = {operation_str};\n"));
                    measure_results.extend(op_results);
                }
            }
        } else {
            let operation_str = operation_call(op, &qubits);
            qsharp_str.push_str(&format!("{indent}{operation_str};\n"));
        };
    }

    if !measure_results.is_empty() {
        // Sort first by q_id, then by c_id
        measure_results.sort_by_key(|(_, (q_id, c_id))| (*q_id, *c_id));
        match measure_results.len() {
            0 => {}
            1 => {
                let (name, _) = measure_results[0].clone();
                qsharp_str.push_str(&format!("{indent}return {name};\n"));
            }
            _ => {
                let results = measure_results
                    .iter()
                    .map(|(name, _)| name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                qsharp_str.push_str(&format!("{indent}return [{results}];\n"));
            }
        }
    }

    qsharp_str.push_str("}\n");
    qsharp_str
}

fn measurement_call(op: &Operation, qubits: &FxHashMap<usize, String>) -> String {
    // Note: for measurements, the controls are their arguments and the targets are the variables where they results are stored.
    // We may want to change this in the future to be more consistent with the other operations.
    // We also ignore a lot of the usual gate info for measurements, like the gate name and display args.

    let args = op
        .controls
        .iter()
        .map(|c| qubits.get(&c.q_id).unwrap().clone())
        .collect::<Vec<_>>();
    let args_count = args.len();

    let args = args.join(", ");
    if args_count == 1 {
        format!("M({args})")
    } else {
        // This is a joint measurement operation.
        // For now, assume PauliZ measurement basis for all measurements.
        let bases = vec!["PauliZ"; args_count].join(", ");
        format!("Measure([{bases}], [{args}])")
    }
}

fn operation_call(op: &Operation, qubits: &FxHashMap<usize, String>) -> String {
    let gate = op.gate.as_str();
    let functors = if op.is_controlled && op.is_adjoint {
        "Controlled Adjoint "
    } else if op.is_controlled {
        "Controlled "
    } else if op.is_adjoint {
        "Adjoint "
    } else {
        ""
    };

    let mut args = vec![];
    if let Some(display_arg) = op.display_args.clone() {
        args.push(display_arg);
    }

    let targets = op
        .targets
        .iter()
        .map(|t| qubits.get(&t.q_id).unwrap().clone())
        .collect::<Vec<_>>();
    args.extend(targets);

    if op.is_controlled {
        let controls = op
            .controls
            .iter()
            .map(|t| qubits.get(&t.q_id).unwrap().clone())
            .collect::<Vec<_>>()
            .join(", ");
        let controls = format!("[{controls}]");
        let args_count = args.len();
        let mut inner_args = args.join(", ");
        if args_count != 1 {
            inner_args = format!("({})", inner_args);
        }
        args = vec![controls, inner_args];
    }

    let args = args.join(", ");
    format!("{functors}{gate}({args})")
}
