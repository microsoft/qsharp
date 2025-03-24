// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use regex_lite::{Captures, Regex};
use rustc_hash::FxHashMap;

use crate::{
    circuit::{CircuitGroup, Measurement, Unitary},
    Circuit, Operation,
};

pub fn circuits_to_qsharp(file_name: String, circuits_json: String) -> String {
    match serde_json::from_str::<CircuitGroup>(circuits_json.as_str()) {
        Ok(circuits) => build_circuits(file_name, circuits.circuits),
        Err(e) => format!("Error: {}", e),
    }
}

pub fn build_circuits(file_name: String, circuits: Vec<Circuit>) -> String {
    if circuits.len() == 1 {
        build_operation_def(file_name, &circuits[0])
    } else {
        let mut qsharp_str = String::new();
        for (index, circuit) in circuits.iter().enumerate() {
            let circuit_name = format!("{file_name}{index}");
            let circuit_str = build_operation_def(circuit_name, circuit);
            qsharp_str.push_str(&circuit_str);
        }
        qsharp_str
    }
}

pub fn build_operation_def(circuit_name: String, circuit: &Circuit) -> String {
    let mut indentation_level = 0;
    let qubits = circuit
        .qubits
        .iter()
        .enumerate()
        .map(|(i, q)| (q.id, format!("qs[{}]", i)))
        .collect::<FxHashMap<_, _>>();

    let parameter = if qubits.is_empty() {
        String::new()
    } else {
        "qs : Qubit[]".to_string()
    };

    // The return type is determined by the number of qubits "children".
    // However, the actual return statement is determined by the variables storing measurements.
    // If there is an inconsistency between these, which would happen if there was a mismatch between
    // the number of qubit children specified on the circuit and the number of qubit children specified
    // on the measurements, incorrect Q# could be generated.
    let return_type = match circuit.qubits.iter().fold(0, |sum, q| sum + q.num_results) {
        0 => "Unit",
        1 => "Result",
        _ => "Result[]",
    };

    // Check if all operations are Unitary
    let is_ctl_adj = circuit.component_grid.iter().all(|col| {
        col.components.iter().all(|op| {
            if let Operation::Unitary(unitary) = op {
                unitary.gate != "|0〉" && unitary.gate != "|1〉"
            } else {
                false
            }
        })
    });

    let characteristics = if is_ctl_adj { "is Ctl + Adj " } else { "" };

    let summary = if qubits.is_empty() {
        String::new()
    } else {
        format!("/// Expects a qubit register of size {}.\n", qubits.len())
    };

    let mut qsharp_str = format!(
        "{summary}operation {circuit_name}({parameter}) : {return_type} {characteristics}{{\n"
    );
    indentation_level += 1;

    let mut measure_results = vec![];
    let indent = "    ".repeat(indentation_level);

    // Add an assert for the number of qubits
    if !qubits.is_empty() {
        let inner_indent = "    ".repeat(indentation_level + 1);
        qsharp_str.push_str(&format!("{indent}if Length(qs) != {} {{\n", qubits.len()));
        qsharp_str.push_str(&format!(
            "{inner_indent}fail \"Invalid number of qubits. Operation {} expects a qubit register of size {}.\";\n",
            circuit_name,
            qubits.len()
        ));
        qsharp_str.push_str(&format!("{indent}}}\n"));
    }

    let mut body_str = String::new();
    let mut should_add_pi = false;

    // ToDo: Add support for children operations
    for col in &circuit.component_grid {
        for op in &col.components {
            match &op {
                Operation::Measurement(measurement) => {
                    let operation_str = measurement_call(measurement, &qubits);
                    let mut op_results = vec![];
                    for reg in &measurement.results {
                        if let Some(c_id) = reg.result {
                            let result = (format!("c{}_{}", reg.qubit, c_id), (reg.qubit, c_id));
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
                            body_str.push_str(&format!("{indent}{operation_str};\n"));
                        }
                        1 => {
                            body_str
                                .push_str(&format!("{indent}let {result} = {operation_str};\n"));
                            measure_results.extend(op_results);
                        }
                        _ => {
                            body_str
                                .push_str(&format!("{indent}let ({result}) = {operation_str};\n"));
                            measure_results.extend(op_results);
                        }
                    }
                }
                Operation::Unitary(unitary) => {
                    if unitary.gate == "|1〉" {
                        // Note "|1〉" will generate two operations: Reset and X
                        let operation_str = operation_call(unitary, &qubits);
                        body_str.push_str(&format!("{indent}{operation_str};\n"));
                        let op_x = Unitary {
                            gate: "X".to_string(),
                            is_adjoint: false,
                            controls: vec![],
                            targets: unitary.targets.clone(),
                            args: vec![],
                            children: vec![],
                        };
                        let operation_str = operation_call(&op_x, &qubits);
                        body_str.push_str(&format!("{indent}{operation_str};\n"));
                    } else {
                        let operation_str = operation_call(unitary, &qubits);
                        body_str.push_str(&format!("{indent}{operation_str};\n"));
                    };
                }
            }

            // Look for a `π` character in the args
            let args = op.args();
            if !should_add_pi && !args.is_empty() {
                should_add_pi = args.iter().any(|arg| arg.contains("π"));
            }
        }
    }

    // This is a hack to get around the fact that Q# doesn't support π as a constant
    if should_add_pi {
        // Add the π constant
        qsharp_str.push_str(&format!("{indent}let π = Std.Math.PI();\n"));
    }

    qsharp_str.push_str(body_str.as_str());

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

    qsharp_str.push_str("}\n\n");
    qsharp_str
}

