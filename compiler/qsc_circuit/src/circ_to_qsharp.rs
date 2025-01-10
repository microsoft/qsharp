// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

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
        Ok(circuit) => test(circuit),
        Err(e) => format!("Error: {}", e),
    }
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
