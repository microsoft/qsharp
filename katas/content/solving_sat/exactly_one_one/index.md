**Inputs:**

1. Three qubits in an arbitrary state $\ket{x}$ (input/query register).
2. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).

**Goal:**
Implement a quantum oracle which calculates the function that checks whether exactly one of the inputs is $1$, i.e., $f(x) = 1$ if exactly one bit of $x$ is $1$, and $0$ otherwise.
    
Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.