fn measurement_call(measurement: &Measurement, qubits: &FxHashMap<usize, String>) -> String {
    let args = measurement
        .qubits
        .iter()
        .map(|q| qubits.get(&q.qubit).unwrap().clone())
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

fn operation_call(unitary: &Unitary, qubits: &FxHashMap<usize, String>) -> String {
    let mut gate = unitary.gate.as_str();

    if gate == "|0〉" || gate == "|1〉" {
        gate = "Reset";
    }

    let is_controlled = !unitary.controls.is_empty();

    let functors = if is_controlled && unitary.is_adjoint {
        "Controlled Adjoint "
    } else if is_controlled {
        "Controlled "
    } else if unitary.is_adjoint {
        "Adjoint "
    } else {
        ""
    };

    let mut args = vec![];

    // // Create the regex for matching integers
    // // let int_regex = Regex::new(r"(?<![\d.])(\d+)(?![\d.])").unwrap();
    // let int_regex = Regex::new(r"([\d.])(\d+)([\d.])").unwrap();

    // for arg in &unitary.args {
    //     // Convert ints to doubles by appending a `.` to the end of the integer
    //     let updated_arg = int_regex.replace_all(arg, "$1.").to_string();

    //     args.push(updated_arg);
    // }

    // Create the regex for matching numbers (both integers and doubles)
    let number_regex = Regex::new(r"((\d+(\.\d*)?)|(\.\d+))").unwrap();

    for arg in &unitary.args {
        // Replace all numbers in the string
        let updated_arg = number_regex
            .replace_all(arg, |caps: &Captures| {
                let number = &caps[0]; // The matched number
                if number.contains('.') {
                    number.to_string() // If it's already a double, leave it unchanged
                } else {
                    format!("{}.", number) // If it's an integer, append a `.`
                }
            })
            .to_string();

        args.push(updated_arg);
    }

    let targets = unitary
        .targets
        .iter()
        .map(|t| qubits.get(&t.qubit).unwrap().clone())
        .collect::<Vec<_>>();
    args.extend(targets);

    if is_controlled {
        let controls = unitary
            .controls
            .iter()
            .filter_map(|c| {
                if c.result.is_none() {
                    Some(qubits.get(&c.qubit).unwrap().clone())
                } else {
                    None
                }
            })
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
