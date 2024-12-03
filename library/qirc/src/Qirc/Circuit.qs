struct Circuit {
    operations: Gates.Gate[]
}

function Append(gates: Gates.Gate[], circuit: Circuit) : Circuit {
    new Circuit {
        operations = circuit.operations + gates
    }
}

operation Execute(circuit: Circuit) : Unit {
    for gate in circuit.operations {
        Message($"Executing ${gate.name}");
        if (gate.name == "H") {
            H(gate.qubits[0].qubit);
        } elif (gate.name == "CNOT") {
            CNOT(gate.qubits[0].qubit, gate.qubits[1].qubit);
        } elif (gate.name == "Measure") {
            let result1 = M(gate.qubits[0].qubit);
            let result2 = M(gate.qubits[1].qubit);
            Message($"Measured {gate.qubits[0].name} to be {result1} and {gate.qubits[1].name} to be {result2}");
        } elif (gate.name == "CZ") {
            CZ(gate.qubits[0].qubit, gate.qubits[1].qubit);
        }
    }
}

export Append, Circuit;