// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rustc_hash::FxHashMap;

use crate::Circuit;

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

pub fn str_test(contents: String) -> String {
    match serde_json::from_str::<Circuit>(contents.as_str()) {
        // Ok(circuit) => test(circuit),
        Ok(circuit) => build_qsharp(circuit),
        Err(e) => format!("Error: {}", e),
    }
}

pub fn build_qsharp(circuit: Circuit) -> String {
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

    let return_type = "Unit";

    let mut qsharp_str = format!("operation PreparePsiMinus({parameters}) : {return_type} {{\n");
    indentation_level += 1;

    for op in circuit.operations {
        let gate = op.gate.as_str();

        let targets = op
            .targets
            .iter()
            .map(|t| qubits.get(&t.q_id).unwrap().clone())
            .collect::<Vec<_>>()
            .join(", ");

        let controls = op
            .controls
            .iter()
            .map(|t| qubits.get(&t.q_id).unwrap().clone())
            .collect::<Vec<_>>()
            .join(", ");

        let args = match (controls.is_empty(), targets.is_empty()) {
            (false, false) => format!("[{controls}], {targets}"),
            (false, true) => format!("[{controls}]"),
            (true, false) => targets,
            (true, true) => "".to_owned(),
        };

        let operation_str = match (op.is_controlled, op.is_adjoint) {
            (false, false) => format!("{gate}({args})"),
            (false, true) => format!("Adjoint {gate}({args})"),
            (true, false) => format!("Controlled {gate}({args})"),
            (true, true) => format!("Controlled Adjoint {gate}({args})"),
        };

        let indent = "    ".repeat(indentation_level);

        qsharp_str.push_str(&format!("{indent}{operation_str};\n"));
    }
    qsharp_str.push_str("}\n");
    qsharp_str
}

pub fn test(_x: Circuit) -> String {
    "
/// # Summary
/// Prepares |Ψ−⟩ = (|01⟩-|10⟩)/√2 state assuming `register` is in |00⟩ state.
operation PreparePsiMinus(register : Qubit[]) : Unit {
    H(register[0]);                 // |+0〉
    Z(register[0]);                 // |-0〉
    X(register[1]);                 // |-1〉
    CNOT(register[0], register[1]); // 1/sqrt(2)(|01〉 - |10〉)
}"
    .to_owned()
}
