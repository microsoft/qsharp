**Inputs:**

1. $N$ qubits in an arbitrary state $\ket{x}$ (input/query register).
2. A qubit in an arbitrary state $\ket{y}$ (output/target qubit).

**Goal:**
Implement a quantum oracle which calculates the AND of the inputs, i.e., $f(x) = x_0 \wedge x_1 \wedge ... \wedge x_{N-1}$.
    
Leave the qubits in the input register in the same state they started in.
Your solution should work on inputs in superposition, and not use any measurements.