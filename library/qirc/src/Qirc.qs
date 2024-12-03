// cirq.NamedQubit
struct NamedQubit {
    name: String,
    qubit: Qubit
}

operation CreateNamedQubit(name: String) : NamedQubit {
    use q = Qubit();
     new NamedQubit { name = name, qubit = q }

}



export NamedQubit;
