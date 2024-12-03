// cirq.NamedQubit
struct NamedQubit {
    name: String,
    qubit: Qubit
}

function CreateNamedQubit(name: String, qubit: Qubit) : NamedQubit {
    new NamedQubit { name = name, qubit = qubit }
}



export NamedQubit;
