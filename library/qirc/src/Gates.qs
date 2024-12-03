import Qirc.NamedQubit;
struct Gate {
    name : String,
    qubits : NamedQubit[]
}

operation H(qubit : NamedQubit) : Gate {
    new Gate { name = "H", qubits = [qubit] }
}

operation CNOT(control : NamedQubit, target : NamedQubit) : Gate {
    new Gate { name = "CNOT", qubits = [control, target] }
}

operation Measure(qubit1 : NamedQubit, qubit2 : NamedQubit) : Gate {
    new Gate { name = "Measure", qubits = [qubit1, qubit2] }
}

operation CZ(qubit1 : NamedQubit, qubit2 : NamedQubit) : Gate {
    new Gate { name = "CZ", qubits = [qubit1, qubit2] }
}